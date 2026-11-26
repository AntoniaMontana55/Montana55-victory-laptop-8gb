# Güvenlik İncelemesi: `fxceless/victory-fork`

**Tarih:** 2026-11-25  
**İncelenen commit:** `2119b0e` (`origin/main`, upstream `Marvisak/VIctory@ffe310e` üzerine 12 commit)  
**Kapsam:** Salt okunur kaynak kod, git geçmişi, CI, PR/issue ve yorum taraması (`C:\CLLL\.calisma\victory-fork` + GitHub API).

---

## 1. İddia nerede, ne diyor?

- **`fxceless/victory-fork` içinde (94 issue, 293 PR, 503 yorumun tamamı tarandı):** "Veri gönderiyor / telemetri / malware / spyware / exfil / webhook" yönünde **hiçbir iddia veya kanıt bulunmadı**. (Discussions ve Wiki kapalı.)
  - Sadece `telemetry` kelimesi [#15](https://github.com/fxceless/victory-fork/issues/15) ve [#321](https://github.com/fxceless/victory-fork/issues/321) numaralı issue'larda geçiyor; ikisi de yerel FPS/VRAM/frame-time ölçüm sayaçları anlamında kullanılmış, ağ aktarımıyla ilgisiz.
  - [#184 yorumunda](https://github.com/fxceless/victory-fork/issues/184#issuecomment-7148271684) geçen `socket` kelimesi Wine `wineserver` yerel soketine atıf yapıyor.
- **`Marvisak/VIctory` içinde:** `fxceless/victory-fork` (eski adı `victory-gta6`) hakkında veri sızdırma/zararlı iddiası **bulunmadı**. [#1645](https://github.com/Marvisak/VIctory/issues/1645) ve [#1820](https://github.com/Marvisak/VIctory/issues/1820) teknik birleştirme/test raporlarıdır; [#1820 yorumunda](https://github.com/Marvisak/VIctory/issues/1820#issuecomment-7147108818) `DesertGamer` yalnızca kendi topluluk birleştirme fork'unun PR'ları şüpheli içeriğe karşı taradığını belirtmiş, `fxceless` için bir suçlama yöneltmemiştir.
- **Genel GitHub ve web araması:** `fxceless/victory-fork` kaynak koduna yönelik doğrulanabilir bir veri gönderimi raporu bulunmadı. Kullanıcının duyduğu iddianın tam kaynağı **doğrulanmadı** (muhtemelen repo dışı Discord/sosyal medya söylentisi veya proje adını kullanan sahte derlenmiş `.exe` dağıtımlarıyla karıştırma).

---

## 2. Ana dalda (`main`) ağ/veri gönderen kod var mı?

**Hayır.** `ffe310e..2119b0e` arasındaki 12 commit'in tamamı (92 dosya) ve tüm takip edilen kaynak dosyaları tarandı:

- **Bağımlılıklar ([Cargo.toml:14-45](file:///C:/CLLL/.calisma/victory-fork/Cargo.toml#L14-L45)):** `reqwest`, `hyper`, `ureq`, `tokio` vb. hiçbir ağ/HTTP crate'i yok. Fork yeni dış bağımlılık eklememiş; yalnızca `crates/victory-loader/Cargo.toml` içine `Win32_System_Performance` eklenmiş.
- **Ağ katmanı (çevrimdışı sandbox korunmuş):**
  - [crates/victory-hle/src/modules/net.rs:1-17](file:///C:/CLLL/.calisma/victory-fork/crates/victory-hle/src/modules/net.rs#L1-L17): Fork tarafından **değiştirilmemiş**. Oyunun loopback (`127.0.0.0/8`, `::1`) dışındaki tüm Winsock/WinHTTP bağlantılarını (`WSAENETUNREACH`, `ERROR_WINHTTP_CANNOT_CONNECT`) ve proxy otomatik algılamayı (`ERROR_WINHTTP_AUTODETECTION_FAILED`) engelliyor.
  - [crates/victory-hle/src/modules/kernel.rs:333-337](file:///C:/CLLL/.calisma/victory-fork/crates/victory-hle/src/modules/kernel.rs#L333-L337): `wininet.dll`, `urlmon.dll` gibi diğer ağ DLL'lerinin yüklenmesini reddediyor.
  - `crates/victory-hle/src/modules/xnetworking.rs`: Fork farkı yalnızca 1 satır yorum değişikliği (`the title` → `the title`).
- **Process / Registry / URL taraması:**
  - Fork diff'inde registry çağrısı veya dış URL yok (yalnızca `README.md` içindeki GitHub/Claude linkleri).
  - Kod içindeki tek URL'ler `LICENSE` ve birim test sahte URL'leri (`media.rs:483`, `xnetworking.rs:198`, `xuser.rs:403`).
  - Fork'un eklediği tek `Command::new` çağrısı [crates/victory-gpu/src/pipeline/host.rs:881](file:///C:/CLLL/.calisma/victory-fork/crates/victory-gpu/src/pipeline/host.rs#L881) satırında olup yalnızca `#[test]` altında yerel Windows SDK `dxc.exe` çalıştırıyor.

---

## 3. CI (`.github/workflows/ci.yml`)

[ci.yml:1-33](file:///C:/CLLL/.calisma/victory-fork/.github/workflows/ci.yml#L1-L33) temiz:
- Tetikleyiciler: `pull_request` ve `push` (`main`). `pull_request_target` **yok**.
- İzinler: `permissions: contents: read`.
- Secret kullanımı veya dış URL'ye veri gönderen adım **yok**; yalnızca `actions/checkout@v5`, `rustup toolchain install`, `Swatinem/rust-cache@v2`, `cargo clippy --locked` ve `cargo test --workspace --locked` çalıştırıyor.

---

## 4. PR'lardaki riskler

- **Birleştirilmiş PR'lar (#1, #11, #14):** Diff taramasında sıfır ağ/HTTP/socket/komut çalıştırma kodu.
- **Açık PR'lar (en çok tartışılan 25 PR'ın diff'i + 279 açık PR'ın başlık/gövde/URL taraması):**
  - Veri sızdıran veya zararlı PR görülmedi; ancak **279 açık PR'ın tamamının satır satır diff'i doğrulanmadı** (birleşmemiş rastgele PR'ları derlemeden önce diff incelenmeli).
  - **Dış ağ bağlantısı veya betik/ikili ekleyen açık PR'lar (birleşmemiş):**
    - **#22, #69, #286 (Agility SDK):** Derleme/kurulum sırasında `curl` ile `https://api.nuget.org/v3-flatcontainer/microsoft.direct3d.d3d12/...` adresinden Microsoft D3D12 Agility SDK indiriyor (SHA-512 hash doğrulamalı; veri gönderimi değil ama derleme anında dış indirme yapar).
    - **#64 (Linux/Nix/Proton):** `scripts/build-linux.sh`, `scripts/run-linux.sh`, `shell.nix` ve `ci.yml` değişikliği içeriyor.
    - **#44 (Kaynak loglayıcı):** `tools/watch-resources.ps1` (yerel CPU/GPU/bellek ve grafik DLL sürümlerini loglar; ağ çıkışı görülmedi) + `ci.yml`.
    - **#86 (PC FSR):** `VICTORY_FFX_DIR` altından harici `amd_fidelityfx_*_dx12.dll` yükler (kendi indirmez).

---

## 5. Dışarıdan dağıtılan dosyalar

- **GitHub Releases:** Hem `fxceless/victory-fork` hem `Marvisak/VIctory` için **0 release** (resmi derlenmiş `.exe` yok).
- **Üçüncü taraf indirme linkleri:** `README.md` ve `fxceless/victory-fork` içindeki tüm issue/PR/yorumlarda `mega.nz`, `mediafire`, `drive.google`, `cdn.discordapp` veya harici `.exe` dağıtımı **bulunmadı**.
- **Not:** Repo dışında bu projelerin adını kullanan sahte "hazır `.exe`" siteleri/Discord paylaşımları zararlı yazılım için bilinen risk vektörüdür; yalnızca kaynaktan derleme yapılmalıdır.

---

**KARAR: GÜVENLİ** — `fxceless/victory-fork` ana dalında (`2119b0e`), CI yapılandırmasında ve birleştirilmiş PR'larda veri gönderen veya zararlı herhangi bir ağ kodu bulunmadı; üstelik oyunun loopback dışı ağ erişimi HLE katmanında aktif olarak engellenmeye devam ediyor.
