# Victory-Fork Optimizasyon ve Ölçüm Planı (i7-12700H · 15.7 GB RAM · RTX 3070 Ti Laptop 8 GB)

Hedef klon: `fxceless/victory-fork` @ `2119b0e` (`C:\CLLL\.calisma\victory-fork`). NVIDIA 32-şeritli wave, Windows 11, 49 GB pagefile.

## 1. Ölçüm araçları (nasıl profil alınır)
- **Derleme ve temel log:** `cargo build --profile diag -p victory --features diagnostics` + `VICTORY_LOG=info` (`crates/victory-core/src/logging.rs:21`). 5 sn'de bir kare satırı basar: `fps, ms/frame (preparing: copies + textures; GPU; copying back MB), draws/frame` (`crates/victory-gpu/src/host.rs:5793`).
- **Bellek kırılımı:** `diag` derlemesinde `memory map` satırı varsayılan açıktır (`VICTORY_MEMORY_MAP=0` kapatır, `crates/victory-gpu/src/copystat.rs:487`). #94'te 24.3 GB private belleğin **10.9 GB'si OS VRAM commit'i (host dokular)**, 4.7 GB'si host bellek kopyaları (3.4 GB pencereler), 4.6 GB'si oyun belleği, 1.3 GB'si heap çıktı.
- **GPU zamanlama:** `VICTORY_GPU_TIMING=1` (`host.rs:1225`) her komut listesini ayrı zamanlar ve en yavaş komutları döker. **Kritik tuzak:** `VICTORY_LATE_WAIT` ve `VICTORY_RELAY`'i sessizce `0` yapar (`host.rs:1678`) — sadece GPU içi kırılım için kullan, FPS karşılaştırması için değil.
- **Tekrarlanabilir test:** Aynı kayıt + `VICTORY_KEYS=10:Enter,15:Enter` (`crates/victory-hle/src/modules/gameinput.rs:1208`) + görsel kontrol için `VICTORY_CAPTURE=<dir>` / `VICTORY_CAPTURE_FRAMES=...` (`crates/victory-gpu/src/capture.rs:15,25`).
- **Kare süresi nereye gidiyor?** Varsayılan modda **CPU + senkron bekleme ağırlıklı** (#413, #168: 488 ms'nin 196 ms'si hazırlık, 182 ms'si GPU, 66 ms'si geri kopya). Çözünürlük 4K→1080p GPU'yu sadece **%5** oynatır (#10); GPU zamanının **%93'ü compute**, ~3.200 adet `max=1` indirect draw (~12 µs/adet = 40/63 ms, #107, #10). Streaming'de doku hattı `MAPPING` kilidinde takılır (#178: medyan 20–203 ms, pik 4 sn). `VICTORY_BVH_BUILDER=1` açılırsa `memory::ready` 4.3 GB tarayıp kareyi 3.3 sn'ye çıkarır (#284).

## 2. Ayarlar (env) tablosu
| Değişken | Ne yapar | Varsayılan | Etki (Hız / Bellek / Kararlılık) | Konum |
|---|---|---|---|---|
| `VICTORY_LATE_WAIT` | `1..3`: GPU beklemeyi ve present'ı erteler | `0` (`RELAY` açıksa `3`) | **Hız ++** (−40..55%); oda duvarı kaybı riski (#168) | `crates/victory-gpu/src/host.rs:1681` |
| `VICTORY_RELAY` | `1`/`2`: geri kopyayı ayrı thread'de indirir (`2` seçici bekler) | kapalı | **Hız ++** (−40%), ama duvar/cilt/altyazı bozar (#89,#168), RAM pik (#321) | `crates/victory-gpu/src/host.rs:1778`, `shadow.rs:473` |
| `VICTORY_WIDE_BACK` | Geri kopya karşılaştırmasını çoklu thread'de yapar | `0` | **Hız +** (`LATE_WAIT=3` ile kopya-geri 66→28 ms, #413) | `crates/victory-gpu/src/shadow.rs:961` |
| `VICTORY_WIDE_REFRESH` | Bağımsız host kopya yenilemelerini paralelleştirir | `0` | **Hız +** (CPU hazırlık ~45–50 ms hedefi, #38) | `crates/victory-gpu/src/host.rs:5172` |
| `VICTORY_WIDE_MERGE` | Geri birleştirmeyi (merge) paralel çalıştırır | `0` | **Hız +** (CPU merge yükünü böler) | `crates/victory-gpu/src/host.rs:1524` |
| `VICTORY_CARD_BUFFERS_MIN_MB` / `_MB` | Kart VRAM'ine alınacak buffer eşiği / bütçesi | `16` / `512` MiB | `1` / `1024` **Hız ++**; 8 GB VRAM'de bütçeyi `512–1024` tut | `crates/victory-gpu/src/shadow.rs:154` |
| `VICTORY_TEXTURE_BUDGET_MB` | Host doku VRAM bütçesi (aşınca yenileme frenlenir) | VRAM'in %90'ı (min 2 GB) | **8 GB VRAM için kritik** (`3072–4096` önerilir, #89) | `crates/victory-gpu/src/resource.rs:1371` |
| `VICTORY_TEXTURE_SWEEP` | Boştaki host doku kopyalarını 5 sn / 4 kare sonra bırakır | `1` (açık) | **Bellek/VRAM korur** — `0` yapma | `crates/victory-gpu/src/resource.rs:1416` |
| `VICTORY_TEXTURE_NO_WAIT` | Remap kilidi tutuluyorsa dokuyu beklemeden kernel'dan okur | `0` | Streaming takılmasını siler (#178), ama kare başına doku erteleyebilir | `crates/victory-gpu/src/texture.rs:306` |
| `VICTORY_INDIRECT_WINDOW` | Ardışık indirect draw'ları pencere halinde clamp'ler | `0` | `main`'de nötr (#107); **PR #220 ile GPU'yu 2.5–3× hızlandırır** | `crates/victory-gpu/src/draw.rs:146` |
| `VICTORY_SHADER_OPT` | Çevrilmiş DXIL'de temizlik geçişleri (−%43 komut, −20 ms) | açık (tümü) | **Hız +**; siyah mozaik görürsen `0` dene (#310) | `crates/victory-shader/src/emit/mod.rs:188` |
| `VICTORY_SHADER_LOOP_LIMIT` | Shader döngü iterasyon üst sınırı (varsayılan 2²⁰) | `1048576` | `256` TDR'ye karşı güvenli orta yol (#3,#15); `32` saçı bozar (#62) | `crates/victory-shader/src/emit/mod.rs:137` |
| `VICTORY_WAVE_WIDE` | 64-şeritli bariyer formu (AMD RDNA3 için) | `0` | **NVIDIA 32-lane'de KAPALI (`0`) tut** (hang/çökme riski, README) | `crates/victory-shader/src/emit/mod.rs:382` |
| `VICTORY_WINDOW_KB` / `_TAILS` | Buffer dışı pencere boyutu / kuyruk kırpma | `4096` KiB / `0` | **Bellek kazandırmaz** (≤0.3 GB, `1024` daha kötü, #54) | `crates/victory-gpu/src/shadow.rs:2864,2903` |
| `VICTORY_INPLACE` / `_MB` | Oyun belleğini kopyalamadan GPU heap olarak açar | `0` / `4096` | Bellek etkisi minimal (#54); `LATE_WAIT` ile uyumsuz (`inplace.rs:79`) | `crates/victory-gpu/src/inplace.rs:90,98` |
| `VICTORY_KEEP_SNAPSHOTS` / `_MB` | CPU snapshot havuzunu elde tutar | `0` / `256` MiB | `RELAY` ile kapalı; düşük RAM'de açma | `crates/victory-gpu/src/shadow.rs:1436,1447` |
| `VICTORY_PIPELINE_WAIT_MS` | Arka plan pipeline derleme kare bekleme süresi | `500` ms | İlk ısınmada takılmayı azaltır (ör. `24` ms, README) | `crates/victory-gpu/src/pipeline/make.rs:101` |
| `VICTORY_DISPLAY_MODE` | Konsola bildirilen çözünürlük (`1920x1080`) | konsol varsayılanı | FPS'e etkisi ~%5 (#10), VRAM'i bir miktar rahatlatır | `crates/victory-gpu/src/dxgi.rs:230` |

## 3. Başkalarının ölçümleri (NVIDIA ve ≤16 GB RAM / 8 GB VRAM)
| Issue | Donanım | Ayarlar | Sonuç (FPS / ms / Bellek) | Sorun / Not |
|---|---|---|---|---|
| **#413** | **RTX 3070 Ti 8 GB** + R7 7700X, 32 GB | Varsayılan (`2119b0e`, 1080p) | 2.05 fps (488 ms: 196 haz / 182 GPU / 66 geri) | Device loss yok, CPU-bound |
| **#413** | **RTX 3070 Ti 8 GB** (aynı) | `RELAY=2` | 3.37 fps (297 ms: 190 / 70 / 159) | −%39.2 süre, device loss yok |
| **#413** | **RTX 3070 Ti 8 GB** (aynı) | `LATE_WAIT=3 WIDE_BACK=1 CARD_BUFFERS_MIN_MB=1 _MB=1024` | **4.55–4.62 fps** (216–220 ms: 75 / 98 / 28) | **−%55.7 süre**, tekrarlanabilir, device loss yok |
| **#89** | **RTX 5070 Laptop 8 GB**, 64 GB | `TEXTURE_BUDGET_MB=4096` + `RELAY=2` vs `1` vs kapalı | Kapalı 1.8 fps (554 ms) → `RELAY=1/2` **3.0 fps** (333–342 ms) | `RELAY=2`: eksik karakter cildi + üst üste altyazı; kapalı temiz |
| **#109** | **RTX 3060 Ti 8 GB** + **16 GB RAM** | 32 GB pagefile; `NO_WATCHDOG=1 LOOP_LIMIT=256` | ~2 fps, prolog açılır | 16 GB RAM + paging yüzünden ara sıra donma / frame pacing |
| **#168** | RTX 4090 24 GB + i9-14900K | `RELAY=2` vs `LATE_WAIT=3` seti | 3.0–3.2 fps (314–331 ms) → **5.0–5.3 fps** (189–198 ms) | İki geç-bekleme modu da ilk odanın duvarlarını kaybediyor |
| **#94** | RTX 3090 24 GB + 32 GB RAM | `LATE_WAIT=3` seti (`2119b0e`) | 1.8 fps (567 ms) → **3.7 fps** (273 ms); 24.3 GB private | 32 GB RAM bile pagefile büyüttü (50/52 GB commit) |
| **#54** | RTX 3080 Ti 12 GB + 32 GB RAM | `WINDOW_KB=1024/256`, `WINDOW_TAILS=1`, `INPLACE=1` | Hepsi 19.7–20.3 GB private (~0.3 GB fark) | `WINDOW_KB=1024` +514 MB daha kötü |
| **#10** | **RTX 4060 Ti 8 GB**, 64 GB | 4K vs 1080p, `LOOP_LIMIT=32` | 656 ms → 621 ms (GPU 258 → 244 ms, −%5) | GPU'nun %93'ü compute; `LOOP_LIMIT=32` saçı bozar (#62) |
| **#15** | RTX 5080 16 GB | `LOOP_LIMIT=256` (varsayılan hız) | 2.29 fps (prolog hareket doğrulandı) | 256 limit saç/geometriyi bozmadan güvenli |
| **#321** | RX 7600 XT 16 GB + 24 GB RAM | `WAVE_WIDE=1` + `RELAY=2` | Kare 1723'te 8.4 MB tahsis OOM | `RELAY=2` bellek pikini artırıyor (PR #360 bunu hedefler) |

## 4. Değerli PR'lar (8 GB VRAM / 16 GB RAM için en iyi 5 — sıralı)
Tümü `2119b0e` (`main`) üzerinde **`MERGEABLE`** durumdadır:
1. **#220 — Indirect draw bariyer + kartta bütçe** (`draw.rs`, `host.rs`, `indirect.rs`, `indirect/bench.rs`): Her draw sonrası `COMMON` bariyerini ve PCIe üstünden atomik bütçeyi kaldırır. **Ölçüm (RTX 3070 8 GB, `RELAY=2` + `INDIRECT_WINDOW=1`): 4.9→7.7 fps (206→130 ms, GPU 113→28–46 ms).** *Çakışma:* `host.rs`+`indirect.rs` (#71, #253 ile aynı dosyalar).
2. **#265 — `VICTORY_LEAN_WINDOWS=1` CPU tarafsız 64 KiB tile pencereler** (`host.rs`, `shadow.rs`): 4 MiB pencerelerin CPU kopyasını kaldırır. **Ölçüm (RTX 3080 Ti): −0.8 GB commit (yüklemede −1.9 GB), −2.1 GB aktif bellek**, görsel fark yok. 15.7 GB RAM'de pagefile thrashing'i azaltan tek gerçek bellek PR'ı. *Çakışma:* `shadow.rs` (#206, #360, #49) + `host.rs`.
3. **#95 — Aralık bazlı `MAPPING` kilidi** (`draw.rs`, `host.rs`, `texture.rs`, `watch.rs`, `xmem.rs`): Doku yenilemesinin ilgisiz remap'leri beklemesini bitirir. **Ölçüm (RTX 5070 Ti): 801 ms'nin 203 ms medyan (389 ms ort.) beklemesi sıfırlanır**, `TEXTURE_NO_WAIT=1` gibi doku atlamaz. *Çakışma:* #270/#393 ile aynı sorunu çözer (birlikte alma); `texture.rs` (#92, #271).
4. **#253 — Kart hash/compare 64-thread grup/parça** (`host.rs`, `indirect.rs`): `CARD_BUFFERS` açıkken GPU hash+compare **74+55 ms → 8.7+9.8 ms** (RX 6800'de GPU bekleme −57 ms). *Alternatif:* **#71** (`host.rs`, `indirect.rs`) buna ek olarak `RELAY` okumasını ayrı `COPY` kuyruğuna taşır (`VICTORY_RELAY_COPY_QUEUE`); #253 ile çakışır, `RELAY` kullanmayacaksan #253 daha temiz.
5. **#206 — `LATE_WAIT`/`RELAY` eksik kopyalanmamış sayfa düzeltmesi** (`shadow.rs`): #168/#89'daki duvar ve cilt kaybı penceresini daraltır. **+ #360** (`shadow.rs`): CPU snapshot OOM öncesi idle havuzu boşaltır (15.7 GB RAM'de #321 OOM koruması). *Çakışma:* ikisi de `shadow.rs` (#265 ile merge gerekir).

*Diğer incelenenler (tümü `MERGEABLE`):* **#38** (`host.rs`, `WIDE_REFRESH` varsayılan açık — env ile zaten açılabilir), **#49** (`shadow.rs`, `pipeline/make.rs`, `dstorage.rs` — i7-12700H 20 thread için worker ölçekleme, orta kazanç), **#92 / #271** (`victory-formats/texture.rs`, `texture.rs` — 4–8 MiB+ dokularda paralel detile, 16 MiB 2.2→0.5 ms), **#346** (`memory.rs`, `victory-shader` — sadece `BVH_BUILDER=1` için 3.3 s→normal), **#270** (`TEXTURE_NO_WAIT=1` varsayılan; #95 daha temiz), **#363** (Linux/Proton copy churn + pipeline library).

## 5. Bu makine için deneme sırası (adım adım)
> Not: İlk çalıştırma pipeline/shader önbelleğini doldurur (ilk 4–10 dk yavaş, #94). Ölçümleri **sıcak önbellek** ile, aynı kayıtta, 300–540 sn arası medyan kare satırıyla al. Windows'ta 15.7 GB RAM dolduğunda error `1455` (commit) almamak için 49 GB pagefile aktif kalsın ve tarayıcı/ağır uygulamaları kapat.

- **Adım 0 — Baz ölçüm (`main` @ `2119b0e`, diag build):**
  `VICTORY_LOG=info VICTORY_DISPLAY_MODE=1920x1080 VICTORY_TEXTURE_BUDGET_MB=3584` (sadece 8 GB VRAM koruması).
  *Ölç:* medyan `ms/frame` (`preparing / GPU / copy-back`), `memory map` private GB, VRAM kullanımı. *Beklenen:* ~2.0 fps (~480–520 ms), ~20–24 GB commit (#413, #109).
- **Adım 1 — Sadece env ile en güvenli hız seti (kod değişmeden):**
  `VICTORY_LATE_WAIT=3 VICTORY_WIDE_BACK=1 VICTORY_WIDE_REFRESH=1 VICTORY_CARD_BUFFERS_MIN_MB=1 VICTORY_CARD_BUFFERS_MB=768 VICTORY_TEXTURE_BUDGET_MB=3584 VICTORY_DISPLAY_MODE=1920x1080`
  *(8 GB VRAM için `CARD_BUFFERS_MB=768` ile başla; VRAM dolmazsa `1024` dene. `WAVE_WIDE` kapalı kalsın; `RELAY=2`'yi şimdilik açma — cilt/duvar/OOM riski #89,#168,#321.)*
  *Ölç:* kare süresi + duvar/cilt bütünlüğü + VRAM. *Beklenen:* **~4.2–4.6 fps (~215–240 ms, −%50..55)** (#413).
- **Adım 2 — Streaming takılması kontrolü (gerekirse):**
  Adım 1 logunda `textures` > 30 ms veya takılma görürsen `VICTORY_TEXTURE_NO_WAIT=1` ekle; `pipeline` bekliyorsa `VICTORY_PIPELINE_WAIT_MS=24`. *Beklenen:* doku takılma pikleri sıfırlanır (#178).
- **Adım 3 — İlk PR: #220 + `VICTORY_INDIRECT_WINDOW=1`:**
  Adım 1 env'lerine ek olarak `VICTORY_INDIRECT_WINDOW=1`.
  *Ölç:* GPU ms ve toplam FPS. *Beklenen:* GPU süresi ~100 ms'den **30–50 ms**'ye iner, toplam **~5.5–7.5 fps** (#220 RTX 3070 8 GB verisi).
- **Adım 4 — Bellek PR'ları: #265 (`VICTORY_LEAN_WINDOWS=1`) + #360:**
  *Ölç:* `memory map` private GB, working set, pagefile disk aktivitesi (donma/stutter azalması). *Beklenen:* **−0.8 GB commit / −2.1 GB aktif bellek** (#265) ve OOM koruması (#360).
- **Adım 5 — Kalan CPU/GPU darboğazı PR'ları: #95 + #253 + #206 (+ isteğe bağlı #49/#92):**
  Adım 3–4 sonrası kare artık CPU hazırlık/copy-back ağırlıklı olacak; #95 doku kilidini (#178'siz) çözer, #253 kart hash/compare'i 5–6× kısaltır, #206 geç-bekleme görsel hatalarını azaltır.
