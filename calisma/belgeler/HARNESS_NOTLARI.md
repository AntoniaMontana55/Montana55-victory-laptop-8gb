# Victory Harness & Çalıştırma Notları

Kaynaklar:
- Harness: `ZSlayerHQ/victory-world` @ `7b3a5d6` (`C:\CLLL\.calisma\vw-harness`)
- Runtime: `ZSlayerHQ/victory-world-runtime` (`next`) @ `02ea29c` (`C:\CLLL\.calisma\vw-runtime`)
- Güvenlik ve başlatıcı: `C:\CLLL\.calisma\repo_durum\GUVENLIK.md`, `C:\CLLL\.calisma\repo_durum\guvenli_baslat.ps1`
- Donanım ayarları: `C:\CLLL\.calisma\victory-fork\AYARLAR.md` (RTX 3070 Ti Laptop 8 GB VRAM, 16 GB RAM, Windows 11 Home)

> **Güvenlik yasakları (`GUVENLIK.md:7-8,28`):** `--presentmon` ve `wpr-capture.ps1` **YASAK** (yönetici ister). `run-queue.sh` yalnız kendi kuyruk dosyamızla (`eval` var). `save-snapshot.mjs --save` yalnız kendi düz isimli snapshot'larımızla. `--gpu-trace` / `--renderdoc` gerekmedikçe yok. Her koşu `guvenli_baslat.ps1` ile başlatılmalı.

---

## 1. Harness Akışı ve Bileşen Envanteri

### Yapılandırma (`victory.toml` ve `harness/local.env`)
- **Runtime config (`victory.toml`):** `victory-core/src/config.rs:48-101` — Öncelik sırası: **ortam değişkeni > `VICTORY_CONFIG` (yoksa `./victory.toml`) > varsayılan**.
  - `[paths]` anahtarları (`vw-runtime/victory.example.toml:6-19`):
    - `game_dir` / `VICTORY_GAME_DIR` (zorunlu: `warm`, `translate`, `census` için; `run` exe yolunu doğrudan argümandan alır)
    - `cache_dir` / `VICTORY_CACHE_DIR` (vars. `%LOCALAPPDATA%\victory\cache`)
    - `data_dir` / `VICTORY_DATA_DIR` (vars. `%LOCALAPPDATA%\victory\data`)
    - `test_titles` / `VICTORY_TEST_TITLES` (`;` ayrılmış)
  - Tüm yollar git deposu dışında olmalıdır (`config.rs:124`, `ensure_outside_repo`).
- **Harness yerel yol dosyası (`harness/local.env`):** `harness/local.env.example:1-23`, `harness/local-env.mjs:18-26`, `harness/local-env.sh:4-11`.
  - Ortam değişkeni dosyayı ezer; `eval` kullanılmaz.
  - **Yalnız 7 izinli yol değişkeni** kabul edilir (diğerleri uyarıyla yok sayılır):
    `GAME_EXE`, `VICTORY_CAPTURES`, `VICTORY_BUILDS`, `VICTORY_REPO`, `VICTORY_WORKTREES`, `PRESENTMON_EXE`, `VICTORY_SAVE_BACKUPS`.
  - **Oyun yolu verme:** `GAME_EXE=<oyun.exe>` (`local.env` veya env) + `victory.toml` içinde `game_dir`. `run-world.sh:135-138` varsayılan olarak `$HERE/../victory/victory.toml` dosyasını `VICTORY_CONFIG` olarak dışa aktarır.

