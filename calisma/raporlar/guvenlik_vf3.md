# Güvenlik İncelemesi (3): `fxceless/victory-fork` (`310c6eb..465b94f5`)

Tarih: 2026-11-25  
Kapsam: Salt okunur kod, bağımlılık ve `build.rs` incelemesi (derleme, test veya çalışma ağacı değişikliği yapılmadı; yalnız `git fetch` + `git log/diff/show`).  
İncelenen aralık: `310c6eb..465b94f5` (8 commit, 47 dosya).  
Onaylanan tam SHA: `465b94f54e2f31abd3624cf336697a57a2d6a083` (`origin/main`, 2026-11-25 16:19 UTC).

---

## 1. Özet Hüküm

| Hedef | Commit Aralığı | Onaylanan Tam SHA | Karar |
| --- | --- | --- | --- |
| `fxceless/victory-fork` (`origin/main`) | `310c6eb` → `465b94f5` (8 commit, 47 dosya) | `465b94f54e2f31abd3624cf336697a57a2d6a083` | **GÜVENLİ** |

Hiçbir kırmızı bayrak (ağ/telemetri, süreç/komut çalıştırma, registry yazma, yeni dış crate, `git=` bağımlılık, başka süreçlere erişim, yönetici yetkisi veya gömülü ikili) bulunmadı. Yeni eklenen `crates/victory-shader/build.rs` ve `windows-sys` feature'ları (`Win32_System_EventLog`, `Win32_System_Diagnostics_ToolHelp`) satır satır incelendi ve güvenli doğrulandı.

---

## 2. Kırmızı Bayrak Taraması

- **`Cargo.toml` / `Cargo.lock`:** `Cargo.lock` **değişmedi**. Yeni crate, `git=` veya `path=` bağımlılığı yok. Tek değişiklik `crates/victory-gpu/Cargo.toml:30`: mevcut workspace `windows-sys` bağımlılığına 2 feature eklendi: `Win32_System_Diagnostics_ToolHelp` (`spread.rs` için) ve `Win32_System_EventLog` (`drvwatch.rs` için).
- **`build.rs`:** 1 yeni dosya: `crates/victory-shader/build.rs:1-65`. Yalnız `CARGO_MANIFEST_DIR/src/**/*.rs` dosyalarını okuyup 128-bit FNV-1a özeti ve `"VICTORY_*"` sabitlerini toplar; stdout'a yalnız `cargo:rerun-if-changed`, `cargo:rustc-env=VICTORY_SHADER_SOURCE_HASH` ve `cargo:rustc-env=VICTORY_SHADER_SWITCHES` basar. Dosya yazma, ağ veya dış komut yok.
- **`proc-macro` / `rust-toolchain` / `.github/workflows`:** Değişmedi / eklenmedi.
- **Ağ API'si (`reqwest`/`ureq`/`hyper`/`TcpStream`/`UdpSocket`/`WinHTTP`/`WinINet`/HTTP URL) ve telemetri:** **YOK.** `net.rs` değişmedi. Diff'teki tek URL `README.md` içindeki GitHub referansıdır (`https://github.com/myokex/victory-amd`). `profile.rs:1656-1681` `SymInitializeW` çağrısına açıkça yalnız `current_exe().parent()` vererek `_NT_SYMBOL_PATH` üzerinden sembol sunucusuna ağ isteği gitmesini engeller.
- **`Command::new` / PowerShell / cmd / `CreateProcess` / `ShellExecute`:** **YOK** (ne üretimde ne testlerde).
- **Dosya sistemine repo/config dışı yazma:**
  - `crates/victory-gpu/src/profile.rs:351-352,2720`: Yalnız `diagnostics` derlemesinde ve `VICTORY_PROFILE=1` açıkken (varsayılan kapalı) `%TEMP%\victory_profile_<pid>.txt` (+ `.folded`, `.part` üzerinden atomik `rename`) veya `VICTORY_PROFILE_OUT` yoluna metin profili yazar.
  - `apps/victory/src/translate.rs` (`translate --warm`): Yalnız `cfg.cache_subdir("shaders")` altındaki mevcut shader önbelleğine yazar.
  - Diğer `fs::write` / `create_dir_all` / `remove_dir_all` çağrıları yalnız `#[cfg(test)]` içindedir (`crates/victory-hle/src/modules/files.rs`, `crates/victory-gpu/src/profile/tests.rs`).
