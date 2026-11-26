# VIctory / the title Koşu Otomasyonu ve Yükleme RAM Optimizasyonu Araştırması

**Temelimiz:** `ZSlayerHQ/zslayer-pc-port-runtime` `next@02ea29c (+PR #10)` (`C:\CLLL\.calisma\vw-pr10`) ve `ZSlayerHQ/zslayer-pc-port-tools` (`C:\CLLL\.calisma\vw-harness`).  
**Tarih:** 2026-11-25

---

## 0. `vw-harness` Mevcut Araç Özeti

| Araç (yol) | Ne yapar? | Kritik Not / Eksik |
|---|---|---|
| `README.md` | Kurulum, kurallar, `harness/run-world.sh` kullanım özeti | `local.env.example` ile makine yolları (`GAME_EXE`, `VICTORY_CAPTURES`, `VICTORY_SAVE_BACKUPS`) ayrılır. |
| `harness/world-run.sh` | Tek giriş noktası: `run.lock` ile tek koşu kilidi + OBS açıksa gerçek GPU koşusunu engeller (`VICTORY_ALLOW_OBS=1` hariç) | `VICTORY_CONFIG` ve `WORLD_RUN_HOLDS_LOCK=1` verip `run-world.sh`'ı çağırır. |
| `harness/run-world.sh` | Ana koşu sarmalayıcı: `--warm`, `--save <ad>`, `--save-as <ad>`, `--route <ad>`, `--keys`, `--settled-seconds` / `--heavy-seconds`, `--presentmon`, `--timing`, `--frames`, `--no-record` | `--warm` çağrısında `--jobs` geçirmez (tüm çekirdekleri kullanır). `VICTORY_KEYS_AFTER` sentinelini set eder (`L177-184`), ancak **`vw-pr10` bunu desteklemiyor**. |
| `harness/save-snapshot.mjs` | `<data_dir>/<title>/saves/<user>/<config>` klasörünü `save-as` / `restore` ile yönetir; geri yüklemeden önce `before-runs/` altına yedekleyip dosya sayısı + bayt doğrulaması yapar | Yalnızca `saves/<user>/<config>` kopyalar; ilk kurulum/legal bayraklarını tutan `storage` klasörünü kopyalamaz. |
| `harness/route-keys.mjs` | `harness/routes/<ad>.keys` dosyasını (`# heavy-seconds:` / `# settled-seconds:` + `<sn>:<tuş>[:<hold>]`) `VICTORY_KEYS` dizesine çevirir | `vw-pr10` yalnız `<sn>:<tuş>` (sabit 200 ms) okur; `:hold` ve `+sn` desteklemez (`vw-pr10/crates/victory-hle/src/modules/gameinput.rs:1206-1238`). |
| `harness/settled-watch.mjs` & `heavy-watch.mjs` | Logda `draws+dropped >= 1000` (ve `settled` için 3 ardışık kare `fps >= 1.5`) görünce süreyi başlatır, `--after-file` oluşturur ve süre bitince `taskkill /F` ile kapatır | Yalnızca bu koşunun PID'ini hedefler (`run-process.ps1 -Strict`). |
| `harness/run-queue.sh` | Kuyruk dosyasındaki (`<tag> <snapshot|lane> [bayraklar]`) deneyleri sırayla çalıştırır, `--max-seconds` (varsayılan 1500 sn) ve `--dry-run` sunar, `<kuyruk>.result.md` üretir | Hata alan satır sonraki koşuyu durdurmaz. |
| `harness/summarize-run.mjs` | Log + PresentMon CSV + `screen.mp4` luma analizinden FPS medyan/min/max, siyah ekran süresi, reddedilen pipeline ve sürücü hatalarını özetler | **RAM zirvesi (`ram_mb` / `peak_commit_mb`) ölçümü yok.** |

---

## 1. Otomatik Tuş / Pad Girdisi

