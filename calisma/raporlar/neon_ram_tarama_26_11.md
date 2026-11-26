# Victory / the title — 72 Saatlik RAM, 8 GB NVIDIA ve Series S Taraması (23–26 Kasım 2026)

- **Tarama penceresi:** 2026-11-23 00:00 UTC – 2026-11-26 00:40 UTC (son 72 saat, salt okunur `gh api` taraması).
- **Hedef donanım & sorun:** RTX 3070 Ti Laptop 8 GB VRAM (ReBAR açık), 16 GB RAM (~8 GB boş). Dünya + 90 sn'de kilitli `NON_LOCAL` 3.5–4.0 GB (pencereler + upload + staging), ~760 MB çift pencere, 17–19 GB private, 4×1 GiB NVIDIA sürücü PSO arenası (`TAHMİN`).
- **Taban:** `avatarmanz/VIctory-7900XTX @0a435665` (26 Kasım itibarıyla HEAD hâlâ `0a435665`; son 24 saatte yeni commit yok).
- **Repo erişim notları:**
  - `fxceless/victory` → 404; aktif repo **`fxceless/victory-fork`** (3.068★).
  - `Marvisak/VIctory` → ana upstream (12.618★).
  - `Belobug/victory` → 404; aktif repo **`Belobug/VIctory-Belobug-AMD`** (1.650★).
  - `CSTCryst/victory` → 404; aktif repo **`CSTCryst/VIctory-Cryst-NVIDIA`** (DexNom tabanlı NVIDIA fork'u).
  - `ZSlayerHQ/zslayer-pc-port-runtime` → aktif (`next` dalı, 89★).
  - `darkworldhm` → GitHub kullanıcısı/reposu bulunamadı (404).

---

## En Önemli 5 Aday

### 1. `Belobug/VIctory-Belobug-AMD` — Commit `0434b3ac` (Sayfa Havuzu Kesitlerini Yerinde Açma)
- **Repo / Referans:** `Belobug/VIctory-Belobug-AMD` @ `0434b3ac` (2026-11-25 16:36 UTC)
- **Başlık:** *Title memory shown page by page is used in place through a view of ours on the same stretch of its section*
- **Durum:** `merged` (main)
- **Değişen dosyalar / satır:** 3 dosya, `+182 / -3` (`crates/victory-gpu/src/watch.rs` +99, `shadow.rs` +80/-1, `copystat.rs` +3/-2)
- **Ne yaptığı:** Oyun streaming yığınlarını 64 KiB'lık ayrı ayrı `MapViewOfFile` görünümleriyle eşlediği için (ör. 75 MiB için 1.200 görünüm) standart `OpenExistingHeapFromAddress` (`INPLACE=1`) bunları tek heap olarak açamıyordu. Bu commit `watch::stretch` ile aynı section içinde hem sanal adreste hem section offset'inde ardışık giden 4 MiB+ serileri tespit edip ikinci bir bitişik `MapViewOfFile` görünümüyle eşliyor ve D3D12 heap'ini bu görünüm üzerinden kopyasız açıyor.
- **Bizim sorunumuza beklenen etkisi:**
  - **Kanıt:** Yazarın ölçümünde (AMD, `run mem2` vs `refl12`) 33 pencere kopyasız açıldı; copy-back 36 → 12 ms/kare, kare süresi 227–266 → 202–216 ms.
  - **İddia / `TAHMİN`:** Bizde reddedilen `INPLACE=1`'in kapsamadığı sayfa havuzu görünümlerini doğrudan hedefliyor; ~760 MB'lık çiftlenmiş pencere + `NON_LOCAL` kilitli RAM'i ciddi oranda düşürmesi beklenir (`TAHMİN`: NVIDIA Windows WDDM üzerinde doğrulanmadı).
  - **Kritik risk:** Kod yorumunda `shortcut: the view of ours is never unmapped` notu var (uzun oturumda sanal adres/pin birikimi yapabilir) ve Windows'ta `OpenExistingHeapFromAddress1` pin kilidi yüzünden **Aday 2 (`Marvisak #2921`)** olmadan remap/VirtualFree sırasında `ERROR_INVALID_PARAMETER (87)` riski taşır.
- **Varsayılan / Env:** Varsayılan **AÇIK** (`VICTORY_STRETCH=0` kapatır).
- **Güvenlik:** Temiz. Yalnızca 3 Rust dosyası (`victory-gpu`), `build.rs`/script/ağ/indirme yok.

---

### 2. `Marvisak/VIctory` — PR `#2112` + `#2088` + `#2921` (256 KiB Pencere + Yerinde Bellek + Windows Pin Güvenliği)
- **Repo / Referans:** `Marvisak/VIctory` PR `#2088` (taban), PR `#2112` (`#2088` üstü), PR `#2921` (Windows pin önkoşulu)
- **Başlıklar:**
  - `#2088`: *GPU: copy loose title memory in 256 KiB windows instead of 4 MiB*
  - `#2112`: *GPU: title memory in place instead of copied, opt-in (`VICTORY_IN_PLACE=1`)*
  - `#2921`: *HLE, GPU: title memory held in place is let go before the title or the runtime changes it*
- **Durum:** Üçü de `open` (25–26 Kasım'da aktif güncellendi)
- **Değişen dosyalar / satır:**
  - `#2088`: 1 dosya (`shadow.rs`), `+224 / -12`
  - `#2112`: 5 dosya (`shadow.rs` +1129/-15, `host.rs` +15, `memory.rs` +4, `watch.rs` +1, `README.md` +15), toplam `+1164 / -15`
  - `#2921`: 5 dosya (`pinned.rs` +431, `memoryapi.rs` +248, `xmem.rs` +6/-3, `lib.rs` +4, `mod.rs` +4), toplam `+693 / -3`
- **Ne yaptığı:** `#2088` kopyalanan gevşek pencereleri 4 MiB'tan 256 KiB'a indirerek kullanılmayan %70 dolguyu kesiyor. `#2112` ise `OpenExistingHeapFromAddress` ile hem gevşek pencereleri hem buffer nesnelerini doğrudan oyun belleğinden açıp host kopyasını ve sayfa korumasını tamamen atlıyor. `#2921`, Windows'ta `OpenExistingHeapFromAddress1` açıkken `VirtualProtect`/`UnmapViewOfFile2`/`VirtualFree` çağrılarının `ERROR_INVALID_PARAMETER (87)` ile çökmesini önlemek için değişiklikten önce pin'li heap'i bırakıp (geç kilit açmada yeniden deneyerek) güvenli hâle getiriyor.
- **Bizim sorunumuza beklenen etkisi:**
  - **Kanıt (`#2088`):** RTX 5080 (Linux/vkd3d) dünya yüklemesinde gevşek pencere belleği 3.4 GB → 2.17 GB, süreç RSS 17.3–20.4 GB → **15.4 GB** (~1.2–2 GB net düşüş).
  - **Kanıt (`#2112`):** Aynı ortamda `VICTORY_IN_PLACE=1` ile gevşek pencere host belleği 3.4 GB → **79 MB** (1.448 pencere yerinde, 0 red).
  - **Kanıt (`#2921`):** RTX 5070 Ti Windows'ta test edildi (40 bırakmanın 32'si <0.01 ms, en yavaşı 5.9 ms).
  - **İddia / `TAHMİN`:** 16 GB Windows makinede `#2088` tek başına bile `NON_LOCAL`'ı ~1.2 GB düşürür; `#2112` + `#2921` (+ Aday 1) birleşimi 3.5–4.0 GB kilitli pencere yükünün büyük bölümünü ortadan kaldırabilir (`TAHMİN`: `#2112` Windows NVIDIA'da henüz uçtan uca oyun ölçümü vermedi).
