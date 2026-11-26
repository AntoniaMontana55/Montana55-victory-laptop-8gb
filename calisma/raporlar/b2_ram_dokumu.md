# B2 RAM dökümü (26.11, salt okunur)

Kaynak: `kosular\20261126-030711_b2-tails\run.log` 03:10:33-38 satırları (16162, 16165, 16168, 16170, 15961-15962), `olcum.csv`. Kod: `vw-avatar` @0a43566. PC: 16 088 MB fiziksel (Win32_ComputerSystem).

## 1. Fiziksel RAM hesabı (~200. sn)

| kalem | MB | kaynak |
|---|---|---|
| Boş (available) | 850 | copystat 16168 |
| Süreç çalışma kümesi (WorkingSetSize, paylaşılan dahil) | 7 558 | copystat.rs:503-512 = K32GetProcessMemoryInfo |
| Karta paylaşılan, kullanımda (NON_LOCAL CurrentUsage) | 4 727 | host.rs:9124-9145 |
| Kalan: OS + çekirdek + sürücü + diğer uygulamalar | 16 088 − 850 − 7 558 − (2 172…4 727) = **3,0…5,5 GB** | hesap |

- Koşu başında boş RAM yalnız **7,9 GB** (olcum.csv 1. satır). Yani oyun dışı ~8 GB doluydu. Oyun bunun 2,5-5 GB'ını çalışma kümesinden sıkıştırarak aldı. Thrash'in ana kaynağı bu.
- **Çakışma (çift sayım) var mı? Kısmen. TAHMİN:**
  - "write-combined 2555 MB in memory" (copystat.rs:647-675) sayfa başına QueryWorkingSetEx'in Valid bitini örnekliyor; 64 KiB'ta 1 sayfa bakılıyor.
  - "PC's shared 4727" ise VidMm'nin sistem segmentinde yerleşik tuttuğu ayırmalar.
  - Bellekteki WC sayfalarının bir kısmı (upload yığınları: scratch 240 + staging 576 + snapshot 122 MB ≈ 0,9 GB) her iki sayıda da var.
  - WC'nin geri kalanı büyük olasılıkla GPU_UPLOAD (BAR) eşlemeleri, yani kartın belleği, RAM değil (host.rs:9293+, shadow.rs:476-512). Kanıt: harita kalemlerinin "in memory" toplamı 9 393 MB. Bu, WorkingSetSize'ın (7 558) **1,8 GB** üstünde, BAR pencere tavanı da ≤1 664 MB ("host copies in the card's memory … 1664 MB", host.rs card_stats out[8]). Bu fark, BAR sayfalarının Valid görünüp RAM sayılmadığını düşündürüyor.
  - Sonuç: oyunun gerçek fiziksel payı **≈ 7,6 + (4,7 − 0,9…2,5) ≈ 9,8…11,4 GB**.
- **Sayfalanamayan (bellek yöneticisinin atamadığı) kalemler:**
  - (a) NON_LOCAL'de yerleşik D3D12 ayırmaları, 4 727 MB. Kilitli; yalnız VidMm Evict/bütçe baskısıyla iner. İçerik: pencere/host kopya shared_buffer'ları (CUSTOM WRITE_BACK L0, host.rs:9211-9260), upload yığınları (host.rs:8419-8450), staging havuzu (pool.rs:352), readback.
  - (b) Çekirdek/sürücü nonpaged.
  - Geri kalanların hepsi sayfalanabilir ve sistem bunları sayfalayarak yer açıyor: başlık heap'i 3 085 (667 bellekte), başlık sayfa görünümleri 5 619 (1 643), process heap 1 275 (586), "other private" 6 145 (2 599), dosyalar 7 557 (1 053). Sayfa hatası fırtınası buradan geliyor.

## 2. Büyük ayırma grupları kimin? (15962)

Bunların hiçbiri Rust ayırıcısından değil. Sayan ayırıcı (copystat.rs:383-427) ≥32 MB blokları listeliyor ve canlı yalnız **251,63 + 82,06 MB** var ("our own blocks of 32 MB and more"). Kodda testler dışında 1 GiB'lık VirtualAlloc yok (host.rs:1005+, inplace.rs:1268+ test modülleri).

