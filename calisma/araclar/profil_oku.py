"""VIctory örnekleyici çıktısını Windows PDB sembolleriyle özetler."""

import argparse
import ctypes as ct
import re
import struct
import sys
from collections import Counter, defaultdict
from contextlib import nullcontext
from ctypes import wintypes as wt
from dataclasses import dataclass
from functools import cache
from pathlib import Path


DEFAULT_EXE = Path(__file__).resolve().parent.parent / "vw-avatar/target/diag/victory.exe"
BASE = 0x140000000
SYMOPT_UNDNAME = 0x2
SYMOPT_DEFERRED_LOADS = 0x4
SYMOPT_LOAD_LINES = 0x10
SYMOPT_IGNORE_NT_SYMPATH = 0x1000
SYMOPT_EXACT_SYMBOLS = 0x400
MAX_NAME = 8192
U64 = ct.c_ulonglong


class SymbolInfo(ct.Structure):
    _fields_ = [
        ("SizeOfStruct", wt.ULONG), ("TypeIndex", wt.ULONG), ("Reserved", U64 * 2),
        ("Index", wt.ULONG), ("Size", wt.ULONG), ("ModBase", U64),
        ("Flags", wt.ULONG), ("Value", U64), ("Address", U64),
        ("Register", wt.ULONG), ("Scope", wt.ULONG), ("Tag", wt.ULONG),
        ("NameLen", wt.ULONG), ("MaxNameLen", wt.ULONG), ("Name", ct.c_wchar * 1),
    ]


class LineInfo(ct.Structure):
    _fields_ = [
        ("SizeOfStruct", wt.DWORD), ("Key", ct.c_void_p), ("LineNumber", wt.DWORD),
        ("FileName", wt.LPWSTR), ("Address", U64),
    ]


@dataclass(frozen=True)
class Frame:
    name: str
    address: int | None = None
    location: str = ""

    @property
    def key(self):
        return self.name, self.address


def image_size(exe):
    with exe.open("rb") as image:
        if image.read(2) != b"MZ":
            raise ValueError(f"Geçersiz PE dosyası ({exe})")
        image.seek(0x3C)
        pe_offset, = struct.unpack("<I", image.read(4))
        image.seek(pe_offset)
        if image.read(4) != b"PE\0\0":
            raise ValueError(f"Geçersiz PE başlığı ({exe})")
        image.seek(pe_offset + 24 + 56)
        size, = struct.unpack("<I", image.read(4))
        return size


class Symbols:
    def __init__(self, exe):
        self.exe = exe.resolve()
        self.size = image_size(self.exe)
        if not self.exe.with_suffix(".pdb").is_file():
            raise FileNotFoundError(f"PDB bulunamadı ({self.exe.with_suffix('.pdb')})")
        # DbgHelp uses this unique, nonzero value only as a session identifier.
        self.handle = wt.HANDLE(id(self))
        self.dbg = ct.WinDLL("dbghelp.dll", use_last_error=True)
        handle, string, dword = wt.HANDLE, wt.LPCWSTR, wt.DWORD
        signatures = {
            "SymSetOptions": (dword, dword),
            "SymInitializeW": (wt.BOOL, handle, string, wt.BOOL),
            "SymCleanup": (wt.BOOL, handle),
            "SymLoadModuleExW": (U64, handle, handle, string, string, U64, dword,
                                 ct.c_void_p, dword),
            "SymFromAddrW": (wt.BOOL, handle, U64, ct.POINTER(U64), ct.POINTER(SymbolInfo)),
            "SymGetLineFromAddrW64": (wt.BOOL, handle, U64, ct.POINTER(dword),
                                     ct.POINTER(LineInfo)),
        }
        for name, (result, *arguments) in signatures.items():
            function = getattr(self.dbg, name)
            function.restype = result
            function.argtypes = arguments

    def __enter__(self):
        self.dbg.SymSetOptions(SYMOPT_UNDNAME | SYMOPT_DEFERRED_LOADS | SYMOPT_LOAD_LINES
                               | SYMOPT_IGNORE_NT_SYMPATH | SYMOPT_EXACT_SYMBOLS)
        if not self.dbg.SymInitializeW(self.handle, str(self.exe.parent), False):
            raise ct.WinError(ct.get_last_error())
        self.base = self.dbg.SymLoadModuleExW(
            self.handle, None, str(self.exe), None, BASE, self.size, None, 0
        )
        if not self.base:
            error = ct.WinError(ct.get_last_error())
            self.dbg.SymCleanup(self.handle)
            raise error
        return self

    def __exit__(self, *_):
        self.dbg.SymCleanup(self.handle)

    def resolve(self, rva):
        if not 0 <= rva < self.size:
            return None
        buffer = ct.create_string_buffer(ct.sizeof(SymbolInfo) + MAX_NAME * ct.sizeof(ct.c_wchar))
        info = SymbolInfo.from_buffer(buffer)
        info.SizeOfStruct = ct.sizeof(SymbolInfo)
        info.MaxNameLen = MAX_NAME
        displacement = U64()
        if not self.dbg.SymFromAddrW(self.handle, self.base + rva,
                                     ct.byref(displacement), ct.byref(info)):
            return None
        name = ct.wstring_at(ct.addressof(info) + SymbolInfo.Name.offset, info.NameLen)
        line = LineInfo(SizeOfStruct=ct.sizeof(LineInfo))
        line_displacement = wt.DWORD()
        for address in dict.fromkeys((info.Address, self.base + rva)):
            if self.dbg.SymGetLineFromAddrW64(
                self.handle, address, ct.byref(line_displacement), ct.byref(line)
            ) and line.FileName:
                return Frame(name, info.Address, f"{line.FileName}:{line.LineNumber}")
        return Frame(name, info.Address)