- **Varsayılan / Env:** `#2088` varsayılan **AÇIK**; `#2112` varsayılan **KAPALI** (`VICTORY_IN_PLACE=1`); `#2921` altyapı (her zaman aktif).
- **Güvenlik:** Temiz. Yalnızca Rust kaynak dosyaları ve README; `build.rs`/script/ağ/indirme yok.

---

### 3. `Marvisak/VIctory` — Commit `e04c0781` / PR `#2143` (Shader Sabit Tablosu Kısaltma — NVIDIA Çoklu-GB Sürücü Rezervasyonu)
- **Repo / Referans:** `Marvisak/VIctory` @ `e04c0781` (PR `#2143`, 2026-11-25 21:54 UTC)
- **Başlık:** *Shaders: a program's constant table starts after its instructions (GBs of NVIDIA video memory)*
- **Durum:** `merged` (main)
- **Değişen dosyalar / satır:** 1 dosya (`crates/victory-shader/src/emit/lower.rs`), `+49 / -7`
- **Ne yaptığı:** Hesaplanan offset ile `code_dword` okuyan shader'larda sabit tablosu 0. bayttan başlayıp tüm 64 KiB shader kodunu kopyalıyordu. Bu commit tabloyu yalnızca komutların bittiği yerden başlatıyor (sabit bir offset daha aşağıyı okumadıkça); böylece 26.084 program içinde ≥16 KiB sabit tablosu olan shader sayısı 25'ten **0**'a (≥8 KiB olanlar 35'ten 3'e) düşüyor.
- **Bizim sorunumuza beklenen etkisi:**
  - **Kanıt:** Linux/vkd3d-proton + RTX 5080'de tek bir 57 KiB sabit tablolu compute shader'ın NVIDIA sürücüsüne tüm yerleşik thread'ler için ~7.0 GB yerel bellek (local memory) ayırttığı `VK_KHR_pipeline_executable_properties` ile ölçüldü; bu yamayla bu ayrım sıfırlandı.
  - **İddia / `TAHMİN`:** Bizim Windows D3D12 NVIDIA sürücüsünde gördüğümüz 4×1 GiB PSO arenasının aynı "büyük sabit dizi → thread başına yerel bellek rezervasyonu" mekanizmasından kaynaklanıp kaynaklanmadığı **kanıtlanmamış (`TAHMİN`)**; ancak yalnızca 56 satır ve shader çıktısını bozmadığı 26.084 programda doğrulanmış olduğu için en düşük riskli / en yüksek olası getirili yama.