- **Registry yazma:** **YOK.**
- **İkili / blob / base64 / unsafe indirme:** **YOK.** (`crates/victory-gpu/src/shader_cache.rs` içindeki `include_bytes!("shader_cache.rs")` yalnız kendi kaynak kodunu cache anahtarına katar.)

---

## 3. Özel İnceleme Noktaları (Dosya:Satır)

### 3.1. Driver Watch (`VICTORY_DRIVER_WATCH`) — `crates/victory-gpu/src/drvwatch.rs`
- **API:** `windows_sys::Win32::System::EventLog::EvtSubscribe` (`wevtapi.dll`, `drvwatch.rs:23,84-93`).
- **Kapsam:** `"System"` olay günlüğü kanalına yalnız gelecek olaylar için (`EvtSubscribeToFutureEvents`) abone olur. XPath filtresi yalnız ekran sürücüsü sağlayıcılarıyla sınırlıdır: `nvlddmkm`, `amdkmdag`, `amdwddmg`, `igfx`, `igfxn`, `Display` (`drvwatch.rs:73-79`).
- **Yalnız okuma mı?** **Evet.** Üretim akışında (`start()`, `on_event()`) hiçbir şey yazmaz; `on_event` (`drvwatch.rs:57-62`) olay içeriğini bile çözümlemez, yalnız `ERRORS` atomik sayacını artırır, `TRIPPED` bayrağını `true` yapar ve `tracing::error!` yazar. (`ReportEventW` çağrısı yalnız `#[cfg(test)] #[ignore]` altındaki manuel testtedir: `drvwatch.rs:154`.)
- **Yönetici yetkisi ister mi?** **Hayır.** Windows'ta `"System"` kanalı standart kullanıcı tarafından okunabilir. Abonelik başarısız olursa hata verip çıkmaz, `tracing::warn!` ile devam eder (`drvwatch.rs:95-99`).
- **Başka etkisi:** İlk ekran sürücüsü hatasında `drvwatch::tripped()` `true` döner ve `VICTORY_INDIRECT_WINDOW` gibi deneysel hızlandırma yollarını o koşu boyunca devre dışı bırakır (`crates/victory-gpu/src/draw.rs`).
- **Varsayılan:** **AÇIK** (`VICTORY_DRIVER_WATCH=0` ile kapatılır).

### 3.2. Çekirdek Afinitesi ve Güç Kısma (`VICTORY_FAST_CORES`, `VICTORY_DISK_CORES`, `VICTORY_EFFICIENCY_MODE`, `VICTORY_SPREAD`)
- **`VICTORY_FAST_CORES` (`crates/victory-gpu/src/framelog.rs:181-244`):**
  - `GetSystemCpuSetInformation` (`framelog.rs:185,188`) ile en yüksek `EfficiencyClass` değerine sahip çekirdekleri okur ve `SetThreadSelectedCpuSets(GetCurrentThread(), ...)` (`framelog.rs:238`) ile yalnızca katmanın kendi yardımcı iş parçacıklarını (`victory-submit`, `victory-late-wait`, `victory-relay`) hızlı çekirdeklere yönlendirir.
  - **Sistem ayarı değiştirmez;** yalnız mevcut iş parçacığı için süreç-içi zamanlayıcı tercihidir, yönetici istemez. Varsayılan: **AÇIK** (`VICTORY_FAST_CORES=0` ile kapatılır).
- **`VICTORY_DISK_CORES=slow` (`crates/victory-hle/src/modules/dstorage.rs`, `framelog.rs:258-266`):**
  - Aynı `SetThreadSelectedCpuSets(GetCurrentThread(), slow, ...)` çağrısını DirectStorage worker iş parçacıkları için yapar (en az 8 yavaş mantıksal çekirdek varsa). Sistem ayarı değiştirmez. Varsayılan: **KAPALI**.
- **`VICTORY_EFFICIENCY_MODE` (`crates/victory-hle/src/runtime.rs:190-210`):**
  - `SetProcessInformation(GetCurrentProcess(), ProcessPowerThrottling, ...)` ile yalnız bu süreçte `PROCESS_POWER_THROTTLING_EXECUTION_SPEED` kapatır (pencere arka plandayken EcoQoS yavaşlatmasını önler). Sistem ayarı değiştirmez. Varsayılan: **EcoQoS kapalı** (`VICTORY_EFFICIENCY_MODE=1` ile eski davranışa dönülür).
