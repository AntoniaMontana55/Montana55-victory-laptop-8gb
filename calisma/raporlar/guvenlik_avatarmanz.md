# Güvenlik İncelemesi: `avatarmanz/VIctory-7900XTX` (`0a435665`)

- **İncelenen hedef:** `avatarmanz/VIctory-7900XTX` — sabit commit `0a43566532c57b34a5397d6fdc91e419de32e8e9` (`2026-11-25T22:14:34Z`, inceleme anında `main` HEAD)
- **Yöntem:** Salt okunur GitHub REST + GraphQL API incelemesi (231/231 kaynak/config/betik dosyası tarandı; klonlama, derleme veya betik çalıştırma yapılmadı).
- **Karar:** **GÜVENLİ** (güvenlik açısından temiz; ancak NVIDIA'da teknik/uyumluluk uyarıları var — bkz. Bölüm 4 ve 5).

---

## 0. Lisans Durumu

- GitHub API bu repoyu ve tabanı olan `fxceless/victory-fork`'u **`GPL-3.0`** olarak döndürüyor (`LICENSE` dosyası `0a435665` ağacında mevcut, 35.149 bayt, GNU GPL v3).
- `Cargo.toml:11` → `license = "GPL-3.0-or-later"`.
- `README.md:112-115` → `GPL-3.0-or-later, as VIctory`, tek istisna için `docs/asset-policy.md`'ye atıf var.
- `docs/asset-policy.md:26-35` → `crates/victory-formats/src/bcpack-tables.bin` (77.982 B) BCPack sabit tablolarıdır ve depo lisansının (GPL) **dışında** bırakılmıştır (upstream Marvisak/fxceless kararı; dosya SHA'sı `fxceless/victory-fork` ile birebir aynı).
- `README.md:108-109` → `ivall/xcl` (GPL-2.0-only) yalnızca davranış analizi için okunduğu, kod kopyalanmadığı belirtiliyor.

---

## 1. Derleme Yüzeyi (`build.rs`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`)

### `build.rs` dosyaları (depodaki tüm 3 adet)
1. **`crates/victory-gpu/build.rs:1-45`** (bu fork'ta eklendi): Yalnız `victory-shader/src`, `victory-shader/Cargo.toml`, `src/shader_cache.rs`, `Cargo.lock` ve `rust-toolchain.toml` dosyalarını okuyup `blake3` ile özetler ve `cargo:rustc-env=VICTORY_TRANSLATOR=...` yazar. Ağ, süreç, dosya yazma yok.
2. **`crates/victory-shader/build.rs:1-65`**: Yalnız `crates/victory-shader/src/*.rs` dosyalarını FNV-1a ile özetler ve `"VICTORY_..."` anahtarlarını listeler (`VICTORY_SHADER_SOURCE_HASH`, `VICTORY_SHADER_SWITCHES`). Temiz.
3. **`tests/fixtures/hle-probe/build.rs:1-4`**: Yalnız test fixture'ı için `/DELAYLOAD:victory-probe-delayed.dll` ve `delayimp` bağlayıcı yönergelerini yazar. Temiz.

### `Cargo.toml`, `[profile]` ve `.cargo/config.toml`
- **`.cargo/config.toml:1-2`**: Yalnız `[alias] xtask = "run --quiet --package xtask --"`. `rustflags`, özel `linker` veya `runner` **yok**.
- **`rust-toolchain.toml:1-5`**: `channel = "1.98.1"`, `x86_64-pc-windows-msvc`, `rustfmt`, `clippy`.
- **`Cargo.toml:67-79` (`[profile]`)**: `dev.package."*"` (`opt-level = 2`), `release` (`debug = "line-tables-only"`), `diag` (`inherits = "release"`). Alışılmadık veya güvensiz bayrak yok.
- **`Cargo.toml:53-65` (`[workspace.lints.clippy]`)**: Fork'ta yeni eklenen GPU kodları için `undocumented_unsafe_blocks = "allow"` dahil birkaç clippy uyarısı gevşetilmiş (güvenlik açığı değil, kod hijyeni notu).
- **Crate `Cargo.toml` farkları (`fxceless` tabanına göre)**:
  - `crates/victory-gpu/Cargo.toml`: `blake3` `build-dependencies` eklendi; `windows-sys` içine `Win32_System_Registry` (salt okunur `tdr.rs` için), `Win32_UI_Input_KeyboardAndMouse` ve `Win32_UI_WindowsAndMessaging` (F3 panel/girdi için) eklendi; `Win32_System_EventLog` çıkarıldı.
  - `crates/victory-hle/Cargo.toml`: `Win32_Security_Cryptography` (`rpcrt4.rs` rastgele UUID için) ve `Win32_System_IO` eklendi.
  - `crates/victory-shader/Cargo.toml`: `[[bench]] name = "guard_runs"` eklendi.

### `Cargo.lock`
- **Git veya harici yol (`path`) bağımlılığı:** **YOK** (tüm dış crate'ler `registry+https://github.com/rust-lang/crates.io-index` kaynağından).
- **`fxceless/victory-fork` ile fark:** **0 eklenen, 0 çıkarılan, 0 sürüm değişen paket.** `Cargo.lock` paket kümesi `fxceless` ile birebir aynı.

---

## 2. Betikler (`scripts/`, kök ve hook'lar)

Depoda `tools/` klasörü veya `.bat`/`.cmd`/`.py` betiği **yok**. Yalnız iki betik mevcut:

1. **`scripts/play.ps1:1-38`** (README'deki "start script"):
   - **Ne yapar:** RX 7900 XTX ayarlarını (`VICTORY_TEXTURE_NO_WAIT=1`, `VICTORY_RELAY=2`, `VICTORY_WIDE_BACK=1`, `VICTORY_CARD_STEPS=2`, `VICTORY_THIN=1`, `VICTORY_JOIN_PARTS=8`, `VICTORY_UPLOAD_AHEAD=1`, `VICTORY_CARD_AWAKE=1`, `VICTORY_AMBIENT=0.8` vb.) geçerli oturumun ortam değişkenlerine yazar (`play.ps1:15-27,35`), `VICTORY_CONFIG`/`GAME_DIR`/`CACHE_DIR`/`DATA_DIR` dışındaki eski `Env:VICTORY_*` değişkenlerini temizler (`play.ps1:33-34`) ve `target\<Build>\victory.exe run <Exe> --execute` çalıştırır (`play.ps1:38`).
   - **Risk kontrolü:** İndirme yok, ağ erişimi yok, kayıt defteri/güvenlik duvarı/Defender dokunuşu yok, dosya silme yok, yönetici yetkisi istemiyor (aksine `play.ps1:7` normal pencerede, yönetici olmadan çalıştırılmasını öneriyor).
2. **`.githooks/pre-commit:1-3`**: Yalnız `exec cargo xtask policy --staged` çağırır (depoya oyun dosyası/ikili sızmasını engellemek için).

---

## 3. Kod Güvenlik Taraması (231 Dosya, `0a435665`)

### Ağ (`std::net`, Winsock, WinHTTP, URLDownload)
- **Host kaynaklı dış ağ bağlantısı / telemetri / indirme YOK.**
- `crates/victory-hle/src/modules/net.rs:3-12,725-877`: Oyunun `ws2_32.dll` / `winhttp.dll` çağrılarını **engelleyen** HLE yalıtım katmanı (loopback dışı `WinHttpConnect` ve DNS çözümlemeleri reddedilir; bu fork'ta `blocked_lookup` sonucu temizleyecek şekilde iyileştirilmiş). `net.rs:954` (`TcpListener::bind("127.0.0.1:0")`), `media.rs:513`, `xnetworking.rs:239,253`, `xuser.rs:403` yalnızca birim testleridir.
- Yeni eklenen `crates/victory-hle/src/modules/rpcrt4.rs:1-40`: Oyunun gerçek `rpcrt4.dll` üzerinden `ws2_32` ağ sızıntısı yapmasını önlemek için GUID çağrılarını yerelde karşılar (güvenlik lehine).

### Süreç Başlatma (`Command::new`, `CreateProcess`, `ShellExecute`)
- `CreateProcess`, `ShellExecute`, `WinExec`, `powershell`, `cmd.exe` **YOK**.
- Tüm `Command::new` çağrıları meşru ve sınırlı:
  - `apps/victory/src/warm.rs:140`: `victory warm --these ...` için kendi ikilisini (`current_exe()`) `IDLE_PRIORITY_CLASS` ile başlatır (pipeline derleme çökmelerini izole etmek için).
  - `crates/victory-hle/src/modules/lifecycle.rs:345` ve `audio/capture.rs:71`: Kendi ikilisini (`current_exe()`) yeniden başlatma / ses yakalama (yalnız `VICTORY_AUDIO_CAPTURE` set ise).
  - `crates/victory-gpu/src/pipeline/host.rs:1101`, `victory-shader/src/emit/diff_tests.rs:674`, `emit/mod.rs:975`: Yalnız test/tanılama için Windows SDK `dxc.exe`.
  - `xtask/src/main.rs:36`, `xtask/src/policy.rs:60,94`: `git` çağrıları.

### Kayıt Defteri (Registry)
- **Kayıt defterine YAZMA/DEĞİŞTİRME YOK.**
- `crates/victory-gpu/src/tdr.rs:11-76`: Yalnız **salt okunur** `RegGetValueW` ile `HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers` altından `TdrLevel`, `TdrDelay`, `TdrDdiDelay` okur; `TdrLevel` 0 veya 1 ise kullanıcıya uyarı loglar.
- `crates/victory-hle/src/modules/xsystem.rs:199-230`: Oyunun konsol modu sorguları için salt okunur HLE `RegOpenKeyW`/`RegQueryValueExW` kancaları.

### Dosya Silme / Yazma ve DLL Yükleme
- Varsayılan yazma hedefleri `%LOCALAPPDATA%\victory\cache` ve `%LOCALAPPDATA%\victory\data` (`crates/victory-core/src/config.rs:137-145`); `ensure_outside_repo` ile depo içi yazma engellenir (`config.rs:132`).
- `remove_file` / `remove_dir_all` çağrıları yalnız geçici/önbellek/kayıt-sürümü temizliği veya testlerdir (`pipeline/known.rs:478` test, `pipeline/library.rs:610,696`, `shader_cache.rs:173`, `loader/src/publish.rs:31`, `pipeline/cache.rs:68`, `xgamesave.rs`).
- Sistem DLL yüklemeleri (`root.rs:67`, `timeline/report.rs:109`, `hle/src/host.rs:154`) `LOAD_LIBRARY_SEARCH_SYSTEM32` bayrağıyla DLL hijacking'e karşı korumalıdır.
- **Not (`crates/victory-loader/src/write_sites.rs:1-45`):** `VICTORY_WRITE_SITES=<dosya>` verilirse yüklenen oyunun bellek içi bazı yazma komutlarını yamalayarak sayfa hatasını azaltır; değişken verilmezse **kapalıdır** ve yalnızca oyun sürecinin kendi belleğine dokunur.

### Gizlenmiş Blob / İkili Dosyalar
- Depoda `.dll`, `.exe`, `.so`, `.dylib` **YOK**.
- Tek ikili dosya `crates/victory-formats/src/bcpack-tables.bin` (77.982 B, `bcpack.rs:124` ile `include_bytes!`): SHA `1e548501764588b06e2ed47aa7fed263cd797f3b`, `fxceless/victory-fork` ile **%100 aynı**.
- Gizlenmiş base64/hex blob yok (`xtask/src/policy.rs:174` zaten uzun hex/base64 dizilerini engelleyen denetim içeriyor).

---

## 4. `fxceless/victory-fork` Farkı ve AMD/NVIDIA Analizi

### Son 2–3 Günlük Commit Özeti (`fxceless` tabanı üzerine)
- `fxceless/victory-fork` (`9c184b35`) ile karşılaştırmada **73 yeni dosya, 109 değiştirilen dosya** ve ~208 `local` commit (100+'ü `2026-11-25` tarihli).
- Başlıca eklemeler:
  - İnce komut listesi + 8 iş parçacıklı çeviri (`VICTORY_THIN`), asenkron gönderim/birleştirme (`VICTORY_ASYNC_SUBMIT`, `VICTORY_JOIN_PARTS`).
  - AVX2 çok iş parçacıklı bellek karşılaştırması, GPU tarafında adım yürütme (`VICTORY_CARD_STEPS`), `VICTORY_RELAY`/`VICTORY_LAND`.
  - Doku ön-yükleme kuyruğu ve yükleme sonrası görünürlük (`VICTORY_UPLOAD_AHEAD`, `VICTORY_SHOW_AFTER`), su/doku geri okuma (`fxceless` PR #999 / Geckomacabre, burada varsayılan açık).
  - Önceden pipeline ısıtma (`apps/victory/src/warm.rs` → `victory warm`) ve kare kayıt/tekrar oynatma (`apps/victory/src/replay.rs`, `crates/victory-gpu/src/replay/*`).
  - Belobug (romainhedouin), DexNom ve ZJOU fork'larından cherry-pick'ler (render target/pipeline önbelleği, RT/BVH düzeltmeleri).

### AMD'ye Özgü Varsayımlar ve NVIDIA'da Sorun Çıkarabilecek Yerler
1. **KRİTİK (Replay modu) — `apps/victory/src/replay.rs:468-469`**:
   `victory_shader::emit::set_host_max_wave(64);` ve `victory_shader::emit::set_host_rdna(true);` **sabit kodlu (hardcoded)**. NVIDIA'da (maks. wave 32) `victory replay` shader çevirisi çalıştırılırsa `WaveSize(64)` / RDNA float kurallarıyla uyumsuz DXIL üretir. *(Normal oyun akışında `crates/victory-gpu/src/host.rs:915-926` `WaveLaneCountMax` ve `VendorId == 0x1002` sorguladığı için NVIDIA'da doğru şekilde 32 / `rdna=false` seçer.)*
2. **DRED/Slot tanılama — `crates/victory-gpu/src/book.rs:20-21,413-417`**:
   Descriptor bit çözümlemesi AMD RDNA 2/3 düzenine göre yazılmış; NVIDIA'da çökmez ama adres çözemez (`VICTORY_SLOT_CHECK` / DRED slot raporları boş kalır).
3. **Indirect dispatch sınırı — `crates/victory-gpu/src/indirect.rs:279-293`**:
   AMD'de (`VendorId == 0x1002`) `1 << 20` gruba izin verirken NVIDIA'da `65_535` ile sınırlar. Güvenli biçimde `amd(&device)` ile korunmuş; ancak çok büyük indirect dispatch'lerde NVIDIA'da kesilme/görsel artefakt ihtimali olabilir.
4. **Saat hızı tutma shader'ı — `crates/victory-gpu/src/host/awake.rs:25-35`**:
   `VICTORY_CARD_AWAKE=1` (yalnız `play.ps1` açar, varsayılan kapalı) her ~2 ms'de 64 thread'lik boş döngü compute shader'ı gönderir. RX 7900 XTX için ayarlanmış; NVIDIA'da gereksiz yüksek güç/GPU kullanımına yol açabilir.
5. **NVIDIA'da test edilmemiş agresif ayarlar — `README.md:14`, `scripts/play.ps1:15-25`**:
   Fork **yalnız tek bir RX 7900 XTX makinede** test edilmiş. NVIDIA'da 1.000.000 descriptor heap sınırı (`descriptor/host.rs:1362,1410,2962`, `draw.rs:3163`) ve `f32tof16`/stencil farkları nedeniyle `play.ps1` bayrakları yerine önce varsayılan ayarlarla veya `-Safe` eşdeğeriyle deneme yapılmalı.
6. **Bilgi — `crates/victory-gpu/src/dxgi.rs:142,157`**:
   Emüle edilen oyuna konsol GPU'su olarak `AMD (0x1002)` bildirir; bu kasıtlı Xbox Series taklididir, NVIDIA host'ta sorun çıkarmaz.

---

## 5. Karar ve Doğrulanacak Satırlar

- **Güvenlik Kararı:** **GÜVENLİ** — Kötü amaçlı kod, harici ağ/telemetri, yetkisiz kayıt defteri yazma, tehlikeli dosya silme, harici git/path bağımlılığı veya yeni/gizlenmiş ikili bulunmadı.
- **Derleme/Çalıştırma öncesi doğrulanacak noktalar (NVIDIA hedefi için):**
  - `apps/victory/src/replay.rs:468-469` (replay kullanılacaksa `set_host_max_wave`/`set_host_rdna` sabit kodlaması).
  - `crates/victory-gpu/src/host.rs:915-926` (çalışma zamanı logunda `host wave lanes` değerinin NVIDIA için `max = 32` ve `rdna = false` geldiği).
  - `scripts/play.ps1` çalıştırılmadan önce `VICTORY_CARD_AWAKE` ve agresif bayrakların NVIDIA'da kademeli denenmesi.