- **Varsayılan / Env:** Varsayılan **AÇIK** (env anahtarı yok).
- **Güvenlik:** Temiz. Tek dosya (`lower.rs`), `build.rs`/script/ağ/indirme yok.

---

### 4. `fxceless/victory-fork` — PR `#1219` (Düşük VRAM'de Büyük Dokuların Üst Mip Seviyelerini Atma)
- **Repo / Referans:** `fxceless/victory-fork` PR `#1219` (Steenax6, 2026-11-26 00:26 UTC; `#1218` üstüne yığılı)
- **Başlık:** *Large title textures are made without their most detailed levels when the card is short of room (`VICTORY_TEXTURE_DROP_LEVELS`)*
- **Durum:** `open`
- **Değişen dosyalar / satır:** 8 dosya (`tiles.rs` +1103/-60, `resource.rs` +424/-22, `texture.rs` +217/-41, `descriptor.rs` +39, `descriptor/host.rs` +30/-6, `host.rs` +20, `README.md` +19, `draw.rs` +5), toplam `+1857 / -129`
- **Ne yaptığı:** ≥1024 px kenarlı placed block-compressed 2D dokuları reserved texture olarak oluşturup kart belleği daraldığında en üst 1 veya 2 mip seviyesini ayırmadan bırakıyor; view'lardaki LOD clamp'i yükseltip boyut bilgisini koruduğu için shader'lar bozulmuyor ve yer açılınca üst seviyeler geri yüklenebiliyor (`VICTORY_TEXTURE_PROMOTE_MB`).
- **Bizim sorunumuza beklenen etkisi:**
  - **Kanıt:** Bizde `TEXTURE_BUDGET_MB=1024` (768'de görüntü bozuldu) yerine mip kesme yaklaşımı, 8 GB NVIDIA kartlarda (RTX 3070 8 GB + 16 GB RAM `fxceless#1083` yorumu ve `soldsawn/victory-nvidia-8gb` RTX 4060 Laptop 8 GB) `VICTORY_TEXTURE_DROP_MIPS=2` ile **6.6/7.0 GB VRAM**, bozulmayan dokular ve **148 ms (Series X 1080p)** olarak bağımsız doğrulandı.
  - **İddia / `TAHMİN`:** Üst 1–2 mip seviyesini hiç oluşturmamak hem 8 GB VRAM taşmasını (WDDM'in dokuları sistem RAM'ine/shared'a itmesini) hem de doku staging/upload RAM patlamasını keser (`TAHMİN`: `#1219`'un 16 GB RAM'deki private/NON_LOCAL ölçümü henüz PR'da paylaşılmadı).
- **Varsayılan / Env:** Varsayılan **AÇIK** (`VICTORY_TEXTURE_DROP_LEVELS=0` kapatır; `force1` / `force2` her koşulda 1–2 mip atar; `VICTORY_TEXTURE_PROMOTE_MB` geri yükleme eşiği).
- **Güvenlik:** Temiz. Yalnızca `crates/victory-gpu` Rust dosyaları + README; `build.rs`/script/ağ/indirme yok.

---

### 5. Series S Two-Plane Present + FSR Stand-in: `fxceless/victory-fork` PR `#1189` vs `ZSlayerHQ/zslayer-pc-port-runtime` PR `#10`
- **Repo / Referans:**
  - `fxceless/victory-fork` PR `#1189` (Tissma95, 2026-11-26 00:37 UTC)
  - `ZSlayerHQ/zslayer-pc-port-runtime` PR `#10` (Cleverdee, 2026-11-25 16:43 UTC)
- **Başlıklar:**
  - `fxceless#1189`: *Series S: interface plane laid over the game, FidelityFX upscaler stand-in*
  - `ZSlayerHQ#10`: *Series S: show the title's picture (two present planes, FSR stand-in) and keep AMD cards from losing the device*
- **Durum:** İkisi de `open`
- **Değişen dosyalar / satır:**
  - `fxceless#1189`: 5 dosya (`list.rs` +349/-1, `fidelityfx.rs` +291, `resource.rs` +17, `draw.rs` +10, `mod.rs` +2), toplam `+669 / -1`
  - `ZSlayerHQ#10`: 8 dosya (`standin.rs` +601, `host.rs` +578/-30, `queue.rs` +150/-2, `draw.rs` +109, `list.rs` +83/-19, `pipeline.rs` +26, `kernel.rs` +17, `lib.rs` +2), toplam `+1566 / -51`
- **Ne yaptığı:** `--console series-s` profilinde oyun 2 düzlem (1080p oyun + 4K arayüz) sunuyor ve 20 DXIL'siz AMD FidelityFX compute shader'ıyla 720p→1080p upscale yapıyor; ikisi de desteklenmediği için ekran siyah kalıyordu. `#1189` bunu HLE `fidelityfx.rs` + compute birleştirme ile (`VICTORY_COMPOSE=1`), `ZSlayerHQ#10` ise 24→72 byte `plane_parameters` stride düzeltmesi + `Host::present_planes` + düşen FSR dispatch'lerinde en büyük küçük renk SRV'sini kopyalayan `standin.rs` ile çözüyor.
- **Bizim sorunumuza beklenen etkisi:**
  - **Kanıt:** `fxceless#1083` (RTX 3070 8 GB + 16 GB RAM) ve `CuteBooty/victory-8gb` (RTX 3070 8 GB, 103 dk kesintisiz ilk görev) Series S profilinin 8 GB NVIDIA kartlarda çok daha düşük VRAM/RAM ayak iziyle kararlı çalıştığını doğruladı. `trustno1-coder/VIctory-NVIDIA` da `fxceless#1189`'u tabanımız `VIctory-7900XTX` üstüne alıp siyah menüleri çözdüğünü doğruladı.
  - **İddia:** Tabanımız `VIctory-7900XTX` (fxceless tabanlı) olduğu için `fxceless#1189` daha küçük (+669 satır) ve çakışması daha az; ancak `ZSlayerHQ#10`'daki 72-byte plane stride kontrolü göz önünde bulundurulmalı.
- **Varsayılan / Env:**
  - `fxceless#1189`: Varsayılan **KAPALI** (`VICTORY_COMPOSE=1`, `VICTORY_PRESENT_PLANE=n`, `VICTORY_PLANE_PREMULTIPLIED=1`).
  - `ZSlayerHQ#10`: Varsayılan **AÇIK** (`VICTORY_PRESENT_PLANES=0` kapatır, `=premul` premultiplied alpha).
- **Güvenlik:** İkisi de temiz (yalnızca Rust kaynak dosyaları; `build.rs`/script/ağ/indirme yok).

---

## Kısa Liste — Diğer İlgili Commit / PR Adayları

| Repo & Ref | Başlık / Özet | Durum | Dosya / Satır | Varsayılan / Env | RAM/VRAM Etkisi (Kanıt vs `TAHMİN`) | Güvenlik |
|---|---|---|---|---|---|---|
| `fxceless/victory-fork` @ `79cfdd96` | `VICTORY_WINDOW_UPLOAD=1`: Gevşek pencereleri ReBAR üzerinden tek GPU upload heap'e alma + `VICTORY_SHOT` | `merged` | 12 dosya, `+1858/-48` (`host.rs`, `shadow.rs`) | **KAPALI** (`VICTORY_WINDOW_UPLOAD=1`, `VICTORY_CARD_WINDOWS_MB`) | Bizim `VICTORY_WINDOWS_BAR_MB` yamasıyla benzer amaçlı; sistem RAM'inden 8 GB VRAM'e taşır (dikkat: 8 GB VRAM'i hızla doldurabilir, `TAHMİN`) | Temiz (Rust + README; `renderdoc.rs` pasif) |
| `Marvisak/VIctory` PR `#2222` (`#2120`+`#2130` üstü) | `host::keep_resident` (RT/DS/UAV → `HIGH`) + dinamik DXGI VRAM bütçesine göre upload heap payı | `open` | 10 dosya, `+2280/-35` (`shadow.rs`, `budget.rs`, `host.rs`) | **KAPALI** (`VICTORY_GPU_UPLOAD=1`, `VICTORY_GPU_UPLOAD_WRITES=1`; öncelikler aktif) | 8 GB VRAM dolduğunda Windows'un RT/DS'leri sistem RAM'ine demote etmesini engeller; sabit 2 GiB upload bütçesini karta göre ölçekler (`TAHMİN`: 8 GB'da ölçüm yok) | `shadow.rs`'de URL/komut eşleşmesi var (büyük ihtimalle yorum satırı; merge öncesi gözle bakılmalı) |
| `Marvisak/VIctory` PR `#2916` | `XMemMapPhysicalPages` ve `map_alias` içinde zaten commit'li section sayfalarını tekrar commit etmeme + placeholder yeniden kullanma | `open` | 1 dosya (`xmem.rs`), `+287/-28` | **AÇIK** (`VICTORY_MAP_LEAN=0` kapatır) | Adres alanı kilidi ve gereksiz commit çağrılarını azaltır; doğrudan kilitli RAM miktarından çok streaming takılmasını düşürür (`TAHMİN`) | Temiz |
| `ZSlayerHQ/zslayer-pc-port-runtime` PR `#4` | Doku staging buffer'larını boyut sınıfına göre havuzda tutup yeniden kullanma (54.279 alloc / 21 GB → yeniden kullanım) | `open` | 3 dosya (`texture.rs` +347, `draw.rs`, `inplace.rs`), `+365/-2` | **AÇIK** (`VICTORY_STAGING_POOL_MB=512`, `=0` kapatır) | **Dikkat:** RTX 5070 12 GB'da CPU upload süresini ciddi düşürdü (kanıt), ancak 512 MB boşta upload buffer tutması 16 GB RAM makinede kilitli `NON_LOCAL` RAM'i artırabilir — 16 GB'da düşük limit (örn. 64–128 MB) gerekir (`TAHMİN`) | Temiz |
| `Belobug/VIctory-Belobug-AMD` @ `f53ae3db` (PR `#30`) | Buffer-to-texture kopyalarının staging'ini yeni committed upload buffer yerine list'in havuzlu scratch'inden alma | `merged` | 2 dosya (`host.rs`, `list.rs`), `+30/-20` | **AÇIK** (env yok) | Küçük, temiz (+30/-20); her kopyada committed upload buffer yaratıp bırakma çalkantısını ve anlık `NON_LOCAL` sıçramalarını azaltır (`TAHMİN`) | Temiz |
| `Belobug/VIctory-Belobug-AMD` PR `#172` | Hiçbir lane'in almadığı dalları DXIL üretmeden önce budama (OOM pipeline creation workaround) | `open` | 5 dosya (`victory-shader`), `+645/-13` | **AÇIK** (env yok) | Sürücü PSO derlerken yaşanan OOM'u ve PSO bellek şişmesini azaltır (`TAHMİN`: AMD'de OOM için yazıldı, NVIDIA 4×1 GiB PSO arenasına etkisi belirsiz) | Temiz |
| `fxceless/victory-fork` PR `#822` | Sistem commit headroom'u düştüğünde boşta doku staging ve gevşek gölge kopyalarının bekleme süresini kısaltıp erken serbest bırakma | `closed` | 5 dosya (`host.rs`, `shadow.rs`, `texture.rs`, `library.rs`, `copystat.rs`), `+33/-31` | **AÇIK** (env yok) | **Kanıt:** Düşük commit replay'inde 7 → 125 present'a çıkardı; yalnızca 33 satır, 16 GB RAM için çok ucuz bir sigorta (fxceless toplu temizlikte kapattı) | Temiz |
| `fxceless/victory-fork` PR `#928` & `#965` | `#928`: Fiziksel sayfa section'larını sistem pagefile yerine sparse temp dosyayla destekleme (`VICTORY_PHYSICAL_BACKING_DIR`). `#965`: VRAM buffer staging'i sparse file + tek 16 MiB upload buffer ile yapma | `closed` | `#928`: 5 dosya `+278/-37`; `#965`: 10 dosya `+423/-40` | **KAPALI** (`VICTORY_PHYSICAL_BACKING_DIR`) | **Kanıt (`#965`):** 96 MiB staging'de yalnızca 0.96 MiB private commit artışı (sentetik probe). Pagefile/RAM baskısını diske taşır; ancak disk I/O gecikmesi ve geç I/O hatası riski var | Temiz (yalnızca `Cargo.toml`/`Cargo.lock` + Rust) |
| `Marvisak/VIctory` PR `#2494` | `SEC_COMMIT` section'lar üzerinde sıfır yeniden-içe-aktarma (zero-reimport) `InplaceArena` RFC prototipi | `open` (Draft) | 3 dosya (`inplace.rs` +2851, `xmem.rs` +893), `+3746/-22` | Pasif (henüz `shadow.rs`'ye bağlı değil) | Doğrudan kullanılamaz; gelecekteki tam kopyasız mimari için referans | Temiz |
| `fxceless/victory-fork` PR `#1178` | Lazy GPU write-back (VEH) + `VICTORY_REBAR=1\|2` + `AdapterKey` PSO | `open` | 4 dosya (`shadow.rs` +6131), `+6939/-532` | **KAPALI** (`VICTORY_LAZY_COPYBACK=1`, `VICTORY_REBAR=1\|2`) | Copy-back bant genişliği odaklı; çok büyük diff (+6.9k satır), RAM'den çok FPS hedefli, yüksek birleştirme riski | İncelenmedi (çok büyük diff) |
| `CSTCryst/VIctory-Cryst-NVIDIA` (`1fa80130`, `df27403e`) & `trustno1-coder/VIctory-NVIDIA` | NVIDIA 1.000.000 view heap sınırı üstündeki slotları slot-map ile yeniden eşleme ve 3 kare sonra geri alma (`VICTORY_DYNAMIC_SLOTS`) | `merged` (kendi fork'ları) | `victory-gpu` | Fork'a özel | RAM düşürmez; NVIDIA'da düşen çizimleri ve dinamik descriptor ring dolmasını önler | `trustno1-coder` içinde `scripts/play-nvidia.ps1` var (alınacaksa yalnızca Rust commit'leri alınmalı) |