- **`VICTORY_SPREAD` (`crates/victory-gpu/src/spread.rs:29-210`):**
  - `CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)` ile **yalnız `entry.th32OwnerProcessID == GetCurrentProcessId()`** filtresiyle kendi sürecinin iş parçacıklarını tarar (`spread.rs:70-81`), `OpenThread(THREAD_QUERY_INFORMATION | THREAD_SET_INFORMATION, 0, id)` (`spread.rs:105`) ve `SetThreadGroupAffinity` (`spread.rs:142`) ile oyunun sıkışık iş parçacığı gruplarına yedek çekirdek maskesi ekler. Başka süreçlere dokunmaz, sistem ayarı değiştirmez. Varsayılan: **KAPALI** (`VICTORY_SPREAD=1` veya `alt`).

### 3.3. Örnekleyici Profiler (`VICTORY_PROFILE`) — `crates/victory-gpu/src/profile.rs`
- **Derleme ve çalışma kapısı:** Yalnız `#[cfg(all(windows, feature = "diagnostics"))]` altında derlenir (`crates/victory-gpu/src/lib.rs`) ve yalnız `VICTORY_PROFILE=1` verildiğinde başlar (`profile.rs:348`). Varsayılan: **KAPALI**.
- **Başka süreçlere dokunuyor mu?** **Hayır.** İş parçacığı keşfi yalnız kendi sürecimizin `KNOWN_THREADS` tablosu ve `NtGetNextThread(GetCurrentProcess(), ...)` (`profile.rs:1050-1057`) ile yapılır. `OpenThread(ACCESS, 0, id)` (`profile.rs:975,1081`) yalnız bu kimliklere açılır. `ReadProcessMemory(GetCurrentProcess(), ...)` (`profile.rs:823,1463`) yalnız kendi sürecinin yığın ve TEB belleğini okur. `OpenProcess` yoktur.
- **Debug privilege (`SeDebugPrivilege`) istiyor mu?** **Hayır.** `AdjustTokenPrivileges` çağrısı yoktur; standart kullanıcı yetkisiyle kendi sürecinin iş parçacıklarını `THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION | SYNCHRONIZE` erişimiyle (`profile.rs:103-104`) örnekler.
- **DLL ve sembol güvenliği:** `LoadLibraryExA(c"dbghelp.dll", null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32)` (`profile.rs:1620-1624`) yalnız `System32` içinden yükler (DLL hijacking korumalı). `SymInitializeW` yalnız `current_exe().parent()` klasörünü arar, ağdaki sembol sunucularına başvurmaz (`profile.rs:1656-1681`).

### 3.4. Diğer Düşük Seviye Mekanikler (Bilgi Amaçlı)
- **`crates/victory-gpu/src/plaincopy.rs` (`VICTORY_COPY_PLAIN=1`, varsayılan KAPALI):** Kendi süreç içi oyun belleğini `global_asm!` `rep movsb` + Vectored Exception Handler ile kopyalar; okuma hatası olursa çekirdek `ReadProcessMemory` yoluna düşer.
- **`crates/victory-loader/src/console_insn.rs` (`VICTORY_CLZERO_LOOP15`, varsayılan AÇIK):** 15 baytlık `clzero` döngüsünü kendi süreç belleğinde tek `write_bytes` çağrısıyla öykünür (`0` ile eski satır-satır davranışa döner).

---

## 4. Commit Bazlı `VICTORY_*` Anahtarları ve Varsayılanları

