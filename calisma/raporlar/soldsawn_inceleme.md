# `soldsawn/victory-nvidia-8gb` ve Neptune Overlay İncelemesi

## 10 Satırlık Özet
1. `soldsawn/victory-nvidia-8gb` (53★, `GPL-3.0-or-later`), `fxceless/victory-fork@f5c03f3b` üstüne **8 GB NVIDIA (Wave32) + Windows 11** için ~35 topluluk PR'ını ve `zm0kiemedia/Neptune_VI` performans overlay'ini birleştiren kaynak-only (hazır exe/release yok) bir fork'tur.
2. **En kritik bulgu:** Repo `--console series-s` DEĞİL, **Series X profili + `VICTORY_RESOLUTION=1920x1080`** (çözünürlük merdiveni tavanı, `037ef58933`) + **`VICTORY_TEXTURE_DROP_MIPS=2`** (`b4ff304f8d`+`7d0e3980d4`) + **`Cargo.toml` `lto = "thin"`** ile çalışır.
3. Ekran görüntüsündeki **2.565 draw/kare**, `soldsawn`'ın ekstra draw elemesi/culling yapmasından **değildir**; `host.rs` aynı sayacı yazar (`soldsawn` README'sinde 3.005–5.027 draw, `fxceless` README'sinde odada bakış açısına göre 1.550–4.500 draw). Kamera açısı + Series X/1080p sahne anlığıdır.
4. **RTX 5060 Laptop vs RTX 3070 Ti Laptop:** Ham raster gücü (3DMark Time Spy Graphics ~11.000–11.500 vs ~11.000–12.000) neredeyse **denktir**; 3070 Ti'nin GPU'su bu hız için engel değil, darboğaz **CPU hazırlık + geri kopya + 16 GB RAM baskısıdır**.
5. **16 GB RAM uyarısı (Kritik):** Ekran görüntüsünde sistem RAM kullanımı **28.2 / 31 GB** (süreç 12.6 GB); yazarın kendi testi 48 GB RAM'de. `soldsawn`'ın `CARD_WINDOWS_MB=2048` + `CARD_BUFFERS_MB=1024` + Series X ayarları 16 GB RAM'de doğrudan kopyalanırsa ağır pagefile/thrash riski taşır (`TAHMİN`).
6. Bizim taban (`avatarmanz/VIctory-7900XTX @ 0a435665`) AMD RDNA 3 (Wave64, 64 GB RAM) odaklı olduğu için **`lto = "thin"`**, **NVIDIA Wave32 CBV 64 KiB clamp (`09b95ca9c9`)**, **Paired Wave64 lowering (`26600c61e3`)**, **`VICTORY_TEXTURE_DROP_MIPS=2`** ve **`VICTORY_RESOLUTION=1920x1080`** bizde **yoktur**.
7. Güvenlik denetimi **temiz**: release/exe/zip yok, `build.rs` ve `overlay.rs` ağ/indirme/shell çağrısı yapmıyor (`queue2.rs`'deki `Command::new` yalnız birim testi), `scripts/play-nvidia-8gb.ps1` yalnız env ayarlayıp yerel binary'yi başlatıyor.
8. **En yüksek getirili, en düşük riskli alınabilirler:** (1) `Cargo.toml` `lto = "thin"` (+1 satır), (2) `09b95ca9c9` NVIDIA Wave32 CBV/shader fix (+122/-8), (3) `b4ff304f8d`+`7d0e3980d4`+`830ca05650` `VICTORY_TEXTURE_DROP_MIPS=2` (+401/-15), (4) `037ef58933` `VICTORY_RESOLUTION=1920x1080` (+265/-1), (5) `d919676c5b` `VICTORY_COPY_PLAIN=1` (+110/-11) veya `4e34eb042e` `VICTORY_SKIP_WC_COMPARE=1`.
9. `soldsawn` PR `#5` (26.11 01:47 UTC, açık, `ad3lre`) ayrıca shadow pool'u 512 MiB ile sınırlayıp scratch RAM şişmesini kesiyor — 16 GB RAM için ek takip adayı.
10. **Önerilen ilk adım:** Kod değiştirmeden önce mevcut `avatarmanz @ 0a435665` üstünde `VICTORY_WIDE_MERGE=1`, `VICTORY_JOIN_PARTS=8`, `VICTORY_FPS_COUNTER=0` ve düşük VRAM/RAM bütçesiyle ölçüm al; ardından `lto = "thin"` + `09b95ca9c9` + `DROP_MIPS=2` portla.

---

## 1. Repo Kimliği, Kökeni, Son Commitler ve Önerilen Ayarlar

### 1.1 Repo ve Köken (Upstream / Parent)
- **Repo:** `soldsawn/victory-nvidia-8gb` (53★, 9 fork, 5 açık PR, 0 release; oluşturulma `2026-11-25T13:52:45Z`, son push `2026-11-25T21:55:34Z`).
- **Parent / Upstream:** `fxceless/victory-fork` (o da `Marvisak/VIctory` fork'u).
- **Merge-base:** `fxceless/victory-fork @ f5c03f3b0f` (`2026-11-25 21:06 UTC`, PR `#999` `VICTORY_TEXTURE_BACK=1`). `soldsawn:main` (`c09a44e97d`), `f5c03f3b0f`'nin **152 commit önünde** (35+ topluluk PR'ı merge edilmiş), güncel `fxceless:main`'in 39 commit gerisinde (66 dosya farklı).
- **Neptune Overlay:** `zm0kiemedia/Neptune_VI` (34★, `GPL-3.0`, yazarlar `Traffic` & `@zm0kie`) kaynaklı yarı saydam performans paneli (`crates/victory-gpu/src/overlay.rs` 620 satır + `pixels.rs` 205 satır). `0e0954d2d7` ve `7c55a8c12c` commit'leriyle `soldsawn 8GB (overlay: Neptune)` başlığıyla gömülmüş; `F10` ile açılıp kapanıyor, açıkken yerleşik `fpsbox`'ı otomatik gizliyor (`b6e20af958`).

### 1.2 Son Commitler (Son 7 Gün — Tümü 25 Kasım 2026)
Repo tek günde (25 Kasım) v1 → v11 olarak birleştirilmiş:
- `c09a44e97d` (21:55 UTC): `chore: keep soldsawn/main history on v11 publish`
- `754a7de31e` (21:54 UTC): README + `scripts/play-nvidia-8gb.ps1` v11 güncellemesi (`f5c03f3` + Thin LTO + overlay)
- `7c55a8c12c` / `b6e20af958` / `0e0954d2d7` (21:26–21:51 UTC): `soldsawn 8GB` pencere başlığı + Neptune overlay + `fpsbox` otomatik gizleme
- `637a03fafc` (21:24 UTC): `fxceless@f5c03f3` (`VICTORY_TEXTURE_BACK=1`) merge
- `a9eb55950c` / `3c85889c9f` (21:05 UTC): `fxceless#1117` (`Azemed`: Thin LTO, `VIEWS RwLock`, lock-free `HostHeap::mirror`, `VICTORY_INPLACE=1`)
- `fa4847c904` / `3f5ecd8427` / `19f2ee63ca` (21:01–21:04 UTC): `fxceless#1122`, `#1120` (doku kopya tabanı ≤ 1/3 bütçe), `#1128` (descriptor sayfa atlama)
- `bd1680bdfd` / `d919676c5b` (20:17 UTC): `fxceless#1032` (`VICTORY_COPY_PLAIN=1` guarded in-place write-back)
- `ea9a6b57c2` / `aa1a1cde3a` / `32fc94da5e` / `d45a242e9a` (20:17–20:19 UTC): `fxceless#1043` (lock churn azaltma), `#1044` (dstorage fiziksel çekirdek), `#1069`/`#1086` (`VICTORY_REMAP_LEAN=1`/`BATCH=1`)
- `daf13a3da7` / `85a6ef0057` (16:40 UTC): `fxceless#1015` (`VICTORY_ALIAS_ONCE=1`), `#1007` (`VICTORY_HEAP_WORKER=1`)
- `a90ef376e2` (15:44 UTC): P-core pinning + lock-free `HostHeap::mirror`
- `830ca05650` (14:54 UTC): `VICTORY_CLAMP_WAITS=0` + `DROP_MIPS` içinde `<5` mip'li 2D UI dokularını koruma
- `fbb0b89ba8` / `80c3f8e3b2` / `9fd722d5ec` / `bae927c15b` (12:59–13:27 UTC): `fxceless#854` (Paired Wave64 + >32 KiB mesh link), `#868` (`VICTORY_SKIP_WC_COMPARE=1`), `#799` (NVIDIA Wave32 CBV 64 KiB clamp), `#610` (`VICTORY_TEXTURE_DROP_MIPS=N`), `soldsawn#1` (`037ef58933` `VICTORY_RESOLUTION=1920x1080`).

### 1.3 Kurulum, Çalıştırma ve Önerilen ENV / CLI Tam Listesi
- **Derleme:** `cargo build --release --bin victory` (`Cargo.toml` içinde `[profile.release]` altında `lto = "thin"` ve `debug = "line-tables-only"` açık).
- **Çalıştırma:** `.\scripts\play-nvidia-8gb.ps1 -Exe "path\to\title.exe"` → CLI olarak **`victory.exe run --execute <Exe>`** çağırır (**`--console series-s` YOK — varsayılan Series X profili**).
- **`scripts/play-nvidia-8gb.ps1` içindeki tam ENV listesi:**

| Değişken | Varsayılan Değer | Kaynak / Açıklama |
|---|---|---|
| `VICTORY_RESOLUTION` | `1920x1080` | `soldsawn#1` (`037ef58933`): Oyunun dahili çözünürlük merdivenini (1440p/2160p) 1080p ile sınırlar; Series X ışıklandırmasını korur. |
| `VICTORY_TEXTURE_DROP_MIPS` | `2` | `fxceless#610` (`b4ff304f8d`+`7d0e3980d4`+`830ca05650`): Büyük dokuların en üst 2 mip seviyesini hostta oluşturmaz ve yenilemez (`<5` mip'li 2D UI hariç). |
| `VICTORY_COPY_PLAIN` | `1` | `fxceless#1032` (`d919676c5b`): `write_mapped` için fault-guarded yerinde geri yazma (`plaincopy.rs`). |
| `VICTORY_INDIRECT_GATHER` | `1` | `fxceless@465b94f5`: Dağınık indirect kayıtları list prelude'da tek pass'te clamp eder. |
| `VICTORY_HEAP_WORKER` | `1` | `fxceless#1007` (`233a65f44a`): İndeksli doku yüklemelerini ayrı iş parçacığına taşır. |
| `VICTORY_ALIAS_ONCE` | `1` | `fxceless#1015`: Prelude alias kopyalarını kaynak yazımı başına bir kez yapar. |
| `VICTORY_JOIN_PARTS` | `1` | Ardışık submission parçalarını birleştirir (~29 → 9 submission/kare). |
| `VICTORY_BACK_OWN_VIEW` | `1` | Shader yazımlarını kendi görünümü üzerinden geri yazar (kendi yazımını tekrar okutmaz). |
| `VICTORY_INDIRECT_WINDOW_DRAWS` | `1` | `fxceless#987`: Indirect pencere çizim yolu. |
| `VICTORY_GUARD_HOLES_FRAME` | `1` | `fxceless#987`: Guard boşluklarını kare bazlı yönetir. |
| `VICTORY_COPY_CHANGED` | `1` | Karttaki kopyaları güncellerken yalnız değişen 256-byte blokları yollar. |
| `VICTORY_CARD_WINDOWS_MB` | `2048` | Karttaki gevşek pencere bütçesi (2 GiB — **8 GB VRAM + 16 GB RAM'de dikkat**). |
| `VICTORY_CARD_BUFFERS_MB` | `1024` | Karttaki buffer kopya bütçesi (1 GiB). |
| `VICTORY_CLAMP_WAITS` | `0` | `830ca05650`: Indirect clamp beklemelerini kapatır. |
| `VICTORY_WIDE_MERGE` | `1` | GPU geri kopya merge aşamasını çok çekirdeğe yayar. |
| `VICTORY_RELAY_OWN_QUEUE` | `1` | Copy-back relay işini kendi kuyruğuna alır. |
| *(Opsiyonel CLI bayrakları)* | `-Borderless`, `-Water`, `-NoOverlay` | `VICTORY_BORDERLESS=1`, `VICTORY_TEXTURE_BACK=1`, `VICTORY_OVERLAY=0`. |
| *(Kodda varsayılan AÇIK)* | `VICTORY_INDIRECT_WINDOW=1`, `VICTORY_LEVELS_ON_DEMAND=1` | `draw.rs` ve `69a8a3ebf2` ile varsayılan açık; `VICTORY_SKIP_WC_COMPARE=1` ve `VICTORY_INPLACE=1` kodda var ama launcher'da **kapalı**. |

---

## 2. Bizim Tabana (`avatarmanz/VIctory-7900XTX @ 0a435665`) Göre Farklar

> **Mimari bağlam:** Bizim taban (`avatarmanz/VIctory-7900XTX @ 0a435665`), AMD RX 7900 XTX (**Wave64, 24 GB VRAM, 64 GB RAM**) için optimize edilmiştir ve `soldsawn`'da **olmayan** şu hızlandırmalara zaten sahiptir: `VICTORY_THIN=1` (8 thread list çevirisi), `VICTORY_CARD_STEPS=2`, `VICTORY_INDIRECT_BATCH=2`, `VICTORY_LAND` / `VICTORY_RELAY=2` / `VICTORY_WIDE_BACK=1`, `VICTORY_UPLOAD_AHEAD=1`, `VICTORY_BIND_CACHE` (RT/pipeline cache), `VICTORY_DECLARE_ONCE`, `VICTORY_TEXTURE_BACK` varsayılan açık.
> Buna karşılık `avatarmanz @ 0a435665`, **NVIDIA Wave32** ve **8 GB VRAM** yamalarının neredeyse hiçbirini içermez.

### 2.1 `soldsawn`'da Olup Bizde (`0a435665`) Olmayan Kritik Farklar

| Kategori | Özellik / Commit SHA | Değişen Dosya ve Satır | Ne Yapar? |
|---|---|---|---|
| **Derleme (Hız)** | **Thin LTO** (`3c85889c9f`, `Cargo.toml`) | `Cargo.toml` (`+1/-0`) | `[profile.release]` içine `lto = "thin"` ekler; tüm crate'ler arası hot-path inlining sağlar. Bizde yok. |
| **8 GB VRAM** | **`VICTORY_TEXTURE_DROP_MIPS=2`** (`b4ff304f8d`, `7d0e3980d4`, `830ca05650`) | `texture.rs` (`+230/-8`), `resource.rs` (`+97/-1`), `descriptor/host.rs` (`+36`), `list.rs` (`+27/-4`) — toplam **`+391/-13`** | Büyük dokuların en üst 2 mip seviyesini hostta hiç ayırmaz ve yenilemez (`<5` mip 2D UI korunur). 8 GB VRAM taşmasını ve doku bozulmasını önler. |
| **8 GB VRAM + GPU Hız** | **`VICTORY_RESOLUTION=1920x1080`** (`037ef58933`, `soldsawn#1`) | `victory-game/src/output.rs` (`+177`), `victory-game/src/lib.rs`, `run.rs`, `lifecycle.rs` — toplam **`+265/-1`** (6 dosya) | Series X profilinde oyunun 1440p/4K çözünürlük merdivenini 1080p'ye kilitler + 2 MiB ring-buffer `int3` assert'ini atlar. Series X ışıklandırmasını koruyup 1.4–2.3 GB VRAM ve ~%40 GPU yükü kazandırır. |
| **8 GB VRAM** | **Doku kopya tabanı ≤ 1/3 bütçe** (`a360025758`, `fxceless#1120`) + unmapped placed doku serbest bırakma (`fxceless#432`) | `resource.rs` (`+13/-2`) + `resource.rs`/`texture.rs` | 8 GB kartlarda doku tabanını bütçenin 1/3'üyle sınırlar, unmapped placed dokuları hemen bırakır. |
| **NVIDIA Wave32** | **Wave32 CBV 64 KiB Clamp & Shader Fix** (`09b95ca9c9`, `fxceless#799`) | `descriptor/host.rs` (`+12/-2`), `emit/lower.rs` (`+67/-6`), `emit/mod.rs` (`+29`), `shader/resource.rs` (`+14`) — **`+122/-8`** (4 dosya) | NVIDIA'nın 64 KiB CBV sınırı ve Wave32 shader çeviri hatalarını düzeltir. Bizim `avatarmanz` ağacında CBV 65536 clamp yok. |
| **NVIDIA Wave32** | **Paired Wave64 Compute Lowering & >32 KiB Mesh Link** (`26600c61e3`, `fxceless#854`) | 18 dosya (`victory-shader`, `pipeline/host.rs`, `translate.rs`) — **`+4.562/-1.321`** | 64-lane konsol compute shader'larını 32-lane NVIDIA GPU'da eşleştirilmiş wave'lerle çalıştırır (+230 program geçer, aydınlatma/gölgeler düzelir). |
| **Geri Kopya (Copy-back)** | **`VICTORY_COPY_PLAIN=1`** (`d919676c5b`, `fxceless#1032`) | `plaincopy.rs` (`+109/-10`), `host.rs` (`+1/-1`) — **`+110/-11`** | `write_mapped` yolunda fault-guarded yerinde geri yazma yapar. |
| **Geri Kopya (Copy-back)** | **`VICTORY_SKIP_WC_COMPARE=1`** (`4e34eb042e`, `fxceless#868`) | 7 dosya (`census.rs` `+364`, `host.rs` `+40/-2`, `inplace.rs` `+25`, vb.) — **`+445/-2`** | Write-combining bellek kopyalarını hash/compare ve GPU→CPU geri kopyasından muaf tutar (opt-in). |
| **Hazırlık / CPU** | **`VICTORY_HEAP_WORKER=1`** (`233a65f44a`) + **`VICTORY_ALIAS_ONCE=1`** (`fxceless#1015`) + **`VICTORY_CLAMP_WAITS=0`** (`830ca05650`) + **P-core pinning** (`a90ef376e2`) | `heap_worker.rs` (`+288`), `indirect.rs` (`+10/-2`), `descriptor/host.rs`/`list.rs` (`+24/-1`) | İndeksli doku yüklemesini ayrı thread'e alır, alias kopyasını azaltır, indirect clamp beklemesini kaldırır. |
| **Overlay** | **Neptune Overlay** (`0e0954d2d7`, `b6e20af958`, `7c55a8c12c`) | `overlay.rs` (`+620`), `pixels.rs` (`+205`), `host.rs`/`lib.rs` — **`+833/-2`** | `F10` ile açılan FPS/30sn min-max-avg/Draws/CPU/GPU/RAM/VRAM paneli. |

### 2.2 Ekran Görüntüsünde Draw Sayısı Neden 2.565/kare (Bizde 3.800–4.800)?
- **Kanıt:** `soldsawn`'ın `host.rs` dosyasında `let draws = (drawn - self.drawn) as f64 / frames; overlay::set_frame(fps, ms, draws);` satırı, upstream `fxceless` ile **birebir aynı** draw sayacını kullanır. `soldsawn` içinde ekstra draw culling / frustum eleme veya kasıtlı draw atlama **yoktur**.
- **Neden 2.565?**
  1. **Kamera açısı / bakış yönü (Ana neden — Kanıt):** `soldsawn` README'sinde aynı repo ve aynı ayarlarla hapishane başlangıcında **`3.005–5.027 draw/kare`** ölçülmüştür. `fxceless` README'si de ilk odalarda kameranın baktığı yöne göre draw sayısının **`1.550` (duvar/hafif açı) ile `4.500+` (oda geneli)** arasında değiştiğini ve kare süresinin draw sayısıyla doğrusal (~32 µs/draw) ölçeklendiğini belgeler.
  2. **Series X vs Series S profil farkı (`TAHMİN`):** Ekran görüntüsü Series X + 1080p ile çalışıyor; Series S'te ekstra FSR/two-plane kompozit veya farklı LOD/pass dağılımı draw sayısını bir miktar değiştirebilir, ancak 2.565 → 4.000 farkının asıl sebebi anlık kamera açısıdır (aynı makinede 30 sn'lik `low 5.7 fps` ~175 ms'ye karşılık gelir, yani kamera dönünce o makine de 4.000+ draw'a ve ~6 fps'ye düşmektedir).

---

## 3. 9–11 FPS Kanıtları, GPU Karşılaştırması ve 16 GB RAM Aktarılabilirliği

### 3.1 9–11 FPS İçin Mevcut Kanıtlar
- **`soldsawn/victory-nvidia-8gb` README (Kanıt):** Intel i5-12450H + **RTX 4060 Laptop 8 GB** + 48 GB RAM + Series X + `1920x1080` + `DROP_MIPS=2` + `COPY_PLAIN=1` + Thin LTO → **`8–10 FPS` (`103.9–129.1 ms/kare`, 3.005–5.027 draw)**, GPU %50–58, CPU %45–50, **VRAM 6.6 / 7.0 GB**. (Aynı sistem 4K merdiveninde `6 FPS / 153–172 ms`, GPU %96, VRAM 7.6 GB.)
- **`soldsawn` PR `#3` (Kanıt — Zayıf Kart/CPU):** GTX 1660 6 GB + i7-4790K: v9 1080p + `DROP_MIPS=2` ile **5–6 FPS (181–240 ms, 4.187 draw, VRAM 3.98 / 4.93 GB)**; 4K'da 1.7–3.3 FPS.
- **`fxceless` Issue `#1083` (Kanıt — Birebir 8 GB + 16 GB RAM):** RTX 3070 8 GB + 16 GB RAM (48–64 GB pagefile) `soldsawn` Series X 1080p + `DROP_MIPS=2` ile **148 ms/kare (~6.8 FPS)** — ekran görüntüsündeki 94.9 ms (11 FPS)'den yavaş çünkü 16 GB RAM + farklı CPU + daha yüksek draw.

### 3.2 RTX 5060 Laptop vs RTX 3070 Ti Laptop (Kaynaklı Karşılaştırma)
- **GPU Gücü (Notebookcheck, 3DMark Time Spy Graphics):**
  - **RTX 5060 Laptop (100–115W, Blackwell, 8 GB GDDR7 128-bit):** ~**11.000 – 11.500** puan (orta-yüksek TGP ~11.164).
  - **RTX 3070 Ti Laptop (125–150W, Ampere, 8 GB GDDR6 256-bit):** ~**11.000 – 12.000** puan (Legion 5 Pro vb. 11.500–11.900).
  - **Sonuç:** Ham rasterizasyon gücü **neredeyse eşit** (hatta yüksek TGP'li 3070 Ti ufak farkla önde). Üstelik ekran görüntüsünde GPU kullanımı yalnız **%48** (bizde GPU süresi ~30 ms). Yani 11 fps vs 6.5 fps farkı **GPU gücünden gelmiyor**.
- **CPU Farkı:** Ekran görüntüsündeki **Ryzen 7 260** (AMD Hawk Point / Zen 4, 8 çekirdek / 16 izlek, 3.8–5.1 GHz, DDR5-5600/LPDDR5x-7500; Ryzen 7 8845HS'nin yeni adı) tek çekirdek ve bellek bant genişliğinde güçlüdür; CPU %74 yükte 94.9 ms'yi besleyebiliyor.

### 3.3 31 GB RAM vs Bizim 16 GB RAM — Ne Kadar Aktarılabilir?
- Ekran görüntüsünde süreç working set'i **12.6 GB**, ancak **toplam sistem RAM kullanımı 28.2 / 31 GB** (sürücü `NON_LOCAL` + sistem + page-pool kilitli bellekleri dahil).
- Yazarın kendi benchmark'ı da **48 GB RAM**'de yapılmış.
- **16 GB RAM için sonuç (`TAHMİN`):**
  - `soldsawn`'ın VRAM yamaları (`VICTORY_TEXTURE_DROP_MIPS=2` + `VICTORY_RESOLUTION=1920x1080`) **doğrudan aktarılabilir** ve 8 GB VRAM'in sistem RAM'ine (shared/NON_LOCAL) taşmasını önlediği için 16 GB RAM'e de **nefes aldırır**.
  - Ancak `play-nvidia-8gb.ps1`'deki `VICTORY_CARD_WINDOWS_MB=2048` + `VICTORY_CARD_BUFFERS_MB=1024` + Series X profili, 16 GB RAM makinede (özellikle pagefile 48–64 GB değilse) **commit/NON_LOCAL şişmesine ve sayfalama takılmasına** yol açar. Bizde `CARD_WINDOWS_MB` ve `CARD_BUFFERS_MB` daha düşük tutulmalı veya `soldsawn` PR `#5` / `Marvisak #2088` / `fxceless #1240` RAM budama yamalarıyla desteklenmelidir.

---

## 4. Güvenlik ve Lisans Denetimi

- **Lisans:** `GPL-3.0-or-later` (`Cargo.toml` ve `LICENSE`). `zm0kiemedia/Neptune_VI` da `GPL-3.0`.
- **Hazır exe / release / zip:** GitHub Releases **boş**; repoda `.exe`, `.dll`, `.zip` yok. (Biz zaten hazır binary KULLANMAYIZ, yalnız kaynak fikir/commit alırız.)
- **Betik ve `build.rs` denetimi:**
  - `crates/victory-shader/build.rs`: Yalnız `src/` altındaki `.rs` dosyalarını okuyup FNV-1a hash ve `VICTORY_*` env listesi üretiyor; ağ/disk yazımı/harici komut **yok**.
  - `tests/fixtures/hle-probe/build.rs`: 4 satır, yalnız test DLL delayload linker bayrağı.
  - `scripts/play-nvidia-8gb.ps1`: 50 satır; yalnız `Env:VICTORY_*` temizleyip ayarlıyor ve `target\release\victory.exe` çalıştırıyor. İndirme/ağ/kayıt defteri yok.
  - Tüm `fxceless@f5c03f3b...soldsawn:main` diff taraması (`http://`, `https://`, `Command::new`, `TcpStream`, `curl`, `Invoke-WebRequest`, `powershell`, `cmd.exe`): Tek gerçek kod eşleşmesi `crates/victory-gpu/src/queue2.rs` içindeki `#[test] fn device_removal_does_not_complete_a_copy_job` birim testinde `Command::new(std::env::current_exe())` çağrısı. `overlay.rs` tamamen temiz (yalnız Win32/DXGI metrik okuma ve piksel çizimi).

---

## 5. Bizim İçin En Değerli 5 Alınabilir Öğe ve Deneme Sırası

### 5.1 En Değerli 5 Somut Öğe (`avatarmanz @ 0a435665` Üstüne)

1. **`Cargo.toml`: `[profile.release] lto = "thin"`** (Commit `3c85889c9f` içinden, 1 satır)
   - **Ne yapar:** Release (ve `inherits = "release"` olan `diag`) derlemesinde crate'ler arası LTO açar.
   - **Beklenen etki:** **KANIT:** `soldsawn` v11'de (diğer PR'larla birlikte) 8–10 FPS hattının parçası. **`TAHMİN`:** Bizim 60–90 ms CPU hazırlık ve 50–65 ms copy-back CPU tarama döngülerinde %5–12 arası bedava hızlanma, sıfır RAM maliyeti, sıfır risk.

2. **`VICTORY_TEXTURE_DROP_MIPS=2` Serisi** (`b4ff304f8d` + `7d0e3980d4` + `830ca05650`, toplam `+401/-15`, 4 dosya)
   - **Ne yapar:** ≥5 mip'li dokularda en üst 2 mip seviyesini host VRAM'de ayırmaz ve staging yenilemesinde atlar; 2D UI keskin kalır.
   - **Beklenen etki:** **KANIT:** RTX 4060 8 GB'da VRAM 7.6 → 6.6 GB, doku/saç bozulması bitti; RTX 3070 8 GB + 16 GB RAM (`fxceless#1083`) 148 ms'de kararlı çalıştı; GTX 1660 6 GB'da 3.98 GB VRAM'e indi. **`TAHMİN`:** Bizde 8 GB VRAM taşmasını ve doku staging `NON_LOCAL` RAM baskısını keser; `TEXTURE_BUDGET_MB=1024` yerine çok daha güvenli VRAM kontrolü sağlar.

3. **NVIDIA Wave32 CBV 64 KiB Clamp & Shader Fix (`09b95ca9c9`, `fxceless#799`, `+122/-8`, 4 dosya)**
   - **Ne yapar:** NVIDIA D3D12 sürücüsünün 64 KiB CBV sınırını uygular ve Wave32 shader çeviri kenar durumlarını düzeltir.
   - **Beklenen etki:** **KANIT:** Tüm NVIDIA 8 GB fork'larının temel düzeltmesi (`soldsawn` + `fxceless#799`). **`TAHMİN`:** Bizim `avatarmanz` (AMD odaklı) tabanında NVIDIA kaynaklı gizli CBV/shader düşmelerini ve sürücü kararsızlığını önler; çok küçük diff (+122 satır).

4. **`VICTORY_RESOLUTION=1920x1080` Çözünürlük Merdiveni Tavanı (`037ef58933`, `soldsawn#1`, `+265/-1`, 6 dosya)**
   - **Ne yapar:** Series X profilinde 1440p/4K merdiven girişlerini bellekte 1080p'ye çeker ve 2 MiB ring-buffer `int3` assert'ini atlar.
   - **Beklenen etki:** **KANIT:** `soldsawn` RTX 4060 8 GB'da GPU yükünü %96 → %50–58'e, kare süresini 153–172 ms → 104–129 ms'ye, VRAM'i ~1.4–2.3 GB aşağı çekti; `fxceless#1083` RTX 3070 8 GB + 16 GB RAM'de Series S 720p hızında (148 ms) Series X 1080p görüntü verdi. **`TAHMİN`:** Series S'e alternatif olarak daha iyi görüntüyle 8 GB VRAM'e sığma imkânı sunar (16 GB RAM'de Series S'ten biraz daha fazla RAM isteyebilir).

5. **`VICTORY_COPY_PLAIN=1` (`d919676c5b`, `+110/-11`, 2 dosya) + `VICTORY_SKIP_WC_COMPARE=1` (`4e34eb042e`, `+445/-2`)**
   - **Ne yapar:** `COPY_PLAIN=1` `write_mapped` yolunu fault-guarded yerinde yapar; `SKIP_WC_COMPARE=1` write-combining bölgeleri GPU→CPU geri kopyadan çıkarır.
   - **Beklenen etki:** **KANIT:** `soldsawn` launcher'ında `COPY_PLAIN=1` aktif ve 8–10 FPS koşusunun parçası. **`TAHMİN`:** Bizde kare süresinin %35–40'ını yiyen **50–65 ms / 45 MB geri kopya** kalemini doğrudan hedef alır (önce `avatarmanz`'daki `VICTORY_RELAY=2` + `VICTORY_WIDE_BACK=1` + `VICTORY_LAND` ile çakışıp çakışmadığı kontrol edilerek portlanmalı). *(Not: Paired Wave64 `26600c61e3` `+4562/-1321` satır olduğu için görsel kalite için çok değerli ama `avatarmanz` üstüne birleştirme maliyeti yüksektir — 2. faz olarak düşünülmeli.)*

### 5.2 Önerilen Deneme Sırası
1. **Aşama 0 — Sıfır Kod, Yalnız ENV (Mevcut `avatarmanz @ 0a435665` üstünde hemen test):**
   - `VICTORY_WIDE_MERGE=1` (bizim `host.rs`'de zaten mevcut, kapalı geliyor — 50–65 ms copy-back CPU merge'ini çok çekirdeğe yayar).
   - `VICTORY_JOIN_PARTS=8` (mevcut `avatarmanz` `play.ps1` ayarı — kapalıysa aç).
   - `VICTORY_FPS_COUNTER=0` (`fxceless#1083` kanıtı: FPS kutusu tek başına ~20 ms/kare kaybettirebiliyor).
   - `VICTORY_THIN=1` + `VICTORY_CARD_STEPS=2` + `VICTORY_INDIRECT_BATCH=2` + `VICTORY_UPLOAD_AHEAD=1` aktifliğini doğrula (bizim 60–90 ms hazırlığı asıl indiren `avatarmanz` anahtarları).
2. **Aşama 1 — Tek Satır & Küçük Düşük-Riskli Portlar:**
   - `Cargo.toml` → `lto = "thin"` (yeniden derle ve ölç).
   - `09b95ca9c9` (NVIDIA Wave32 CBV 64 KiB clamp, +122/-8).
3. **Aşama 2 — 8 GB VRAM & 1080p Hattı:**
   - `b4ff304f8d` + `7d0e3980d4` + `830ca05650` (`VICTORY_TEXTURE_DROP_MIPS=2`).
   - `037ef58933` (`VICTORY_RESOLUTION=1920x1080` — Series X 1080p vs Series S karşılaştırması).
4. **Aşama 3 — Copy-back ve RAM İnce Ayarı:**
   - `d919676c5b` (`VICTORY_COPY_PLAIN=1`) ve/veya önceki rapordaki `Marvisak #2088` + `fxceless #1230/#1233` + `soldsawn PR #5` (512 MiB shadow pool sınırı).
