# Victory Ayar Envanteri (`zslayer/next` @ `02ea29c`)

Hedef donanım: **RTX 3070 Ti Laptop (8 GB VRAM) / 16 GB RAM / NVIDIA**.  
Kaynak: `git -C C:\CLLL\.calisma\victory-fork` üzerinde yalnızca `zslayer/next` (`02ea29c`) ağacı.

---

## 1. Komut Satırı Alt Komutları ve Bayrakları

Kaynak: `apps/victory/src/main.rs` (`clap`), bakım aracı `xtask/src/main.rs`.

| Komut / Bayrak | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `victory run <exe>` | — | Oyun çalıştırılabilirini hazırlar, eşler ve bağlar | Konsol / Çalıştırma | `apps/victory/src/main.rs:94` |
| `--execute` (`run`) | `false` | Oyunu gerçekten başlatır; verilmezse bağlamadan sonra durur | Çalıştırma | `apps/victory/src/main.rs:104` |
| `--console <series-x\|series-s>` (`run`) | `series-x` | Oyunun gördüğü konsol profilini (cihaz tipi ve bellek) seçer (`SeriesX`: 10 GiB GPU + 3.5 GiB std, `SeriesS`: 7,776 MiB std) | Konsol profili | `apps/victory/src/main.rs:107`, `crates/victory-gpu/src/console.rs:16` |
| `--unimplemented <trap\|zero>` (`run`) | `trap` | Uygulanmamış import çağrılınca çökertir (`trap`) veya `0` döndürür (`zero`) | Teşhis | `apps/victory/src/main.rs:97` |
| `--strict` (`run`, `census`) | `false` | Host DLL pass-through'u kapatır (her şey HLE olmalı) | Teşhis | `apps/victory/src/main.rs:44,101` |
| `-- <title_args...>` (`run`) | boş | Oyunun kendi komut satırına iletilen argümanlar | Diğer | `apps/victory/src/main.rs:111` |
| `victory warm [paths...]` | konfig yolları | Oyun arşivlerindeki shader'ları önceden çevirir ve compute pipeline'ları önbelleğe alır | Shader / Önbellek | `apps/victory/src/main.rs:84` |
| `--jobs <n>` (`warm`, `translate`) | tüm mantıksal çekirdek | Çeviri/derleme işçi iş parçacığı sayısı | Shader / Önbellek | `apps/victory/src/main.rs:79,88` |
| `--no-pipelines` (`warm`) | `false` | Yalnızca DXIL çevirisi yapar, host D3D12 pipeline oluşturmaz | Shader / Önbellek | `apps/victory/src/main.rs:91` |
| `victory translate [paths...]` | konfig yolları | Bulunan tüm shader konteynerlerini / SGD5 dosyalarını çevirip raporlar | Shader / Teşhis | `apps/victory/src/main.rs:68` |
| `--validate` (`translate`) | `false` | Üretilen DXIL'i `dxil.dll` (`VICTORY_DXIL` veya Windows SDK) ile doğrular | Shader / Teşhis | `apps/victory/src/main.rs:74` |
| `--pipelines` (`translate`) | `false` | Çevrilen her shader için host D3D12 pipeline oluşturmayı dener | Shader / Teşhis | `apps/victory/src/main.rs:77` |
| `--list` (`translate`, `lib-exports`) | `false` | Her program/export için tek satır çıktı verir | Teşhis | `apps/victory/src/main.rs:54,79` |
| `victory inspect <file>` (`--imports`, `--exports`) | `false` | PE başlık, bölüm, TLS, build-id ve import/export özetini basar | Teşhis | `apps/victory/src/main.rs:30` |
| `victory census [paths...]` (`--strict`, `--missing`) | `false` | PE importlarını toplar ve HLE kapsamasını listeler | Teşhis | `apps/victory/src/main.rs:40` |
| `victory shader-info [paths...]` (`--verbose`, `--disasm`) | `false` | Shader konteyner düzenini ve isteğe bağlı GCN/RDNA disassemble çıktısını basar | Shader / Teşhis | `apps/victory/src/main.rs:58` |
| `victory config` | — | Çözümlenen `game_dir`, `cache_dir`, `data_dir` yollarını ve politika denetimini gösterir | Diğer | `apps/victory/src/main.rs:115` |
| `cargo xtask policy [--staged]` / `install-hooks` | — | Depo varlık politikasını denetler / `.githooks` kurar | Diğer | `xtask/src/main.rs:19` |

---

## 2. Ortam Değişkenleri (`VICTORY_*`) Envanteri

### (a) Bellek / VRAM Bütçesi ve Kopya Yönetimi

