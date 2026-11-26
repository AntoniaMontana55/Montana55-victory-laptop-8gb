# Güvenlik İncelemesi: ZSlayerHQ/victory-world-runtime & victory-world

Tarih: 2026-11-25  
Kapsam: Salt okunur kod ve betik incelemesi (derleme/çalıştırma yapılmadı).

---

## 1. Özet Sonuçlar

| Hedef | Commit / Aralık | Sonuç |
| --- | --- | --- |
| `ZSlayerHQ/victory-world-runtime` (`next`) | `c96b4f2` → `99f8834aa16b98d7f3943b8de262a8d4857bda7b` | **GÜVENLİ** |
| `ZSlayerHQ/victory-world` (`harness/`) | `566d5be7222255800ef4ea004d96473ca6ed5295` | **GÜVENLİ** *(yerel/güvenilir kuyruk dosyası varsayımıyla; 2 dikkat noktası aşağıda)* |

---

## 2. Kısım 1: `victory-world-runtime` Fark İncelemesi (`c96b4f2..99f8834`)

- **Kapsam:** 36 dosya (+5548 / -430). Değişikliklerin tamamı GPU/D3D12 (`crates/victory-gpu`), shader çeviricisi (`crates/victory-shader`), HLE (`dstorage.rs`, `files.rs`, `kernel.rs`, `xmem.rs`) ve CLI (`apps/victory`: `warm` komutu + çıkış kodu loglaması).
- **Kırmızı bayrak taraması:**
  - Yeni/değişen `build.rs`: **Yok.**
  - Ağ API'si / `crates/*/src/**/net.rs` değişikliği: **Yok.** (`http://`/`https://` yalnızca `README.md` bağlantılarında.)
  - Registry yazma, kullanıcı/tarayıcı/token okuma, telemetri, gömülü ikili blob, CI (`.github/`) değişikliği: **Yok.**
  - `Command::new`: Üretim kodunda **yok**; yalnızca Windows birim testinde mevcut (aşağıda).

### Yeni Bağımlılık Listesi
- **Yeni dış crate yok.**
- `apps/victory/Cargo.toml:22` ve `Cargo.lock:639`: Zaten workspace'te bulunan `tracing` (`tracing.workspace = true`) `victory` ikili paketine eklendi. Yeni `git=` veya özel registry kaynağı yok.

### İncelenen Dikkat Çekici Noktalar (Temiz)
- `crates/victory-hle/src/modules/kernel.rs:704-710` (`#[cfg(windows)] #[test]`):
  ```rust
  let exe = std::env::current_exe().unwrap();
  let mut child = std::process::Command::new(exe)
      .args(["--exact", "modules::kernel::tests::a_title_exit_process_ends_as_windows_does_with_its_code", "--nocapture"])
  ```
  Yalnızca `cargo test` sırasında `ExitProcess` HLE davranışını sınamak için test binary'sini yeniden başlatır.
- `crates/victory-gpu/src/watch.rs` (`VirtualProtect`, testlerde `CreateFileMappingW`/`MapViewOfFile`): Çoklu görünüm (multi-view) bellek yazma korumasının parçası; yalnızca oyun belleği üzerinde çalışır.
- `crates/victory-hle/src/modules/files.rs:547-572` (`fread`) ve `dstorage.rs:866-884`: Kernel yazmalarının guard sayfasında hata vermesini önlemek için ara tampona (bounce buffer) okuyup kullanıcı modunda kopyalar; ek dosya yolu veya ağ erişimi yoktur.
- `crates/victory-gpu/src/shader_cache.rs:59-64`: Cache anahtarı için kendi Rust kaynak dosyalarını `include_str!` ile hash'ler; dış dosya okumaz.

---

## 3. Kısım 2: `victory-world/harness/` İncelemesi (`566d5be`)

- **İndirme / Ağa veri gönderme:** Hiçbir `.mjs`, `.ps1` veya `.sh` dosyasında `curl`, `wget`, `Invoke-WebRequest`, `fetch`, HTTP/HTTPS isteği veya soket kullanımı **yok**.
- **Registry / Yönetici yetkisi:** Registry yazma **yok**. Betikler kendileri UAC/yönetici yetkisi istemez; yalnızca `--presentmon` açılırsa Intel PresentMon ETW oturumu için Windows'ta Yönetici veya *Performance Log Users* yetkisi gerekir.

### Şüpheli / Dikkat Gerektiren Yerler
1. **`harness/run-queue.sh:349-351`** — Kuyruk satırını `eval` ile ayrıştırma (Command Injection riski, güvenilmeyen kuyruk dosyası verilirse):
   ```bash
   set -f
   if eval "TOKENS=($clean)" 2>/dev/null; then
   ```
   Yalnızca kendi oluşturduğunuz güvenilir kuyruk dosyalarıyla çalıştırılmalı.
2. **`harness/set-volume.ps1:10-156`** — Çalışma zamanında C# derleme (`Add-Type`) ile CoreAudio COM erişimi:
   ```powershell
   Add-Type -TypeDefinition @'
   ...
   volume.SetMasterVolume(level, ref none);
   ```
   Yalnızca `run-process.ps1 -Strict` ile eşleşen bu koşunun `victory.exe` PID'inin ses seviyesini varsayılan `%4`'e çeker; başka süreçlere dokunmaz.