### Araç Gereksinimleri (Makine Kontrol Listesi)
- **Zorunlu (harness için):** Git Bash (`bash`, `tee`, `sed`, `awk`), **Node.js 18+** (CI'da Node 22: `.github/workflows/harness-tests.yml:22` — neredeyse tüm yardımcı betikler `.mjs`!), `powershell.exe`, `tasklist`, `taskkill`.
- **Ekran kaydı ve özet zaman çizelgesi için:** `ffmpeg` (`gfxcapture` filtreli) ve `ffprobe` (`vw-harness/README.md:12-14`, `record-screen.mjs:52-55`, `contact-sheets.mjs:49-53`, `screen-timeline.mjs:32-49`). Yoksa `--no-record` ile kayıt kapatılabilir.
- **Not:** Makinede Git Bash ve Node.js varlığı henüz doğrulanmadı; Node.js yoksa harness `.mjs` betikleri çalışmaz (ancak `victory.exe` + `guvenli_baslat.ps1` doğrudan PowerShell ile çalışır).

### Bileşen Tablosu

| Bileşen | Ne yapar | Girdi | Çıktı dosyaları | Araç | Yönetici? | Dosya:Satır |
|---|---|---|---|---|---|---|
| `world-run.sh` | Tek koşu kilidi (`run.lock`) alır, `obs64.exe` açıksa (WARP değilse) reddeder, `VICTORY_CONFIG` ve `WORLD_RUN_HOLDS_LOCK=1` set edip `run-world.sh`'yi çağırır | `run-world.sh` bayrakları, `VICTORY_DIR`, `VICTORY_CONFIG` | `harness/run.lock/owner` (geçici) | `bash`, `node`, `tasklist` | Hayır | `harness/world-run.sh:1-45` |
| `run-world.sh` | Tek koşu orkestratörü: env/build başlığı yazar, opsiyonel `--save` geri yükler, `--warm` çalıştırır, öncelik/ses/kayıt/watcher başlatır, `victory.exe run` koşturur, çıkışta sheet + driver olayları + özet üretir | `--tag`, `--console`, `--profile`, `--warm`, `--keys` / `--route`, `--heavy-seconds` / `--settled-seconds`, `--no-record`, `--volume`, `--frames`, `--dump-targets`, `--save`/`--save-as`, vb. | `harness/logs/<STAMP>-<TAG>.log`, `<LOG>.md`, `<LOG>.warm` (`--warm` ile), `<CAPTURE>/*` | `bash`, `node`, `powershell`, `ffmpeg`/`ffprobe` (kayıt açıksa) | Hayır | `harness/run-world.sh:1-388` |
| `victory warm` (`--warm`) | Oyun arşivlerindeki shader'ları önceden çevirir ve compute pipeline'larını önbelleğe alır (`run-world.sh` önceliğini yükseltir ve `precompile-progress.mjs` ile izler) | `$(dirname "$GAME_EXE")`, `VICTORY_CONFIG`, ops. `--jobs N`, `--no-pipelines` | `<LOG>.warm`, `<CAPTURE>/translate-report.txt`, `precompile.json`, `<cache_dir>/shaders` | `victory.exe`, `node`, `powershell` | Hayır | `run-world.sh:249-277`, `apps/victory/src/main.rs:90-98` |
| `precompile-progress.mjs` | `.log.warm` dosyasını 0.5 sn'de bir okuyup OBS/stream overlay için sayısal ilerleme JSON'u yazar (yol/ad içermez) | `--log <warm.log>`, `--out <json>`, `--started-ms`, `--one-phase`, `--exit` | `$HOME/.config/victory-harness/stream/precompile.json` (`VICTORY_PRECOMPILE_JSON` ile değişir) | `node` | Hayır | `harness/precompile-progress.mjs:19,93-191` |
| `run-queue.sh` | Kuyruk dosyasındaki satırları sırayla `world-run.sh` üzerinden koşturur; `--max-seconds` (vars. 1500 sn) aşımında oyunu ve ağaç sürecini öldürür. **Kural:** yalnız kendi kuyruk dosyamız (`eval`) | `[--max-seconds N] [--dry-run] <queue_file>` (satır: `<tag> <snapshot\|lane> [flags...]`) | `<queue_file>.result.md` + her koşunun kendi log/capture'ı | `bash`, `node`, `powershell` | Hayır | `harness/run-queue.sh:1-377` (`eval`: `355`) |
| `route-keys.mjs` (`--route`) | `harness/routes/<name>.keys` dosyasını okuyup `VICTORY_KEYS` listesi ve `# heavy-seconds:` / `# settled-seconds:` başlıklarını çıkarır (`--keys` ile birlikte kullanılamaz) | `keys\|heavy-seconds\|settled-seconds <routes_dir> <name>` | stdout (virgüllü tuş dizisi / saniye) | `node` | Hayır | `harness/route-keys.mjs:1-78`, `run-world.sh:84-98` |
| `heavy-watch.mjs` (`--heavy-seconds`) | Logda `draws + dropped >= 1000` olan ilk 5 sn kare satırından N sn sonra `run-process.ps1 -Strict` PID'ini `taskkill /F` ile kapatır | `<log> <seconds> [--poll-ms 250]` | Loga `harness: ended by --heavy-seconds after N s of heavy scene` satırı ekler | `node`, `powershell`, `taskkill` | Hayır | `harness/heavy-watch.mjs:1-249` |
| `settled-watch.mjs` (`--settled-seconds`) | Ağır sahnede art arda 3 kare satırında `fps >= 1.5` (vars.) olunca "settled" sayar, `keys-after` dosyasını oluşturur, N sn sonra oyunu kapatır; `--settled-timeout` (vars. 600 sn) dolarsa keser | `<log> <seconds> [--timeout 600] [--floor-fps 1.5] [--waiting K] [--after-file <path>]` | `<CAPTURE>/keys-after`, loga `harness: settled at ...` ve `harness: ended by --settled-seconds ...` | `node`, `powershell`, `taskkill` | Hayır | `harness/settled-watch.mjs:1-330` |
| `record-screen.mjs` + `window-handle.ps1` | Oyun penceresinin HWND'sini bekler (120 sn), `ffmpeg` `gfxcapture` ile 2 fps H.264 kayıt alır (`--no-record` ile kapalı) | `<CAPTURE>/screen.mp4`, `RUN_WORLD_GAME_EXE`, `RUN_WORLD_STARTED_EPOCH` | `<CAPTURE>/screen.mp4` + loga `recording: started ...` | `node`, `powershell`, `ffmpeg`, `ffprobe` | Hayır | `harness/record-screen.mjs:1-166`, `window-handle.ps1:1-17` |
| `contact-sheets.mjs` + `screen-timeline.mjs` | `screen.mp4`'den 3×3 (9 sn/sayfa) zaman damgalı iletişim sayfaları ve saniyelik siyah/görüntü/cut zaman çizelgesi üretir | `<CAPTURE>/screen.mp4`, `[--log <run.log>]` | `<CAPTURE>/sheets/sheet-001.jpg`, özet içinde `## What was on screen` | `node`, `ffmpeg` | Hayır | `harness/contact-sheets.mjs:1-60`, `screen-timeline.mjs:1-215` |
| `raise-priority.ps1` / `set-volume.ps1` | Koşu başladıktan 4 sn sonra `victory.exe` önceliğini `AboveNormal`, Windows mikser sesini `%4` (`--volume N\|off`) yapar | `RUN_WORLD_*` env, `-Translate` / `-Percent` | stdout / log satırı | `powershell` | Hayır | `harness/raise-priority.ps1:1-22`, `set-volume.ps1:1-157` |
| `driver-events.ps1` | Koşu zaman aralığındaki Windows `System` günlüğünden yalnız NVIDIA `nvlddmkm` olaylarını (MMU Fault, Warp Error, id 13/153) loga ekler | `<start_epoch> <end_epoch>` | Loga `driver event: ...` satırları | `powershell` | Hayır | `harness/driver-events.ps1:1-60` |
| `save-snapshot.mjs` (`--save`/`--save-as`) | `<data_dir>/<title>/saves/<user>/<config>` klasörünü yedekleyip doğrular, sonra snapshot'tan geri yükler veya yeni snapshot alır. **Kural:** yalnız düz isimli kendi snapshot'larımız | `restore\|save-as <name> --config <victory.toml>` | `<VICTORY_SAVE_BACKUPS>/{snapshots,before-runs}/...` | `node` | Hayır | `harness/save-snapshot.mjs:1-209` |
| `summarize-run.mjs` / `compare-runs.mjs` / `experiment-table.mjs` | Tek koşu özeti, 2+ koşu fark tablosu ve çoklu koşu deney tablosu üretir | `<log>... [--json]` | `<LOG>.md` / stdout Markdown veya JSON | `node` (+ `ffmpeg`/`ffprobe` özet için) | Hayır | `summarize-run.mjs`, `compare-runs.mjs`, `experiment-table.mjs` |
| `shader-compile-report.mjs` / `clean-failures.mjs` / `name-hashes.mjs` | `--warm` derleme raporu, çeviri hata listesi (`stage\|hash\|reason`) ve hash→shader adı ekleme | `<warm.log>`, `<report.txt>`, `<run.log>` | stdout / çıktı dosyası | `node` | Hayır | `shader-compile-report.mjs`, `clean-failures.mjs`, `name-hashes.mjs` |
| `snapshot-build.sh` | Temiz git worktree commit'ini `--locked` derleyip `victory.exe` + `*.dll` + `victory.pdb` + `snapshot.txt` olarak `<VICTORY_BUILDS>/<sha>-<profile>` altına dondurur | `<lane> [--profile diag\|release]` / `--list` / `--prune N` | `<VICTORY_BUILDS>/<sha>-<profile>/*` | `bash`, `git`, `cargo` | Hayır | `harness/snapshot-build.sh:1-219` |
| **YASAK / DIŞ:** `presentmon.mjs` (`--presentmon`), `wpr-capture.ps1` | PresentMon ETW kare ölçümü / Windows Performance Recorder `.etl` izi | `--presentmon` / `wpr-capture.ps1` | `presentmon.csv` / `wpr/trace-*.etl` | `PresentMon` / `wpr.exe` | **EVET (YASAK)** | `presentmon.mjs:243-253`, `wpr-capture.ps1:8-11` |
| **GEREKMEDİKÇE YOK:** `gpu-trace.mjs`, `renderdoc.mjs`, `--aftermath` | Nsight Graphics GPU Trace, RenderDoc kare yakalama, NVIDIA Aftermath çökme dökümü (`run-world.sh:325` bu commit'te `--gpu-trace`/`--renderdoc` dallanmasını henüz bağlamamış, yalnız bayrak/log işliyor) | `--gpu-trace --trace-after-ms`, `--renderdoc --capture-after-ms`, `--aftermath` | `<CAPTURE>/{gputrace,rdoc}` / Aftermath dump | `ngfx.exe`, `renderdoccmd.exe`, Aftermath SDK | Hayır | `gpu-trace.mjs`, `renderdoc.mjs`, `run-world.sh:102-127,191-195` |

---

## 2. Harness Olmadan Doğrudan Çalıştırma (En Kısa Komut)

### Gerekli Config Alanları
`victory.toml` (`vw-runtime/victory.example.toml:6-19`, `crates/victory-core/src/config.rs:48-127`):
```toml
[paths]
game_dir = 'D:\victory-data\game'      # Zorunlu (warm/config için); repo dışında olmalı
cache_dir = 'D:\victory-data\cache'    # Opsiyonel (vars. %LOCALAPPDATA%\victory\cache); repo dışında
data_dir = 'D:\victory-data\data'      # Opsiyonel (vars. %LOCALAPPDATA%\victory\data); repo dışında
```
Doğrulama: `cargo run -p victory --locked -- config` (`vw-runtime/README.md:65`).

### En Kısa Doğrudan Komut (PowerShell, Repo Dışından)
Runtime, göreceli yolları çalışma dizinine yazdığı için **git deposu dışındaki** bir klasörden başlatılmalıdır (`vw-runtime/README.md:80-90`):
```powershell
$env:VICTORY_CONFIG = "C:\CLLL\.calisma\vw-runtime\victory.toml"
Set-Location D:\victory-run   # repo dışında herhangi bir klasör
& "C:\CLLL\.calisma\vw-runtime\target\diag\victory.exe" run "D:\victory-data\game\<oyun.exe>" --execute --console series-s 2> D:\victory-run\run.log
```
- **En kısa `guvenli_baslat.ps1` çağrısı** (`GUVENLIK.md:28`, `guvenli_baslat.ps1:1-47`):
  ```powershell
  $env:VICTORY_CONFIG = "C:\CLLL\.calisma\vw-runtime\victory.toml"
  powershell.exe -NoProfile -ExecutionPolicy Bypass -File "C:\CLLL\.calisma\repo_durum\guvenli_baslat.ps1" `
    -Exe "C:\CLLL\.calisma\vw-runtime\target\diag\victory.exe" `
    -WorkDir "D:\victory-run" `
    -ArgumentList 'run "D:\victory-data\game\<oyun.exe>" --execute --console series-s' `
    -MaxMinutes 5
  ```
- **Bulunan `run-world.sh` hatası (not):** `run-world.sh:157` `RUN_CWD` klasörünü repo dışında seçip satır `219`'da logluyor, ancak satır `325`'te (ve `warm` için `270`'te) hâlâ `cd "$VICTORY_DIR"` ile başlatıyor! `$VICTORY_DIR` bir git deposuysa bu repo-içi çalışma dizini riski taşır; doğrudan veya değiştirilmiş çalıştırmada `RUN_CWD` kullanılmalı.

---

## 3. Harness ile `guvenli_baslat.ps1` Birleştirme Önerisi (Değişiklik Yapılmadı)

### Harness `victory.exe`'yi Nasıl Başlatıyor?
- **Başlatma noktası `run-process.ps1` DEĞİLDİR.** `run-process.ps1:14-49` yalnızca çalışan `victory.exe` PID'lerini `Get-CimInstance Win32_Process` ile sorgular (`Name == "victory.exe"`, `CommandLine` içinde `" run "` + `$GAME_EXE` ve `CreationDate >= RUN_WORLD_STARTED_EPOCH - 2`).
- **Gerçek başlatma satırları:**
  - Oyun koşusu: `harness/run-world.sh:325-327`:
    ```bash
    (cd "$VICTORY_DIR" && env "${ENV_ARGS[@]}" "$BIN" run "$GAME_EXE" --execute --console "$CONSOLE") \
      2>&1 | tee -a "$LOG"
    ```
  - Warm-up: `harness/run-world.sh:270`:
    ```bash
    (cd "$VICTORY_DIR" && env "${WARM_ENV[@]}" "$BIN" warm "$(dirname "$GAME_EXE")") >"$LOG.warm" 2>&1
    ```

### Araya `guvenli_baslat.ps1` Koymak İçin Değişmesi Gerekenler (Öneri)
1. **`harness/run-world.sh:325` (ve isteğe bağlı `270`):** Doğrudan `"$BIN"` yerine ortamdan gelen bir başlatıcı değişkeni (örn. `VICTORY_SAFE_LAUNCHER="C:/CLLL/.calisma/repo_durum/guvenli_baslat.ps1"`) tanımlıysa `powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$VICTORY_SAFE_LAUNCHER" -Exe "$BIN" -WorkDir "$RUN_CWD" -ArgumentList "run \"$GAME_EXE\" --execute --console $CONSOLE" -LogDir "$CAPTURE/guvenli"` çağrılır. (Aynı yerde `cd "$VICTORY_DIR"` → `cd "$RUN_CWD"` düzeltmesi de yapılmalı.)
2. **KRİTİK — `stdout`/`stderr` akışı (`guvenli_baslat.ps1:214-224`):**
   - `guvenli_baslat.ps1` şu an `CreateProcess(..., inheritHandles: false, ...)` ile `StartupInfo.StdOut`/`StdErr` bağlamadan başlatır. Bu yüzden `victory.exe`'nin `stderr` logu `run-world.sh`'nin `2>&1 | tee -a "$LOG"` borusuna **düşmez**; bu olmadan `heavy-watch.mjs`, `settled-watch.mjs` ve `summarize-run.mjs` log okuyamaz!
   - **Çözüm önerisi:** `guvenli_baslat.ps1` içine opsiyonel `-StdOutLog` / `-StdErrLog` (veya `STARTF_USESTDHANDLES` + `inheritHandles: true` ile üst sürecin borusuna aktarım) eklenmeli; böylece `victory.exe` logu hem `run-world.sh` borusuna akar hem `olcum.csv` + `ozet.txt` üretilir.
3. **Öncelik çakışması (`run-world.sh:289` vs `guvenli_baslat.ps1:201-202`):**
   - `guvenli_baslat.ps1` Job Object'e `PriorityClass = BelowNormal` (0x4000) kilitler; `run-world.sh:289` ise 4 sn sonra `raise-priority.ps1` ile `AboveNormal` yapmaya çalışır. `VICTORY_SAFE_LAUNCHER` aktifken `raise-priority.ps1` (ve `warm` için satır `267-269`) atlanmalı.
4. **Uyumlu kalan parçalar (değişiklik gerekmez):**
   - `guvenli_baslat.ps1:218-223` doğrudan `victory.exe` sürecini `"<exe>" run "<GAME_EXE>" --execute ...` komut satırıyla başlattığı için `run-process.ps1 -Strict`, `heavy-watch.mjs`, `settled-watch.mjs`, `set-volume.ps1` ve `record-screen.mjs` bu süreci **doğrudan tanır**.
   - Watcher `taskkill /F` ile `victory.exe`'yi bitirdiğinde `guvenli_baslat.ps1:467` `ActiveProcesses() == 0` görerek döngüden temiz çıkar ve `olcum.csv` / `ozet.txt` yazar.

---

## 4. Ölçüm Sistemi ve Koşu Tablosu Sütunları

### Metrik Kaynakları, Dosyalar ve Formatlar

| Kaynak | Dosya ve Format | Kaydedilen Metrikler | Dosya:Satır |
|---|---|---|---|
| Runtime `stderr` (`tracing`) | `harness/logs/<STAMP>-<TAG>.log` (düz metin) | **5 sn kare satırı:** `fps`, `ms/frame`, `preparing`, `copies ms/MB`, `textures ms/MB`, `on the GPU`, `copying back ms/MB`, `draws/frame`, `dropped`, `being made`, `indirect`, `textures picked by index (uploaded MB, waiting)`. **`card clock:` satırı:** `sync_ms`, `title_ms`, `prelude_ms`, `compare_ms`, `fetch_ms`, `own_ms`, `sum_ms`, `wall_ms`, `cpu_core_pct`, `cpu_all_pct`, `private_bytes`, `working_set_bytes`, `io_read/write_*`, `game_vol_*`. **Settle/Hız satırları:** `submissions sent ahead` (flights, settles, card wait ms, neden kırılımı), `table constants`, `buffer copies`, `card clock threads`. **VRAM:** `VICTORY_CARD_ACCOUNTING=1` ile DXGI VRAM bütçe/kullanım satırı (`AYARLAR.md:173,245`). | `parse-run.mjs:25-44`, `run-lines.mjs:17-28`, `compare-runs.mjs:37-43` |
| `summarize-run.mjs` | `<LOG>.md` (Markdown) veya `--json` | Süre, son kare, bitiş nedeni, ekran kaydı + `What was on screen` (siyah/yakın-siyah/görüntü sn, sahne kesmeleri), FPS (samples/median/min/max/last), son kare maliyet kırılımı, en kötü 5 kare, pipeline sağlığı (refused, translated, slow submissions/lists/cmds, left out), Card & CPU per frame, top 12 thread, eksik dokular, hung komutlar, NVIDIA `nvlddmkm` driver olayları, hatalar. | `summarize-run.mjs:88-250` |
| `compare-runs.mjs` | stdout Markdown / `--json` | Settled (yoksa ilk ağır kare) penceresinde: `fps median`, **`Frame ms median / p10 / p90`**, `Preparing / Copies / Textures / GPU / Copy-back` medyanları, `Draws`, `Settles`, `Waiting in settles ms`, `Top settle reasons`, `Textures waiting`, `Dropped`, `Refused pipelines`, `Table constants`, `Buffer copies`, tüm 18 `card clock` medyanı + ilk koşuya göre **mutlak & % fark tablosu**. | `compare-runs.mjs:201-285` |
| `guvenli_baslat.ps1` | `<LogDir>/olcum.csv` (UTF-8 BOM CSV, 1 Hz) + `ozet.txt` | **Sütunlar:** `Zaman,BosRAM_GB,BosCommit_GB,PrivateBytes_GB,WorkingSet_GB,CPU_Yuzde,GPU_Sicaklik_C,VRAM_Kullanilan_MiB,VRAM_Toplam_MiB,GPU_Yuzde`. `ozet.txt`: başlangıç/bitiş, çıkış kodu, durdurma sebebi, tepe PrivateBytes, en düşük boş RAM, tepe GPU °C, tepe VRAM MiB. | `guvenli_baslat.ps1:429-447,579-588` |

> **p95 & GPU busy notu:** PresentMon yasak olduğundan (`GUVENLIK.md:7`), `presentmon.csv` üretilmez ve `experiment-table.mjs`'deki `p95 frame ms` / `GPU median ms` sütunları eklenmez. Bunun yerine kare dağılımı için `compare-runs.mjs`'nin **`Frame ms p10 / median / p90`** değerleri, kart beklemesi için runtime `card clock` / `settleMs` satırları, VRAM ve gerçek GPU yükü/sıcaklığı için `guvenli_baslat.ps1`'in `olcum.csv` dosyası (`nvidia-smi`) kullanılır. (`KNOWN-FAULTS.md:69-74`: pencere başlığındaki `gpu N% busy` aslında GPU yükü değil, gönderim thread'inin bekleme oranıdır.)

### Koşu Tablosu Sütunları
- **`experiment-table.mjs` (`harness/experiment-table.mjs:98-121`):**
  `tag` | `seconds` | `last frame` | `median fps` | *(yalnız PresentMon varsa: `median frame ms` | `p95 frame ms` | `GPU median ms`)* | `ended` | `hung list` | `hung command` | `hung op` | `pipeline` | `stage` | `DRED last op` | `DRED hung op` | `null slots` | `slowest submission` | `dropped draws` | `skip pipelines` | `loop limit` | `wave64` | `GPU timing` | `debug layer` | `GBV`
  *(Altında ayrıca `## Translation failures — distinct hashes` çapraz tablosu.)*
- **`run-queue.sh` sonuç tablosu (`<queue>.result.md`, `harness/run-queue.sh:337`):**
  `tag` | `binary` | `exit` | `how it ended` | `log`

---

## 5. İlk Koşu Sırası ve Tekrarlanabilir Rota / Tuş Dizisi

### Önerilen İlk Koşu Sırası
1. **Güvenlik & Derleme (koşu öncesi):**
   - Oyun klasörü gelince `GUVENLIK.md:15-20` sırası: kaynak kaydı → Defender tarama → dosya listesi kontrolü → exe/DLL SHA-256 doğrulaması.
   - `victory.toml` (`game_dir`, repo dışı `cache_dir`/`data_dir`) oluştur → `cargo run -p victory --locked -- config` ile doğrula.
   - Derleme: `cargo build --profile diag -p victory --features diagnostics --locked` (`vw-runtime/README.md:72`).
   - PC'yi yeniden başlat; Brave/Discord/Steam/Broadcast/qBittorrent kapalı (`GUVENLIK.md:28`).
2. **Adım 1 — Önbellek Isıtma (`victory warm`):**
   - `VICTORY_ACTIVATE_RAYS=0`, `VICTORY_RT_RESOLVE=0` ile (`AYARLAR.md:271-272`) ve 16 GB RAM'de tepe belleği dizginlemek için `--jobs 4` (veya `6`) vererek `guvenli_baslat.ps1` altında `victory.exe warm <game_dir>` koştur.
3. **Adım 2 — Koşu 1 (Menüye kadar, 2-3 dk, kısa baz ölçüm):**
   - `guvenli_baslat.ps1` (`-MaxMinutes 4`) + `AYARLAR.md` başlangıç seti + `--console series-s`, tuş script'i olmadan elle menüye kadar gir.
   - Amaç: 30 fps menü kararlılığı, açılış commit/RAM/VRAM/sıcaklık baz çizgisi (`olcum.csv` + `run.log`), `crashes.log` yokluğu.
4. **Adım 3 — Koşu 2 (İlk oynanabilir sahne / hapishane başı sabit sahne):**
   - İlk açılış ekranlarını geçip ilk oynanabilir sahneye (autosave noktasına) ulaş; çıkışta kendi düz isimli snapshot'ını al (`save-snapshot.mjs save-as hapishane-basi --config ...`).
   - Sonraki tüm karşılaştırma koşularında bu snapshot'ı geri yükleyip (`--save hapishane-basi`) sabit sahnede 60-90 sn (`--heavy-seconds 60` veya `--settled-seconds 60`) ölçüm al ve `compare-runs.mjs` ile kıyasla.

### Tekrarlanabilir Rota / Tuş Dizisi Durumu (Kritik Runtime Uyumsuzluğu!)
- Harness'ta `--route <name>` (`route-keys.mjs`) `harness/routes/<name>.keys` dosyalarını okur; `route-keys.test.mjs:88` bu klasörde en az bir `.keys` rotası bulunduğunu gösterir (salt-okunur/komutsuz modda dizin listelenmedi; koşu öncesi `harness/routes/` içine bakılmalı).
- **KRİTİK BULGU — `vw-runtime @ 02ea29c` tuş motoru sınırı (`crates/victory-hle/src/modules/gameinput.rs:1201-1238`):**
  - `VICTORY_KEYS` yalnız `diagnostics` derlemesinde aktiftir (`#[cfg(feature = "diagnostics")]`).
  - Yalnız `<saniye>:<Tuş>` biçimini (sabit `200 ms` basış) destekler: `Enter`, `Esc`, `Tab`, `Back` (**`Backspace` değil!**), `Up`, `Down`, `Left`, `Right` veya tek ASCII harf/rakam.
  - `route-keys.mjs`'nin beklediği `:<hold>` süresi (örn. `70:J:8`) ve `settled-watch.mjs`'nin kullandığı `+...` / `VICTORY_KEYS_AFTER` göreceli zamanlama **bu runtime commit'inde (`02ea29c`) henüz yoktur** (`split_once(':')` yüzünden `70:J:8` sessizce yok sayılır!).
- **Öneri:** İlk koşularda karmaşık `.keys` rotalarına güvenmek yerine (a) kendi düz isimli `save-snapshot` kaydımızla sabit sahneye dönmek, (b) gerekirse yalnız basit `--keys '55:Enter,65:Enter'` kullanmak gerekir.

---

## 6. Series S + 1080p + `AYARLAR.md` Ayarlarının Harness'ta Konumu

> **Önemli:** `harness/local.env` yalnız 7 yol değişkenini kabul eder (`local-env.mjs:18-26`); aşağıdaki `VICTORY_*` ayarları `local.env`'ye **konamaz** (yok sayılır). Tüm `VICTORY_*` değişkenleri `world-run.sh` / `run-world.sh` çağrılmadan önce **süreç ortamına** verilir; `run-world.sh:213-217` bunları otomatik loglar ve `victory.exe`'ye iletir.

| Ayar (`AYARLAR.md:266-276`) | Değer | Harness'ta Nereye Konur? | Not |
|---|---|---|---|
| Konsol profili | `series-s` | CLI bayrağı: `--console series-s` (`run-world.sh:56`) | Harness varsayılanı `series-x` (`run-world.sh:26`) — mutlaka verilmeli |
| Derleme profili | `diag` | CLI bayrağı: `--profile diag` (varsayılan, `run-world.sh:18`) | `VICTORY_KEYS` ve `VICTORY_CAPTURE` için `diag` şart |
| Ekran kaydı (ops.) | kapalı | CLI bayrağı: `--no-record` (`run-world.sh:68`) | `ffmpeg` yoksa veya 8 GB VRAM / 16 GB RAM'de ek yük istenmiyorsa |
| Çözünürlük (1080p) | `1920x1080` | Ortam: `VICTORY_DISPLAY_MODE=1920x1080` | Vars. 4K (`3840x2160`) |
| Kart tampon bütçesi | `512` | Ortam: `VICTORY_CARD_BUFFERS_MB=512` | Vars. 1024 MiB |
| Doku VRAM payı | `85` | Ortam: `VICTORY_TEXTURE_ROOM_PERCENT=85` | Vars. 90% |
| Bellek pencere ızgarası | `512` | Ortam: `VICTORY_WINDOW_KB=512` | Vars. 4096 KiB |
| Pencere kuyruk bölme | `1` | Ortam: `VICTORY_WINDOW_TAILS=1` | Vars. 0 |
| RT resolve atlama | `0` | Ortam: `VICTORY_RT_RESOLVE=0` | Vars. 1 (~1000 RT shader derlemesini atlar) |
| ActivateRays atlama | `0` | Ortam: `VICTORY_ACTIVATE_RAYS=0` | Vars. 1 (~85 sn / >1.3 GB derleyici RAM tasarrufu) |
| VRAM muhasebesi (teşhis) | `1` | Ortam: `VICTORY_CARD_ACCOUNTING=1` | İlk koşularda VRAM doluluğunu loglar; stabil olunca kaldır |
| `VICTORY_RELAY` / `LATE_WAIT` | `2` / `3` | Değiştirmeye gerek yok | Kodda zaten varsayılan açık (`AYARLAR.md:274`) |
| `VICTORY_LOG` / `DRED` | `info` / `1` | `run-world.sh:172` zaten sabit ekler | Ekstra vermeye gerek yok |
| `VICTORY_PIPELINE_WAIT_MS` | `1000` (ops.) | Ortam: `VICTORY_PIPELINE_WAIT_MS=1000` | Yalnız `warm` yapılmadıysa geçici donma azaltıcı (`AYARLAR.md:279`) |

**Örnek Git Bash ortam bloğu (harness çağrısı öncesi):**
```bash
export VICTORY_DISPLAY_MODE=1920x1080
export VICTORY_CARD_BUFFERS_MB=512
export VICTORY_TEXTURE_ROOM_PERCENT=85
export VICTORY_WINDOW_KB=512
export VICTORY_WINDOW_TAILS=1
export VICTORY_RT_RESOLVE=0
export VICTORY_ACTIVATE_RAYS=0
export VICTORY_CARD_ACCOUNTING=1
# Sonra (guvenli_baslat.ps1 entegrasyonu ile):
# bash harness/world-run.sh --tag ilk-kosu --console series-s --profile diag --no-record --heavy-seconds 60
```