| grup | sahibi | ne tutuyor | küçülten |
|---|---|---|---|
| 47×64 MB WC (3 008) | **9 tanesi kanıtlı bizim:** doku staging havuzu (BLOCK 64 MB pool.rs:30, STAGING_KEPT=8 pool.rs:37, log "576 MB of their staging buffers"). Kalan 38 (2 432 MB) 00:08:56'dan beri sabit. **TAHMİN:** BAR pencere blokları ya da sürücünün upload alt-ayırması. | upload (WC) | staging için env yok, yalnız kod (pool.rs:37) ya da `VICTORY_TEXTURE_POOL=0` (riskli) |
| 124×32 MB WC (3 856) | **TAHMİN:** sürücünün (NVIDIA UMD) committed upload/GPU_UPLOAD kaynaklarını 32 MB parçalara alt-ayırması. 18 → 124 artışı dünya yüklemesiyle birlikte. Bizim upload'larımızın tamamı ≤1 752 MB ("made so far 408 upload buffers", birikimli). | upload + BAR pencereler | `WINDOWS_BAR_MB`; pencere sayısı |
| 106×32 MB, WC değil (1 974; 00:08:56'da 2 571) | **TAHMİN:** shared_buffer (CUSTOM **WRITE_BACK**, WC değil, host.rs:9214-9218) alt-ayırması. Pencereler (sistem tarafı ≈ 2 692 − BAR ≤1 664 ≈ 1,0-1,5 GB) ve başlık buffer host kopyaları (3 497 − 2 692 ≈ 0,8 GB) burada. Pencereler bırakılınca 2 571 → 1 974'e düştü. | kilitli sistem RAM'i | daha çok pencereyi BAR'a almak (§4) |
| 4×1024 MB, WC değil (1 314 → 2 563, 36. sn'den beri) | Bizim değil, host adreslerinde. Commit'i büyüyen ayrılmış alan. **TAHMİN:** sürücü derleyicisinin/PSO arenası (nvwgf2umx / nvgpucomp). "pipeline being made" sürerken büyüyor. | sayfalanabilir; "other private 2 599 MB in mem" bunun içinde | ısıtılmış önbellek + `VICTORY_PIPELINE_WORKERS` düşük (**TAHMİN**) |

Sahipliği kanıtlamak için ucuz yol (~20 satır, yalnız diag): upload_buffer / shared_buffer / mapped_card_buffer Map'ten sonra `VirtualQuery(data)` ile AllocationBase'i alıp türe göre "taban → bizim tür" tablosu tutmak. copystat.rs:690-713'teki grup satırı da her grubu "ours: upload/shared/bar" diye etiketlemeli.

## 3. diagnostics feature'ın RAM maliyeti

- **"our own allocations 1 656 MB in 1,56 M blocks" diag maliyeti değil.** Sayan ayırıcı (copystat.rs:383-427) yalnız iki atomik sayaç tutuyor. 1,6 GB tüm sürecin Rust yığını ve release'te de aynen kalır.
- **book.rs:** açık, çünkü `D3D12_DRED=light` → `dred_any()` (book.rs:70-80). Canlı nesne başına bir Entry (`what: String`, `notes: String`, book.rs:39-46) ve en çok 400 000 "gone" kaydı tutuyor (GONE, book.rs:32). Canlı nesne sayısı ~35 bin (14,5k doku + 5,9k kopya + birikimli 9,9k shared / 5,1k card…). **TAHMİN 60-150 MB**; 1,54 M küçük bloğun önemli kısmı bu olabilir.
- **Memory map yürüyüşü:** 212k bölge × 48 B ≈ 10 MB geçici. RAM'den çok her 6. satırda 0,76 sn bellek çağrılarını tutmasıyla maliyetli (copystat.rs:563-567).
- **Release'te düşüş toplam ≈ 0,1-0,2 GB (TAHMİN).** Diag derlemede de `VICTORY_BOOK=0` ve `VICTORY_MEMORY_MAP=0` (copystat.rs:530-533) ile aynı kazanç alınır, derleme değiştirmeye gerek yok. Asıl sorun bu değil.

## 4. Kilitli belleği en çok düşürecek TEK kod değişikliği

**Öneri: BAR pencere kapısını %80'den ayarlanabilir %88'e çekmek.** Yer: `shadow.rs:496-512` `window_mapped_for` içindeki `used + size > budget / 10 * 8` satırı (shadow.rs:503-505).
- `VICTORY_WINDOWS_BAR_PERCENT` env'i eklenir, varsayılan 80 kalır. Yaklaşık 6-8 satır.
- **Neden:** kart 6 138 / 7 249 MB = %84,7. Kapı %80'de, bu yüzden B2'de yeni pencerelerin **hiçbiri** BAR'a gitmiyor; hepsi kilitli shared_buffer'a düşüyor. WINDOWS_BAR_MB=2048 tavanı bağlayıcı değil, canlı BAR ≤1 664 MB. Darboğaz yüzde kapısı.
  - %88 ≈ 6 379 MB → yaklaşık 580 MB oda açılır.
  - %88'in üstüne çıkılmamalı. Doku kuyruğu bütçenin %90'ında duruyor (shadow.rs:390-396 yorumu, `resource::room`); daha yukarısı dokuları aç bırakır (siyah zemin riski).
- **Beklenen etki:** kilitli −0,3…−0,6 GB (**TAHMİN**). Yalnız yeni yaratılan pencereler yer değiştirir; mevcut pencereler taşınmaz.
- **Doğruluk riski düşük:** aynı yol zaten ~1,6 GB pencereyi BAR'da taşıyor. CPU yalnız yazıyor (shadow.rs:476-484).
- **Performans/kararlılık riski orta:** kart bütçeye yaklaşınca OS dokuları ve render hedeflerini NON_LOCAL'e indirebilir. Bu kilitli belleği tersine **artırır**. Doğrulama: `video memory … card's own in use` < bütçe ve "the system changed the budget" satırı yok.

**Daha küçük ikinci aday:** `pool.rs:37 STAGING_KEPT 8 → 2`, 1 satır. Kanıtlı −384 MB WC/kilitli, risk düşük; tek bedeli upload buffer yeniden yaratma.

**Seçilmeyenler:**
- `host.rs:1837 SPARE_KEPT` (1 GiB scratch): şu an yalnız 89 MB yedek var ("101 chunks 89 MB spare"). Kazanç ≤ 0,09 GB.
- Boştaki pencereleri Evict / MakeResident etmek: 2 022 pencerenin 2 017'sini descriptor slot'ları gösteriyor (16170), bindless okumada hangisinin okunacağı bilinemez. Evict → page fault → device lost. 25.11'deki iki kayıp tam olarak buydu (host.rs:349-360). **Yüksek risk, önerilmez.**
- Pencere çakışmasını birleştirmek: 2 692 MB pencere 1 933 MB başlık belleği üstünde, yani 759 MB çift. Kazancı büyük ama find/slot kimliği yüzünden (#2856 kararı) yüksek risk.

## Env ile en etkili 2 ayar (koşu öncesi ≥10 GB boş RAM şartından sonra)

1. **`VICTORY_TEXTURE_BUDGET_MB=768`:** kartta ~250 MB daha yer açar; %80 kapısının altına inildiğinde yeni pencereler BAR'a gider. B1'de 1536 → 1024 adımı kartı −365, kilitliyi −150 MB indirmişti. **TAHMİN** −0,1…−0,3 GB; doku kalitesini görsel kontrol etmek gerek.
2. **`VICTORY_BOOK=0 VICTORY_MEMORY_MAP=0`** (ya da `D3D12_DRED` kapalı): **TAHMİN** −0,1…0,2 GB ve 0,76 sn'lik takılmalar gider. Kilitli belleği değil, çalışma kümesini düşürür.
- Etkisiz: `WINDOWS_BAR_MB`'ı yükseltmek (tavan bağlayıcı değil). `VICTORY_KEEP_FREED=0` (<12 GiB boşta zaten 5 dk tutmuyor, host.rs:374-378; üstelik device-loss riskini geri getirir).