@dataclass(frozen=True)
class Thread:
    name: str
    cpu_ms: int
    samples: int


def read_profile(path):
    threads, stacks = {}, []
    with path.open(encoding="utf-8-sig") as profile:
        for number, line in enumerate(profile, start=1):
            kind, _, data = line.rstrip("\r\n").partition(" ")
            try:
                if kind == "thread":
                    # The writer emits two adjacent spaces if the start address is empty.
                    tid, _start, cpu, samples, name = data.split(" ", 4)
                    threads[int(tid)] = Thread(name or f"Adsız ({tid})", int(cpu), int(samples))
                elif kind == "stack":
                    tid, phase, count, frames = data.split(" ", 3)
                    stacks.append((int(tid), int(phase), int(count), tuple(frames.rstrip(";").split(";"))))
                elif kind not in {"", "rounds", "phase"}:
                    raise ValueError(f"bilinmeyen kayıt türü {kind!r}")
            except ValueError as error:
                raise ValueError(f"{path}, satır {number} ({error})") from error
    return threads, stacks


def is_victory(raw):
    return raw.partition("+")[0].casefold() == "victory.exe"


def symbolize(raw, symbols):
    module, separator, rva = raw.partition("+")
    if is_victory(raw) and symbols and separator:
        value = symbols.resolve(int(rva, 16))
        if value is not None:
            return value
        return Frame(raw + "!?")
    return Frame(module.casefold() + "!?")


def shorten(name):
    depth = 0
    start = 0
    parts = []
    previous_end = 0
    for match in re.finditer(r"<|(?<!-)>", name):
        if match[0] == "<":
            if not depth:
                start = match.start()
            depth += 1
        elif depth:
            depth -= 1
            if not depth:
                parts.extend((name[previous_end:start], "<...>"))
                previous_end = match.end()
    return "".join(parts) + name[previous_end:]


def md(value):
    return re.sub(r"([\\|`<>*_\[\]&~])", r"\\\1", str(value))


def table(columns, rows):
    print("| " + " | ".join(columns) + " |")
    print("| " + " | ".join("---" for _ in columns) + " |")
    for row in rows:
        print("| " + " | ".join(md(value) for value in row) + " |")
    print()


def percent(count, total):
    return f"{count / total:.2%}" if total else "0.00%"


