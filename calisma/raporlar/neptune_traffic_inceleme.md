# Neptune (`Victory Edit by Traffic & zm0kie`) ve `zm0kiemedia/Neptune_VI` İncelemesi

## 10 Satırlık Özet
1. **"Neptune (Victory Edit by Traffic & zm0kie)" iki katmanlıdır:** Tabanı **Traffic & imgone**'un `2026-11-25` akşamı paket (`victory.exe` + `neptune-ve-source.zip`) olarak dağıttığı `fxceless/victory-fork` tabanlı yapı; halka açık GitHub reposu ise bunu orphan kök commit (`347cac92`) olarak alıp üstüne NVIDIA/Ryzen yamaları ekleyen **`zm0kiemedia/Neptune_VI`** (34★, `GPL-3.0-or-later`, yazar Paul H / `zm0kie`).
2. **`build ef869ec4` hiçbir kamuya açık GitHub reposunda yoktur:** `crates/victory-gpu/build.rs` derleme anında `git rev-parse --short HEAD` değerini `VICTORY_BUILD` olarak gömer. `ef869ec4`, Traffic'in yerel git ağacından derlediği akşam binary'sinin SHA önekidir (`docs/NEPTUNE-README.txt`'deki tek test makinesi birebir ekran görüntüsündeki **i7-14700 + RTX 5070 Ti 16 GB + 64 GB RAM**).
3. **Draw başına maliyet:** Ekran görüntüsünde `111.4 ms / 4.652 draw = 23.9 µs/draw` (Neptune README açık dünya 6.500 draw / 90 ms = `13.8 µs/draw`); bizde `170 ms / ~4.000 draw = 42.5 µs/draw`. Draw başına maliyet bizden **~%44 daha düşük** (~1.78×).
4. **Donanım vs kod ayrımı (`TAHMİN`):** 170 ms → 111 ms (~59 ms) farkın **~%45–55'i donanımdan** (i7-14700 tek çekirdek %15–20 + sürekli 5.4 GHz turbo, RTX 5070 Ti 2.2–2.5× GPU + 16 GB VRAM/64 GB RAM sıfır sayfalama baskısı), **~%45–55'i kod/ayarlardan** gelir.
5. **İlginç bulgu:** `zm0kiemedia/Neptune_VI` `Cargo.toml`'da **`lto = "thin"` bile yok**; buna rağmen `CLAMP_AFTER_WRITER`, `UNDER_WRITES_WORDS/MASKS`, `QUIET_BACK`, `COPY_CHANGED`, `WATCH_4K`, `placement.rs` ve `UM_TEXLOOK` ile yüksek verim alıyor.
6. **16 GB RAM + 8 GB VRAM uyarısı:** Ekran görüntüsünde süreç **18.7 GB RAM** (sistem 39.0/64 GB), **11.3/14.0 GB VRAM** kullanıyor; `play.ps1` `CARD_WINDOWS_MB=4096` + `CARD_BUFFERS_MB=1536` + `LIST_POOL_MB=2048` ile 16 GB kart/64 GB RAM için ayarlı — 8 GB VRAM / 16 GB RAM'e doğrudan kopyalanamaz.
7. **Güvenlik temiz, 2 küçük not var:** `Neptune_VI`'de hazır exe/release yok; `overlay.rs` ağa çıkmıyor. Ancak `apps/victory/src/discord.rs` yerel `\\.\pipe\discord-ipc-N` üzerinden Discord Rich Presence açıyor (`VICTORY_DISCORD=0` ile kapalı), `xtask/src/agility.rs` ise yalnız manuel `cargo xtask agility` çalıştırıldığında `api.nuget.org`'dan SHA-512 doğrulamalı Microsoft `D3D12Core.dll` indiriyor.
8. **Bizde (`avatarmanz @ 0a435665`) zaten olup ENV ile hemen denenebilecekler:** `VICTORY_WIDE_MERGE=1`, `VICTORY_WIDE_REFRESH=1`, `VICTORY_ASK_ONCE=1`, `VICTORY_TEXTURE_NO_WAIT=1`, `VICTORY_CARD_REHASH_RUNS=1`, `VICTORY_JOIN_COPIES=1`, `VICTORY_WINDOW_TAILS=1`, `VICTORY_THIN=1`.
9. **Neptune'den portlanabilecek en değerli 5 yeni öğe** (`lto = "thin"`, `DROP_MIPS`, Wave32 CBV hariç): (1) `CLAMP_AFTER_WRITER` + `CLAMP_BATCH` (`09222fa7`+`ce39c721`+`61f22060`), (2) `placement.rs` placed texture heap'leri (`25fbf693`), (3) `DROP_DXIL` + `windowup.rs` + `LEVELS_ON_DEMAND`, (4) `UNDER_WRITES_WORDS/MASKS` + `QUIET_BACK` + `RELAY_OWN_QUEUE` + `COPY_CHANGED`, (5) `UM_TEXLOOK` (`0f39c0df`) + `WATCH_4K` (`25fbf693`) + `prof.rs` (`6cc42d53`).
10. **İlk adım:** Önce sıfır kodla `avatarmanz @ 0a435665` içinde zaten var olan 8 Neptune/upstream ENV anahtarını doğrula; ardından `DROP_DXIL` + `placement.rs` + `CLAMP_AFTER_WRITER` sırasıyla portla.

---

## 1. "Neptune (Victory Edit by Traffic & zm0kie)" Nedir ve `ef869ec4` Nereden Geliyor?

### 1.1 Repo, Köken ve `ef869ec4` İz Sürümü
- **Traffic & imgone paketi (`Neptune (VIctory Edit.)`):** Kamuya açık bir GitHub reposu yok. `zm0kiemedia/Neptune_VI` içindeki `docs/NEPTUNE-README.txt` doğrudan bu paketin kendi README'sidir: `2026-11-25, evening` tarihli pakette `victory.exe`, `play.ps1`, `neptune-ve-source.zip` ve `LICENSE` (GPL-3.0-or-later) dağıtılmış.
  - Tabanı: `fxceless/victory-fork` + `myokex/victory-amd` (`LEVELS_ON_DEMAND`, descriptor heap sync) + `soldsawn/victory-nvidia-8gb` (shader çevirici) + `Belobug`/`ShakiraRED-dt` (ambient ışık) + `ZSlayerHQ`, `JoshuuDev`, `skornuta` (`frametime.csv`), `Trexies`, `zm0kie` (overlay, `windowup.rs`, `ramline.rs`) + DexNom FidelityFX yönlendirme yaması.
  - `docs/NEPTUNE-README.txt`'deki tek test makinesi: **RTX 5070 Ti 16 GB, i7-14700, 64 GB RAM, Windows 11** — yani kullanıcının gördüğü ekran görüntüsündeki makineyle **birebir aynı sistem**.
- **Halka açık GitHub reposu (`zm0kiemedia/Neptune_VI`):** 34★, 4 fork (`destroyerreviews/Neptune-Optimized`, `bhenzy/Neptune_VI-Bhenz`, `The-laouza-buzz/Neptune_VI`, `oddiums/Neptune_VI`), 0 release, `GPL-3.0-or-later`. Yazar: Paul H (`zm0kie`). Traffic'in akşam kaynağını `347cac92` (`new base (2026-11-25 evening)`) orphan root commit'i olarak alıp üstüne kendi yamalarını eklemiş. Eski geçmiş `old-victory-fork` (`aee44cde`) dalında duruyor.
- **`build ef869ec4` nerede?**
  - `crates/victory-gpu/build.rs` derleme anında `git rev-parse --short HEAD` çalıştırıp `VICTORY_BUILD` env'ine yazar; `overlay.rs` bunu `"build {}"` olarak ekrana basar (`host.rs` pencere başlığına `"Neptune (Victory Edit by Traffic & zm0kie)"` yazar).
  - Global GitHub commit aramasında ve ilişkili 9 repoda (`zm0kiemedia/Neptune_VI`, 4 fork'u, `fxceless/victory-fork`, `Marvisak/VIctory`, `avatarmanz/VIctory-7900XTX`, `soldsawn/victory-nvidia-8gb`, `skornuta/victory-fork`) `ef869ec4` **yoktur**.
  - **Sonuç (Kanıt + Çıkarım):** `ef869ec4`, Traffic'in kendi yerel git reposundan derlediği `2026-11-25` akşamı binary'sinin commit önekidir; `zm0kiemedia/Neptune_VI` bu kaynağı `neptune-ve-source.zip` üzerinden yeni kök commit (`347cac92`) olarak yüklediği için orijinal git SHA korunmamıştır.

### 1.2 Son 7 Gün Commitleri (`zm0kiemedia/Neptune_VI:main`, 25–26 Kasım 2026)
- `b900c646` / `82454af5` / `ce39c721` / `09222fa7` (26.11 01:07–01:25 UTC): `VICTORY_CLAMP_AFTER_WRITER=1` (indirect argümanları yazıcı sonrası tek seferde sanitize etme, `play-zm.ps1` varsayılanı).
- `61f22060` (26.11 00:54 UTC): `VICTORY_CLAMP_BATCH=1` + eski doku slotlarını yenisi yüklenene kadar tutma (siyah gökyüzü düzeltmesi).
- `2c415c1d` (26.11 00:38 UTC): Upstream `ab7c9cf` `VICTORY_THIN=1` portu (`list/thin.rs`, `+1317/-13`).
- `6cc42d53` (26.11 00:34 UTC): `VICTORY_GPU_TIME=1` (`prof.rs`, GPU pass başına ms profili).
- `25fbf693` (25.11 23:58 UTC): Upstream `game-client 93d95bf` birleştirmesi (`placement.rs`, `WATCH_4K=1`, `cores.rs`, `ambient.rs` güncellemesi, `+2376/-142`).
- `0f39c0df` (25.11 23:52 UTC): `U-WW` write-watch sayaçları (`VICTORY_UM_WATCH`) + `U2` epoch tabanlı doku kontrolü (`VICTORY_UM_TEXLOOK`, `epoch.rs`).
- `7efb355c` (25.11 23:20 UTC): Unified memory çalışma belgeleri, `apps/um-spikes`, `U1` aralık birleştirme (`VICTORY_UM_COALESCE_KB`, `coalesce.rs`).
- `cd2a5744` / `eb27b506` / `c5546395` / `c25ee37a` / `fb0c93c0` / `1bf102d2` / `347cac92` (25.11 20:14–22:30 UTC): Yeni akşam tabanı, DRED, `VICTORY_INDIRECT_RAW=3`, Win10 Agility SDK (`fxceless#402`), `VICTORY_OCEAN_READBACK=1` varsayılanı.

### 1.3 Önerilen Profil, Ayarlar, ENV ve Derleme (`Cargo.toml`)
- **LTO:** `Cargo.toml` içinde `[profile.release]` yalnız `debug = "line-tables-only"` içerir — **`lto = "thin"` YOKTUR**.
- **Profil / Çözünürlük:**
  - **Traffic (`tools/play.ps1`):** Varsayılan **`--console series-s`** (720p dahili → PC `amd_fidelityfx_upscaler_dx12.dll` ile 1080p upscale; OptiScaler veya PC oyunu DLL'i gerektirir). DLL yoksa otomatik Series X (4K) açar. İlk odada 16–17 FPS (Series S) / ~14 FPS (Series X 4K); açık dünyada 6.500–9.000 draw'da 9–11 FPS.
  - **`zm0kie` (`tools/play-zm.ps1`):** README'de `-SeriesX` (4K) öneriyor, 32 GB RAM makineler için `-Ram32` (`LIST_POOL_MB=512`, `LIST_POOL_MOST=1024`, `TEXTURE_STAGE_MB=256`, `CARD_WINDOWS_MB=3072`) eklemiş.
- **Öne çıkan ENV seti (`play.ps1` / `play-zm.ps1`):**
  - *16 GB VRAM / 64 GB RAM bütçeleri (8 GB/16 GB'da düşürülmeli):* `CARD_WINDOWS_MB=4096`, `CARD_BUFFERS_MB=1536`, `LIST_POOL_MB=2048`, `LIST_POOL_MOST=2048`, `TEXTURE_STAGE_MB=512`, `WIDE_THREADS=12`.
  - *Hız & copy-back:* `RELAY=2`, `RELAY_OWN_QUEUE=1`, `WIDE_MERGE=1`, `WIDE_REFRESH=1`, `UNDER_WRITES_WORDS=1`, `UNDER_WRITES_RUNS=100000`, `UNDER_WRITES_MASKS=1`, `UNDER_WRITES_MASKS_LEAST=1`, `QUIET_BACK=1`, `ASK_QUIET=8`, `ASK_ONCE=1`, `COPY_CHANGED=1`, `JOIN_COPIES=1`, `CARD_REHASH_RUNS=1`, `WINDOW_TAILS=1`, `TEXTURE_NO_WAIT=1`, `CARD_WINDOW_UPLOAD=1`, `THIN=1`, `WATCH_4K=1`.
  - *Indirect & bellek:* `INDIRECT_RAW=2` (Traffic) veya `3` + `CLAMP_AFTER_WRITER=1` + `CLAMP_BATCH=1` (zm0kie), `LEVELS_ON_DEMAND=1`, `DROP_DXIL=1`, `DESC_DATA=1`, `VRAM_IDLE_MS=15000,2500,1000`, `PIPELINE_WAIT_MS=2500`, `TEXTURE_HASH_EVERY=32`, `WAVE_OWN_LDS_GROUPS=64,128`, `OCEAN_READBACK=1` (zm0kie).

---

## 2. Ne Çözmüşler? (Bizim `avatarmanz @ 0a435665` Tabanına Göre Farklar ve Draw Maliyeti)

### 2.1 Draw Başına Maliyet Karşılaştırması
- **Ekran görüntüsü (Neptune, i7-14700 + RTX 5070 Ti 16 GB + 64 GB RAM):** `111.4 ms / 4.652 draw = 23.9 µs/draw` (Neptune README'de açık dünya 6.500 draw / 90 ms = `13.8 µs/draw`, dağılım: %48 recording bekleme, %41 submit/memory/write-watch/texture, %11 oyun kodu, GPU meşguliyeti %35).
- **Bizim durum (`avatarmanz @ 0a435665`, i7-12700H + RTX 3070 Ti Laptop 8 GB + 16 GB RAM):** `170 ms / ~4.000 draw = 42.5 µs/draw` (3.800–4.400 draw aralığında `38.6–44.7 µs/draw`).
- **Sonuç (Kanıt):** Ekran görüntüsünde draw başına maliyet bizden **~%44 daha düşük** (`23.9` vs `42.5 µs/draw`). Ancak bunun tamamı kod değil (Bölüm 3'te ayrıştırılmıştır).

### 2.2 `avatarmanz @ 0a435665` Tabanında Olmayan Neptune Yamaları
*(Not: `VICTORY_WIDE_MERGE`, `WIDE_REFRESH`, `ASK_ONCE`, `TEXTURE_NO_WAIT`, `CARD_REHASH_RUNS`, `JOIN_COPIES`, `WINDOW_TAILS`, `THIN`, `PIPELINE_WAIT_MS` bizim `0a435665` ağacımızda **zaten mevcuttur**; aşağıdaki tablo yalnız bizde **eksik** olanları listeler.)*

| Hedef | Özellik / Commit SHA | Dosya & Satır | Ne Yapar? |
|---|---|---|---|
| **CPU/GPU Indirect** | `VICTORY_CLAMP_AFTER_WRITER=1` (`09222fa7` + `ce39c721`) | `indirect.rs` (`+234/-12`), `draw.rs` (`+52/-17`), `host.rs` (`+21/-1`), `shadow.rs` (`+5`) — **`+312/-30`** | Indirect argüman tamponlarını her draw öncesinde değil, GPU yazıcı pass'ından hemen sonra tek seferde sanitize eder; okuyucular raw çalışır. **Kanıt (`zm0kie`):** 4.6 → 8.4 FPS, GPU hang yok. |
| **CPU/GPU Indirect + Görsel** | `VICTORY_CLAMP_BATCH=1` + stale slot koruma (`61f22060`) | `descriptor/host.rs` (`+19/-2`), `draw.rs` (`+19/-2`), `host.rs` (`+9/-3`) — **`+49/-7`** | In-place indirect clamp'leri tek dispatch'te toplar; yenisi yüklenene kadar eski doku slotunu koruyarak siyah gökyüzü/nesne yanıp sönmesini giderir. |
| **CPU Hazırlık / Streaming** | `placement.rs` — Placed Texture Heaps (`25fbf693`, `Marvisak 11e9a544`) | `placement.rs` (`+288`), `resource.rs` (`+245/-2`) | Her doku için ayrı committed resource açmak yerine önceden ayrılmış heap'lere yerleştirir. **Kanıt (kod içi ölçüm):** 300 doku / 430 MB oluşturma **255 ms → 3 ms**. |
| **CPU Hazırlık / Upload** | `VICTORY_WATCH_4K=1` (`25fbf693`) + `VICTORY_UM_TEXLOOK=1` (`0f39c0df`) + `VICTORY_UM_COALESCE_KB` (`7efb355c`) | `watch.rs` (`+169/-12`), `epoch.rs` (`+70`), `texture.rs` (`+62/-3`), `coalesce.rs` (`+57`) | 4 KiB sayfa takibi; doku yenileme kontrollerini 64 KiB yazım epoch'u değişmediyse atlar; karta giden küçük upload aralıklarını birleştirir. |
| **Geri Kopya (Copy-back)** | `UNDER_WRITES_WORDS/MASKS/MASKS_LEAST` + `QUIET_BACK=1` + `RELAY_OWN_QUEUE=1` + `COPY_CHANGED=1` + `ASK_QUIET=8` (Neptune tabanı `347cac92`) | `host.rs`, `shadow.rs`, `watch.rs` | Shader yazımı altındaki küçük CPU değişikliklerinin bekleme üretmesini engeller, geri kopyalanan veriyi tekrar okutmaz, copy-back relay'i ayrı GPU kuyruğuna alır, yalnız değişen baytları yükler. |
| **RAM & VRAM** | `VICTORY_DROP_DXIL=1` + `VICTORY_CARD_WINDOW_UPLOAD=1` (`windowup.rs`) + `VICTORY_LEVELS_ON_DEMAND=1` + `LIST_POOL_MOST/MB` (`347cac92`) | `windowup.rs` (`272` satır), `pipeline.rs`, `texture.rs`, `resource.rs`, `list.rs` | Pipeline sonrası DXIL baytlarını RAM'den atar; transient pencere upload'ı için havuz kullanıp CPU kopyası tutmaz; yalnız kullanılan mip'leri VRAM'e alır; command list havuzunu sınırlayıp yeniden kullanır. |
| **Teşhis** | `VICTORY_GPU_TIME=1` (`6cc42d53`) | `prof.rs` (`+272`), `host.rs` (`+101`), `draw.rs` (`+15`) — **`+409/-1`** | Her 30 karede bir direct queue GPU zaman damgalarıyla en pahalı 25 shader/pass'ı 5 sn'de bir loglar (kapalıyken tek bayrak testi). |

---

## 3. Donanım Farkı (Kaynaklı Oranlar) ve Kazancın Ayrımı (`TAHMİN`)

### 3.1 Kaynaklı Donanım Oranları
- **CPU Tek Çekirdek — Core i7-14700 (Masaüstü) vs Core i7-12700H (Laptop):**
  - **Geekbench 6 Single-Core:** ~2.600–2.780 vs ~2.350–2.650 → **~%10–15** önde (Geekbench / NanoReview).
  - **Cinebench R23 Single-Core:** ~2.100–2.200+ vs ~1.800–1.850 → **~%15–20** önde.
  - **Cinebench 2024 Single-Core:** ~125–130 vs ~103 → **~%20** önde (NanoReview).
  - **Sürdürülebilir yük farkı:** i7-14700 5.4 GHz turbo ve 65W–219W masaüstü güç bütçesiyle saat hızını korurken, i7-12700H 4.7 GHz tepe / 45W mobil termal sınırda uzun yükte düşer; gerçek oyun döngüsünde efektif CPU farkı **~%15–25** (`TAHMİN`).
- **GPU — RTX 5070 Ti 16 GB (Masaüstü) vs RTX 3070 Ti Laptop 8 GB:**
  - **3DMark Time Spy Graphics:** ~28.000–32.000+ (Blackwell GB203, 16 GB GDDR7) vs ~11.000–13.000 (Ampere GA104 125–150W, 8 GB GDDR6) → ham raster gücü **~2.2× – 2.5×** (TechPowerUp / Notebookcheck).
  - **VRAM & RAM:** 16 GB VRAM + 64 GB RAM vs 8 GB VRAM + 16 GB RAM. Ekran görüntüsünde **11.3 GB VRAM** ve **18.7 GB süreç RAM'i (39.0/64 GB sistem)** tüketiliyor — bizim donanımda bu bütçe doğrudan shared VRAM / pagefile thrash'ine düşer.

### 3.2 170 ms → 111 ms (~59 ms) Farkın Ayrımı (`TAHMİN`)
- **Donanım payı (~%45–55 / ~27–33 ms):**
  - *GPU (~12–15 ms):* Bizim ~30 ms GPU süresi, 2.2–2.5× hızlı 5070 Ti'de ~12–15 ms'ye iner (ekran görüntüsünde GPU %47 — GPU ana darboğaz değil).
  - *CPU tek çekirdek + sürekli boost (~15–20 ms):* 60–90 ms hazırlık + copy-back CPU taraması ~%15–20 daha hızlı tek çekirdek ve geniş DDR5 bant genişliğiyle kısalır.
  - *Bellek rahatlığı:* 64 GB RAM + 16 GB VRAM, 16 GB RAM sistemdeki hard-fault/pagefile takılmalarını sıfırlar.
- **Kod / ayar payı (~%45–55 / ~26–32 ms):**
  - *Copy-back & senkronizasyon:* `UNDER_WRITES_WORDS/MASKS`, `QUIET_BACK=1`, `RELAY_OWN_QUEUE=1`, `COPY_CHANGED=1` (bizdeki 50–65 ms copy-back kaleminin en büyük açığı).
  - *Indirect & doku hazırlığı:* `INDIRECT_RAW=2/3` / `CLAMP_AFTER_WRITER`, `ASK_QUIET=8`, `placement.rs`, `LIST_POOL_MOST`.
  - *(Not: Neptune'de `lto = "thin"` olmadığı için aynı kod LTO ile ~%5–10 daha da hızlanabilir.)*

---

## 4. Güvenlik ve Lisans Denetimi

- **Lisans:** `GPL-3.0-or-later` (`Cargo.toml` + `LICENSE`).
- **Hazır exe / release / zip:** `zm0kiemedia/Neptune_VI` GitHub Releases **boş**, repoda `.exe`/`.dll`/`.zip` yok. (Traffic'in dışarıda paylaştığı pakette `victory.exe` vardı — biz yalnız kaynak fikir/commit inceliyoruz, hazır binary kullanmıyoruz.)
- **`build.rs` dosyaları (Temiz):**
  - `apps/victory/build.rs`, `crates/victory-gpu/build.rs`, `crates/victory-shader/build.rs`: Yalnız `D3D12\D3D12Core.dll` varlığına göre MSVC `/EXPORT` bayrağı basıyor, `victory-shader/src` FNV-1a hash'i üretiyor ve `git rev-parse --short HEAD` ile `VICTORY_BUILD` yazıyor. Ağ/indirme/zararlı komut yok.
- **İndirme yapan kod (Yalnız manuel xtask):**
  - `xtask/src/agility.rs` (`c5546395`, Win10 Agility SDK): Sadece kullanıcı `cargo xtask agility` çalıştırırsa `curl` ile `api.nuget.org`'dan pinlenmiş sürüm + SHA-512 doğrulamalı Microsoft `D3D12Core.dll` indirir. Normal build ve runtime'da çalışmaz.
- **PowerShell betikleri (`tools/*.ps1`):** `play.ps1`, `play-zm.ps1`, `start-neptune.ps1`, `memwatch.ps1` ağ/indirme yapmıyor (sadece yerel env ayarlıyor, `-FrameGen` verilirse yerel OptiScaler klasöründen `opti\`'ye kopyalıyor; `start-neptune.ps1` ve `play-zm.ps1 -Stats` içinde `D:\Neptune-VE\` sabit yolları var).
- **Overlay ve Discord IPC:**
  - `crates/victory-gpu/src/overlay.rs`: Ağ/telemetri **yok**; yalnız Win32 GDI/DXGI yerel sayaçları.
  - `apps/victory/src/discord.rs`: Doğrudan ağ açmıyor ama yerel `\\.\pipe\discord-ipc-0..9` pipe'ı üzerinden Discord istemcisine Rich Presence (`application_id = 1575009406737846333`, `"Neptune (VIctory Edit.)"`) gönderiyor. `play-zm.ps1` bunu `VICTORY_DISCORD=0` ile kapatmış; Traffic'in `play.ps1`'inde varsayılan açık. Portlamaya gerek yok.

---

## 5. Bizim İçin En Değerli 5 Alınabilir Öğe ve Deneme Sırası

*(Not: `lto = "thin"` zaten deneniyor; `VICTORY_TEXTURE_DROP_MIPS=2` ve NVIDIA Wave32 CBV `09b95ca9c9` önceki `soldsawn_inceleme.md` raporunda kapsandığı için aşağıda Neptune'e özgü **yeni** 5 öğe listelenmiştir.)*

### 5.1 En Değerli 5 Somut Öğe (`avatarmanz @ 0a435665` Üstüne)
1. **`VICTORY_CLAMP_AFTER_WRITER=1` + `VICTORY_CLAMP_BATCH=1` + Stale Slot Fix** (`09222fa7` + `ce39c721` + `61f22060`, toplam `+361/-37`, 4 dosya)
   - **Beklenen etki:** **KANIT:** `zm0kie` RTX 5070 Ti'de per-draw clamp'ten bu yola geçince **4.6 → 8.4 FPS** ölçtü ve siyah gökyüzü düzeldi. **`TAHMİN`:** Bizim 60–90 ms hazırlık/indirect yolunda 10–25 ms kazanç + NVIDIA'da indirect hang koruması.
2. **`placement.rs` — Önceden Ayrılmış Heap'te Placed Textures** (`25fbf693` içinden `placement.rs` + `resource.rs`, `Marvisak 11e9a544` kaynaklı)
   - **Beklenen etki:** **KANIT:** 300 doku oluşturma **255 ms → 3 ms**. **`TAHMİN`:** 8 GB VRAM + 16 GB RAM'de kamera dönüşü ve açık alana çıkış anındaki doku oluşturma takılmalarını ciddi keser.
3. **`VICTORY_DROP_DXIL=1` + `VICTORY_CARD_WINDOW_UPLOAD=1` (`windowup.rs`, 272 satır) + `VICTORY_LEVELS_ON_DEMAND=1`** (`347cac92` tabanından)
   - **Beklenen etki:** **KANIT:** `play.ps1` ve `play-zm.ps1`'de varsayılan açık; pipeline sonrası DXIL belleğini ve transient pencere CPU kopyalarını atar, yalnız kullanılan mip'leri VRAM'e koyar. **`TAHMİN`:** Bizdeki **16 GB RAM / 8 GB VRAM** darboğazında commit/working-set şişmesini doğrudan azaltır.
4. **Copy-Back & Alt-Yazım Paketi: `UNDER_WRITES_WORDS/MASKS` + `QUIET_BACK=1` + `RELAY_OWN_QUEUE=1` + `COPY_CHANGED=1`** (`347cac92` tabanından)
   - **Beklenen etki:** **KANIT:** Traffic'in 9–11 FPS (6.500–9.000 draw) ayar setinin ana copy-back omurgası. **`TAHMİN`:** Bizde karenin %35'ini yiyen **50–65 ms geri kopya** süresinde 15–30 ms potansiyel düşüş (`avatarmanz`'daki `RELAY=2` + `WIDE_MERGE=1` ile uyumlu portlanmalı).
5. **`VICTORY_UM_TEXLOOK=1` (`0f39c0df`, `epoch.rs`, `+220/-17`) + `VICTORY_WATCH_4K=1` (`25fbf693`, `watch.rs`, `+121/-8`) + `VICTORY_GPU_TIME=1` (`6cc42d53`, `prof.rs`, `+409/-1`)**
   - **Beklenen etki:** **KANIT:** `UM_TEXLOOK` ve `WATCH_4K` gereksiz doku taramalarını ve 64 KiB yanlış dirty upload'larını kesmek için yazılmış; `prof.rs` sıfır tahminle GPU pass ms dökümü verir. **`TAHMİN`:** 60–90 ms CPU hazırlık içinde 5–15 ms ek tasarruf + kendi donanımımızda net profil verisi.

### 5.2 Önerilen Deneme Sırası
1. **Aşama 0 — Sıfır Kod, Yalnız ENV (Mevcut `avatarmanz @ 0a435665` içinde zaten VAR olanlar):**
   - `VICTORY_WIDE_MERGE=1`, `VICTORY_WIDE_REFRESH=1`, `VICTORY_ASK_ONCE=1`, `VICTORY_TEXTURE_NO_WAIT=1`, `VICTORY_CARD_REHASH_RUNS=1`, `VICTORY_JOIN_COPIES=1`, `VICTORY_WINDOW_TAILS=1`, `VICTORY_THIN=1` (hangileri kapalıysa açıp ölçüm al; `CARD_WINDOWS_MB` / `LIST_POOL_MB` değerlerini 16 GB RAM için kesinlikle Neptune'deki 4096/2048'e **çıkarma**).
2. **Aşama 1 — RAM/VRAM Nefes Alma & Streaming (Küçük/Orta Portlar):**
   - `VICTORY_DROP_DXIL=1` + `placement.rs` (`25fbf693`) — `soldsawn`'dan `DROP_MIPS=2` ile birlikte 8 GB VRAM / 16 GB RAM'i korur.
3. **Aşama 2 — CPU Hazırlık & Indirect:**
   - `09222fa7` + `ce39c721` + `61f22060` (`CLAMP_AFTER_WRITER` + `CLAMP_BATCH` + siyah gökyüzü düzeltmesi) ve `0f39c0df` (`UM_TEXLOOK`).
4. **Aşama 3 — Copy-Back İndirimi:**
   - `QUIET_BACK=1` + `RELAY_OWN_QUEUE=1` + `COPY_CHANGED=1` + `UNDER_WRITES_WORDS/MASKS` (en yüksek copy-back getirisi, `avatarmanz` `land.rs`/`host/` yapısına dikkatli uyarlama ister).
