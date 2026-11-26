# Victory / the title — Delta Taraması (26.11 ~00:40 → 01:35 UTC)

- **Kapsam:** Önceki iki raporda (`neon_ram_tarama_26_11.md`, `yerinde_port_inceleme.md`) yer alan maddeler tekrarlanmadı; yalnız **26.11 ~00:40 UTC sonrası** yeni commit/PR/issue'lar ve son 24 saatte yeni keşfedilen **16 GB RAM / 8 GB / 6 GB laptop NVIDIA** fork'ları raporlandı.
- **Taban (`avatarmanz/VIctory-7900XTX`) durumu:** `main` dalında **YENİ COMMIT YOK** — HEAD hâlâ `0a435665` (son push `2026-11-25 22:18 UTC`). Ancak 00:40 UTC sonrası repo üstüne doğrudan bizim tabana uyan **5 yeni açık PR** geldi (`#48`, `#68`, `#66`, `#58`, `#28`).
- **Diğer izlenen repolarda yeni durum özeti:**
  - `Marvisak/VIctory`: `main`'de yeni kod commiti yok (`63bcca43` yalnız README); ancak çok sayıda yeni PR açıldı (`#2948`, `#2958`, `#2955`, `#2960`).
  - `fxceless/victory-fork`: `main`'de `0fc6b066` (`VICTORY_HEAP_SYNC_CUT`, run 286'da kazanç yok ölçüldü), `36bdd515`, `025b042a`; PR tarafında **`#1240`**, **`#1229`–`#1233`**, **`#1237`**, **`#1227`** yeni.
  - `Belobug/VIctory-Belobug-AMD`: Varsayılan dal `rx9070xt`'ye geçti; yeni commit `81ebb7bd` (RX 9070 XT odaklı) ve PR `#215` (`VICTORY_LATE_SIGNALS=1`, Radeon 8060S'te -%12 kare süresi).
  - `ZSlayerHQ/zslayer-pc-port-runtime`, `CSTCryst/VIctory-Cryst-NVIDIA`, `soldsawn/victory-nvidia-8gb`, `edsuuu/VI-BR-PORT`, `trustno1-coder/VIctory-NVIDIA`, `CuteBooty/victory-8gb`: 00:40 UTC sonrası yeni kod commiti yok (yalnız `JozefDeTremmerie/CCS03` PR `#47` shader raporu ve `#45` texture bütçe/lock-free view bitleri).

---

## En Önemli 5 Yeni Aday