| Commit | Anahtar (`VICTORY_*`) | Varsayılan (`465b94f5`) | Açıklama / Kapsam |
| --- | --- | --- | --- |
| `8dd30ab` | `VICTORY_VIEW_GUARD` | **AÇIK** (`0` kapatır) | Fiziksel sayfanın tüm view'larını birlikte korur (`watch.rs`). |
| `8dd30ab` / `a26859f` | `VICTORY_PAGE_WRITES` | **AÇIK (`guards`=1)** (`0` yalnız sayar; `1`/`slots`/`all`) | `8dd30ab`'de kapalıydı, `a26859f`'de varsayılan `guards` (açık) oldu (`watch.rs`). |
| `8dd30ab` | `VICTORY_FULL_CARD_PASS` | **AÇIK** (`0` kapatır) | Reddedilen doku slotlarını geride tutup arkasındaki slotlara devam eder (`draw.rs`). |
| `8dd30ab` | `VICTORY_DSTORAGE_RETRIES` | **12** (`0` tekrar yok) | Başarısız sayfa-hedefli DirectStorage okumalarını yeniden dener (`dstorage.rs`). |
| `8dd30ab` | `VICTORY_SHADER_CACHE_KEY` | **source** (`exe` eski mod) | Shader cache anahtarını derlenen exe yerine çevirici kaynağına bağlar (`shader_cache.rs`). |
| `8dd30ab` / `a26859f` | `VICTORY_CARD_ROOM_FOR_BUFFERS` | **AÇIK** (`0` kapatır) | `8dd30ab`'de kapalıydı, `a26859f`'de **AÇIK** oldu (`shadow.rs`). |
| `8dd30ab` | `VICTORY_LEVELS_ON_DEMAND` | **KAPALI** (`1` açar) | Doku mip seviyelerini talep üzerine VRAM'e ayırır (`tiles.rs`). |
| `8dd30ab` / `a26859f` | `VICTORY_LEVELS_MIN_KB` | **Varsayılan eşik** (sayısal) | `LEVELS_ON_DEMAND` için en küçük seviye eşiği (`tiles.rs`). |
| `8dd30ab` | `VICTORY_TILES_TEST_CARD` | **KAPALI** (`1` kartta test) | Yalnız `#[cfg(test)]` altında gerçek GPU'da tiles testi (`tiles.rs`). |
| `8dd30ab` | `VICTORY_MERGE_RACE` | **KAPALI** (`1` açar) | Copy-back birleştirme yarışı sayacı (`shadow.rs`). |
| `8dd30ab` | `VICTORY_REFRESH_CHECK` | **KAPALI** (`1` açar) | Refresh doğrulama sayacı (`shadow.rs`). |
| `8dd30ab` | `VICTORY_WARM_SECONDS` | **Yok (sınırsız)** | `translate --warm` için saniye sınırı (`translate.rs`). |
| `a26859f` | `VICTORY_HEAP_SYNC` | **AÇIK** (`0` kapatır) | Bayt kopyalanan descriptor kayıtlarını yansıtır (`descriptor.rs`). |
| `a26859f` | `VICTORY_VIEW_OWN_COPY` | **AÇIK** (`0` kapatır) | Kendi kopyası dışına taşan view'lara kendi baytlarını verir (`descriptor/host.rs`). |
| `a26859f` | `VICTORY_CARD_ROOM_FOR_TARGETS_MB` | **3072 MB** (`0` kapalı, maks 8192) | Render target'lar için VRAM tamponu ayırır (`resource.rs`). |
| `a26859f` | `VICTORY_FIRST_HALF` | **AÇIK** (`0` kapatır) | Yalnız reddedilen shader'larda ilk-yarı interpolasyon çiftleri (`lower.rs`). |
| `9376fd6` | `VICTORY_HEAP_SYNC_TABLES` | **KAPALI** (`1` açar) | Draw-table slotlarını descriptor sync'ten ayırır (`descriptor/host.rs`). |
| `9376fd6` | `VICTORY_CARD_FETCH_INFLIGHT` | **KAPALI** (`1` açar) | Karttan veri çekmeyi uçuş (in-flight) içinde yapar (`host.rs`). |
| `9376fd6` | `VICTORY_CARD_FETCH_INFLIGHT_MB` | **64 MB** | In-flight fetch tampon sınırı (`host.rs`). |
| `9376fd6` | `VICTORY_VIEW_ASK` / `VICTORY_GUARD_ASK` | **KAPALI** (`1` eski davranış) | Bilinen korumalı view belleği için write-watch sormayı atlar (`watch.rs`). |
| `f85101b` | `VICTORY_INDIRECT_CARD_BUDGET` | **AÇIK** (`0` kapatır) | Dolaylı çizim kart bütçesi (`host.rs`). |
| `f85101b` | `VICTORY_QUEUE2_REFETCH` | **AÇIK** (`0` kapatır; yalnız `QUEUE2` ile) | İkinci kuyruk yeniden çekme kontrolü (`host.rs`). |
| `f85101b` | `VICTORY_FILL_WAIT` | **KAPALI** (`1` eski bekleme) | Doldurma beklemesini atlar (`host.rs`). |
| `f85101b` | `VICTORY_MERGE_RUNS` | **512** (`1..65536`) | Birleştirme çalışma sayısı sınırı (`host.rs`). |
| `f85101b` | `VICTORY_MERGE_GAP` | **0** (`0..64`) | Birleştirme boşluk eşiği (`shadow.rs`). |
| `f85101b` | `VICTORY_STAGING_POOL` | **AÇIK** (`0` kapatır) | Doku yükleme staging havuzu (`texture.rs`). |
| `f85101b` | `VICTORY_ASYNC_SUBMIT` | **KAPALI** (`1` açar) | Gönderimleri ayrı `victory-submit` iş parçacığına alır (`queue.rs`). |
| `f85101b` | `VICTORY_QUEUE2` | **KAPALI (`0`)** (`1`/`2`/`3`) | Kopyalar için ayrı D3D12 copy kuyruğu (`queue2.rs`). |
| `f85101b` | `VICTORY_QUEUE2_LANDING_MB` | **96 MB** | `QUEUE2` landing buffer tavanı (`queue2.rs`). |
| `f85101b` | `VICTORY_INDIRECT_WINDOW_CHECK` | **KAPALI** (`1` açar) | Dolaylı çizim penceresi doğrulaması (`indirect.rs`). |
| `f85101b` | `VICTORY_PROFILE` (+ `OUT`, `HZ`=1000, `EVERY`=30, `AFTER`=0, `FOR`, `DEPTH`=96, `STACK_KB`=128, `MAX_THREADS`, `BUSY`, `INLINE`=1, `REUSE`=1, `BUDGET`, `JITTER_US`=250, `THREADS`) | **KAPALI** (yalnız `diagnostics` + `=1`) | Süreç-içi örnekleyici profiler (`profile.rs`). |
| `a253706` | `VICTORY_DRIVER_WATCH` | **AÇIK** (`0` kapatır) | System olay günlüğünde ekran sürücüsü hatalarını izler (`drvwatch.rs`). |
| `a253706` | `VICTORY_FRAME_LOG` | **Diagnostics'te AÇIK, release'de KAPALI** (`0`/`1`) | Kare süresi ve uzun kare dökümü (`framelog.rs`). |
| `a253706` | `VICTORY_CARD_METER` | **Diagnostics'te AÇIK, release'de KAPALI** (`0`/`1`) | GPU timestamp list bazlı süre ölçümü (`cardmeter.rs`). |
| `a253706` | `VICTORY_CAPTURE_WORLD_EVERY` | **8** (`>0`) | Dünya karesi yakalama sıklığı (`capture.rs`). |
| `a253706` | `VICTORY_HEAP_SYNC_PAGE` | **AÇIK** (`0` kapatır) | Değişen sayfa kayıtlarını tek kilitte senkronlar (`descriptor.rs`). |
| `a253706` | `VICTORY_EFFICIENCY_MODE` | **KAPALI** (EcoQoS kapalı; `1` eski mod) | Süreç için Windows EcoQoS yavaşlatmasını kapatır (`runtime.rs`). |
| `a253706` | `VICTORY_COPY_PLAIN` | **KAPALI** (`1` açar) | `rep movsb` + VEH ile yerinde bellek kopyalama (`plaincopy.rs`). |
| `a253706` | `VICTORY_BVH_HOLD` | **KAPALI** (`1` açar; reddedilmiş deney) | 64 BVH programını tutar (`pipeline.rs`). |
| `a253706` | `VICTORY_CARD_REHASH_RUNS` | **KAPALI** (`1`/`alt`) | Kart yeniden hash deneyi (`host.rs`). |
| `a253706` / `3973179` | `VICTORY_REFRESH_ONCE`, `VICTORY_BACK_OWN_VIEW`, `VICTORY_ASYNC_PRESENT`, `VICTORY_VIEW_KNOWN` | **KAPALI** (`1`/`alt`) | Deneysel hızlandırmalar; `3973179`'da `alt` (20 sn aç/kapa) modu eklendi. |
| `0b0074e` | `VICTORY_FAST_CORES` | **AÇIK** (`0` kapatır) | Katman yardımcı iş parçacıklarını hızlı çekirdeklere yönlendirir (`framelog.rs`). |
| `0b0074e` | `VICTORY_DISK_CORES` | **KAPALI** (`slow` açar) | DirectStorage iş parçacıklarını yavaş çekirdeklere yönlendirir (`dstorage.rs`). |
| `3973179` | `VICTORY_CLZERO_LOOP15` | **AÇIK** (`0` kapatır) | 15 baytlık `clzero` döngüsünü topluca öykünür (`console_insn.rs`). |
| `3973179` | `VICTORY_JOIN_PARTS` (+ `JOIN_MOST`=4, `JOIN_LISTS`=48) | **KAPALI** (`1`/`alt`) | Ardışık kuyruk parçalarını tek host gönderiminde birleştirir (`queue.rs`). |
| `3973179` | `VICTORY_BACK_OWN_CHECK` | **KAPALI** (`1` açar) | `BACK_OWN_VIEW` doğrulama sayacı (`shadow.rs`). |
| `3973179` | `VICTORY_CARD_METER_PIPELINES` | **KAPALI** (`1` açar) | Pipeline bazlı GPU timestamp ölçümü (`cardmeter.rs`). |
| `3973179` | `VICTORY_WINDOW_JUMP` | **KAPALI** (`1`/`alt`) | Hatırlanan adımda pencere açma deneyi (`draw.rs`). |
| `3973179` | `VICTORY_SPREAD` (+ `VICTORY_SPREAD_SPARE`) | **KAPALI** (`1`/`alt`) | Sıkışık oyun iş parçacığı gruplarına yedek çekirdek ekler (`spread.rs`). |
| `465b94f` | `VICTORY_INDIRECT_GATHER` | **KAPALI** (`1`/`alt`) | Dağınık dolaylı kayıtları liste başında tek geçişte sınırlar (`draw.rs`). |