| Yöntem | Nasıl yapılıyor? | Repo / Kanıt | Bizde var mı? | Güvenlik / Kalite Riski |
|---|---|---|---|---|
| **Ekran/render durumuna göre otomatik boot (`VICTORY_FAST_BOOT=1` + `VICTORY_SKIP_CUTSCENES=1`)** | Kare başına render-target bind sayısı `>= 100` olup 3 ardışık kare sürünce ("dünyaya girildi") durur (`VICTORY_FAST_BOOT_MAX_S=420` fallback). Öncesinde her 2 karede bir `A`, 8 karede bir `START` basar ve menü frame-pacing uyumasını kaldırır. | [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort) — `runtime/victory/crates/victory-core/src/fastboot.rs`, `runtime/victory/crates/victory-hle/src/modules/gameinput.rs` | **Yok** (`vw-pr10` ve `vw-harness` içinde yok) | Orta: ilk açılışta yasal/kurulum ekranlarını kullanıcı onayı olmadan tıklayabilir (`docs/RUNTIMES.md`: "testing only"). Fiziksel pad çakışmasına dikkat. |
| **Zamanlayıcılı otomatik `A` (`VICTORY_AUTO_INPUT=1`)** | İlk 120 sn boyunca her ~2 sn'de bir `A` tuşu enjekte eder (odak gerektirmez). | [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort) (`gameinput.rs`), [vitutu1/victory-perf](https://github.com/vitutu1/victory-perf) (`crates/victory-hle/src/modules/gameinput.rs`) | **Yok** | Düşük-Orta: kör zamanlama olduğu için oyunun yavaş açıldığı koşularda yanlış menüye basabilir veya ilk kurulumu onaylayabilir. |
| **Canlı dosya tabanlı girdi (`VICTORY_PRESS_FILE` / `XCL_PRESS_FILE`)** | Dış script (veya ajan) metin dosyasına `"A"`, `"DOWN DOWN A"` veya stick token yazar; runtime her token'ı 150 ms basıp dosyayı boşaltır. | [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort) (`gameinput.rs`, `scripts/play-victory.ps1`), [xcl-dev/xcl](https://github.com/xcl-dev/xcl), [DesignWithTommy/Neonbridge](https://github.com/DesignWithTommy/Neonbridge) (`src/runtimes.rs`) | **Yok** | Düşük: log/ekran durumunu izleyen dış harness ile birleştirilince en esnek yöntem; dosya yolu repo dışında tutulmalı. |
| **Aralık / periyodik / basılı tutma `VICTORY_KEYS`** | `VICTORY_KEYS=30:Enter,12-280/4:Enter,285-300:W` biçimiyle aralık, `/N` periyot ve uzun basma destekler. | [Neo-AzTeK/Causeway](https://github.com/Neo-AzTeK/Causeway) (`docs/debugging.md`) | **Yok** (`vw-pr10` `gameinput.rs:1206-1238` yalnız `<sn>:<Tuş>`, 200 ms) | Düşük: sadece `diag` derlemede çalışır. Not: [PastiiJay/klbf-lab](https://github.com/PastiiJay/klbf-lab) (`docs/klbf/results.md`) `VICTORY_KEYS` varken fiziksel pad'in yalnız pause'a ulaşabildiğini raporlar. |
| **Sentinel sonrası göreli tuşlar (`VICTORY_KEYS_AFTER`)** | `settled-watch.mjs` sahne oturunca `keys-after` dosyası yazar; `+sn:Tuş:süre` girişleri bu andan itibaren sayar. | [ZSlayerHQ/zslayer-pc-port-tools](https://github.com/ZSlayerHQ/zslayer-pc-port-tools) (`harness/run-world.sh:177-184`, `settled-watch.mjs:214-220`) | **Kısmen:** `vw-harness`'te **var**, `vw-pr10` runtime'da **yok** | Düşük: deterministik kamera/hareket rotaları için ideal; runtime tarafına eklenmesi küçük bir yama. |
| **Doğrudan pad tuş takvimi (`XCL_PRESS_AT`)** | `XCL_PRESS_AT=14:A,18.5:DOWN,20:A` — klavye eşlemesi yerine doğrudan pad buton adları. | [xcl-dev/xcl](https://github.com/xcl-dev/xcl) (`crates/xcl/src/hle/gameinput.rs`) | **Yok** (bizde klavye→pad tablosu var) | Düşük. |

---

## 2. Belli Bir Kayıttan (Save / Snapshot) Başlama

| Yöntem | Nasıl yapılıyor? | Repo / Kanıt | Bizde var mı? | Güvenlik / Kalite Riski |
|---|---|---|---|---|
| **Kayıt dizini ve doğrulamalı snapshot (`save-snapshot.mjs`)** | Kayıtlar `<data_dir>/<TITLEID>/saves/<user>/<config>` altında. `save-as` ile temiz checkpoint alınır; `restore` öncesi mevcut klasör `before-runs/` altına yedeklenip dosya+bayt doğrulanır. | `vw-harness/harness/save-snapshot.mjs:8-154`, `run-world.sh:237-244` | **Var** (`vw-harness`) | Düşük: yedek doğrulanmadan silme yapmaz. **Uyarı:** `storage` klasörünü kapsamaz; ilk kurulum/legal ekranı tekrar çıkabilir. |
| **İlk kurulum + legal atlayan `saves`+`storage` kopyası** | İlk autosave öncesi durdurulmuş bir koşunun `saves` ve `storage` klasörleri yeni `VICTORY_DATA_DIR/<TITLEID>/` altına kopyalanır; kurulum/yasal ekranlar geçilmiş sayılır. Autosave varsa "Continue" ile checkpoint'e girilir (otomatik değil, menüde Enter/A gerekir). | `vw-pr10/README.md:70-72` (ve `myokex/victory-amd`) | **Var** (manuel / `VICTORY_DATA_DIR` ile) | Düşük: kayıt dosyaları asla git reposuna eklenmemeli (asset policy). |
| **Yerleşik kayıt slotları (`VICTORY_SAVE_SLOT=<ad>` + `victory saves`)** | `<data>/<TITLEID>/slots/<ad>/{storage,saves}` yapısıyla hem `storage` hem `saves` birlikte izole edilir; `create`, `copy`, `delete` (`trash/`'a taşır). | [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort) — `runtime/victory/crates/victory-core/src/saves.rs`, `apps/victory/src/main.rs` | **Yok** | Düşük: path traversal doğrulaması (`validate`) ve çöp kutusu koruması var. |
| **Yasal onay korumalı data kopyalama (`--data-from`)** | `cargo xtask smoke --data-from <klasör>` kurulumu bitmiş data klasörünü kopyalar; taze klasörde `--accept-terms` olmadan otomatik tuş basmayı reddeder. | [Neo-AzTeK/Causeway](https://github.com/Neo-AzTeK/Causeway) (`docs/debugging.md`) | **Yok** | Çok düşük (güvenlik artıran koruma). |

---

## 3. İntro / Logo / Legal Atlama ve Doğrudan Boot Bayrakları

| Yöntem | Nasıl yapılıyor? | Repo / Kanıt | Bizde var mı? | Güvenlik / Kalite Riski |
|---|---|---|---|---|
| **Bink video atlama (`VICTORY_SKIP_MOVIES=1` / `all`)** | `bink*.dll` importlarını (`BinkSetWillLoop`, `BinkWait`, `BinkNextFrame`, `BinkClose`) loader'da sarar. Döngüsüz logo/intro videolarında ilk tick'te `BinkGoto(u32::MAX)` ile son kareye zıplar; oyunun kendi kapanış/ses akışı bozulmaz. **Release derlemede de çalışır.** | [ka1qux/BRVIctory-ka1qux](https://github.com/ka1qux/BRVIctory-ka1qux) — `crates/victory-hle/src/modules/bink.rs`, `apps/victory/src/run.rs` | **Yok** | Düşük: bellek ve frame buffer düzeni gerçek Bink'te kalır; `1` modu döngülü arka plan videolarına dokunmaz (`all` onları da bir kez son kareye atar). |
| **Fast Boot + Ara Sahne Atlama (`VICTORY_FAST_BOOT=1` + `VICTORY_SKIP_CUTSCENES=1`)** | Menü frame-pacing'i kapatır + dünyaya girilince 25 sn daha `A` basarak açılış ara sahnesini geçer. | [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort) (`fastboot.rs:51-57,96-102`) | **Yok** | Orta: gerçek zamanlama/ses senkronu test ediliyorsa kapatılmalı. |
| **Doğrudan "boot-to-save" (menüsüz yükleme) flag'i** | İncelenen hiçbir repoda (Marvisak, fxceless, myokex, ScarlettPort, Causeway, XCL, soldsawn) oyunun iç menü durum makinesini atlayıp doğrudan save yükleyen bir komut satırı/env flag'i **bulunamadı**. | Tüm izlenen repolar | **Yok** | — (Oyun EXE'sini yamalama/bellek yazma gerektireceğinden asset/güvenlik politikasına aykırı olur). |

---

## 4. Tekrarlanabilir Ölçüm (Sabit Sahne, FPS/RAM Kaydı, Koşu Kuyruğu)

| Yöntem | Nasıl yapılıyor? | Repo / Kanıt | Bizde var mı? | Güvenlik / Kalite Riski |
|---|---|---|---|---|
| **Saniyelik FPS + RAM + CPU CSV dökümü (`VICTORY_FRAMETIME_DUMP=1`)** | Her ~1 sn özet aralığında `<cache>/frametime.csv` (veya verilen yol) dosyasına `fps`, `ms_per_frame`, `cpu_prepare_ms`, `gpu_ms`, `draws`, `dropped`, **`ram_mb`**, **`cpu_pct`** yazar; başlıkta donanım ve tüm `VICTORY_*` bayraklarını kaydeder. | [skornuta/victory-fork](https://github.com/skornuta/victory-fork) — `crates/victory-gpu/src/frametime.rs`, `host.rs`, `apps/victory/src/run.rs` | **Yok** | Çok düşük: sadece sayısal sayaçlar yazar (oyun verisi içermez). |
| **Harness koşu kuyruğu + PresentMon + yerleşmiş sahne saati** | `run-queue.sh` + `world-run.sh` + `--save` + `--settled-seconds` + `--presentmon` + `summarize-run.mjs` (siyah ekran luma kontrolü dahil). | `vw-harness/harness/run-queue.sh`, `run-world.sh`, `settled-watch.mjs`, `summarize-run.mjs` | **Var** (`vw-harness`) | Düşük. Eksik: `ram_mb` kaydı yok; `run-world.sh` içine PowerShell/Node ile periyodik WorkingSet/Commit örnekleyici eklenebilir. |
| **Baseline karşılaştırmalı smoke testi (`cargo xtask smoke`)** | 300 sn katılımsız koşu, `report.json` + 60 satırlık `digest.md`, `--baseline <report.json>` ile delta raporu, `VICTORY_SURVIVE_REMOVAL=1`. | [Neo-AzTeK/Causeway](https://github.com/Neo-AzTeK/Causeway) (`docs/debugging.md`) | **Yok** | Düşük. |
| **Commit RAM zirvesi izleyen smoke (`scripts/smoke.ps1`)** | `XCL_RUN_SECONDS` + `XCL_ACTIVITY=1` ile koşup logdan `peak_commit_mb`, `removals`, `crashes` çıkarır ve `manifest.json` yazar. | [xcl-dev/xcl](https://github.com/xcl-dev/xcl) (`scripts/smoke.ps1`) | **Yok** | Düşük. |

---

## 5. Yükleme RAM Zirvesini Düşürme Hileleri

| Yöntem | Nasıl yapılıyor? | Repo / Kanıt | Bizde var mı? | Güvenlik / Kalite Riski |
|---|---|---|---|---|
| **Ayrı `warm` sürecini düşük `--jobs` ile önceden çalıştırma** | `victory.exe warm --jobs 2` (veya `4`) ayrı süreçte çalıştırılıp kapatılır; böylece oyun belleği ile paralel shader çevirici/derleyici RAM'i aynı anda tepe yapmaz. (Dikkat: aynı derleme profili ve aynı shader flag'leriyle çalıştırılmalı, aksi halde cache anahtarı tutmaz.) | `vw-pr10/apps/victory/src/main.rs:90-98`, `vw-pr10/README.md:38-51` | **Var** (`vw-pr10` destekliyor; `vw-harness --warm` `--jobs` geçirmiyor) | Çok düşük: en güvenli ve en etkili yöntem. |
| **RT resolve & ağır probe shader'larını kapatma (`VICTORY_RT_RESOLVE=0`, `VICTORY_ACTIVATE_RAYS=0`)** | `VICTORY_BVH_BUILDER` kapalıyken ilk dünya karesinde kuyruğa giren ~1.000 `CS_RaytracingResolve` programını ve tek başına 1.3+ GB RAM (eski AMD sürücüde 47 GB) + 85 sn yiyen `CS_ScreenProbes_ActivateRays` derlemesini tamamen atlar. | `vw-pr10/crates/victory-gpu/src/pipeline/make.rs:68-97` | **Var** (`vw-pr10` içinde hazır, **varsayılan kapalı değil — açık!** `=0` verilmeli) | Çok düşük: `BVH_BUILDER` zaten kapalıyken görsel kayıp yok, yükleme RAM'i ve süresi ciddi düşer. |
| **Çalışma zamanı pipeline worker sayısını sınırlama (`VICTORY_PIPELINE_WORKERS=<n>`)** | Sürücü derleyicisi paralel çalıştığında her thread yüzlerce MB–GB RAM tüketir. Worker sayısını `1` veya `2` yapmak yükleme RAM zirvesini doğrudan keser. (`XCL_DXC_THREADS=N` / `XCL_PIPELINE_THREADS=N` XCL'deki karşılığı.) | [rpatty-dev/PercyScarlett](https://github.com/rpatty-dev/PercyScarlett) (`pipeline/make.rs`), [skornuta/victory-fork](https://github.com/skornuta/victory-fork) (`pipeline/make.rs`), [crt4ever/DumbFloridaMoron](https://github.com/crt4ever/DumbFloridaMoron) | **Yok** (`vw-pr10/crates/victory-gpu/src/pipeline/make.rs:547` `(cores/2).clamp(2,8)` sabit) | Düşük: yükleme biraz uzar; çok yavaşlarsa 30 sn oyun GPU watchdog'u tetiklenebilir (`VICTORY_PIPELINE_WAIT_MS` / watchdog'a dikkat). |
| **Grafik + compute ön-derleme (`translate --cache --pipelines --jobs N`)** | Upstream `Marvisak/VIctory` grafik pipeline'larını da çevrimdışı önbellekler (`vw-pr10`'daki `warm` yalnız shader + compute pipeline yapar). | [Marvisak/VIctory](https://github.com/Marvisak/VIctory) (`apps/victory/src/prebuild.rs`) | **Kısmen** (`vw-pr10` `translate --pipelines` ve `warm` var, upstream `prebuild.rs` yok) | Düşük. |
| **Mip düşürme / isteğe bağlı mip / 1080p (`VICTORY_TEXTURE_DROP_MIPS=1\|2`, `VICTORY_LEVELS_ON_DEMAND=1`, `VICTORY_RESOLUTION=1920x1080`)** | En büyük N doku seviyesini atar veya 1080p render ister; VRAM ve sistem RAM kopyalarını GB'larca düşürür. | [soldsawn/victory-nvidia-8gb](https://github.com/soldsawn/victory-nvidia-8gb) (`README.md`, `fxceless/victory-fork#610`), [rpatty-dev/PercyScarlett](https://github.com/rpatty-dev/PercyScarlett) (`docs/PROLOGUE.md`), [JozefDeTremmerie/CCS03](https://github.com/JozefDeTremmerie/CCS03) | **Yok** | Orta (kalite): [MaDeInCCCP2/victory-fork](https://github.com/MaDeInCCCP2/victory-fork) (`docs/soldsawn-fork-analysis.md`) konuk verisi attığı için "temiz görsel/FPS kıyaslamasında" reddeder; yalnızca düşük RAM/VRAM stabilitesi için uygundur. |
| **Watchdog susturma (`VICTORY_NO_WATCHDOG=1` / `VICTORY_WATCHDOG=0`)** | Düşük worker sayısıyla yavaş derlerken oyunun 30 sn stall watchdog'unun süreci kapatmasını önler. | [ZigZagBoi1/ScarlettPort](https://github.com/ZigZagBoi1/ScarlettPort) (`docs/RUNTIMES.md`), [rpatty-dev/PercyScarlett](https://github.com/rpatty-dev/PercyScarlett) (`CHANGELOG.md`) | **Yok** | Düşük-Orta: gerçek bir kilitlenmeyi gizleyebilir; harness'ın `--max-seconds` / `--settled-timeout` zaman aşımıyla birlikte kullanılmalı. |

---

## Bizim İçin Önerilen Koşu Akışı (vw-pr10 + vw-harness)

1. **Tek seferlik düşük RAM'li önbellek ısıtma (`warm`):** Oyunu başlatmadan önce ayrı süreçte, testle aynı derleme ve shader bayraklarıyla `target\diag\victory.exe warm --jobs 2` çalıştırıp kapat (oyun belleğiyle derleyici RAM'i çakışmasın).
2. **Yükleme RAM'ini düşüren hazır bayrakları her koşuda aç:** `vw-pr10`'da zaten mevcut olan `VICTORY_RT_RESOLVE=0` ve `VICTORY_ACTIVATE_RAYS=0` değerlerini ortamda ver (`BVH_BUILDER` kapalıyken ~1000 gereksiz RT resolve + 1.3 GB'lık probe derlemesini atlar).
3. **Temiz `saves` + `storage` fikstürü hazırla:** Bir kez kurulum/legal ekranlarını geçip istediğin checkpoint'te (ör. ilk oynanabilir oda) durdur; `save-snapshot.mjs save-as <ad>` ile kaydet ve `storage` klasörünün de `data_dir` altında sabit kaldığını doğrula.
4. **Hızlı port edilecek 2 küçük runtime yaması (en yüksek getiri):**
   - `ka1qux/BRVIctory-ka1qux`'tan `VICTORY_SKIP_MOVIES=1` (`bink.rs`) → logo/intro videolarını saniyelere indirir.
   - `rpatty-dev/PercyScarlett` / `skornuta/victory-fork`'tan `VICTORY_PIPELINE_WORKERS=2` (`pipeline/make.rs:547`) + `VICTORY_FRAMETIME_DUMP=1` (`ram_mb` + `fps` CSV) → yükleme RAM zirvesini kontrol eder ve otomatik kaydeder.
5. **Menü geçişi için `VICTORY_KEYS` / `VICTORY_FAST_BOOT`:** Kısa vadede `vw-pr10` ile uyumlu `<sn>:Enter` listesiyle `--save <ad>` kullan; orta vadede `ScarlettPort`'tan `VICTORY_FAST_BOOT=1` (ekran/bind durumuna duyarlı) veya `ZSlayerHQ`'nun `VICTORY_KEYS_AFTER` sentinel desteğini `gameinput.rs`'e ekle.
6. **Ölçüm penceresini `settled-watch` ile sabitle:** `harness/world-run.sh --save <ad> --settled-seconds 45 --presentmon --tag <deney>` ile sahne gerçekten oturduktan (`draws+dropped >= 1000`, 3 kare `fps >= 1.5`) sonra sabit süreli ölçüm al.
7. **Toplu A/B karşılaştırması için `run-queue.sh`:** Değişiklikleri `run-queue.sh --max-seconds 900 <kuyruk>` ile sıraya koy; `summarize-run.mjs` çıktısını (FPS, PresentMon, siyah ekran) ve `frametime.csv` içindeki `ram_mb` zirvesini birlikte kıyasla.

---

## 5 Satırlık Özet

1. `vw-harness` zaten güçlü bir otomasyon iskeleti (`world-run.sh`, doğrulamalı `save-snapshot.mjs`, `settled-watch.mjs`, `run-queue.sh`, `summarize-run.mjs`) sunuyor; ancak `vw-pr10` runtime'ında `VICTORY_KEYS_AFTER` ve RAM kaydı eksik.
2. Kod değiştirmeden hemen kazanç için: `victory warm --jobs 2` ayrı süreçte çalıştırılmalı ve `vw-pr10`'da zaten var olan `VICTORY_RT_RESOLVE=0` + `VICTORY_ACTIVATE_RAYS=0` açılmalı.
3. Diğer repolardaki en yüksek getirili açılış hızlandırmaları `ka1qux/BRVIctory-ka1qux`'taki `VICTORY_SKIP_MOVIES=1` (Bink kancası) ve `ZigZagBoi1/ScarlettPort`'taki render-durumuna duyarlı `VICTORY_FAST_BOOT=1` + `VICTORY_SAVE_SLOT`.
4. Yükleme RAM zirvesini kesmek ve ölçmek için en temiz yamalar `VICTORY_PIPELINE_WORKERS=<n>` (`PercyScarlett`/`skornuta`) ve saniyelik `ram_mb` + `fps` CSV yazan `VICTORY_FRAMETIME_DUMP=1` (`skornuta/victory-fork`).
5. Hiçbir repoda doğrudan "boot-to-save" flag'i yok; en hızlı tekrarlanabilir akış: önceden `warm --jobs 2` → sabit `saves`+`storage` snapshot → `VICTORY_SKIP_MOVIES=1` + otomatik `Enter`/`FAST_BOOT` → `settled-watch` + `run-queue.sh`.