> **Kritik yerleşik kurallar (kod içi):**
> - **%80 `card_for` kuralı:** `crates/victory-gpu/src/shadow.rs:212-215` (`card_for`) ve `shadow.rs:272-275` (`window_card_for`) — DXGI `video_memory()` üzerinden `used + size > budget / 10 * 8` ise yeni kart tamponu ayrılmaz (VRAM bütçesinin en az %20'si dokulara/sistem payına boş bırakılır). Ayrıca tek tampon `< 1 GiB` olmalıdır.
> - **Doku bütçesi (`resource::room`):** `crates/victory-gpu/src/resource.rs:1367-1384` — `VICTORY_TEXTURE_BUDGET_MB` verilmezse ilk `2 GiB` (`FLOOR`) her zaman açılır; sonrasında `used < budget / 100 * room_percent()` (varsayılan **%90**, clamp `50..=97`). DXGI bütçesi okunamazsa `4 GiB` (`WITHOUT_BUDGET`) sınırı uygulanır.
> - **Basınç altında otomatik süpürme süreleri:** `crates/victory-gpu/src/host.rs:124-156` — `vram_idle_ms`: VRAM kullanımı `<75%` → `5000 ms`, `>=75%` → `2500 ms`, `>=85%` → `1000 ms`. `ram_idle_ms`: boş fiziksel RAM `>=4 GiB` → `10000 ms`, `<4 GiB` → `5000 ms`, `<2 GiB` → `2000 ms`.

| Ad | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `VICTORY_CARD_BUFFERS` | `1` (açık) | Başlık tamponlarının VRAM tarafı kopyalarını ve GPU hash/karşılaştırma geri-kopyasını açar (`0`: kapatır) | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:139` |
| `VICTORY_CARD_BUFFERS_MIN_MB` | `1` (min 1) | Kart belleğine (VRAM) alınacak en küçük başlık tamponu boyutu (MiB) | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:157` |
| `VICTORY_CARD_BUFFERS_MB` | `1024` | Kart tamponu kopyalarının toplam üst sınırı (MiB; "before" ikizleri hariç) | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:157` |
| `VICTORY_CARD_WINDOWS_MB` | `0` (kapalı) | Tampon dışı bellek pencerelerinin VRAM'e kopyalanabilecek üst sınırı (MiB) | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:252` |
| `VICTORY_TEXTURE_BUDGET_MB` | unset (DXGI %90) | İndeksle seçilen doku kopyaları için sabit VRAM üst sınırı (MiB); verilirse DXGI yüzdesinin yerine geçer | Bellek/VRAM | `crates/victory-gpu/src/resource.rs:1371` |
| `VICTORY_TEXTURE_ROOM_PERCENT` | `90` (`50..97`) | Yeni doku kopyası açmaya devam edilecek DXGI VRAM bütçesi yüzdesi | Bellek/VRAM | `crates/victory-gpu/src/resource.rs:1398` |
| `VICTORY_LAZY_TEXTURES` | `1` (açık) | Doku host kopyalarını oluşturma anında değil ilk kullanımda ayırır (`0`: anında) | Bellek/VRAM | `crates/victory-gpu/src/resource.rs:1254` |
| `VICTORY_SWEEP` | `1` (açık) | Saniyede bir kullanılmayan pencere, staging ve doku kopyalarını boşaltır (`0`: kapatır) | Bellek/VRAM | `crates/victory-gpu/src/host.rs:189` |
| `VICTORY_TEXTURE_SWEEP` | `1` (açık) | Boşta kalan doku host kopyalarını bırakır (`0`: kapatır; `descriptor/host.rs`'de `count` sadece sayar) | Bellek/VRAM | `crates/victory-gpu/src/resource.rs:1637`, `descriptor/host.rs:1132` |
| `VICTORY_WINDOW_KB` | `4096` (4 MiB) | Uzunluklu bellek pencerelerinin kesileceği ızgara boyutu (KiB, min 64, 2'nin kuvveti); düşürmek RAM'i azaltır | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:2973` |
| `VICTORY_WINDOW_TAILS` | `0` (kapalı) | `VICTORY_WINDOW_KB < 4096` iken uzunluksuz pencereleri daha küçük kuyruklarla bölerek RAM tasarrufu sağlar (`1`: açık) | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:3012` |
| `VICTORY_KEEP_SNAPSHOTS` | `0` (kapalı) | GPU geri-kopya için CPU tarafı anlık görüntüleri tutar (`1`: açar, `VICTORY_RELAY`'i kapatır, RAM harcar) | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:1446` |
| `VICTORY_KEEP_SNAPSHOTS_MB` | `256` | `VICTORY_KEEP_SNAPSHOTS=1` için RAM üst sınırı (MiB) | Bellek/VRAM | `crates/victory-gpu/src/shadow.rs:1457` |
| `VICTORY_INPLACE` | `0` (kapalı) | Uygun özel başlık belleğini kopyalamadan doğrudan D3D12 heap olarak açar (`1`: pencereler, üstü deneysel) | Bellek/VRAM | `crates/victory-gpu/src/inplace.rs:90` |
| `VICTORY_INPLACE_MB` | `4096` | `VICTORY_INPLACE` ile fiziksel RAM'e kilitlenecek toplam başlık belleği sınırı (MiB) | Bellek/VRAM | `crates/victory-gpu/src/inplace.rs:98` |
| `VICTORY_SCRATCH_POOL` | `1` (açık) | Geçici GPU scratch tamponlarını havuzdan yeniden kullanır (`0`: her seferinde ayır) | Bellek/VRAM | `crates/victory-gpu/src/host.rs:1422` |
| `VICTORY_LIST_POOL` | `1` (açık) | Tamamlanan D3D12 komut listelerini havuzda yeniden kullanır (`0`: her seferinde yeni) | Bellek/VRAM | `crates/victory-gpu/src/list.rs:232` |
| `VICTORY_LIBRARY_MAP` | `1` (açık) | Pipeline kütüphanesini bellek eşleme (mmap) ile okur/yazar (`0`: tam RAM tamponu) | Bellek/VRAM | `crates/victory-gpu/src/pipeline/library.rs:81` |
| `VICTORY_LIBRARY_HANDLE` | `1` (açık) | Eşlenen pipeline kütüphanesi dosya tutamacını açık tutar (`0`: kapatır, sayfalar sıfırlanabilir) | Bellek/VRAM | `crates/victory-gpu/src/pipeline/library.rs:90` |
| `VICTORY_LIBRARY_TRIM` | `1` (açık) | Pipeline yükleme patlamasından sonra eşlenen sayfaları çalışma kümesinden kırpar (`0`: tutar) | Bellek/VRAM | `crates/victory-gpu/src/pipeline/library.rs:370` |
| `VICTORY_LIBRARY_LOCK` | `0` (kapalı) | Tüm pipeline kütüphanesi çağrılarını tek mutex altında serileştirir (`1`: açık) | Bellek/VRAM | `crates/victory-gpu/src/pipeline/library.rs:427` |
| `VICTORY_HEAP_TEXTURES` | `1` (açık) | İndeksle seçilen (bindless) dokuların arka plan/geçiş yüklemesini açar (`0`: kapatır) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:1692` |
| `VICTORY_TEXTURE_STAGE_MB` | `192` | Kare başına bindless doku staging bütçesi (MiB) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:1700` |
| `VICTORY_TEXTURE_STAGE_MS` | `250` | Kare başına bindless doku staging süre sınırı (ms) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:1767` |
| `VICTORY_TEXTURE_CATCHUP` | `1` (açık) | İlk sahne kuyruğunda staging bütçesini 512 MB / 1.5 s'ye çıkarır (`0`: normal bütçe) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:1780` |
| `VICTORY_TEXTURE_GATE_S` | `60` | Doku yakalama (catch-up) yüksek bütçesinin toplam süre ödeneği (saniye; `0`: yok) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:362` |
| `VICTORY_EARLY_TEXTURES_MS` | `12` | Bindless shader istemeden önce her karede doku yüklemeye ayrılan süre (ms; `0`: yok) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:94` |
| `VICTORY_BAKE_TEXTURES_MS` | `1500` | Dekorasyon bake çiziminden hemen önce doku yükleme süresi (ms; `0`: özel işlem yok) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:94` |
| `VICTORY_REUPLOAD_WHEN_FULL` | `1` (açık) | VRAM doluyken mevcut host kopyası olan dokuları güncellemeye devam eder (`0`: etmez) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:414` |
| `VICTORY_HEAP_RECHECK` | `1` (açık) | Yüklenen bindless dokuların sonradan değişip değişmediğini yeniden denetler (`0`: etmez) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:402` |
| `VICTORY_PLACED_FILLER` | `0` (kapalı) | Placed doku anahtarında dolgu (filler) sözcüklerini korur (`1`: korur, kopya sayısını şişirir) | Bellek/VRAM | `crates/victory-gpu/src/resource.rs:2334` |
| `VICTORY_VIEW_INLINE` | `1` (açık) | CBV ve placed view kayıtlarını satır içi tutarak bellek harcamasını azaltır (`0`: tablo girişi) | Bellek/VRAM | `crates/victory-gpu/src/descriptor.rs:394` |
| `VICTORY_ROOT_VIEW_MB` | sınırsız (`u64::MAX`) | Root SRV/CBV görünüm kopya uzunluğuna üst sınır koyar (MiB) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:474` |
| `VICTORY_ROOT_UAV_MB` | sınırsız (`u64::MAX`) | Root UAV görünüm kopya/geri-kopya uzunluğuna üst sınır koyar (MiB) | Bellek/VRAM | `crates/victory-gpu/src/draw.rs:1192` |
| `VICTORY_DEPTH_TWINS` | `1` (açık) | Depth UAV ikiz doku mekanizması (`0`: kapalı, `stencil`: sadece stencil, `nodepth`: derinliksiz) | Bellek/VRAM | `crates/victory-gpu/src/resource.rs:447,458,533` |
| `VICTORY_ALIAS_COPY` | `2` (tam kopya) | Çakışan (aliased) kaynak kopyasını yönetir (`0`: kapalı, `1`: kısmi, unset/diğer: tam) | Bellek/VRAM | `crates/victory-gpu/src/resource.rs:1212,1218` |

### (b) Konsol Profili ve Ekran

| Ad | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `VICTORY_DISPLAY_MODE` | `3840x2160` | Oyuna bildirilen ekran çözünürlüğünü `WxH` (`640..7680` × `360..4320`, 60 Hz) olarak ayarlar | Konsol profili | `crates/victory-gpu/src/dxgi.rs:230` |
| (`--console` CLI) | `series-x` | Oyunun göreceği konsol modeli (`series-x` / `series-s`) | Konsol profili | `apps/victory/src/main.rs:107` |

### (c) Bekleme, Senkronizasyon ve Geri-Kopya (Readback / Relay)

| Ad | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `VICTORY_RELAY` | `2` (açık, test hariç) | GPU uçuşlarını ayrı iş parçacığında inişe geçirir (`0`: kapalı, `1`: temel, `2`: genişletilmiş); açıkken `LATE_WAIT=3`'ü etkinleştirir | Bekleme/Senkron | `crates/victory-gpu/src/shadow.rs:482`, `host.rs:1890` |
| `VICTORY_LATE_WAIT` | `3` (relay açıkken; `GPU_TIMING` ile `0`) | Gönderimleri GPU beklemeden öne sürer (`0`: senkron, `1`: temel, `2`: kısmi birleştirme, `3`: asenkron present) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:1790` |
| `VICTORY_LATE_WAIT_LISTS` | `256` (min 1) | Havada bekleyebilecek en fazla host komut listesi sayısı | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:1992` |
| `VICTORY_LATE_WAIT_WRITES` | `1` (açık) | Bottom-of-pipe yazmalı gönderimlerin havada kalmasına izin verir (`0`: anında settle) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:2002` |
| `VICTORY_LATE_WAITS` | `0` (kapalı) | Geç kalan kare beklemesini bir sonraki tick'e kadar uyutur (`1`: eski davranış, +17 ms/kare) | Bekleme/Senkron | `crates/victory-gpu/src/frame.rs:138` |
| `VICTORY_WIDE_BACK` | `1` (açık) | Kart geri-kopyasını geniş bloklarla yapar (`0`: kapalı) | Bekleme/Senkron | `crates/victory-gpu/src/shadow.rs:971` |
| `VICTORY_WIDE_MERGE` | `0` (kapalı) | Değişen aralıkları geniş birleştirmeyle işler (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:1628` |
| `VICTORY_WIDE_REFRESH` | `0` (kapalı) | Bellek yenilemesini geniş aralıklarla yapar (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:5510` |
| `VICTORY_CARD_REHASH_RUNS` | `0` (kapalı) | Karta yüklenen bitişik aralıkların hash geçişlerini birleştirir (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:1610` |
| `VICTORY_JOIN_COPIES` | `0` (kapalı) | Bitişik başlık tamponu kopyalarını tek kopyada birleştirir (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:2707` |
| `VICTORY_READ_BACK` | `1` (açık) | GPU'nun yazdığı tamponların başlık belleğine geri kopyalanmasını açar (`0`: kapatır) | Bekleme/Senkron | `crates/victory-gpu/src/list.rs:1060` |
| `VICTORY_DIRECT_WRITES` | `1` (açık) | Shader yazmalarını doğrudan eşlenmiş belleğe yazar (`0`: eski yavaş yol) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:5786` |
| `VICTORY_WRITE_SKIP` | `1` (açık) | Gereksiz yazma aralığı beyanlarını atlayarak CPU snapshot/karşılaştırma yükünü azaltır (`0`: hepsini beyan et) | Bekleme/Senkron | `crates/victory-gpu/src/draw.rs:119` |
| `VICTORY_CUT_WRITES` | `0` (kapalı) | Komut listesini yazmalarda böler (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/list.rs:2000` |
| `VICTORY_QUEUE_WAIT` | `1` (açık) | Kuyruk fence beklemelerini uygular (`0`: beklemeden gönder) | Bekleme/Senkron | `crates/victory-gpu/src/queue.rs:72` |
| `VICTORY_SPLIT_WAITS` | `1` (açık) | Beklemeleri tüm gönderim yerine liste bazında böler (`0`: tüm gönderimi tut) | Bekleme/Senkron | `crates/victory-gpu/src/queue.rs:91` |
| `VICTORY_CUT_WAITS` | `1` (açık) | Komut listesini bellek değeri beklemelerinde alt listelere böler (`0`: listeyi bölme) | Bekleme/Senkron | `crates/victory-gpu/src/queue.rs:110` |
| `VICTORY_WAIT_FORCE` | `1` (açık) | Present anında kuyrukta bekleyen işleri zorla çalıştırır (`0`: kareler arası tutar, kilitlenebilir) | Bekleme/Senkron | `crates/victory-gpu/src/queue.rs:120` |
| `VICTORY_PIPELINE_WAIT_MS` | `12000` | Arka planda derlenen pipeline için kare başına bekleme bütçesi (ms) | Bekleme/Senkron | `crates/victory-gpu/src/pipeline/make.rs:184,201` |
| `VICTORY_PIPELINE_GATE_S` | `120` | İlk yükleme sırasında pipeline bekleme ödeneğinin toplam sınırı (saniye; `0`: kapalı) | Bekleme/Senkron | `crates/victory-gpu/src/pipeline/make.rs:230` |
| `VICTORY_PIPELINE_WAIT_SHARED` | `window` | Kare pipeline bekleme bütçesinin thread'ler arası paylaşımı (`window`, `reserve`, `0`) | Bekleme/Senkron | `crates/victory-gpu/src/pipeline/make.rs:327` |
| `VICTORY_ASK_ONCE` | `1` (açık) | Gönderim başında çekirdeğe write-watch sorusunu aralık başı yerine tek seferde sorar (`0`: eski yol) | Bekleme/Senkron | `crates/victory-gpu/src/watch.rs:168` |
| `VICTORY_GUARD_ASK` | `0` (kapalı) | Bellek korumalarını önbellek yerine her seferinde çekirdeğe sorar (`1`: yavaş) | Bekleme/Senkron | `crates/victory-gpu/src/watch.rs:414` |
| `VICTORY_GUARD_HOLES` | `1` (açık) | Eşlenmemiş delikli bellek aralıklarını tek tek tarar (`0`: yalnızca başlangıç adresi) | Bekleme/Senkron | `crates/victory-gpu/src/watch.rs:490` |
| `VICTORY_VIEW_GUARD` | `1` (açık) | Korunan sayfanın tüm eşlenmiş görünümlerini birlikte salt-okunur yapar (`0`: tek görünüm) | Bekleme/Senkron | `crates/victory-gpu/src/watch.rs:559` |
| `VICTORY_TEXTURE_GUARD` | `1` (açık) | Doku değişimlerini write-watch/guard ile izler, hash'i 8 karede bir alır (`0`: her denetimde hash) | Bekleme/Senkron | `crates/victory-gpu/src/texture.rs:744` |
| `VICTORY_TEXTURE_NO_WAIT` | `0` (kapalı) | Bellek yeniden eşlenirken doku yenilemesinin beklemeden çekirdekten okumasını sağlar (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/texture.rs:314` |
| `VICTORY_TEXTURE_COPY` | `0` (kapalı) | Tam eşli dokuları yerinde okumak yerine çekirdek kopyasından okur (`1`: yavaş) | Bekleme/Senkron | `crates/victory-gpu/src/texture.rs:376` |
| `VICTORY_COMMITTED_QUERY` | `0` (kapalı) | Sayfa durumunu hızlı kontrol yerine `VirtualQuery` ile sorar (`1`: çok yavaş) | Bekleme/Senkron | `crates/victory-gpu/src/texture.rs:1071` |
| `VICTORY_MAPPED_BY_PAGE` | `0` (kapalı) | 64 KiB blok yerine 4 KiB sayfa bazında okunabilirlik kontrolü yapar (`1`: yavaş) | Bekleme/Senkron | `crates/victory-gpu/src/texture.rs:1156` |
| `VICTORY_COPY_ASK` | `0` (kapalı) | Başlık belleğine kopyadan önce sayfa commit durumunu sorar (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:5660` |
| `VICTORY_WAIT_ASK` | `0` (kapalı) | Liste beklemesi öncesi sayfa durumunu sorar (`1`: açık) | Bekleme/Senkron | `crates/victory-gpu/src/list.rs:1798` |
| `VICTORY_UNWATCHED_FRAMES` | `1` | Write-watch dışı tamponların kaç karede bir yenileneceğini belirler | Bekleme/Senkron | `crates/victory-gpu/src/draw.rs:1499` |
| `VICTORY_LAZY_PRELUDE` | `1` (açık) | Komut listesi başlangıç (prelude) nesnelerini gerekince oluşturur (`0`: baştan) | Bekleme/Senkron | `crates/victory-gpu/src/host.rs:870` |
| `VICTORY_CLZERO_ASK` | `0` (kapalı) | `CLZERO` talimatı öykünmesinde her seferinde bellek sorgusu yapar (`1`: çok yavaş) | Bekleme/Senkron | `crates/victory-loader/src/console_insn.rs:28` |

### (d) Shader / Wave / Ray Tracing

> **Shader önbellek anahtarına girenler (`crates/victory-gpu/src/shader_cache.rs:64-95`):** `VICTORY_SHADER_LOOP_LIMIT`, `VICTORY_GROUP_SYNC`, `VICTORY_SHADER_OPT`, `VICTORY_WAVE_WIDE`, `VICTORY_WAVE_DEEP_SKIPS`, `VICTORY_WAVE_OWN_BITS`, `VICTORY_WAVE_LOCAL`, `VICTORY_F16_NEAREST`, `VICTORY_STRICT_JOINS`, `VICTORY_MESH_OUTPUT_STRICT` (ayrıca `view_heap_limit` ve `host_max_wave`). Bunları değiştirmek kendi ayrı shader cache girişlerini üretir.

| Ad | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `VICTORY_SHADER_OPT` | `ALL` (hepsi açık) | Çevrilen DXIL üzerindeki temizleme geçişlerinin bit maskesi (`0`: kapalı; 1/2/4/8/16/32) | Shader/Wave | `crates/victory-shader/src/emit/mod.rs:188` |
| `VICTORY_WAVE_WIDE` | `0` (kapalı, wave64→32) | Wave64→2×wave32 dönüşümünde wave-wide dallanma/barrier formunu açar (`1`: NVIDIA'da TDR riski; `0`: trace fallback'i de kapatır) | Shader/Wave | `crates/victory-shader/src/emit/mod.rs:382,472` |
| `VICTORY_WAVE_LOCAL` | `1` (açık) | Wave64 programlarını yerel host wave ile yaklaştırmaya izin verir (`0`: kapalı, `idiom`: yalnızca kesin deyimler) | Shader/Wave | `crates/victory-shader/src/emit/mod.rs:338,344` |
| `VICTORY_WAVE_DEEP_SKIPS` | `1` (açık) | Skaler takibiyle derin per-lane skip'leri kaldırıp wave64→32 değişimini mümkün kılar (`0`: kapalı) | Shader/Wave | `crates/victory-shader/src/emit/mod.rs:392` |
| `VICTORY_WAVE_OWN_BITS` | `1` (açık) | Yalnız kendi lane maskesini okuyan wave64 ifadelerini barrier'sız indirger (`0`: kapalı) | Shader/Wave | `crates/victory-shader/src/emit/lower.rs:1016` |
| `VICTORY_FIRST_HALF` | `1` (açık) | Wave64'ün ilk yarısı optimizasyonunu kullanır (`0`: kapalı) | Shader/Wave | `crates/victory-shader/src/emit/lower.rs:1501` |
| `VICTORY_FIRSTLANE_OWN` | `0` (kapalı) | Iraksak blokta host wave'in kendi ilk lane'ini alır (`1`: açık, yaklaşım) | Shader/Wave | `crates/victory-shader/src/emit/lower.rs:7477` |
| `VICTORY_GROUP_SYNC` | `2` (hepsi bekler) | Grup barrier davranışı (`2`: tam sync, `1`: yalnızca tüm lane'lerin girdiği bloklarda, `0`: beklemesiz/teşhis) | Shader/Wave | `crates/victory-shader/src/emit/mod.rs:164` |
| `VICTORY_F16_NEAREST` | `0` (kapalı) | FP32→FP16 dönüşümünde en yakına yuvarlamayı öykünür (`1`: açık, NVIDIA'da ekstra ~12 komut) | Shader/Wave | `crates/victory-shader/src/emit/lower.rs:1003` |
| `VICTORY_ACTIVATE_RAYS` | `1` (açık) | Ağır `CS_ScreenProbes_ActivateRays` shader'ını derler (`0`: atlar, ~85 sn ve >1.3 GB derleyici RAM tasarrufu) | Shader/Wave | `crates/victory-gpu/src/pipeline/make.rs:77` |
| `VICTORY_RT_RESOLVE` | `1` (açık) | BVH builder kapalıyken ~1.000 `CS_RaytracingResolve` shader'ını derler (`0`: atlar) | Shader/Wave | `crates/victory-gpu/src/pipeline/make.rs:94` |
| `VICTORY_BVH_BUILDER` | `0` (kapalı) | Ray-tracing BVH kurucu compute shader'larına izin verir (`1`: deneysel) | Shader/Wave | `crates/victory-gpu/src/pipeline.rs:306`, `make.rs:93` |
| `VICTORY_BVH_QUARANTINE` | `0` (kapalı) | BVH kurucu shader'larını karantinaya alır (`1`: açık) | Shader/Wave | `crates/victory-gpu/src/pipeline.rs:282` |
| `VICTORY_D3D12_BACKGROUND` | `1` (açık) | Sürücünün D3D12 arka plan shader yeniden derlemesine izin verir (`0`: kapatır) | Shader/Wave | `crates/victory-gpu/src/host.rs:614` |
| `VICTORY_ROOT_SRV_DIRECT` | `1` (açık) | Root SRV'leri doğrudan bağlar (unset: açık, herhangi bir değer: kapalı) | Shader/Wave | `crates/victory-gpu/src/pipeline/host.rs:279` |
| `VICTORY_SKIP_PIPELINES` | boş | Virgülle ayrılmış 16 haneli shader hash'lerini içeren tüm çizim/dispatch'leri atlar | Shader/Wave | `crates/victory-gpu/src/pipeline.rs:463` |
| `VICTORY_SHADER_LOOP_LIMIT` | `2^20` | Çevrilen shader döngülerinin lane başına iterasyon sınırı (yalnız `diagnostics` derlemesinde etkili) | Shader/Wave | `crates/victory-shader/src/emit/mod.rs:137` |
| `VICTORY_MESH_OUTPUT_STRICT` | `0` (kapalı) | 32 KiB host sınırını aşan mesh shader'ları reddeder (set ise açık) | Shader/Wave | `crates/victory-shader/src/emit/lower.rs:4149` |
| `VICTORY_STRICT_JOINS` | `0` (kapalı) | Eksik phi/join değerlerinde çeviriyi durdurur (set ise açık) | Shader/Wave | `crates/victory-shader/src/emit/lower.rs:11032` |
| `VICTORY_HOST_WAVE` / `VICTORY_HOST_MAX_WAVE` | cihazdan okunur | `translate` komutunda hedef host wave boyutunu (ör. `32`) zorlar | Shader/Wave | `apps/victory/src/main.rs:172`, `translate.rs:165` |
| `VICTORY_VIEW_LIMIT` | cihazdan okunur | `translate` komutunda view heap sınırını zorlar | Shader/Wave | `apps/victory/src/main.rs:168` |
| `VICTORY_DXIL` | Windows SDK | `dxil.dll` doğrulayıcı yolunu belirtir | Shader/Wave | `crates/victory-shader/src/emit/validator.rs:122` |

### (e) Teşhis, Ölçüm ve Yakalama

| Ad | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `VICTORY_LOG` (yedek `RUST_LOG`) | `info` | `tracing` log filtresi (`stderr`'e yazar; `debug`/`trace` yalnız `diagnostics` feature'lı derlemede) | Teşhis/Ölçüm | `crates/victory-core/src/logging.rs:21` |
| `VICTORY_GPU_TIMING` | `0` (kapalı) | Her komut listesini tek tek GPU timestamp ile ölçer, yavaş komutları loglar (`LATE_WAIT`'i `0` yapar!) | Teşhis/Ölçüm | `crates/victory-gpu/src/host.rs:1329,1787` |
| `VICTORY_D3D12_DEBUG` | `0` (kapalı) | D3D12 debug katmanını açar (yalnız `diagnostics` derlemesi) | Teşhis/Ölçüm | `crates/victory-gpu/src/host.rs:558` |
| `VICTORY_D3D12_GBV` | `0` (kapalı) | `D3D12_DEBUG` ile birlikte GPU-based validation açar (çok yavaş) | Teşhis/Ölçüm | `crates/victory-gpu/src/host.rs:569` |
| `VICTORY_D3D12_DRED` | `0` (kapalı) | GPU device-removed anında DRED ekmek kırıntısı ve bellek bilgilerini loglar (`diagnostics`) | Teşhis/Ölçüm | `crates/victory-gpu/src/host.rs:1314` |
| `VICTORY_D3D12_WARP` / `VICTORY_WARP` | `0` (kapalı) | Yazılım bağdaştırıcısı (WARP) üzerinde çalıştırır (`diagnostics` / test) | Teşhis/Ölçüm | `crates/victory-gpu/src/host.rs:701`, `victory-shader/src/emit/host.rs:85` |
| `VICTORY_CAPTURE` | unset | Sunulan kareleri (2'nin kuvveti kareler) `<dir>/frame-<n>.png` olarak kaydeder (`diagnostics`) | Teşhis/Ölçüm | `crates/victory-gpu/src/capture.rs:15` |
| `VICTORY_CAPTURE_FRAMES` | boş | Ek olarak yakalanacak ve çizimleri trace edilecek virgüllü kare numaraları | Teşhis/Ölçüm | `crates/victory-gpu/src/capture.rs:26` |
| `VICTORY_CAPTURE_WORLD` / `..._WORLD_ALSO` / `..._HEAVY_FRAMES` / `..._MENUS` | kapalı/boş | 500+ draw'lu ilk 3D sahne (+2,+6,+30,+80), ek ofsetler, ağır sahne ve menü karelerini yakalar | Teşhis/Ölçüm | `crates/victory-gpu/src/capture.rs:44,59,164,436` |
| `VICTORY_DUMP_TARGETS` / `VICTORY_DUMP_AFTER` | kapalı/boş | Yakalanan karelerde tüm render target'ları / belirli çizim sonrasını PNG olarak kaydeder | Teşhis/Ölçüm | `crates/victory-gpu/src/capture.rs:322,632` |
| `VICTORY_CARD_ACCOUNTING` | `0` (kapalı) | VRAM dolduğunda ve her 5 sn'de bir doku/VRAM muhasebe satırı loglar | Teşhis/Ölçüm | `crates/victory-gpu/src/draw.rs:1614` |
| `VICTORY_TRAIL` | `1` (açık) | Son 96 GPU nesne/library olayını tutar, device-removed olursa loglar (`0`: kapalı) | Teşhis/Ölçüm | `crates/victory-gpu/src/host.rs:259` |
| `VICTORY_MEMORY_MAP` | `1` (açık) | `diagnostics` derlemesinde adres uzayı haritasını loglar (`0`: atlar) | Teşhis/Ölçüm | `crates/victory-gpu/src/copystat.rs:487` |
| `VICTORY_COPY_MODE` | normal | Kopya hızı ölçümü için deney modu (`plain`: korumasız kopya, `none`: kopyayı atla) | Teşhis/Ölçüm | `crates/victory-gpu/src/copystat.rs:141` |
| `VICTORY_BARRIER_TEST` | kapalı | Barrier deneyi (`groups`: grup barrier'sız, `indirect`: indirect barrier'sız) | Teşhis/Ölçüm | `crates/victory-gpu/src/copystat.rs:237` |
| `VICTORY_SCISSOR_TEST` / `VICTORY_MESH_TEST` | kapalı | Scissor (`full`) ve mesh draw (`skip`, `scissor`) hata ayıklama deneyleri | Teşhis/Ölçüm | `crates/victory-gpu/src/draw.rs:286,295,309` |
| `VICTORY_NULL_MAGENTA` | `0` (kapalı) | Henüz yüklenmemiş 2D renk dokularını siyah yerine eflatun gösterir (`1`: açık) | Teşhis/Ölçüm | `crates/victory-gpu/src/descriptor/host.rs:2530` |
| `VICTORY_REFRESH_CHECK` | `0` (kapalı) | Yenilenen tamponların bayat kalıp kalmadığını 64 KiB'de bir örnekleyip denetler (`1`: açık) | Teşhis/Ölçüm | `crates/victory-gpu/src/shadow.rs:2438` |
| `VICTORY_WALLS_TRACE` | `0` (kapalı) | DStorage ve GPU yenileme ayrıntı izi (`1` veya `2`) | Teşhis/Ölçüm | `crates/victory-gpu/src/host.rs:2760`, `dstorage.rs:452` |
| `VICTORY_INPLACE_PROBE` / `..._PROBE_MB` | `0` / `64` | İlk sweep'te sürücü in-place heap desteğini sınar ve 30/150. sn'de sayım basar | Teşhis/Ölçüm | `crates/victory-gpu/src/inplace.rs:63,1080` |
| `VICTORY_STACKS` | unset | Belirtilen saniyelerde (`20,60`) tüm thread'lerin çağrı yığınlarını basar | Teşhis/Ölçüm | `crates/victory-loader/src/stacks.rs:27` |
| `VICTORY_WATCH` | unset | Verilen hex RVA'lardaki `u32` değerlerini değişince loglar | Teşhis/Ölçüm | `apps/victory/src/run.rs:164` |
| `VICTORY_CRASH_SAVE` | unset | Çökmede `<rva>:<len>,...` bellek aralıklarını cache klasörüne kaydeder | Teşhis/Ölçüm | `apps/victory/src/run.rs:144` |
| `VICTORY_AUDIO_CAPTURE` / `..._OF` | unset | Oyun sesini 48 kHz stereo 16-bit WAV dosyasına kaydeder (`diagnostics`) | Teşhis/Ölçüm | `crates/victory-hle/src/modules/audio/capture.rs:49,69,84` |
| `VICTORY_XMA_DUMP` | unset | İlk 32 sesin XMA giriş ve bağlam dosyalarını `<dir>` içine kaydeder | Teşhis/Ölçüm | `crates/victory-hle/src/modules/audio.rs:636` |
| `VICTORY_SHADER_SAVE` / `VICTORY_LIST_SLOTS` | unset | Çevrilen shader'ları klasöre yazar / belirtilen PS hash'lerinin doku slotlarını listeler | Teşhis/Ölçüm | `crates/victory-gpu/src/pipeline.rs:160`, `draw.rs:3331` |
| `VICTORY_WAVE_WHY` / `VICTORY_DESC_WHY` | kapalı | Wave64→32 ve descriptor düşürme kararlarını `stderr`'e basar | Teşhis/Ölçüm | `crates/victory-shader/src/emit/mod.rs:326`, `lower.rs:5709` |
| `VICTORY_SHADER_CENSUS` | unset | `translate` komutunda her çevrilen program için belirtilen dosyaya tek satır ekler | Teşhis/Ölçüm | `apps/victory/src/translate.rs:566`, `census.rs:63` |
| `VICTORY_TRANSLATE_ONLY` / `VICTORY_PAIRS` / `VICTORY_DUMP_MATCH` / `VICTORY_FAIL_DUMP` / `VICTORY_EMIT_DUMP` | unset | `translate` alt komutu için filtreleme ve hata/çıktı döküm değişkenleri | Teşhis/Ölçüm | `apps/victory/src/translate.rs:180,399,424,453,614` |

### (f) Girdi / Otomasyon

| Ad | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `VICTORY_KEYS` | boş | Belirtilen saniyelerde pencere odakta olmasa da tuş basar (ör. `30:Enter,35:X`) | Girdi/Otomasyon | `crates/victory-hle/src/modules/gameinput.rs:1208` |

### (g) Ağ Kilidi (`crates/victory-hle/src/modules/net.rs`)

- **Ortam değişkeni / override YOK.** Ağ kilidi kodda sabittir:
  - Winsock (`connect`, `ConnectEx`, `sendto`, `WSASendMsg` vb.) yalnızca `127.0.0.0/8` ve `::1` loopback adreslerine izin verir; diğer her hedef `WSAENETUNREACH` döner.
  - DNS (`getaddrinfo`, `GetAddrInfoW` vb.) yalnızca `localhost` ve IP literallerini çözer; diğerleri `WSAHOST_NOT_FOUND`.
  - `WinHttpConnect` loopback dışı hedeflerde `ERROR_WINHTTP_CANNOT_CONNECT` döner; proxy ve WPAD/PAC otomatik algılama kapalıdır.
  - `0.0.0.0` / `::` bind çağrıları loopback'e daraltılır.

### (h) Diğer (Yollar, İnce Ayar, İç/Test Değişkenleri)

| Ad | Varsayılan | Ne yapar | Kategori | Dosya:Satır |
|---|---|---|---|---|
| `VICTORY_CONFIG` | `./victory.toml` | Yapılandırma TOML dosyasının yolu | Diğer | `crates/victory-core/src/config.rs:49` |
| `VICTORY_GAME_DIR` | `victory.toml` | Oyun dosyalarının bulunduğu klasör | Diğer | `crates/victory-core/src/config.rs:94` |
| `VICTORY_CACHE_DIR` | `%LOCALAPPDATA%\victory\cache` | Türetilmiş önbellek (çevrilen shader'lar, pipeline library) | Diğer | `crates/victory-core/src/config.rs:86` |
| `VICTORY_DATA_DIR` | `%LOCALAPPDATA%\victory\data` | Oyun kayıtları, ayarlar ve `crashes.log` | Diğer | `crates/victory-core/src/config.rs:90` |
| `VICTORY_TEST_TITLES` | `victory.toml` | `;` ile ayrılmış ek test oyun klasörleri | Diğer | `crates/victory-core/src/config.rs:80` |
| `VICTORY_DSTORAGE_RETRIES` | `12` | DirectStorage okuma yeniden deneme sayısı | Diğer | `crates/victory-hle/src/modules/dstorage.rs:1079` |
| `VICTORY_INDIRECT_WINDOW` | `0` (kapalı) | Ardışık indirect draw kayıtlarını pencere halinde kıstırır (`1`: açık) | Diğer | `crates/victory-gpu/src/draw.rs:184` |
| `VICTORY_STALE_TEXTURES` | `1` (açık) | Üzerine yeni view yazılan eski doku slotlarını bayat işaretler (`0`: kapalı) | Diğer | `crates/victory-gpu/src/descriptor/host.rs:264` |
| `VICTORY_TABLED_TEXTURES` | `0` (kuyruktan çıkar) | Tablo üzerinden okunan dokuları bindless kuyruğunda tutar (`1`: eski davranış) | Diğer | `crates/victory-gpu/src/descriptor/host.rs:326` |
| `VICTORY_VIEW_EXTENT` | `1` (açık) | Tampon görünümünü slotun gösterdiği host kopyasında günceller (`0`: yalnız aralıkla ara) | Diğer | `crates/victory-gpu/src/descriptor/host.rs:1885` |
| `VICTORY_KEEP_VIEWS` | `1` (açık) | Aynı placed dokuya yeni view yazılırken yenisi hazır olana dek eskisini gösterir (`0`: anında null) | Diğer | `crates/victory-gpu/src/descriptor/host.rs:2494` |
| `VICTORY_RELAUNCHED_FROM` | iç kullanım | Oyunun kendini yeniden başlatmasında eski süreç tutamacını taşır | İç | `crates/victory-hle/src/modules/lifecycle.rs:234,272` |
| Test-only değişkenler | — | `VICTORY_TESTS_ALLOW_SOFTWARE_SKIP`, `VICTORY_BCPACK_TABLES/CASES`, `VICTORY_XMA_STREAMS/REFERENCE`, `VICTORY_CONSOLE_DXC`, `VICTORY_PC_DXC`, `VICTORY_DIFF_ONLY`, `VICTORY_REF_PS/VS`, `VICTORY_SAVE_STOP/CHILD/OP`, `VICTORY_EXIT_CHILD`, `VICTORY_TRAP_LOG`, `VICTORY_TEST_EXIT_PROCESS_CHILD` | Test | ilgili `*_tests.rs` / modül testleri |

---

## 3. Pencere Başlığı ve Log Ölçümleri (Nasıl Hesaplanıyor?)

Kaynak: `crates/victory-gpu/src/host.rs:6093-6355` (`frame`), `host.rs:439-447` (`card_clock_line`), `host.rs:2562-2580` (`late_line`), `host.rs:6047-6055` (`caption`).

**Pencere başlığı biçimi** (her ≥1 saniyede hesaplanır, ayrı `victory-caption` thread'i 100 ms'de bir `SetWindowTextW` ile günceller):
```
ZSlayerHQ retail game | {fps} fps {ms} ms/frame | gpu {gpu_pct}% busy cpu {cpu_pct}% ram {ram_mb} MB | {draws} draws/frame | up {m:ss}
```

| Metrik | Nerede görünür | Nasıl hesaplanır |
|---|---|---|
| `fps` | Başlık + 5 sn log | `frames / elapsed_secs` (≥1 sn penceresinde sunulan kare sayısı) |
| `ms/frame` | Başlık + 5 sn log | `elapsed_ms / frames` |
| `gpu % busy` (başlık) | Başlık + 5 sn log | `(SUBMISSION_US[1] / 1000 / frames) / ms * 100` — gönderim thread'inin GPU beklediği sürenin kare süresine oranı (relay açıkken kart saatine `card_clock_line` daha doğru bakar) |
| `card busy %` / `card wait` | Her ≥1 sn `tracing::info!` (`card_clock_line` & `late_line`) | `CARD_CLOCK_US`: kartta iş yokken geçen süre (`idle`), gönderim thread'i çalışırken kartın boş kaldığı süre (`behind`) ve gönderim thread'inin kartı beklediği süre (`stood`). `card busy % = (1 - idle / ms) * 100` |
| `cpu %` | Başlık + 5 sn log | `delta_process_cpu_us / 1e6 / elapsed_secs / logical_processors * 100` (tüm makine CPU payı) |
| `ram MB` | Başlık + 5 sn log | `process_working_set() / (1024 * 1024)` (süreç çalışma kümesi) |
| `draws/frame` | Başlık + 5 sn log | `delta_drawn / frames` (host'ta gerçekten çalıştırılan draw sayısı) |
| `VRAM` | 5 sn log & `VICTORY_CARD_ACCOUNTING=1` | Pencere başlığında **yoktur**; 5 sn'lik `tracing::info!` satırında kart tamponu MB (`card_stats`), bindless doku MB (`picked_mb`) ve `VICTORY_CARD_ACCOUNTING=1` ile DXGI bütçe/kullanım ayrıntısı basılır |

### Kare zamanlaması / profil verisi dosyaya yazılıyor mu?
- **Hayır, yerleşik CSV/JSON kare-zamanlama dosyası yok.**
- Tüm kare zamanlama, `card_clock_line`, `late_line` ve `copystat` (yalnız `diagnostics` derlemesinde) çıktıları `tracing_subscriber` üzerinden **`stderr`'e** yazılır (`crates/victory-core/src/logging.rs:28-30`).
- Kalıcı ölçüm kaydı almak için standart yönlendirme gerekir: `victory run ... 2> run.log`.
- Dosyaya yazan tek yerleşik araçlar: `VICTORY_CAPTURE=<dir>` (PNG + `frame-<n>-ops.txt`), `VICTORY_SHADER_CENSUS=<file>` (yalnız `translate` komutu), `VICTORY_AUDIO_CAPTURE=<wav>`, `crashes.log`.

---

## 4. RTX 3070 Ti Laptop (8 GB VRAM / 16 GB RAM / NVIDIA) İlk Koşu Önerisi

### Önerilen Başlangıç Seti

Önce bir kez shader önbelleğini ısıt:
```powershell
victory warm
```

Sonra ilk koşu ortam değişkenleri + CLI:
```powershell
$env:VICTORY_DISPLAY_MODE = "1920x1080"       # Varsayılan 3840x2160 (4K); 8 GB VRAM ve piksel yükünü 4x rahatlatır
$env:VICTORY_CARD_BUFFERS_MB = "512"          # Varsayılan 1024; "before" ikizleriyle VRAM'i boğmamak için
$env:VICTORY_TEXTURE_ROOM_PERCENT = "85"      # Varsayılan 90; 8 GB VRAM'de Windows shared-RAM spill'ine karşı pay bırakır
$env:VICTORY_WINDOW_KB = "512"                # Varsayılan 4096 (4 MiB); 16 GB RAM'de 3+ GB pencere şişmesini keser
$env:VICTORY_WINDOW_TAILS = "1"               # Varsayılan 0; WINDOW_KB < 4096 ile birlikte RAM tasarrufunu tamamlar
$env:VICTORY_RT_RESOLVE = "0"                 # Varsayılan 1; BVH builder kapalıyken ~1000 RT resolve shader derlemesini atlar
$env:VICTORY_ACTIVATE_RAYS = "0"              # Varsayılan 1; 66K komutlu shader'ın ~85 sn / >1.3 GB derleyici RAM zirvesini atlar
$env:VICTORY_CARD_ACCOUNTING = "1"            # İlk teşhis koşusunda VRAM/doku doluluk durumunu logda görmek için (stabil olunca kaldır)
# VICTORY_RELAY (2) ve VICTORY_LATE_WAIT (3) kodda zaten varsayılan AÇIK — ayrıca set etmeye gerek yok.

victory run <oyun.exe> --execute --console series-s 2> run.log
```

> Not: `victory warm` yapmadan doğrudan ilk kez giriyorsan, 12 sn'lik varsayılan `VICTORY_PIPELINE_WAIT_MS` kareleri çok dondurursa geçici olarak `$env:VICTORY_PIPELINE_WAIT_MS = "1000"` verilebilir.

### "Dokunma" Listesi (Bu Donanımda Değiştirme)

| Ayar | Neden dokunma |
|---|---|
| `VICTORY_WAVE_WIDE=1` | Varsayılan kapalı (`0`). NVIDIA wave32'de wave64→2×wave32 barrier formu sürücü kilitlenmesine (TDR / device lost) yol açtığı için özellikle kapatılmış. |
| `VICTORY_GROUP_SYNC=0` | Varsayılan `2`. Barrier'ları kaldırır ve bozuk görüntü/hesap üretir (yalnızca TDR teşhisinde `1` denenebilir). |
| `VICTORY_INPLACE=1` | Varsayılan `0`. 16 GB RAM'de sayfaları fiziksel RAM'e kilitler; önce `VICTORY_INPLACE_PROBE=1` ile sürücü testi yapmadan açma. |
| `VICTORY_KEEP_SNAPSHOTS=1` | Varsayılan `0`. `VICTORY_RELAY`'i devre dışı bırakır ve sistem RAM'ini hızla tüketir. |
| `VICTORY_SWEEP=0`, `VICTORY_TEXTURE_SWEEP=0`, `VICTORY_LAZY_TEXTURES=0`, `VICTORY_LIBRARY_TRIM=0` | 8 GB VRAM + 16 GB RAM'de boşta kalan nesnelerin temizlenmesini kapatır; OOM / VRAM spill garanti eder. |
| `VICTORY_D3D12_DEBUG=1`, `VICTORY_D3D12_GBV=1`, `VICTORY_GPU_TIMING=1`, `VICTORY_D3D12_WARP=1` | Ağır teşhis modları. `GPU_TIMING` `LATE_WAIT`'i zorla `0` yapar; `GBV` ve `WARP` kare hızını yerle bir eder. |
| `VICTORY_F16_NEAREST=1` | Varsayılan `0`. NVIDIA'da her f16 dönüşümüne ~12 ekstra komut ekler. |
| `VICTORY_CLZERO_ASK=1`, `VICTORY_COMMITTED_QUERY=1`, `VICTORY_MAPPED_BY_PAGE=1`, `VICTORY_GUARD_ASK=1` | Eski yavaş çekirdek sorgu yollarına döner; ciddi CPU darboğazı yaratır. |