---

## 16 GB RAM / 8 GB NVIDIA Saha Raporları ve Kullanılan Ayarlar

1. **`fxceless/victory-fork` Issue `#1083` (Birebir donanım sınıfı: RTX 3070 8 GB + 16 GB RAM + Win 11):**
   - **Kritik bulgu:** 16 GB fiziksel RAM ile **48–64 GB manuel pagefile** kullanmış; 10 dakikalık Series S koşusunda **18.5 GB private bellekte düz (flat)** kalıp çökmemiş (bizim 1.5–10 dk thrash sorunumuzda pagefile boyutu + Series S + `TEXTURE_BUDGET_MB` kombinasyonu belirleyici).
   - **Kullanılan env seti (144 ms/kare median, ~7 FPS):**
     `--console series-s`, `VICTORY_COPY_PLAIN=1`, `VICTORY_JOIN_PARTS=1`, `VICTORY_BACK_OWN_VIEW=1`, `VICTORY_ASYNC_SUBMIT=1`, `VICTORY_QUEUE2=1`, `VICTORY_TEXTURE_NO_WAIT=1`, `VICTORY_INDIRECT_WINDOW=1`, `VICTORY_INDIRECT_GATHER=1`, `VICTORY_INDIRECT_SINGLE=1`, `VICTORY_MONITOR_FENCE_WAIT=1`, `VICTORY_TEXTURE_BUDGET_MB=4096`, `VICTORY_FPS_COUNTER=0` (FPS kutusu tek başına 138→159 ms kaybettiriyor!).
   - **Aynı makinede `soldsawn/victory-nvidia-8gb` yorumu:** Series X **1080p** + `VICTORY_TEXTURE_DROP_MIPS=2` ile **148 ms/kare** (Series S 720p ile aynı hızda, daha iyi görüntü).