---

## 5. Varsayılan Davranışı Değiştiren Önemli Değişiklikler

Hiçbir `VICTORY_*` ortam değişkeni ayarlanmadan `465b94f5` derlenip çalıştırıldığında `310c6eb`'ye göre değişen varsayılan davranışlar:

1. **Bellek ve VRAM senkronizasyonu (AÇIK):** `VICTORY_VIEW_GUARD`, `VICTORY_PAGE_WRITES=guards`, `VICTORY_FULL_CARD_PASS`, `VICTORY_DSTORAGE_RETRIES=12`, `VICTORY_HEAP_SYNC`, `VICTORY_HEAP_SYNC_PAGE`, `VICTORY_VIEW_OWN_COPY`, `VICTORY_CARD_ROOM_FOR_BUFFERS`, `VICTORY_CARD_ROOM_FOR_TARGETS_MB=3072`, `VICTORY_STAGING_POOL`, `VICTORY_INDIRECT_CARD_BUDGET`, `VICTORY_FIRST_HALF` ve korumalı view belleğinde write-watch atlama (`VICTORY_VIEW_ASK`/`GUARD_ASK` kapalı) artık varsayılan olarak etkindir.
2. **Sürücü izleme (AÇIK):** `VICTORY_DRIVER_WATCH` açık; Windows `"System"` olay günlüğünde ekran sürücüsü hatası görülürse loglar ve deneysel çizim pencerelerini kapatır.
3. **Çekirdek tercihi ve EcoQoS (AÇIK):** `VICTORY_FAST_CORES` hibrit işlemcilerde katman yardımcı iş parçacıklarını P-core'lara yönlendirir; `runtime.rs` süreç için Windows Efficiency Mode (EcoQoS) yavaşlatmasını kapatır (`VICTORY_EFFICIENCY_MODE=1` ile geri alınabilir).
4. **Shader cache anahtarı ve `clzero` öykünmesi (AÇIK):** Shader önbelleği exe yerine çevirici kaynak koduna göre anahtarlanır (`build.rs`); 15 baytlık `clzero` döngüleri topluca işlenir (`VICTORY_CLZERO_LOOP15`).
5. **Diagnostics derlemesinde ek gözlemleme (AÇIK):** `VICTORY_FRAME_LOG` ve `VICTORY_CARD_METER` yalnız `diagnostics` feature'ıyla derlenen ikililerde varsayılan açıktır (release derlemede kapalı).
6. **Kapalı bırakılan büyük deneyler:** `VICTORY_LEVELS_ON_DEMAND`, `VICTORY_ASYNC_SUBMIT`, `VICTORY_QUEUE2`, `VICTORY_COPY_PLAIN`, `VICTORY_PROFILE`, `VICTORY_JOIN_PARTS`, `VICTORY_BACK_OWN_VIEW`, `VICTORY_SPREAD`, `VICTORY_INDIRECT_GATHER` varsayılan olarak **KAPALI**dır.
