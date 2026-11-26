# Güvenlik İncelemesi (2): ZSlayerHQ/victory-world-runtime & victory-world

Tarih: 2026-11-25  
Kapsam: Salt okunur kod, betik ve CI iş akışı incelemesi (derleme/çalıştırma yapılmadı).  
Önceki inceleme: [guvenlik_zslayer.md](file:///C:/CLLL/.calisma/repo_durum/guvenlik_zslayer.md) (`99f8834` ve `566d5be` GÜVENLİ).

---

## 1. Özet Sonuçlar

| Hedef | Commit Aralığı | Onaylanan Tam SHA | Karar |
| --- | --- | --- | --- |
| `ZSlayerHQ/victory-world-runtime` (`zslayer` / `next`) | `99f8834` → `02ea29c` (9 commit, 7 dosya) | `02ea29c7eeb22da87156676c332dbb157cb9b681` | **GÜVENLİ** |
| `ZSlayerHQ/victory-world` (`main`) | `566d5be` → `7b3a5d6` (6 commit, 42 dosya) | `7b3a5d62d4af8b9c498ad653fdf88007d1feac08` | **GÜVENLİ** *(aşağıdaki kullanım kuralları ve dikkat maddeleriyle)* |

---

## 2. Parça A: `victory-world-runtime` (`99f8834..02ea29c`) — **GÜVENLİ**

### Kırmızı Bayrak Taraması
- **`Cargo.lock` / `Cargo.toml` / `build.rs`:** Değişmedi (`git diff --stat 99f8834 02ea29c -- Cargo.lock '**/Cargo.toml' '**/build.rs'` → boş). Yeni crate, `git=` bağımlılık veya build script yok.
- **Ağ API'si / `net.rs`:** Değişmedi. Loopback kilidi dokunulmamış.
- **`Command::new` / shell / PowerShell / registry / kullanıcı klasörü / base64 / gömülü ikili:** Yok.
- **CI (`.github/workflows/`):** Secret kullanımı yok, `pull_request_target` yok, izin genişletme yok (`permissions: contents: read` korundu). Action sürümleri commit SHA ile sabitlendi (supply-chain sertleştirmesi).

### Dosya Bazlı Bulgular
1. **`.github/workflows/ci.yml:6,25-29,40-41`:**
   - Tetikleme dalları `[main]` → `[next, world]` olarak güncellendi.
   - `actions/checkout` (`fbc6f39...`) ve `Swatinem/rust-cache` (`d1527c8...`) commit SHA'sına sabitlendi.
   - Yalnız `Tests` adımına `VICTORY_TESTS_ALLOW_SOFTWARE_SKIP: "1"` ortam değişkeni eklendi.
2. **`.github/workflows/screen.yml:1-80` (YENİ):**
   - `on: pull_request`, `permissions: contents: read`, `persist-credentials: false`.
   - PR kodunu **çalıştırmadan** yalnız `git diff --name-status` / `--numstat` / `-U0` ile hassas dosya (`Cargo.toml`, `build.rs`, `.github/`, betikler, ikililer) ve riskli desen (`Command::new`, `TcpStream`, `WinHttp`, `USERPROFILE`, `base64` vb.) taraması yapan salt okunur güvenlik kapısı.
3. **`README.md`:** Yalnız belge güncellemesi (dal açıklamaları, kurulum adımları, fork katkı çağrısı, krediler).
4. **`crates/victory-gpu/src/host.rs:681-696,769,6317`:**
   - Yalnız `#[cfg(test)]` altında `Host::is_software(&self)` metodu eklendi (`IDXGIAdapter1::GetDesc1()` ile `DXGI_ADAPTER_FLAG_SOFTWARE` veya `VendorId == 0x1414 && DeviceId == 0x8c` kontrolü).
   - Pencere başlığı `"ZSlayerHQ the title"` → `"ZSlayerHQ retail game"` olarak değiştirildi (ve ilgili birim testi güncellendi).
5. **`crates/victory-gpu/src/pipeline/host.rs:1089-1094`:**
   - Yalnız `mod tests` içinde: `host.is_software()` doğruysa ve `VICTORY_TESTS_ALLOW_SOFTWARE_SKIP=1` tanımlıysa depth-bounds testi atlanır.
6. **`crates/victory-shader/src/emit/host.rs:65-80,111-115,138-141`:**
   - Mevcut yazılım adaptör algılama bloğu `fn software(device: &ID3D12Device) -> bool` yardımcı fonksiyonuna çıkarıldı ve `Host::is_software(&self)` dışa açıldı.
7. **`crates/victory-shader/src/emit/valu_tests.rs:256-283,312-316,1646-1649`:**
   - Testlerde `skip_software(&host)` ile `VICTORY_TESTS_ALLOW_SOFTWARE_SKIP=1` durumunda yazılım adaptörü atlanır; tüm cihazlar atlandıysa test başarısız olmak yerine skipped olarak döner.

> **`host.rs` Değişikliklerinin Özeti:** GPU adaptör seçim mantığı değişmemiştir; her iki `host.rs` dosyasına yalnızca seçili D3D12 aygıtının Microsoft yazılım adaptörü (WARP / Basic Render Driver) olup olmadığını `GetDesc1()` ile sorgulayan `is_software()` metodu eklendi. Bu metod, ekran kartı olmayan CI ortamında `VICTORY_TESTS_ALLOW_SOFTWARE_SKIP=1` değişkeniyle donanım gerektiren shader ve depth-bounds testlerinin güvenle atlanması için kullanılır (ayrıca `victory-gpu/src/host.rs` pencere başlığını `"retail game"` yaptı).

---

## 3. Parça B: `victory-world` (`566d5be..7b3a5d6`) — **GÜVENLİ (Kurallı)**

### Kırmızı Bayrak Taraması
- **Ağ erişimi / indirme / veri gönderme:** `harness/` altındaki yeni ve değişen hiçbir betikte `curl`, `wget`, `Invoke-WebRequest`, `fetch`, HTTP/HTTPS isteği veya dış soket bağlantısı **yok**. (Yalnız `renderdoc.mjs:40-44` içindeki opsiyonel Python tetikleyici betiği yerel makinede `''` / port `38920` RenderDoc target control'e bağlanır.)
- **Registry yazma / base64 / gömülü ikili / `eval` / `Invoke-Expression`:** Yeni eklenen dosyalarda **yok**. `local-env.sh:4-11` açıkça `eval` kullanmadan tek kelime `export` yapar.
- **CI (`.github/workflows/harness-tests.yml`, `.github/workflows/screen.yml`):**
  - `permissions: contents: read`, `pull_request` (`pull_request_target` yok), secret yok.
  - Tüm action'lar commit SHA'ya sabitlenmiş (`actions/checkout@fbc6f39...`, `actions/setup-node@49933ea...`).
  - `harness-tests.yml:24-27`: `choco install ffmpeg -y --no-progress` + `node --test`.
  - `screen.yml:1-80`: PR kodunu çalıştırmadan yalnız `git diff` ile dosya/desen kontrolü.

### Dikkat Gerektiren Yeni Noktalar (Dosya:Satır)
1. **`harness/wpr-capture.ps1:9-23` (YENİ — YÖNETİCİ YETKİSİ İSTER):**
   - `WindowsBuiltInRole::Administrator` kontrolü yapar; yönetici değilse çıkar.
   - `wpr.exe -start GeneralProfile -start GPU -filemode` / `-stop` ile sistem geneli ETW izi (`.etl`) kaydeder (`%VICTORY_CAPTURES%\wpr\` veya `%USERPROFILE%\victory-captures\wpr\`).
   - Sistem geneli süreç adları, dosya yolları ve call stack içerdiği için **kullanılmaz**; kullanılırsa `.etl` dosyası asla paylaşılmaz/commitlenmez.
2. **`harness/save-snapshot.mjs:126-143` (YENİ — KLASÖR SİLME / GERİ YÜKLEME):**
   - `restore()` fonksiyonu `fs.rmSync(path.join(folder, entry), { recursive: true })` ile oyun save klasörünün içeriğini silip snapshot'tan geri yükler.
   - Güvenlik önlemleri: Önce `copyVerified(folder, backup)` ile `before-runs/` altına yedekleyip dosya sayısı + byte toplamını doğrular; `saveFolder()` her seviyede (`title`/`user`/`config`) tam 1 klasör şartı arar; `run.lock` varsa reddeder.
   - **Dikkat:** `saveAs()` satır 156'da `/^[A-Za-z0-9._-]+$/` ad kontrolü varken `restore()` satır 118'de bu kontrol yok (`path.join(snapshots, name)`). Yalnız kendi bildiğimiz düz snapshot adlarıyla kullanılmalı.
3. **`harness/settled-watch.mjs:98-127` (YENİ — SÜREÇ SONLANDIRMA):**
   - `heavy-watch.mjs` ile aynı desen: `powershell.exe -NoProfile -ExecutionPolicy Bypass -File run-process.ps1 -Pids -Strict` ile yalnız bu koşunun `victory.exe` PID'ini alır ve `taskkill /F /PID <pid>` ile kapatır.
4. **`harness/gpu-trace.mjs:123` ve `harness/renderdoc.mjs:115` (YENİ — DIŞ PROFİLLEME ARAÇLARI):**
   - Sırasıyla NVIDIA Nsight Graphics (`ngfx.exe`) ve RenderDoc (`renderdoccmd.exe`) süreçlerini `spawnSync` ile başlatır. Varsayılan akışta çalışmazlar (`--gpu-trace` veya `--renderdoc` + gecikme bayrağı gerekir).
5. **`harness/precompile-progress.mjs:19` ve `harness/run-world.sh:258` (YENİ — KULLANICI KLASÖRÜNE YAZMA):**
   - Varsayılan olarak `$HOME/.config/victory-harness/stream/precompile.json` dosyasına yalnızca sayısal shader derleme ilerlemesi yazar (yol veya program adı içermez; `VICTORY_PRECOMPILE_JSON` ile değiştirilebilir).
6. **`harness/local-env.mjs:18-26` ve `harness/local-env.sh:4-11` (YENİ — YEREL YOL YÜKLEYİCİ):**
   - `harness/local.env` dosyasından **yalnızca 7 izinli yol değişkenini** (`GAME_EXE`, `VICTORY_CAPTURES`, `VICTORY_BUILDS`, `VICTORY_REPO`, `VICTORY_WORKTREES`, `PRESENTMON_EXE`, `VICTORY_SAVE_BACKUPS`) okur; `VICTORY_ALLOW_OBS` gibi anahtarları yok sayar. `eval` kullanmaz; ortam değişkeni dosyaya baskındır.
7. **Diğer Yeni/Güncellenen Betikler (Temiz — Salt Yerel Okuma/Raporlama):**
   - `disk-usage.mjs` (`fs.statfsSync` ile yalnız sürücü boş/toplam GB), `route-keys.mjs` (`.keys` dosyası okuma), `run-cwd.mjs` (`.git` üst dizin kontrolü + `cwd/` oluşturma), `run-lines.mjs`, `shader-compile-report.mjs`, `compare-runs.mjs`, `experiment-table.mjs`, `parse-run.mjs`, `summarize-run.mjs`, `run-process.ps1` (`translate --cache` → `warm` eşleşmesi).

---

## 4. Harness Kullanım Kuralları (Güncel Durum)

### Önceki Rapordan Devam Eden Kurallar
- **`harness/run-queue.sh` (`eval "TOKENS=($clean)"`):** Hâlâ geçerli (yalnız `local-env.sh` satırları eklendi, `eval` duruyor) → **Yalnızca kendi oluşturduğumuz güvenilir kuyruk dosyalarıyla çalıştırılır.**
- **`--presentmon` (`harness/presentmon.mjs`):** Hâlâ geçerli (yönetici / Performance Log Users gereksinimi sürüyor) → **Kullanılmaz.**
- **`harness/set-volume.ps1`:** Değişmedi → **Hâlâ geçerli** (yalnız bu koşunun `victory.exe` PID'inin sesini ayarlar).

### Bu İncelemede Eklenen Yeni Kurallar
- **`harness/wpr-capture.ps1`:** Yönetici PowerShell ister ve sistem geneli `.etl` izi alır → **Kullanılmaz.**
- **`--save` / `harness/save-snapshot.mjs`:** Save klasörünü silip geri yükler (`restore` tarafında `name` regex doğrulaması yok) → Yalnız kendi oluşturduğumuz düz isimli snapshot'larla kullanılır.
- **`--gpu-trace` / `--renderdoc`:** Opsiyonel dış profilleme/yakalama araçlarını (`ngfx.exe`, `renderdoccmd.exe`) çalıştırır → Gerekmedikçe kullanılmaz.
- **`harness/local.env`:** Yalnız kendi yerel yollarımızla elle oluşturulur (dışarıdan kopyalanmaz).