2. **`soldsawn/victory-nvidia-8gb` (50★, RTX 4060 Laptop 8 GB, 48 GB RAM, Win 11):**
   - `VICTORY_RESOLUTION=1920x1080` + `VICTORY_TEXTURE_DROP_MIPS=2` + `VICTORY_COPY_PLAIN=1` + Thin LTO → **8–10 FPS (104–129 ms)**, **6.6 / 7.0 GB VRAM**, saç/doku/aydınlatma sağlam. (48 GB RAM'de ölçüldüğü için 16 GB RAM davranışı `TAHMİN`.)
3. **`fxceless/victory-fork` Issue `#109` (RTX 3060 Ti 8 GB + 16 GB RAM + 32 GB pagefile):**
   - 32 GB pagefile ile oyun açılıyor ve ~2 FPS çalışıyor ancak prologda sayfalama yüzünden donmalar yaşanıyor (48–64 GB pagefile + Series S ile `#1083`'teki gibi stabilize olduğu görülüyor).
4. **`CuteBooty/victory-8gb` (RTX 3070 8 GB, 32 GB RAM, Series S):**
   - İlk hikâye görevini baştan sona 103 dk çökmeden bitirmiş (3–7 FPS); en büyük kazanç upstream `683ad7a9` (table constants in place) portundan gelmiş.
5. **Diğer aktif 8 GB / laptop fork'ları (takip edilmeye değer):**
   - `JozefDeTremmerie/CCS03` (11★, `victory-fork` üstüne 8 GB NVIDIA/laptop + Series S + üst mip atma; en uzun koşu 31 dk).
   - `edsuuu/VI-BR-PORT` (5★, RTX 4060 Ti 8 GB 1080p; shader'ların gerçekten okuduğu dokuları işaretleyip diğerlerini önce çıkarma + `VICTORY_TEXTURE_SHORT_HOLD=1`).
   - `naixue2233/Victory-naixue2233` (OOM/crash yamaları + drop-mips + `NtReadVirtualMemory`).

---

## Kısa Öneri Sırası (16 GB RAM + 8 GB VRAM İçin)
1. **Hemen / Sıfır Risk:** `Marvisak e04c0781` (+49/-7, shader sabit tablosu) + `Marvisak #2088` (256 KiB gevşek pencere → ~1.2 GB RAM tasarrufu) + `fxceless #822` (+33/-31, düşük commit'te erken staging/shadow temizliği) + `Belobug f53ae3db` (+30/-20, scratch staging).
2. **Asıl Kilitli RAM Kırıcı (Orta Risk):** `Marvisak #2921` (Windows pin güvenliği) + `Belobug 0434b3ac` (sayfa havuzu kesitlerini yerinde açma, unmap sızıntısı kapatılarak) + `Marvisak #2112` (`VICTORY_IN_PLACE=1`).
3. **8 GB VRAM & Series S:** `fxceless #1219` (`VICTORY_TEXTURE_DROP_LEVELS`) + `fxceless #1189` (Series S two-plane + FSR stand-in).