def report(threads, stacks, symbols, short, query):
    @cache
    def resolve(raw):
        return symbolize(raw, symbols)

    total = sum(count for _, _, count, _ in stacks)
    phases, group_phases = Counter(), defaultdict(Counter)
    exclusive, inclusive, callers = Counter(), Counter(), Counter()
    frame_info = {}
    matched = 0
    unresolved = 0
    for tid, phase, count, raw_frames in stacks:
        frames = [resolve(raw) for raw in raw_frames]
        keys = [frame.key for frame in frames]
        for frame in frames:
            if frame.location or frame.key not in frame_info:
                frame_info[frame.key] = frame
        thread_name = threads[tid].name if tid in threads else f"Adsız ({tid})"
        phases[phase] += count
        group_phases[thread_name][phase] += count

        first_victory = next((frame for frame in frames if frame.address is not None), None)
        leaf = frames[0]
        if not is_victory(raw_frames[0]):
            exclusive[leaf.key, first_victory.key if first_victory else None] += count
        else:
            exclusive[(first_victory or leaf).key, None] += count
        for key in dict.fromkeys(keys):
            inclusive[key] += count
        if any(is_victory(raw) and frame.address is None for raw, frame in zip(raw_frames, frames)):
            unresolved += count

        if query is not None:
            # Count a matching stack once, using the caller of the outermost match.
            for index in range(len(frames) - 1, -1, -1):
                frame = frames[index]
                if frame.address is not None and query.casefold() in frame.name.casefold():
                    matched += count
                    caller = keys[index + 1] if index + 1 < len(keys) else None
                    callers[caller] += count
                    break

    def frame_name(key):
        return shorten(key[0]) if short else key[0]

    print("## İş parçacıkları\n")
    table(("Ad", "CPU ms", "Örnek"), (
        (thread.name, thread.cpu_ms, thread.samples)
        for thread in sorted(threads.values(), key=lambda t: (-t.cpu_ms, t.name))
    ))
    print(f"Kaydedilmiş yığın örneği: {total}. İş parçacığı sayıları boş yığınları da içerir.\n")
    if unresolved:
        print(f"{unresolved} örnekte en az bir VIctory adresi çözülemedi.\n")
    print("Kare başına ms hesaplanamadı. Profil dosyası örnekleme aralığını, "
          "pencere süresini ve kare sayısını kaydetmiyor. Örnekler bekleme süresini de kapsıyor.\n")

    print("## Faz dağılımı\n")
    table(("Faz", "Örnek", "Pay"), (
        (phase, count, percent(count, total)) for phase, count in sorted(phases.items())
    ))
    print("### İş parçacığı adına göre\n")
    table(("İş parçacığı", "Faz", "Örnek", "İş parçacığındaki pay"), (
        (name, phase, count, percent(count, counts.total()))
        for name, counts in sorted(group_phases.items())
        for phase, count in sorted(counts.items())
    ))

    print("## En çok örnek alan 30 fonksiyon — DIŞLAYICI\n")
    rows = []
    for (key, caller), count in exclusive.most_common(30):
        name = frame_name(key)
        if caller:
            name += f" (çağıran: {frame_name(caller)})"
        rows.append((name, count, percent(count, total), frame_info[caller or key].location))
    table(("Fonksiyon", "Örnek", "Pay", "Kaynak"), rows)

    print("## En çok örnek alan 30 fonksiyon — KAPSAYICI\n")
    table(("Fonksiyon", "Örnek", "Pay", "Kaynak"), (
        (frame_name(key), count, percent(count, total), frame_info[key].location)
        for key, count in inclusive.most_common(30)
    ))

    if query is not None:
        print("## Fonksiyon araması\n")
        print(f"**{md(query)}** — {matched}/{total} örnek ({percent(matched, total)}).\n")
        print("### En sık 5 çağıran\n")
        table(("Çağıran", "Örnek", "Tüm örneklerdeki pay"), (
            (frame_name(key) if key else "Yığında görünmüyor", count, percent(count, total))
            for key, count in callers.most_common(5)
        ))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profil", type=Path, metavar="profil.txt")
    parser.add_argument("--exe", type=Path, default=DEFAULT_EXE, help="PDB ile eşleşen VIctory exe")
    parser.add_argument("--kisa", action="store_true", help="generic parametreleri <...> olarak göster")
    parser.add_argument("--ara", metavar="ALT_DİZE", help="çözülen fonksiyon adlarında ara")
    parser.add_argument("--thread", metavar="ALT_DİZE", help="iş parçacığı adına göre filtrele")
    args = parser.parse_args()
    try:
        threads, stacks = read_profile(args.profil)
        if args.thread is not None:
            search = args.thread.casefold()
            threads = {tid: t for tid, t in threads.items() if search in t.name.casefold()}
            stacks = [stack for stack in stacks if stack[0] in threads]
        needs_symbols = any(is_victory(raw) for _, _, _, frames in stacks for raw in frames)
        with Symbols(args.exe) if needs_symbols else nullcontext() as symbols:
            report(threads, stacks, symbols, args.kisa, args.ara)
    except (OSError, ValueError, struct.error) as error:
        parser.exit(1, f"Hata: {error}\n")


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    main()
