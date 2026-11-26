# `victory` (`vw-pr10`) RAM Kaynakları ve NVIDIA/RAM Aday Kataloğu

- **İncelenen kod tabanı:** `C:\CLLL\.calisma\vw-pr10` (`ZSlayerHQ` `02ea29c` + PR #10 `e7c05277`)
- **Hedef donanım & gözlem:** RTX 3070 Ti Laptop 8 GB VRAM, 16 GB RAM, Win 11. Yüklemede `victory.exe` private commit **7.8 → 11.9 GB** (working set tepe 5.9 GB), VRAM **6.2–6.3 GB**.

---

## 1. BÖLÜM — `vw-pr10` RAM Tüketicileri (Kod Kanıtlı)

### 1.1. Ana Bellek Tüketicileri

| Kaynak | Kod Kanıtı (`dosya:satır`) | Ne Kadar Ayırır (Sabit / Dinamik) | Küçülten Ayar (`VICTORY_*` / CLI) | Küçültmenin Riski |
|---|---|---|---|---|
| **1. Guest (konsol) fiziksel/sanal belleği** | `crates/victory-gpu/src/console.rs:22-27`, `crates/victory-hle/src/modules/xmem.rs:87-166`, `xmem.rs:397-458`, `xmem.rs:504,776,794-800`, `xmem.rs:922-961` | **Dinamik (oyun talebi):** `SeriesX`: **13.5 GiB** (10 GiB GPU + 3.5 GiB std), `SeriesS`: **7,776 MiB (~7.59 GiB)**. `XMemAllocatePhysicalPages` tek seferde 6.1 GiB'a kadar `SEC_RESERVE` section açar (`xmem.rs:411-427`), `map()`/`View::map()` anında `VirtualAlloc(..., MEM_COMMIT)` ile commit eder (`xmem.rs:799`). `XMemGetWorkingSetStatistics` oyuna tüm süreç `PagefileUsage`'ını bildirir (`xmem.rs:935`). | `--console series-s` (CLI) | Series S'te PR #10 (`VICTORY_PRESENT_PLANES`) gerekir; yine de yüklemede oyun + katman ~12 GB commit'e çıkıyor. |
| **2. Host shadow tamponları ve 4 MiB bellek pencereleri (`L0` PC RAM)** | `crates/victory-gpu/src/host.rs:7203-7251` (`shared_buffer`, `D3D12_MEMORY_POOL_L0`), `crates/victory-gpu/src/shadow.rs:28-80, 234-248, 2957-3029` | **Dinamik (büyük):** Tampon dışı erişimler varsayılan **4 MiB** hizalı pencere (`WINDOW = 4 << 20`, `shadow.rs:2957`) olarak PC RAM'inde ayrılır. Kod içi ölçüm: oynanışta **782–861 pencere = 3.2–3.55 GB** (çoğu hiç okunmayan sıfırlar; toplam L0 paylaşımlı ~5.4 GB, `shadow.rs:234-235, 2962, 2998`). `Card` açıkken bile `Card::staging` PC RAM'de kalır (`shadow.rs:88, 246`). | `VICTORY_WINDOW_KB` (`64..4096`, vars. `4096`), `VICTORY_WINDOW_TAILS=1` (vars. `0`), `VICTORY_SWEEP=1` (vars. `1`, <2 GiB boş RAM'de 2 sn idle sweep: `host.rs:147-162`) | `WINDOW_KB` küçüldükçe aynı bellek birden çok pencerede yinelenebilir (ek kopya/CPU); `WINDOW_TAILS=1` kısa→uzun kuyruk geçişinde yeni host tamponu açar (`shadow.rs:3003-3021`). |
| **3. Doku staging (upload) tamponları (`HEAP_TYPE_UPLOAD` = PC RAM)** | `crates/victory-gpu/src/texture.rs:35-63, 1211-1287`, `crates/victory-gpu/src/host.rs:5965-6010`, `crates/victory-gpu/src/draw.rs:1699-1701, 1766-1786` | **Dinamik (yükleme zirvesi):** Her başlık dokusu için ayrı `UPLOAD` tamponu (`STAGING` haritası) açılır; `SWEEP` ile `ram_idle_ms` (2–10 sn) sonra atılır. Kare başı akış varsayılan **192 MiB/kare**, ancak `VICTORY_TEXTURE_CATCHUP=1` ile ilk 60 sn (`GATE_S=60`) **512 MiB/kare** (`draw.rs:1776-1786`). | `VICTORY_TEXTURE_CATCHUP=0`, `VICTORY_TEXTURE_STAGE_MB` (vars. `192`), `VICTORY_TEXTURE_STAGE_MS` (vars. `250`), `VICTORY_TEXTURE_GATE_S=0`, `VICTORY_LAZY_TEXTURES=1` | Doku akışı yavaşlar; ilk sahnede dokular daha geç (siyah/eksik) gelir. |
| **4. Pipeline derleyici işçileri & RT shader RAM zirvesi** | `crates/victory-gpu/src/pipeline/make.rs:68-97, 543-548` | **Dinamik (dev zirve):** `(logical_cores/2).clamp(2, 8)` işçi (12700H'de **8 paralel derleyici**). `CS_ScreenProbes_ActivateRays` tek başına **~1.3 GB** (85 sn) derleyici RAM zirvesi (`make.rs:73`); ~1.000 `CS_RaytracingResolve` ilk dünya karesinde kuyruğa girer (`make.rs:82-86`). | `VICTORY_ACTIVATE_RAYS=0` (vars. `1`), `VICTORY_RT_RESOLVE=0` (vars. `1`) | `BVH_BUILDER=0` iken görsel kayıp yok (RT vuruşu zaten üretilmiyor); `BVH_BUILDER=1` açılırsa `RT_RESOLVE=0` zaten etkisiz kalır. |
| **5. Snapshot havuzu (`POOL`) & `before` kopyaları** | `crates/victory-gpu/src/shadow.rs:819-856` (`give_back`), `shadow.rs:1444-1459` | **Sınırlandırılmış sabit:** `POOL` en fazla 8 adet `Vec<u8>`, toplam **`KEPT = 512 MiB`** (`shadow.rs:845`) PC RAM'de kalıcı tutar (env yok). `VICTORY_KEEP_SNAPSHOTS=1` açılırsa ek **`256 MiB`** resident RAM (`shadow.rs:1457`). | `VICTORY_KEEP_SNAPSHOTS=0` (vars. zaten `0`); `POOL` (512 MiB) env ile küçültülemez (kod değişikliği gerekir). | `KEEP_SNAPSHOTS=1` açılırsa `RELAY` kapanır ve 16 GB'ta OOM riski doğar. |
| **6. Kart tamponları (`VICTORY_CARD_BUFFERS*`) ve VRAM→RAM taşması** | `crates/victory-gpu/src/shadow.rs:137-160, 215-224, 249-254`, `crates/victory-gpu/src/resource.rs:1365-1403` | **Ayarla değişen (VRAM + dolaylı RAM):** Kart tamponları vars. **1024 MiB** (+ "before" ikizleriyle ~2 GB'a kadar VRAM, %80 `card_for` sınırı). Doku bütçesi DXGI **%90** (`FLOOR=2 GiB`). 8 GB VRAM dolunca WDDM kaynakları PC paylaşımlı RAM'ine taşır (`resource.rs:1362-1363`). | `VICTORY_CARD_BUFFERS_MB` (vars. `1024`), `VICTORY_CARD_BUFFERS=0`, `VICTORY_TEXTURE_BUDGET_MB`, `VICTORY_TEXTURE_ROOM_PERCENT` (vars. `90`), `VICTORY_DISPLAY_MODE=1920x1080` | `CARD_BUFFERS=0` veya düşük değer CPU/PCIe kopya yükünü ve kare süresini artırır; düşük doku bütçesi dokuları eksik bırakır. |
| **7. Shader tablosu (`SHADERS`) & pipeline kütüphanesi (`host-pipelines.bin`)** | `crates/victory-gpu/src/pipeline.rs:59-119`, `crates/victory-gpu/src/pipeline/library.rs:26-48, 79-91, 365-393` | **Monoton artan:** `SHADERS` tüm konteyner ve çevrilmiş DXIL'leri **hiç bırakmadan** RAM'de tutar (`pipeline.rs:78`). `host-pipelines.bin` (warm'da 76 MB, uzun koşuda 250–1,490 MB) `VICTORY_LIBRARY_MAP=1` ile mmap edilir (commit değil working set), `VICTORY_LIBRARY_TRIM=1` ile boşta kırpılır (`library.rs:365-389`). | `VICTORY_LIBRARY_MAP=1` (vars. `1`), `VICTORY_LIBRARY_TRIM=1` (vars. `1`). `SHADERS` için env yok. | `LIBRARY_MAP=0` tüm kütüphaneyi (1.4 GB'a kadar) private RAM'e kopyalar; `LIBRARY_HANDLE=0` device-removed yapar. |
| **8. Scratch havuzu & readback tamponları** | `crates/victory-gpu/src/host.rs:1476-1580` (`SPARE_MOST`), `host.rs:5926-5963` | **Sınırlı sabit:** Scratch upload tamponları `SCRATCH_FIRST=256 KiB` / `SCRATCH_CHUNK=4 MiB`, boşta havuz tavanı **`SPARE_MOST = 128 MiB`** (`host.rs:1482`). Readback tamponları (`HEAP_TYPE_READBACK`) yalnız kart geri-kopya/capture anında. | `VICTORY_SCRATCH_POOL=1` (vars. `1`), `VICTORY_LIST_POOL=1` (vars. `1`) | Kapatılırsa her komut listesi kendi 4 MiB upload tamponunu ayırır (57×4 MiB/kare; `host.rs:1489`). |
| **9. Ham adres dizini (`memory.rs`)** | `crates/victory-gpu/src/memory.rs:59-68, 95-115` | **Koşullu:** Yalnız raw 40-bit adres okuyan shader (BVH/RT) çalışırsa `directory` + `faults` `shared_buffer` açar; en fazla `COPIES=1023` pencereyi (~4 GiB tavan) kalıcı tutar. | `VICTORY_BVH_BUILDER=0` (vars. `0`) | BVH kapalıyken tetiklenmez; açılırsa 16 GB RAM'i hızla doldurur. |
| **10. Oyun dosyası / paket okuma (`dstorage.rs`, `run.rs`)** | `crates/victory-hle/src/modules/dstorage.rs:117-151, 222-228, 336, 425, 874-896`, `apps/victory/src/run.rs:22-38` | **Küçük / akışlı:** `.rpf`/`.ttpf` dosyaları **tamamı belleğe okunmaz ve mmap edilmez**; `FileExt` ile parça parça okunur. Eşzamanlı staging sınırı `STAGING = 32 MiB` (`dstorage.rs:336`; oyun ayarlar), worker başı geçici `vec![0; size]` bounce/inflate tamponu. `exe` (`114.7 MB`) `run.rs:37-38`'de `drop()` edilir. | Yok (oyun `set_staging_buffer_size` belirler). | — |
| **11. Log / trace / teşhis tamponları** | `crates/victory-gpu/src/host.rs:253-255`, `crates/victory-gpu/src/copystat.rs:700-710`, `crates/victory-gpu/src/capture.rs:15,261` | **İhmal edilebilir (kapalıysa):** `TRAIL` 96 satır (`<100 KiB`). `VICTORY_CAPTURE` kapalıyken 0; açıkken 4K/1080p readback + PNG tamponları. | `VICTORY_TRAIL=0`, `VICTORY_MEMORY_MAP=0`, `VICTORY_CAPTURE` (unset) | `TRAIL=0` device-removed teşhis izini kapatır; RAM kazancı ihmal edilebilir. |

---

### 1.2. Bellekle İlgili Tüm `VICTORY_*` Ortam Değişkenleri (`vw-pr10`)

| Değişken | Varsayılan | Dosya:Satır | Kısa Etki |
|---|---|---|---|
| `VICTORY_CARD_BUFFERS` | `1` (açık) | `crates/victory-gpu/src/shadow.rs:139` | Büyük başlık tamponlarının VRAM kopyalarını açar/kapatır |
| `VICTORY_CARD_BUFFERS_MIN_MB` | `1` | `crates/victory-gpu/src/shadow.rs:158` | VRAM'e alınacak en küçük tampon boyutu (MiB) |
| `VICTORY_CARD_BUFFERS_MB` | `1024` | `crates/victory-gpu/src/shadow.rs:158` | Kart tamponları toplam VRAM tavanı ("before" ikizleri hariç, MiB) |
| `VICTORY_CARD_WINDOWS_MB` | `0` (kapalı) | `crates/victory-gpu/src/shadow.rs:252` | Tampon dışı pencerelerin VRAM kopya tavanı (MiB) |
| `VICTORY_WINDOW_KB` | `4096` (4 MiB) | `crates/victory-gpu/src/shadow.rs:2973` | Uzunluklu pencere ızgara boyutu (KiB, 64..4096, 2'nin kuvveti) |
| `VICTORY_WINDOW_TAILS` | `0` (kapalı) | `crates/victory-gpu/src/shadow.rs:3012` | Uzunluksuz (root view) pencerelerini 256 KiB adımlı kuyruk yapar |
| `VICTORY_KEEP_SNAPSHOTS` | `0` (kapalı) | `crates/victory-gpu/src/shadow.rs:1446` | CPU tarafı snapshot'ları RAM'de tutar (`RELAY`'i kapatır) |
| `VICTORY_KEEP_SNAPSHOTS_MB` | `256` | `crates/victory-gpu/src/shadow.rs:1457` | `KEEP_SNAPSHOTS=1` için RAM tavanı (MiB) |
| `VICTORY_INPLACE` | `0` (kapalı) | `crates/victory-gpu/src/inplace.rs:90` | Kopyasız D3D12 heap (RAM'i fiziksele kilitler — 16 GB'ta riskli) |
| `VICTORY_INPLACE_MB` | `4096` | `crates/victory-gpu/src/inplace.rs:98` | `INPLACE` kilitli RAM üst sınırı (MiB) |
| `VICTORY_TEXTURE_BUDGET_MB` | unset (DXGI %90) | `crates/victory-gpu/src/resource.rs:1371` | Bindless doku kopyaları sabit VRAM üst sınırı (MiB) |
| `VICTORY_TEXTURE_ROOM_PERCENT` | `90` (`50..97`) | `crates/victory-gpu/src/resource.rs:1398` | Yeni doku kopyası için DXGI VRAM bütçesi yüzdesi |
| `VICTORY_LAZY_TEXTURES` | `1` (açık) | `crates/victory-gpu/src/resource.rs:1254` | Doku host kopyasını ilk kullanımda ayırır |
| `VICTORY_SWEEP` | `1` (açık) | `crates/victory-gpu/src/host.rs:189` | Saniyede bir boşta pencere, staging ve dokuları bırakır |
| `VICTORY_TEXTURE_SWEEP` | `1` (açık) | `crates/victory-gpu/src/resource.rs:1637` | Boşta kalan doku host kopyalarını bırakır |
| `VICTORY_HEAP_TEXTURES` | `1` (açık) | `crates/victory-gpu/src/draw.rs:1692` | Bindless doku yükleme geçişini açar/kapatır |
| `VICTORY_TEXTURE_STAGE_MB` | `192` | `crates/victory-gpu/src/draw.rs:1700` | Kare başına bindless doku staging bütçesi (MiB) |
| `VICTORY_TEXTURE_STAGE_MS` | `250` | `crates/victory-gpu/src/draw.rs:1767` | Kare başına bindless doku staging süre sınırı (ms) |
| `VICTORY_TEXTURE_CATCHUP` | `1` (açık) | `crates/victory-gpu/src/draw.rs:1780` | Uzun kuyrukta staging'i 512 MiB / 1.5 s'ye çıkarır |
| `VICTORY_TEXTURE_GATE_S` | `60` | `crates/victory-gpu/src/draw.rs:362` | Catch-up yüksek staging bütçesi toplam süre ödeneği (sn) |
| `VICTORY_REUPLOAD_WHEN_FULL` | `1` (açık) | `crates/victory-gpu/src/draw.rs:414` | VRAM doluyken mevcut host dokularını yeniden yükler |
| `VICTORY_PLACED_FILLER` | `0` (kapalı) | `crates/victory-gpu/src/resource.rs:2334` | Placed doku anahtarında dolgu sözcüklerini korur (kopya artırır) |
| `VICTORY_VIEW_INLINE` | `1` (açık) | `crates/victory-gpu/src/descriptor.rs:394` | CBV/placed view kayıtlarını satır içi tutar (RAM tasarrufu) |
| `VICTORY_ROOT_VIEW_MB` | `u64::MAX` | `crates/victory-gpu/src/draw.rs:474` | Root SRV/CBV kopya uzunluğu üst sınırı (MiB) |
| `VICTORY_ROOT_UAV_MB` | `u64::MAX` | `crates/victory-gpu/src/draw.rs:1192` | Root UAV kopya/geri-kopya uzunluğu üst sınırı (MiB) |
| `VICTORY_DEPTH_TWINS` | `1` (açık) | `crates/victory-gpu/src/resource.rs:447` | Depth UAV ikiz doku mekanizması (`0` kapatır) |
| `VICTORY_ALIAS_COPY` | `2` (tam) | `crates/victory-gpu/src/resource.rs:1212` | Çakışan (aliased) kaynak kopya modu |
| `VICTORY_SCRATCH_POOL` | `1` (açık) | `crates/victory-gpu/src/host.rs:1494` | Liste scratch upload tamponlarını paylaşır (128 MiB havuz) |
| `VICTORY_LIST_POOL` | `1` (açık) | `crates/victory-gpu/src/list.rs:232` | D3D12 komut listelerini havuzda yeniden kullanır |
| `VICTORY_LIBRARY_MAP` | `1` (açık) | `crates/victory-gpu/src/pipeline/library.rs:81` | `host-pipelines.bin`'i mmap ile açar (`0`: RAM'e kopyalar) |
| `VICTORY_LIBRARY_HANDLE` | `1` (açık) | `crates/victory-gpu/src/pipeline/library.rs:90` | Eşlenen kütüphane dosya tutamacını açık tutar |
| `VICTORY_LIBRARY_TRIM` | `1` (açık) | `crates/victory-gpu/src/pipeline/library.rs:370` | Boşta eşlenen kütüphane sayfalarını working set'ten kırpar |
| `VICTORY_ACTIVATE_RAYS` | `1` (açık) | `crates/victory-gpu/src/pipeline/make.rs:77` | `0`: 66K komutlu shader'ın ~1.3 GB derleyici RAM zirvesini atlar |
| `VICTORY_RT_RESOLVE` | `1` (açık) | `crates/victory-gpu/src/pipeline/make.rs:94` | `0`: BVH kapalıyken ~1.000 RT resolve shader derlemesini atlar |
| `VICTORY_BVH_BUILDER` | `0` (kapalı) | `crates/victory-gpu/src/pipeline.rs:306` | `1`: BVH shader'larını ve `memory.rs` (4 GiB'a kadar) dizinini açar |
| `VICTORY_DISPLAY_MODE` | `3840x2160` | `crates/victory-gpu/src/dxgi.rs:230` | Oyuna bildirilen çözünürlük (render target/VRAM yükünü belirler) |
| `VICTORY_MEMORY_MAP` | `1` (açık) | `crates/victory-gpu/src/copystat.rs:487` | Diag'da periyodik adres haritası taraması |
| `VICTORY_TRAIL` | `1` (açık) | `crates/victory-gpu/src/host.rs:259` | Son 96 GPU olayı halkası |

---

### 1.3. 16 GB RAM'e Sığdırmak İçin İlk Denenecek 3 Tek-Değişkenli Ayar

1. **`VICTORY_WINDOW_KB=256`** *(veya `512`)*
   - **Tahmini kazanç:** **TAHMİN: 0.8–1.5 GB** private commit / `L0` sistem RAM tasarrufu *(ardından 2. adım olarak `VICTORY_WINDOW_TAILS=1` eklenirse **TAHMİN: +0.5–1.0 GB** daha)*.
   - **Neden:** Kod içi ölçümler (`shadow.rs:234-235, 2962, 2998`) 4 MiB hizalı pencerelerin oynanışta tek başına **3.2–3.55 GB** PC RAM'i tükettiğini ve bunun neredeyse yarısının hiç okunmayan sıfırlar olduğunu gösteriyor. `L0` paylaşımlı tamponlar doğrudan private commit'e girer.

2. **`VICTORY_ACTIVATE_RAYS=0`**
   - **Tahmini kazanç:** **~1.3 GB** (kodda belgelenmiş, `pipeline/make.rs:73`) derleyici RAM zirvesi tasarrufu *(yükleme sırasında `CS_ScreenProbes_ActivateRays` tetikleniyorsa doğrudan tepeyi keser; hemen ardından `VICTORY_RT_RESOLVE=0` ile ~1.000 RT shader'ın paralel derleme RAM yükünden **TAHMİN: 0.5–1.2 GB** daha kesilebilir)*.
   - **Neden:** `BVH_BUILDER=0` iken hiçbir görsel katkısı olmayan tek bir 66.501-komutlu compute shader, sürücü derleyicisinde 85 sn boyunca 1.3 GB RAM tüketiyor.

3. **`VICTORY_TEXTURE_CATCHUP=0`**
   - **Tahmini kazanç:** **TAHMİN: 0.4–1.0 GB** yükleme anı staging RAM (`HEAP_TYPE_UPLOAD`) ve WDDM VRAM→RAM taşma zirvesi tasarrufu.
   - **Neden:** Yükleme ekranından dünyaya geçerken >2.000 doku kuyruğa girdiğinde `draw.rs:1776-1786` kare başı staging bütçesini 192 MiB'den **512 MiB**'ye fırlatıyor. Bu `UPLOAD` tamponları PC RAM'inde ayrılıp `ram_idle_ms` (2–10 sn) boyunca `STAGING` haritasında tutuluyor ve aynı anda 8 GB VRAM'i hızla doldurarak WDDM'yi sistem RAM'ine taşmaya itiyor.

---

## 2. BÖLÜM — NVIDIA / RAM Aday Kataloğu

| Kimlik (Repo + Commit/PR/Env) | Ne Yapıyor | Hangi Soruna | Güvenlik İncelemesi | Durumu | Kaynak Rapor |
|---|---|---|---|---|---|
| `ZSlayerHQ/zslayer-pc-port-runtime` **PR #10** (`e7c05277`) | Series S 2-plane present compose + FSR4 stretch/pack stand-in + submit/dispatch bölme | Siyah ekran (Series S), TDR | **Yapıldı (GÜVENLİ)** | **Derlendi / Denendi** (Koşu 5 RAM eşiğinde durdu) | `guvenlik_pr10.md`, `STATUS.md`, `nvidia_siralama.md` |
| `fxceless/victory-fork` **`310c6eb..465b94f5`** (8 commit: `8dd30ab`..`465b94f`) | Submit thread, copy queue (`QUEUE2`), staging pool, `COPY_PLAIN`, `JOIN_PARTS`, `BACK_OWN_VIEW`, `INDIRECT_GATHER` (RTX 4070 4.5→10.3 fps) | FPS, CPU/GPU darboğazı | **Yapıldı (GÜVENLİ)** | **Derlendi** (`vf-465` diag hazır, henüz denenmedi) | `guvenlik_vf3.md`, `watcher.md`, `STATUS.md`, `nvidia_siralama.md` |
| `fxceless/victory-fork` **#413 env reçetesi** (`LATE_WAIT=3`, `WIDE_BACK=1`, `CARD_BUFFERS=1..1024`, `DISPLAY_MODE=1920x1080`) | RTX 3070 Ti 8 GB'ta 2.05→4.62 fps tekrarlanabilir bekleme/tampon ayar seti | FPS, VRAM | Yerel tabanda mevcut | **Denendi** (Koşu 4–5; 16 GB RAM duvarına takıldı) | `STATUS.md`, `nvidia_siralama.md` |
| `fxceless/victory-fork` **PR #261** / `Marvisak#1630` (`VICTORY_D3D12_ADAPTER` + High-Performance DXGI) | Hibrit/Optimus laptoplarda Intel iGPU yerine NVIDIA dGPU seçimi | Siyah ekran / yanlış GPU | Yapılmadı *(geçici çözüm: HKCU `GpuPreference=2` uygulandı)* | **Bulundu** (registry workaround **Kabul**) | `nvidia_siralama.md`, `STATUS.md` |
| `fxceless/victory-fork` **`VICTORY_LEVELS_ON_DEMAND=1`** (`8dd30ab`, `vf-465`) | Doku mip seviyelerini talep üzerine VRAM'e ayırır | RAM / 8 GB VRAM | **Yapıldı (GÜVENLİ)** (`guvenlik_vf3.md`) | **Derlendi** (`vf-465` içinde, henüz denenmedi) | `guvenlik_vf3.md`, `nvidia_siralama.md` |
| `Itsbatt/victory-unified` **`shadow.rs` 64 MiB snapshot havuz tavanı** | `shadow.rs` snapshot havuzunu 512 MiB'den 64 MiB'ye indirir (8 GB RTX 3070 50 dk testli) | RAM (16 GB) | Yapılmadı | **Bulundu** | `nvidia_siralama.md` |
| `soldsawn/victory-nvidia-8gb` **`09b95ca` (PR #799)** | NVIDIA Wave32 CBV 64 KiB clamp (`cbv_size`), `0x887A0001` device-removed önler | Çökme / NVIDIA TDR | Ön tarama temiz | **İncelendi** | `nvidia8gb.md`, `nvidia_siralama.md` |
| `soldsawn/victory-nvidia-8gb` **`4ae2d11`+`8f73867` (PR #432)** | Unmap edilen placed texture host kopyalarını kare başı anında serbest bırakır (kayıpsız) | RAM / 8 GB VRAM | Ön tarama temiz | **İncelendi** | `nvidia8gb.md`, `nvidia_siralama.md` |
| `soldsawn/victory-nvidia-8gb` / `fxceless` **`26600c6`+`0751197` (PR #854)** | NVIDIA Wave32 paired Wave64 compute lowering + >32 KiB mesh-pixel link (+230 shader) | Siyah ekran / eksik çizim / NVIDIA | Ön tarama temiz | **İncelendi** | `nvidia8gb.md`, `watcher.md`, `nvidia_siralama.md` |
| `soldsawn/victory-nvidia-8gb` **`b212918`+`31cb370` (PR #936)** & **`8654c67`+`0fdeee8` (PR #964)** | `Shadow` drop öncesi `SNAPS` flush (UAF) + copy queue fence device-loss yayılımı | Çökme / NVIDIA TDR | Ön tarama temiz | **İncelendi** | `nvidia8gb.md` |
| `soldsawn/victory-nvidia-8gb` **`b4ff304..68e6f6a` (PR #610, `VICTORY_TEXTURE_DROP_MIPS`)** | 2D dokuların en büyük 1–2 mip seviyesini ayırmaz (kayıplı; 8 GB VRAM taşmasını önler) | RAM / 8 GB VRAM | Ön tarama temiz | **İncelendi** (opsiyonel, vars. 0) | `nvidia8gb.md`, `nvidia_siralama.md` |
| `Belobug/VIctory-Belobug-AMD` **`8629211` (PR #45)** / `soldsawn` **`e5028d8`+`a0e6f89` (PR #922)** | Integer render target'larda `BlendEnable` kapatır (D3D12 `E_INVALIDARG` önler) | Eksik çizim / çökme | **Yapıldı (GÜVENLİ)** (`belobug.md`) | **İncelendi** (ilk port adayı) | `belobug.md`, `nvidia8gb.md`, `STATUS.md` |
| `Belobug/VIctory-Belobug-AMD` **`788ea46`** / `soldsawn` **`34a7ebd`+`bfbc2af` (PR #932)** | Dinamik descriptor slot arama önbelleği / imleç sarma (NVIDIA 65K sınırında 7–11 sn donma önler) | FPS / donma / NVIDIA | **Yapıldı (GÜVENLİ)** (`belobug.md`) | **İncelendi** | `belobug.md`, `nvidia8gb.md`, `STATUS.md` |
| `Belobug/VIctory-Belobug-AMD` **`ccab973`+`76387c2` (`VICTORY_HOIST_CLAMPS=7`)** | Indirect draw clamp dispatch'lerini ön listeye alır (GPU 225→72–80 ms iddiası) | FPS / GPU barrier | **Yapıldı (GÜVENLİ)** (`belobug.md`) | **İncelendi** | `belobug.md`, `STATUS.md` |
| `Belobug/VIctory-Belobug-AMD` **`0a32e89`+`e11fb06` (`VICTORY_SWEEP_TILES=2`)** | Geçersiz başlık belleği tile'larını bölge bazlı ve kilit dışında geri verir | RAM / VRAM şişmesi | **Yapıldı (GÜVENLİ)** (`belobug.md`) | **İncelendi** | `belobug.md`, `watcher.md`, `STATUS.md` |
| `Belobug/VIctory-Belobug-AMD` **`a0a4591` (PR #18)** | Paralel sparse copy-back + submission içi tekrar aralık atlama + tek kilit | FPS / CPU kopya | **Yapıldı (GÜVENLİ)** (`belobug.md`) | **İncelendi** | `belobug.md`, `watcher.md`, `STATUS.md` |
| `Belobug` **`4f45ab8`..`cd9f78c` (`VICTORY_AMBIENT`)** (ayrıca `Itsbatt`, `VItoriaBr`) | RT dolaylı ışık yerine sabit ortam ışığı + derinlik karartması | Karanlık görüntü (görsel) | **Yapıldı (GÜVENLİ)** (`belobug.md`) | **İncelendi** (isteğe bağlı) | `belobug.md`, `nvidia_siralama.md` |
| `Marvisak/VIctory` **`8a3d9759`**, **`ea812894`**, **`11e9a544`**, **`315cea18`** | Büyük upload buffer'dan texture staging havuzu, tek barrier batch, önceden heap yerleştirme | RAM / VRAM / FPS | Yapılmadı | **Bulundu** | `watcher.md`, `STATUS.md` |
| `Marvisak/VIctory` **PR #1799** | 22 sn `s_barrier` TDR düzeltmesi | NVIDIA TDR | Yapılmadı | **Bulundu** | `watcher.md` |
| `ZSlayerHQ/zslayer-pc-port-runtime` **PR #7** (RT pass atlama) & **PR #4** (staging havuzu) | RT yapıları yokken RT pass'lerini atla (~%20 hız) / texture staging havuzu | FPS / RAM | Yapılmadı | **Bulundu** | `watcher.md`, `nvidia_siralama.md` |
| `LooperSalty/AnyXboxX` **PR #216** (`VICTORY_STAGING_POOL_MB=1024`) | Index-picked doku staging tamponu yeniden kullanımı (RTX 5080 yükleme 2× hızlı) | RAM / FPS (yükleme) | Yapılmadı | **Bulundu** | `watcher.md`, `nvidia_siralama.md` |
| `JoshuaO22/victory-amd` **`003beef2` ("memory-gap")** | Remap kilidi 16 MB başı, xmem hazır sayfa atlama, 64 adres başı kilit | FPS / CPU kilit | Yapılmadı | **Bulundu** | `watcher.md` |
| `xcl-dev/xcl` **`08294413`**, **`83ad6a55`**, **PR #2345** | Bölünmüş dispatch scratch tavanı, `globallycoherent` azaltma, unmapped tile karantinası | RAM / NVIDIA TDR (mimari fikir) | Yapılmadı (farklı kod tabanı) | **Bulundu** | `watcher.md`, `nvidia_siralama.md` |
| `ThatRecompDev/five-star-recomp` **boş bellek tabanlı derleme kabulü** & **`--skip-pipelines`** | Boş RAM + sistem commit'ine göre derleme işçisi kısma (8 GB pay bırakma) | RAM (16 GB) (mimari fikir) | Yapılmadı (farklı kod tabanı) | **Bulundu** | `watcher.md` |
| `soldsawn` **`4e34eb0` (`VICTORY_SKIP_WC_COMPARE=1`)** & **default-flip commit'leri** | WC bellek copy-back karşılaştırmasını atlar / ölçüsüz varsayılan açma | Doğruluk riski | Ön tarama temiz | **Ret** (önerilmiyor) | `nvidia8gb.md` |
| `Belobug/VIctory-Belobug-AMD` **tam taban değişimi** | AMD RDNA4 16 GB/64 GB odaklı taban; Wave32'de çapraz-lane compute düşürür | NVIDIA uyumsuzluğu | **Yapıldı (GÜVENLİ)** | **Ret** (yalnız parça seçimi) | `belobug.md`, `STATUS.md` |
| `fxceless` **`VICTORY_ASYNC_PRESENT=1`** | Asenkron present (`nvwgf2umx.dll`'de çöküyor) | Çökme (NVIDIA) | **Yapıldı (GÜVENLİ)** | **Ret** (kapalı tutulmalı) | `nvidia_siralama.md`, `guvenlik_vf3.md` |
| Yerel `vw-pr10` bellek env seti (`WINDOW_KB`, `WINDOW_TAILS`, `ACTIVATE_RAYS=0`, `RT_RESOLVE=0`, `TEXTURE_CATCHUP=0`) | Mevcut tabanda sıfır kod değişikliğiyle RAM/derleyici/staging zirvesini düşürür | RAM (16 GB) / VRAM (8 GB) | Mevcut taban (GÜVENLİ) | **İncelendi** (tek-değişkenli deneme sırası hazır) | `AYARLAR.md`, §1.3 |

---

## 5 Satırlık Özet

1. `vw-pr10`'da 7.8→11.9 GB commit'in ana kaynakları: oyunun `XMemAllocatePhysicalPages` commit'i (Series S'te 7.59 GiB'a kadar), `L0` sistem RAM'indeki 4 MiB shadow pencereleri (3.2–3.55 GB), 8 paralel shader derleyicisinin RT derleme zirvesi (`ActivateRays` ~1.3 GB) ve 512 MiB/kare catch-up doku staging tamponlarıdır; `.rpf` dosyaları belleğe tam yüklenmez (32 MiB akışlı).
2. `shadow.rs` snapshot `POOL` tavanı sabit **512 MiB**'dir (env yok; `Itsbatt` bunu 64 MiB'ye çekmiştir), `SHADERS` tablosu ise çevrilen tüm shader'ları hiç bırakmadan RAM'de tutar.
3. 16 GB RAM'de sıfır kod değişikliğiyle ilk denenecek 3 tek-değişkenli ayar: (1) `VICTORY_WINDOW_KB=256`, (2) `VICTORY_ACTIVATE_RAYS=0`, (3) `VICTORY_TEXTURE_CATCHUP=0`.
4. Güvenlik incelemesi bitmiş en hazır kod adayları: `vf-465` (`LEVELS_ON_DEMAND=1` + hızlanma, zaten derli) ile `Belobug`'tan seçme portlar (`8629211` int RT blend, `788ea46` 65K slot önbelleği, `0a32e89+e11fb06` `SWEEP_TILES=2`).
5. RAM/8 GB VRAM için sonraki port önceliği `soldsawn` PR #432 (unmapped placed texture serbest bırakma) + PR #799 (NVIDIA CBV 64 KiB clamp) ve `Itsbatt`'ın `shadow.rs` 64 MiB havuz tavanıdır.