### 1. `timibloozinger/VIctory-compact` — `VICTORY_PAGE_MIRROR=1` + Commit `14115c5a` (Yalnız Dolu Sayfalara Tile Veren Sayfa Aynası)
- **Repo / Referans:** `timibloozinger/VIctory-compact` @ `14115c5a` (2026-11-25 19:46 UTC; repo 21:41 UTC'de açıldı)
- **Başlık:** *gpu: the page mirror gives tiles only where the title has memory (`VICTORY_PAGE_MIRROR_ALL=1`: as before)*
- **Durum:** `merged` (fork `main`)
- **Dosya / Satır:** 2 dosya, `+83 / -2` (`crates/victory-gpu/src/pagemirror.rs` +66/-1, `shadow.rs` +17/-1; tam `pagemirror.rs` modülü fork içinde)
- **Ne yaptığı:** Oyunun 4 MiB pencerelerinin yalnızca bir kısmı gerçekte eşlenmiş fiziksel sayfa içeriyor (yazarın ölçümünde 1.004 pencere / 4.316 MB alanın yalnız 1.700 MB'ı eşlenmiş havuz sayfası; kalanı boş). `VICTORY_PAGE_MIRROR=1` tüm pencereleri 64 KiB tile'lardan oluşan ortak bir reserved-buffer önbelleğinde topluyor; `14115c5a` ise yalnız Windows'un committed+readable bildirdiği sayfalara tile ayırıp boş sayfaları tile'sız bırakıyor (tile olmayan sayfa D3D12'de zaten 0 okunuyor).
- **Bizdeki beklenen etkisi:**
  - **Kanıt:** RTX 4050 Laptop 6 GB + 16 GB RAM, Windows 11, Series S koşularında (300 sn) PC→kart kilitli/paylaşılan bellek (**`NON_LOCAL` / shared GPU memory**) **5.4 GB → 4.0 GB** (-1.4 GB), oyun içi boş RAM **0.4–0.8 GB → 1.3 GB** oldu; kare süresi bozulmadı (180–195 ms, 5–5.5 fps).
  - **`TAHMİN`:** Bizdeki ~3.6 GB kilitli `NON_LOCAL` yükünün ve 87 bin dağınık 64 KiB görünümün yarattığı boş pencere şişmesini (yerinde pin riskine girmeden) doğrudan 1+ GB azaltma potansiyeli yüksek; RTX 3070 Ti 8 GB + ReBAR üzerinde henüz ölçülmedi.
- **Varsayılan / Env:** `VICTORY_PAGE_MIRROR=1` (varsayılan **KAPALI**); `14115c5a` davranışı mirror açıkken **AÇIK** (`VICTORY_PAGE_MIRROR_ALL=1` eski tam-tile davranışına döner).
- **Güvenlik:** Temiz (`14115c5a` yalnız 2 Rust dosyası; `build.rs`/script/ağ/indirme yok).

---

### 2. `fxceless/victory-fork` — PR `#1240` (Düşük RAM Havuz Budayıcı + Mip Sınırı + Toplu Descriptor Aynalama)
- **Repo / Referans:** `fxceless/victory-fork` PR `#1240` (`RealAzemed`, 2026-11-26 01:29 UTC)
- **Başlık:** *GPU: opt-in mip cap (`VICTORY_TEXTURE_MAX_DIM`), low-RAM pool trimmer (`VICTORY_RAM_TRIM_MB`), batched descriptor mirror & bind cache*
- **Durum:** `open`
- **Dosya / Satır:** 8 dosya, `+476 / -19` (`descriptor/host.rs` +257/-8, `draw.rs` +78/-1, `texture.rs` +63/-2, `host.rs` +29/-4, `list.rs` +24, `shadow.rs` +11, `resource.rs` +8/-4, `descriptor.rs` +6)
- **Ne yaptığı:** Boş fiziksel RAM `VICTORY_RAM_TRIM_MB` altına düşünce her 500 ms'de `shadow::POOL` (tutulan CPU snapshot `Vec<u8>`'leri), `texture::SPARE` (staging havuzu) ve `list::IDLE` (boşta host komut listeleri, taban 16) havuzlarını aktif olarak boşaltıyor. Ayrıca büyük 2D dokuları `VICTORY_TEXTURE_MAX_DIM` üstünde tiled mip-cap ile sınırlıyor ve `mirror_copied_batch` ile descriptor aynalamada tüm mutex'leri slot başına değil sayfa başına bir kez alıp sıfır sayfaları atlıyor.
- **Bizdeki beklenen etkisi:**
  - **Kanıt:** RTX 3070 8 GB + **16 GB tek kanal RAM** (Proton/vkd3d-proton, 420 sn çok sahneli açık dünya): RSS tepe **13.5 GB**, en düşük boş RAM **607 MB** (önceden açılış odasında 15.8 GB RSS + 6.5 GB swap ile OOM/thrash); `TEXTURE_MAX_DIM=1024` ile 2.93 GB VRAM tasarrufu (toplam 5.9–6.3 GiB / 8 GiB); descriptor sync 2.2–7.9 ms/kare.
  - **`TAHMİN`:** Bizde boş RAM sürekli ~0 olduğu için `VICTORY_RAM_TRIM_MB=2048` + `list::IDLE` taban 16 budaması sürücü/komut-listesi iç ayırmalarını ve boşta tutulan CPU/staging tamponlarını dizginleyebilir; Windows D3D12 üzerindeki `NON_LOCAL` düşüşü henüz ölçülmedi.
- **Varsayılan / Env:** `VICTORY_RAM_TRIM_MB=<MiB>`, `VICTORY_TEXTURE_MAX_DIM=<px>`, `VICTORY_BIND_CACHE=1` varsayılan **KAPALI**; `mirror_copied_batch` varsayılan **AÇIK**.
- **Güvenlik:** Temiz. Yalnızca `crates/victory-gpu/src/` altında 8 Rust dosyası; `build.rs`/script/ağ/indirme yok.

---

### 3. `Marvisak/VIctory` — PR `#2948` (Salt-Okunan Root UAV'lerde Geri Kopyayı Atla)
- **Repo / Referans:** `Marvisak/VIctory` PR `#2948` (`0xkn0wh3r3`, 2026-11-26 01:19 UTC; `DexNom/VIctory@bc3cd4b` kaynaklı)
- **Başlık:** *Shaders, GPU: a root UAV a shader only reads is not written back*
- **Durum:** `open`
- **Dosya / Satır:** 15 dosya, `+84 / -9` (çekirdek: `victory-shader/src/emit/shader.rs` +35, `victory-gpu/src/draw.rs` +14/-8, `shader_cache.rs` +9/-1; kalanı test imzaları)
- **Ne yaptığı:** Mevcut host, bir shader'a bağlı her root UAV'yi yazılmış sayıp (`Shadow::gpu_writes`, 1 MiB'a kadar) submission öncesi snapshot alıyor ve sonrası geri kopyalayıp karşılaştırıyordu. Çevirici artık `BufferLoad`/`GetDimensions`/`RawBufferLoad` dışındaki işlemleri (store/atomic) izleyip `HostShader::written_root_uavs` listesi üretiyor; host yalnız gerçekten yazılan root UAV'leri geri kopya listesine ekliyor (`VSH2` cache sürümü).
- **Bizdeki beklenen etkisi:**
  - **Kanıt:** Birim testi (`only_root_uavs_stored_to_or_reached_by_atomics_are_written`) geçti. Kaynak `DexNom@bc3cd4b` bütün yamada (sayfa dizini yazım takibi + bu parça) 12.8 → 2.7 GB/kare düşürmüş, ancak bu PR yazarının belirttiği gibi büyük pay zaten `main`'deki sayfa dizini takibinde; bu parçanın tekil oyun içi ms kazancı henüz ölçülmedi.
  - **`TAHMİN`:** Bizde kare süresinin en büyük kalemi geri kopya (**50–65 ms, 45 MB/kare**); salt-okunan root UAV'lerin elenmesi doğrudan gereksiz readback baytını ve snapshot/compare yükünü düşürür (küçük, +84/-9 satır, çok düşük risk). *Not: `vw-avatar` tabanında bu kontrolün mevcut olup olmadığı port öncesi doğrulanmalı.*
- **Varsayılan / Env:** Varsayılan **AÇIK** (env anahtarı yok; `VSH1` shader cache'ini `VSH2`'ye çevirir).
- **Güvenlik:** Temiz. Yalnızca Rust kaynak ve test dosyaları; `build.rs`/script/ağ/indirme yok.

---

### 4. `fxceless/victory-fork` — PR `#1229` + `#1230` + `#1231` + `#1232` + `#1233` (Geri Kopya / Readback Hızlandırma & Profil Serisi)
- **Repo / Referans:** `fxceless/victory-fork` PR `#1229`, `#1230`, `#1231`, `#1232`, `#1233` (`spk-c`, 2026-11-26 00:59–01:31 UTC)
- **Başlıklar:**
  - `#1230`: *perf: coalesce contiguous GPU readback commands* (`+96/-6`, `host.rs`)
  - `#1229`: *perf: avoid redundant readback selection allocations* (`+51/-27`, `host.rs`)
  - `#1232`: *perf: compact merged write intervals in place* (`+47/-6`, `host.rs`)
  - `#1233`: *perf: opt-in AVX2 changed-run detection* (`+131/-7`, `shadow.rs`)
  - `#1231`: *diagnostics: add heap and readback phase profiling* (`+69/-0`, `draw.rs`, `host.rs`)
- **Durum:** Beşi de `open` (her biri tek commit, izole CI'dan geçti)
- **Ne yaptığı:** Doğrudan geri kopya (readback) boru hattının CPU ve GPU kayıt aşamalarını küçültüyor: `#1230` aynı kaynaktaki bitişik GPU readback komutlarını tek `CopyBufferRegion`'da birleştiriyor (1.024 bitişik 64 KiB parça → 1 kopya komutu); `#1229` ve `#1232` readback seçim ve yazım aralığı birleştirmedeki geçici `Vec` tahsislerini sıfırlıyor; `#1233` değişen bayt taramasını AVX2 ile hızlandırıyor; `#1231` ise bizim 50–65 ms'lik geri kopyanın tam olarak seçim/kayıt/fence-bekleme/merge kırılımını ölçmeye yarıyor.
- **Bizdeki beklenen etkisi:**
  - **Kanıt:** Sentetik parity ve regresyon testleri geçti (1.024 kombinasyon, hizasız AVX2 testi); **uçtan uca oyun içi FPS/ms ölçümü YOK**.
  - **`TAHMİN`:** `#1231` (`VICTORY_READBACK_PROFILE=1`) bizim 50–65 ms copy-back'in ne kadarının fence bekleme, ne kadarının CPU merge/tarama olduğunu kesinleştirir; `#1230` + `#1233` özellikle 45 MB/kare geri kopyada komut kaydı ve CPU byte-diff süresini kısaltır.
- **Varsayılan / Env:** `#1230` **KAPALI** (`VICTORY_READBACK_COALESCE=1`), `#1233` **KAPALI** (`VICTORY_CHANGED_RUNS_AVX2=1`), `#1231` **KAPALI** (`VICTORY_READBACK_PROFILE=1`, `VICTORY_HEAP_PROFILE=1`); `#1229` ve `#1232` varsayılan **AÇIK**.
- **Güvenlik:** Temiz. Tümü yalnız `host.rs` / `shadow.rs` / `draw.rs` içinde; `build.rs`/script/ağ/indirme yok.

---

### 5. `avatarmanz/VIctory-7900XTX` — PR `#48` (GPU Bitiminde Dinamik Descriptor Aralıklarını Erken Geri Alma)
- **Repo / Referans:** `avatarmanz/VIctory-7900XTX` PR `#48` (`gomeaw`, 2026-11-26 00:11 UTC; son güncelleme 01:11 UTC)
- **Başlık:** *Reclaim dynamic descriptor ranges at GPU completion*
- **Durum:** `open` (doğrudan bizim taban `0a435665` üstüne yazılmış!)
- **Dosya / Satır:** 5 dosya, `+131 / -16` (`host.rs` +72/-2, `draw.rs` +26/-14, `descriptor.rs` +19, `list.rs` +9, `descriptor/host.rs` +5)
- **Ne yaptığı:** Geç gönderim (`VICTORY_BURY_LAG`) altında bir komut listesi GPU işi bittikten sonra bile 64 submission boyunca tutuluyor ve dinamik descriptor aralıklarını serbest bırakmıyordu; bu da sabit descriptor havuzunu (NVIDIA'da 1M sınır) tüketip diğer kayıt thread'lerini daraltıyordu. Bu PR, tamamlanan her flight'ın descriptor aralıklarını GPU fence tamamlanır tamamlanmaz (CPU readback'ten önce, liste henüz `BURY_LAG` ile tutulurken) geri alıyor.
- **Bizdeki beklenen etkisi:**
  - **Kanıt:** Windows WARP testiyle doğrulandı (fence bitmeden aralık korunuyor, fence bitince eski liste hâlâ flight içindeyken tüm aralık geri alınıyor, device-removed sentinel güvenli). Oyun içi RTX 3070 Ti ölçümü yok.
  - **`TAHMİN`:** Doğrudan bizim taban (`avatarmanz @0a435665`) üstünde çakışmasız uygulanabilir; dinamik descriptor baskısını ve NVIDIA 1M view heap taşma/tekrar ayırma baskısını azaltır.
- **Varsayılan / Env:** Varsayılan **AÇIK** (env anahtarı yok).
- **Güvenlik:** Temiz. Yalnızca 5 Rust dosyası (`victory-gpu`); `build.rs`/script/ağ/indirme yok.

---

## Diğer Yeni Adaylar (Kısa Tablo)

| Repo & Ref | Başlık / Ne Yaptığı | Durum | Dosya / Satır | Varsayılan / Env | Beklenen Etki (Kanıt vs `TAHMİN`) | Güvenlik |
|---|---|---|---|---|---|---|
| `Marvisak/VIctory` PR `#2958` (01:30 UTC) | *One lane of a wave sets a mark its wave shares*: Compute shader'larda `write_mark` / `read_mark` bitmap atomiklerini wave başına tek lane'e indiriyor (`waveReadLaneFirst`). | `open` | 2 dosya (`lower.rs`, `lower/marks.rs`), `+40/-0` | **AÇIK** (env yok) | **Kanıt:** RTX 4070 SUPER'de %25 daha kısa kare (mark bitmap `shared_buffer`'da olduğu için PCIe trafiğini kesiyor; yazar bitmap'i VRAM'e taşımanın CPU copy-back'i 2.8 sn'ye fırlattığını da kanıtladı). **`TAHMİN`:** Bizde ~30 ms GPU süresini ve PCIe/NON_LOCAL bus çekişmesini düşürür. | Temiz (2 Rust dosyası) |
| `AbsolutelyAryan/pluton` @ `57404798` (00:15 UTC) | 6 GB GPU + 16 GB RAM laptop fork'u: **`VICTORY_LIST_ALLOCATORS=pooled`** (komut listelerini yalnız kayıt sırasında alıp havuzlama), **`VICTORY_POOL_CHUNK_MB`** (64 MiB kesitlerle in-place), **`VICTORY_LAZY_COMMIT=1`**, **`VICTORY_CARD_BUFFERS_MIN_MB=1`**. | `merged` (fork `main`) | 208 dosya (tam fork snapshot'ı) | Tümü **KAPALI** (opt-in) | **Kanıt:** RTX 3050 6 GB + 16 GB RAM'de `CARD_BUFFERS_MIN_MB=1` copy-back **134 → 29 ms**; `LIST_ALLOCATORS=pooled` simülasyonda komut belleğini **9x** düşürdü (oyunda şimdilik device-lost!). **`TAHMİN`:** 2 GB "sahipsiz" `NON_LOCAL` sürücü tahsisinde komut ayırıcı payını araştırmak için `pooled` fikri ve memory-map teşhisi değerli. | **Dikkat:** `crates/victory-shader/build.rs`, `settings/laptop.bat.example`, `tools/perf/auto5.sh`/`auto6.sh` içeriyor — alınacaksa yalnız tekil Rust modülleri okunmalı. |
| `timibloozinger/VIctory-compact` @ `bfc12434` & `5481e233` (20:07–21:15 UTC) | `bfc12434`: Küçük dokuları 64 KiB committed yerine 4 KiB hizalı 16 MiB shared heap'lere yerleştirme (`smalltex.rs`). `5481e233`: Absent bloklar için her sorguda `VirtualQuery` (11.000/kare, 16.8 ms) yerine her 8. karede süpürme. | `merged` (fork `main`) | `bfc12434`: 4 dosya `+540/-14`; `5481e233`: 2 dosya `+60/-8` | `VICTORY_SMALL_TEXTURES=1` (**KAPALI**); `5481e233` **AÇIK** (`VICTORY_ABSENT_SWEEP=1` kapatır) | **Kanıt:** RTX 4050 6 GB'da 13.124 küçük doku 2.052 MB tutuyordu; `uncommitted_end` 16.8 ms/kare. **`TAHMİN`:** 8 GB VRAM'de doku sığdırmayı ve hazırlık (60–90 ms) süresini azaltır. | Temiz (yalnız Rust dosyaları) |
| `fxceless/victory-fork` Issue `#1089` / `zaidlutfi/xs-layer` @ `adbfe02c` (00:48 UTC) | Fiziksel bellek section'larını tek dev section yerine **2 MiB section**'lara bölme (`VICTORY_SECTION_MB=2`). `xperf` izinde Windows `MiRebuildLargePages` kilidinin 4 sn'de 2.06 sn map/unmap beklettiği kanıtlandı. | `open` (issue) / `merged` (`xs-layer`) | `xmem.rs` (+91/-38) | `xs-layer`'da **AÇIK** (`VICTORY_SECTION_MB=0` kapatır) | **Kanıt:** `xs-layer` (RTX 4060 Laptop 8 GB + 16 GB RAM, Series S) açık dünyada 2.8 → **3.8 fps** (yeni `main` ile birlikte). **`TAHMİN`:** Bizim 87 bin 64 KiB `SEC_RESERVE` görünümündeki map/unmap takılmalarını azaltabilir; ancak `beside()` / `Cover::Run` ile etkileşimi kontrol edilmeli. | `xs-layer` içinde `run.ps1` ve standart `build.rs` var; yalnız `xmem.rs` yaması incelenmeli. |
| `fxceless/victory-fork` PR `#1237` & `#1227` (00:53–01:26 UTC) | `#1237`: Thread-local weak `shadow::lookup` önbelleği (245.000 kilitli aramayı kaldırıyor). `#1227`: Send-and-hash listesini 3 adıma (kopyalar, tek barrier, hash) bölme. | `open` | `#1237`: 2 dosya `+98/-0`; `#1227`: `victory-gpu` | `#1237` **AÇIK**; `#1227` **AÇIK** | **Kanıt:** `#1237` 117 → **104 ms** (-13 ms CPU hazırlık); `#1227` send-and-hash GPU süresi **29 → 6 ms**. **`TAHMİN`:** Bizdeki 60–90 ms hazırlık ve 30 ms GPU kalemlerine doğrudan katkı. | Temiz (yalnız Rust) |
| `avatarmanz/VIctory-7900XTX` PR `#68` & `#28` (01:29 UTC) | `#68`: `watch::protect_here`/`protect_alias` içinde başlığın kendi `PAGE_GUARD` sayfasını ezmeme + bitişik blokları tek run'da koruma (`VICTORY_GUARD_RUNS`). `#28`: `VICTORY_LATE_WAITS=1` ters boolean bug düzeltmesi + shader cache hit + `warm-bad.txt`. | `open` | `#68`: 3 dosya `+584/-49`; `#28`: 3 dosya `+81/-13` | `#68` bug fix **AÇIK**, run batching env ile; `#28` **AÇIK** | **Kanıt:** `#68` sentetik `PAGE_GUARD` testinde `0a435665` bug'ını kanıtladı. **`TAHMİN`:** `0a435665` üstünde `VICTORY_LATE_WAITS=1` kullanıyorsak `#28` bug'ı doğrudan etkili. | `#28` içinde `scripts/play.ps1` (+1/-1) var; alınacaksa yalnız `ambient.rs` alınmalı. `#68` tamamen temiz. |
| `Marvisak/VIctory` PR `#2955` (01:30 UTC) | `placement.rs`: `free()` içinde ertelenmiş COM refcount reap, O(log H) `BTreeMap` heap indeksi, O(1) boş alan metaverisi, TLS `fits→place` önbelleği. | `open` | 1 dosya (`placement.rs`), `+325/-33` | **AÇIK** | **Kanıt:** Sentetik benchmark hızlanması. **`TAHMİN`:** Doku yerleştirme/eviction CPU yükünü düşürür (RAM miktarına değil CPU hızına etkili). | Temiz (tek Rust dosyası) |
| `fxceless/victory-fork` PR `#772` **yeni yorum** (00:52 UTC) | `UwUCammie` testi (RX 7900 XT, Wine/RADV): `VICTORY_INPLACE=4` + `VICTORY_INPLACE_POOL=1` 10.093 pencereyi import edip **9.99 GiB bellek kilitledi** ve beyaz geçiş ekranında kaldı. | `open` | (Mevcut PR'a yeni kanıt) | **KAPALI** | **Uyarı (Kanıt):** Tüm havuzu sınırsız `INPLACE` açmak 16 GB RAM makinede tam tersine kilitli belleği patlatıyor; `pluton`'un 64 MiB kesit sınırı veya `VICTORY_INPLACE_MB` tavanı şart. | — |

---

## 16 GB RAM / 8 GB NVIDIA — Yeni Saha Ölçümleri ve Env Setleri (26.11)

1. **`timibloozinger/VIctory-compact` (RTX 4050 Laptop 6 GB + 16 GB RAM + Win 11, Series S, 300 sn):**
   - **Sonuç:** **180–195 ms (5–5.5 fps)**, GPU 36–48 ms, `NON_LOCAL` (shared GPU memory) **5.4 → 4.0 GB**, boş RAM **1.3 GB** (süreç commit 15.6 GB).
   - **Env:** `VICTORY_PAGE_MIRROR=1`, `VICTORY_CARD_BUFFERS_MB=768`, `VICTORY_CARD_FETCH_INFLIGHT=0`, `VICTORY_ASYNC_SUBMIT=1`, `VICTORY_QUEUE2=1`, `VICTORY_TEXTURE_NO_WAIT=1`, `VICTORY_PIPELINE_WAIT_MS=24`, `VICTORY_FIDELITYFX_DIR=<FSR3.1>`, `--console series-s`.
2. **`fxceless/victory-fork` PR `#1240` (RTX 3070 8 GB + 16 GB 1-ch DDR5, Proton/vkd3d-proton, 420 sn):**
   - **Sonuç:** Peak RSS **13.5 GB**, min boş RAM **607 MB**, swap maks 4.5 GB, VRAM **5.9–6.3 GiB / 8 GiB** (2.93 GB VRAM tasarrufu), 11.9 fps tepe / 1.40 fps medyan.
   - **Env:** `VICTORY_LEVELS_ON_DEMAND=1`, `VICTORY_TEXTURE_MAX_DIM=1024`, `VICTORY_RAM_TRIM_MB=2048`, `VICTORY_BIND_CACHE=1`.
3. **`zaidlutfi/xs-layer` (RTX 4060 Laptop 8 GB + 16 GB RAM + 32 GB pagefile + Win 11, Series S, 10 dk):**
   - **Sonuç:** Çökmesiz, safehouse açık dünya (5.200 draw) **2.8 → 3.8 fps** (RT kapalı); RT açık 1.1–1.5 fps.
   - **Env:** `--console series-s`, `VICTORY_SECTION_MB=2` (varsayılan açık), `fxceless 32b31f75` tabanı.
4. **`AbsolutelyAryan/pluton` (RTX 3050 6 GB + 16 GB RAM + Ryzen 7 7840HS):**
   - **Sonuç:** `VICTORY_CARD_BUFFERS_MIN_MB=1` ile hazırlık **289 → 140 ms**, copy-back **134 → 29 ms**; ayrıca tüm page-pool'u tek heap'te in-place açmanın 16 GB PC'de sürücüye birkaç GB kilitletip 1+ dk stall ve device-lost verdiğini belgeledi (`VICTORY_POOL_CHUNK_MB=64` çözümü).
5. **`Cosatal/VIctory-4060` (RTX 4060 8 GB + 32 GB RAM + Win 11, Series X 1080p, doğrudan `avatarmanz/VIctory-7900XTX` fork'u, 13 dk):**
   - **Sonuç:** Orijinal `VIctory-7900XTX` 5.8 fps ama VRAM **7.35 / 6.65 GB** taşarken, 8 GB ayarlarıyla **5.6 fps (tepe 9)**, VRAM **6.75 / 6.67 GB**, 166.000 doku yükleme, çökmesiz. *(32 GB RAM olduğu için 16 GB davranışı `TAHMİN`.)*
6. **`Marvisak/VIctory` PR `#2960` (RTX 4060 8 GB + Win 11, Series X 1080p):**
   - **Sonuç:** `VICTORY_ASYNC_SUBMIT=1` + `VICTORY_QUEUE2=1` + `VICTORY_PIPELINE_WAIT_MS=24` + `VICTORY_PIPELINE_WORKERS=4` ile 17 s → **0.5–1.0 s** (bizim tabanda async/queue2 zaten var; `PIPELINE_WORKERS=4` kombinasyonu doğrulandı).