3. **`harness/run-world.sh:129,147,152,207`**, **`harness/record-screen.mjs:59`**, **`harness/heavy-watch.mjs:599`** — `-ExecutionPolicy Bypass`:
   ```bash
   powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$HERE/raise-priority.ps1"
   ```
   Yalnızca aynı dizindeki (`$HERE`/`$PSScriptRoot`) yerel `.ps1` betiklerini çağırır.
4. **Süreç sonlandırma ve dosya silme:**
   - `harness/heavy-watch.mjs:621`: `taskkill /F /PID <pid>` (yalnız `run-process.ps1 -Strict` PID'i).
   - `harness/run-queue.sh:644,658`: Zaman aşımında `Stop-Process -Force` (zaman damgası + komut satırı filtreli `victory.exe`) ve `kill_tree`.
   - `harness/snapshot-build.sh:398`: `--prune N` bayrağında `rm -rf -- "$dir"` (yalnız `$BUILDS` altındaki `snapshot.txt` içeren klasörler).

### Betik Özeti
- `raise-priority.ps1`: `victory.exe` önceliğini `AboveNormal` yapar.
- `set-volume.ps1`: Yalnız bu koşunun `victory.exe` ses oturumunu `%4` (veya `--volume N`) yapar.
- `driver-events.ps1`: Windows `System` günlüğünden yalnızca `nvlddmkm` (NVIDIA) olaylarını okur (salt okunur).
- `window-handle.ps1`: `run-process.ps1 -Strict` ile pencere handle'ını (`MainWindowHandle`) okur; hiçbir şeyi değiştirmez.
- `run-process.ps1`: `Get-CimInstance Win32_Process` ile `victory.exe` PID'lerini süzer.
- `record-screen.mjs`: `ffmpeg` (`gfxcapture` filtresi) ile yalnız oyun penceresini 2 fps `screen.mp4` kaydeder, `ffprobe` ile kare sayar.
- `presentmon.mjs`: Logda ağır sahne (`draws + dropped >= 1000`) görünce PresentMon'u başlatır ve CSV özetler.
- `run-world.sh` / `world-run.sh`: Kilit (`run.lock`), `obs64.exe` koruması (`tasklist`), ortam değişkenleri ve tüm yardımcı betiklerin orkestrasyonu.
- `snapshot-build.sh`: Temiz git worktree'den `cargo build --locked` alıp `victory.exe`, `*.dll`, `victory.pdb` kopyalar.
- Diğer `.mjs` dosyaları (`build-line`, `clean-failures`, `compare-runs`, `contact-sheets`, `experiment-table`, `name-hashes`, `parse-run`, `screen-timeline`, `summarize-run`): Salt yerel log/CSV/video okuma ve raporlama (`git log`, `ffmpeg`, `ffprobe`).

### Harness'ın Dış Araç Gereksinimleri (ve Nereden Çağrıldıkları)
| Araç | Nereden Çağrılıyor | Kullanıldığı Dosyalar |
| --- | --- | --- |
| `bash`, `node`, `git`, `cargo` | `PATH` | `run-world.sh`, `world-run.sh`, `run-queue.sh`, `snapshot-build.sh`, `build-line.mjs` |
| `powershell.exe`, `tasklist`, `taskkill` | Windows `PATH` (`System32`) | `run-world.sh`, `world-run.sh`, `run-queue.sh`, `record-screen.mjs`, `heavy-watch.mjs` |
| `ffmpeg` (`gfxcapture` destekli), `ffprobe` | `PATH` | `record-screen.mjs`, `contact-sheets.mjs`, `screen-timeline.mjs`, `summarize-run.mjs` |
| `PresentMon-2.6.0-x64.exe` *(opsiyonel, `--presentmon`)* | `$PRESENTMON_EXE` veya `<project>/tools/presentmon/PresentMon-2.6.0-x64.exe` | `presentmon.mjs` |
| NVIDIA Nsight Aftermath DLL *(opsiyonel, `--aftermath`)* | `$VICTORY_AFTERMATH_DLL` veya `<root>/tools/nsight-aftermath-sdk-2026.3/lib/x64` | `run-world.sh` |

---

## 4. Claude'un Elle Bakması Gereken 4 Yer

1. **`harness/run-queue.sh:349-351`** — `eval "TOKENS=($clean)"` satırı; kuyruk dosyası dış kaynaktan gelmeyecekse kabul edilebilir, istenirse `read -ra` ile güvenli hale getirilebilir.
2. **`harness/set-volume.ps1:10-156`** — `Add-Type` C# COM kodunun (CoreAudio GUID'leri ve `ISimpleAudioVolume::SetMasterVolume`) gerçekten yalnız hedef PID'e dokunduğunun hızlı göz kontrolü.
3. **`crates/victory-hle/src/modules/files.rs:547-572`** ve **`dstorage.rs:866-884`** — Yeni `fread` ve `DirectStorage` bounce-buffer (`copy_nonoverlapping`) mantığında boyut/taşma (integer overflow veya kısmi okuma) kenar durumları.
4. **`crates/victory-gpu/src/watch.rs`** — `VirtualProtect(..., PAGE_READONLY, ...)` ile eklenen çoklu görünüm (`VICTORY_VIEW_GUARD`) mekanizmasının başka bellek bölgelerini yanlışlıkla salt-okunur bırakıp bırakmadığı.
