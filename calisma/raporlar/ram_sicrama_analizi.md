# Yüklemeden dünyaya geçişte commit sıçraması: analiz (vw-pr10, Series S, 16 GB RAM)

Salt okunur inceleme, 25.11.2026. Kaynaklar: koşu 6-9'un `run.log`, `olcum.csv`, `ayarlar.txt`, `ozet.txt` dosyaları; kod `C:\CLLL\.calisma\vw-pr10`; başlatıcı `C:\CLLL\.calisma\repo_durum\guvenli_baslat.ps1`.
Saat notu: `run.log` UTC (18:02), `olcum.csv` yerel saat (21:02); aradaki fark 3 saat.

## Özet

1. **Tahmin büyük ölçüde doğru.** Kartın belleğindeki (DEFAULT heap) D3D12 kaynakları bu süreçte private bytes'a sayılıyor. Dört koşunun hepsinde `private − host textures` 5.1-6.3 GB'ta sabit kalıyor. Private bytes 5.7 GB'tan 13 GB'a çıkarken artışı doku kopyaları taşıyor. En sağlam kanıt: koşu 6, 7 ve 8'de 128×128 küp doku gibi **küçük DEFAULT dokular** `E_OUTOFMEMORY` ile reddedildi. O anda VRAM 6.6/8.2 GB'taydı, yani bütçenin (7249 MB) altındaydı. Ret, private bytes tam 13.000 GB'a ulaştığı saniyede geldi.
2. **Koşu 9'un son ~4,3 sn'si (18:02:29,5 → 18:02:33,8, +4,27 GB) şöyle dağılıyor:**
   - Doku kopyaları ~1,3-1,4 GB (VRAM ve commit; 4096 MB bütçesi 18:02:31,25'te doldu).
   - Kart tamponları: 79 tampon, 656 MB kart tarafı, ayrıca 656 MB PC RAM'inde staging. "before" ikizleri 0-656 MB ekler. Toplam 1,3-2,0 GB, hepsi 18:02:31,9-32,72 arasında oluştu.
   - Shader ve pipeline derleme ~0,2-0,8 GB (143 çeviri, 8 işçi).
   - Upload tamponları ~0,1 GB.
   - Bu kalemlerin toplamı 3,0-4,3 GB. VRAM ölçümü +2,1 GB, working set ölçümü +1,6 GB ile tutarlı.
3. **13 GB commit tavanı bu makinede aslında "RAM + VRAM ≤ 13 GB" anlamına geliyor.** Fiziksel RAM'in tepesi (working set) yalnız 7,2-7,4 GB. Sistemde hâlâ 25 GB commit boştu, yani sistem commit'i darboğaz değil.
4. **`VICTORY_CARD_BUFFERS_MB=1024` ve `CARD_BUFFERS_MIN_MB=1` koddaki varsayılanlar** (`shadow.rs:158`). #413 reçetesindeki bu iki ayar etkisiz; zararı da kazancı da yok. RAM'e asıl etki MIN=1'den geliyor: 1 MiB üstündeki her tampon kart kopyası alıyor, bu tampon başına 2-3 kat bellek demek.
5. **`WINDOW_KB=512` ve `WINDOW_TAILS=1` bu evrede bir şey kazandırmıyor.** Tampon dışı pencereler toplam 9 MB (`run.log` 18:02:29 "windows: the 9 MB").
6. **Önerilen sıra:**
   - (0) Başlatıcıda commit tavanını yükselt; fiziksel RAM'i boş RAM ve working set ile koru.
   - (1) Diğer programları kapat: koşu başında boş RAM yalnız 9,28 GB'tı.
   - (2) `VICTORY_TEXTURE_BUDGET_MB=2048`.
   - (3) `VICTORY_CARD_BUFFERS_MIN_MB=16` ya da `VICTORY_CARD_BUFFERS=0`.
   - (4) `VICTORY_ACTIVATE_RAYS=0` ve `VICTORY_RT_RESOLVE=0`. Koşu 9'da bu ikisi kaldırılmıştı, yani **açıktı**.
7. **Gözden kaçan uyarılar:** aynı compute shader dört koşunun hepsinde `0x80070057` ile reddediliyor (bellekle ilgisi yok). Görünüm heap'i 1.000.000'da kesiliyor (1.245.184 istendi). Çöküşten sonra sistemin boş commit'i 13-16 GB düşüyor; bu büyük olasılıkla çökme dökümü (TAHMİN).

---

## 1. Son 5-10 saniyedeki sıçramanın kaynağı

### 1.1 Koşu 9 zaman çizelgesi (kanıt)

| Zaman (UTC / yerel) | Olay | Kaynak |
|---|---|---|
| 18:02:29,53 | Son bellek dökümü: **8730 MB private** (5593 MB'ı bellekte), host textures 15771 alive **3163 MB**; title fills 2780 MB; render/yazma kopyaları 383 MB; 9330 doku hâlâ kopyasız (**6018 MB** başlık belleği bekliyor); kart 3663/7249 MB | `run.log:1139,1141` |
| 21:02:30 → :31 | priv 9,04 → 10,33 GB, VRAM 3667 → 4284 MiB | `olcum.csv` |
| 18:02:31,25 | `textures picked by index: the card has no room for more; they wait`: 4096 MB `TEXTURE_BUDGET` doldu | `run.log` (WARN, once) |
| 18:02:31,86 → 18:02:34 | 143 shader çevrildi (saniyeye göre 22/74/43/4), ~129 host pipeline kuruldu | `run.log` |
| 18:02:31,91 | `host pipeline refused 0x80070057 shaders=[compute 02384a60fda6f2d0]` | `run.log` |
| 18:02:31,91 → 18:02:32,72 | **78 adet** `a buffer's host copy is in the card's memory`, toplam **656 MB**; 6'sı ≥16 MiB ("large buffer", 195 MB) | `run.log`; toplam Select-String ile hesaplandı |
| 21:02:32 / :33 / :34 | priv 10,88 / **12,66** / **13,00** GB; WS 6,25 / **7,18** / 6,44; VRAM 5105 / 5389 / **5766** MiB; boş RAM 1,61 / **0,338** / 0,83 GB | `olcum.csv` |
| 18:02:33,82 | İlk `no shared buffer … (0x8007000E) size=1048576`; bunu 21.347 tekrar izliyor | `run.log` |
| çıkış | Kod 3221226505 = **0xC0000409** (fail-fast, Rust/LLVM abort) | `ozet.txt` |

Diğer koşular da aynı deseni gösteriyor:
- Koşu 8: priv 9,8 → 13,0 GB (20:59:34-42); VRAM 4,7 → 6,8 GB; WS 5,87 → 7,36 GB; boş RAM en düşük **0,018 GB** (20:59:45).
- Koşu 6 (doku bütçesi yok): 17:50:48'de host textures **6218 MB**, priv 12483 MB.

### 1.2 Sabit taban: `private − host textures`

| Koşu | İlk döküm | Son döküm | priv − tex (ilk → son) |
|---|---|---|---|
| 6 (4 MiB ızgara) | 17:50:21 priv 5962 / tex 822 | 17:50:48 priv 12483 / tex 6218 | 5140 → 6265 |
| 7 (WINDOW 512) | 17:56:30 5747 / 622 | 17:57:00 13109 / 6769 | 5125 → 6340 |
| 8 (CATCHUP 0) | 17:59:09 7756 / 2188 | 17:59:36 10330 / 4527 | 5568 → 5803 |
| 9 (BUDGET 4096) | 18:02:03 5745 / 608 | 18:02:29 8730 / 3163 | 5137 → 5567 |

Private bytes 7 GB oynarken taban yalnız ~0,4-1,2 GB oynuyor. Artışın ~%85'i host doku baytına bire bir uyuyor. Host dokular `CreateCommittedResource` + `D3D12_HEAP_TYPE_DEFAULT` ile, bayrak olmadan (`D3D12_HEAP_FLAG_NONE`) kuruluyor (`resource.rs:1896-1915`). Bunlar PC RAM'inde değil, kartta duruyor.

### 1.3 Adaylar: kanıt ve MB tahmini (koşu 9, son ~4,3 sn, +4,27 GB)

| Aday | Kod | Kanıt | Bu pencerede tahmin |
|---|---|---|---|
| **(a1) Host doku kopyaları (VRAM, commit'e sayılıyor)** | `resource.rs:1842-1925` (DEFAULT committed); bütçe `resource.rs:1365-1385` yalnız `KINDS[1]` ("title fills") baytını sayıyor | 2780 MB → 4096 MB sınırı 18:02:31,25'te doldu; render/yazma kopyaları (383 MB) bütçeye **dahil değil** | **+1,3-1,4 GB** (VRAM ve commit) |
| **(a2) Kart tamponları + staging + "before"** | `shadow.rs:2896-2923`: her kopya önce `shared_buffer` (CUSTOM, `MEMORY_POOL_L0`, WRITE_BACK = PC RAM, `host.rs:7203-7251`), sonra `card_for` → `uav_buffer` (DEFAULT, `host.rs:7255-7295`); "before" ikizi shader yazınca aynı boyutta (`shadow.rs:2569-2582`). Sınır 1024 MB yalnız kart tarafını sayıyor, ikizleri saymıyor (`shadow.rs:142-160, 220`) | 78 kopya / 656 MB, 18:02:31,9-32,72 arasında | Kart tarafı **656 MB** + PC RAM staging **656 MB** + ikizler **0-656 MB** = **1,3-2,0 GB** |
| **(b) Pipeline derleyicileri** | `make.rs:543-548`: `(mantıksal çekirdek/2).clamp(2,8)` işçi, env yok. `ACTIVATE_RAYS` ve `RT_RESOLVE` varsayılan açık (`make.rs:75-97`); koşu 9'da `<kaldırıldı>` (`ayarlar.txt`) | 143 çeviri + ~129 pipeline 3 sn'de; koşu 7 `LLVM ERROR: out of memory` ile bitti (derleyici malloc'u tavana çarptı) | **0,2-0,8 GB** (TAHMİN; WS'te görünür, sürücü derleyicisi tek tek ölçülmüyor) |
| **(c) Oyunun guest belleği** | `xmem.rs:397-458`: pagefile destekli `SEC_RESERVE` section; `map()` sırasında `VirtualAlloc(MEM_COMMIT)` (`xmem.rs:776, 794-800`) | Bu bir **paylaşılan section** commit'i. Windows'ta bu, süreç PrivateUsage'ına (private bytes) değil sistem commit'ine yazılır (TAHMİN, doğrulanmadı). Döküm tabanı (`priv − tex`) bu pencerede ~sabit | **~0** (private için); WS'e katkısı olabilir |
| **(d) Upload / shared / staging** | `host.rs:5965-6010` (UPLOAD); `made so far: upload buffers` | 441 → 546 MB (5 sn, kümülatif); "staging of textures 6 alive 7 MB"; "our own allocations" 790 → 804 MB | **~0,1 GB** |
| **(e) Pencereler (tampon dışı)** | `shadow.rs:2957-3021` | "windows lie over … 9 MB", "0-7 windows outside buffers" | **≈0** (bu evrede önemsiz) |

Kontrol:
- **VRAM:** ölçülen +2,1 GB (3663 → 5767 MiB). (a1) 1,35 + (a2) kart tarafı 0,66 + ikizlerin bir kısmı = 2,0-2,7 ✓
- **Working set:** ölçülen +1,6 GB (5,59 → 7,18). Staging 0,66 + derleyici + upload ✓
- **Toplam:** (a1) + (a2) + (b) + (d) ≈ 3,0-4,3 GB; ölçülen +4,27 ✓

**Sonuç:** Sıçramayı tek bir şey yaratmıyor. Dünya açılırken kısa bir pencerede üç kaynak üst üste biniyor: doku kuyruğu (6 GB bekliyordu), 1 MiB üstündeki tüm başlık tamponlarının kart kopyası ve PC tarafı ikizi, ve shader dalgası. Bunların VRAM'de duran yarısı da commit'e sayıldığı için, 13 GB tavanına fiziksel RAM dolmadan çarpılıyor.

## 2. VRAM kaynakları private bytes'a sayılıyor mu? Doğru ölçüt hangisi?

**Kodda kullanılan heap tipleri** (hepsi `CreateCommittedResource`, `D3D12_HEAP_FLAG_NONE`; `CreateHeap` ve tahliye bayrağı yok):
- **DEFAULT (kart):** host dokular `resource.rs:1896-1915`; depth ikizleri `resource.rs:472-551`; `uav_buffer` (kart tamponları ve "before" ikizleri) `host.rs:7255-7295`; indirect `indirect.rs:590-613, 1429-1479`; standin `standin.rs:435-444`.
- **CUSTOM + `MEMORY_POOL_L0` + WRITE_BACK (PC RAM, eşlenmiş):** `shared_buffer` `host.rs:7203-7251`; `descriptor/host.rs:2553-2576`; `indirect.rs:547-573`.
- **UPLOAD / READBACK (PC RAM):** `host.rs:5928-5992`, `inplace.rs:532-547`, `resource.rs:2906-3241`.
- **Placed:** yalnız `inplace.rs:259`, `OpenExistingHeapFrom*` ile (`VICTORY_INPLACE`, varsayılan kapalı).

**Sayıldığına dair kanıt (üçü birbirinden bağımsız):**
1. §1.2'deki sabit taban: dört koşuda priv − host doku MB'ı ≈ sabit.
2. Koşu 9'un son 4 sn'sinde priv +4,27 GB, ama WS yalnız +1,6 GB. Farkın (~2,7 GB) VRAM artışıyla (+2,1 GB) örtüşmesi, commit'in kart tarafını da içerdiğini gösteriyor.
3. **Küçük DEFAULT doku ayırmalarının OOM ile reddi:**
   - Koşu 8, 17:59:42,53: `no host texture … 0x8007000E` (128×128×6, format 24; ardından 30×17). O saniyede (`olcum.csv` 20:59:42) VRAM 6611/8192 MiB, bütçe 7249 MB, private **13,000 GB**.
   - Koşu 6, 17:50:50,77: 1280×720 doku.
   - Koşu 7, 17:57:02,04: 2560×1440 doku.

   Kartta yer varken bunun tek açıklaması süreç commit sınırı (Job `ProcessMemory`, `guvenli_baslat.ps1:75, 228`). Not: WDDM, tahliye edilebilir yerel bellek ayırmaları için yedekleme commit'i ayırır. Bu, Windows'un bilinen davranışıyla uyumlu; ama sürücü içinden ayrıca doğrulanmadı.

**Sonuç: Job `PROCESS_MEMORY` (commit) tavanı bu iş için yanlış metrik.** 8 GB kartta commit ≈ (PC RAM'i) + (VRAM'de duran kaynaklar). 13 GB tavan, 7 GB WS + 6 GB VRAM'de dolup oyunu öldürüyor, oysa fiziksel RAM daha dolmamış oluyor. Sistemin commit'i bol: çökme anında 25,6 GB boştu, başlangıçta 43,6 GB.

**Fiziksel RAM'i korumak için doğru ölçütler:**
- **Sistem boş (Available) RAM:** `PhysicalAvailable`, başlatıcıda zaten var (`guvenli_baslat.ps1:348`, `MinFreeRamGB=1.5`, `Sustain=5`). Koşu 8'de 0,018, koşu 9'da 0,338 GB'a indi ama 5 sn sürmediği için tetiklenmedi.
- **Süreç working set'i:** ikincil ölçüt.
- **Commit tavanı:** yalnız kaçak bir sızıntıya karşı emniyet olarak, yüksek tutulmalı (≈ RAM + VRAM bütçesi − pay ≈ 20 GB).

**Fiziksel RAM tutarlılığı (WS 7,18 GB'ta boş RAM 0,34 GB):**
- Koşu başında, victory henüz 0 GB'tayken boş RAM **9,28 GB**'tı (`olcum.csv` 21:01:36). Yani diğer programlar ve çekirdek ~6,4 GB tutuyordu.
- Beklenen boş RAM: 15,6 − 6,4 − 7,18 ≈ 2,0 GB. Ölçülen 0,34. Arada ~1,7 GB açık var.
- Açığın adayları (TAHMİN):
  - Sürücünün PC'de kartla paylaştığı bellek: "476-546 MB of the PC's memory shared with it" (`run.log:1063, 1139`).
  - Pagefile'a yazılmayı bekleyen değiştirilmiş sayfalar (Available'a sayılmaz).
  - dxgkrnl/NVIDIA havuz ayırmaları.
  - Guest section sayfaları.
- Sonuç: büyük resim tutarlı. Fiziksel tarafta asıl kaldıraç diğer süreçlerin 6,4 GB'ı ve PC RAM'indeki staging/derleyici zirvesi. VRAM'deki dokular fiziksel RAM tüketmiyor (VRAM 5,8/8 GB, bütçenin altında; boşaltma yok).

## 3. 16 GB'ta geçirmek için ölçülebilir adımlar (en olası kazançtan başlayarak)

Her adım tek değişkenli olmalı; koşu 9 ayarları taban alınmalı.

### A. Yalnız ayar (kod değişmeden)

| # | Ne | Nerede / varsayılan | Beklenen | Başarı ölçütü |
|---|---|---|---|---|
| A0 | **Başlatıcı:** `-MaxCommitGB 20`; RAM koruması `-MinFreeRamGB 0.5 -SustainSeconds 3` | `guvenli_baslat.ps1:16, 20, 36` (13 / 1,5 / 5) | Yanlış metrik kalkar; 13 GB'ta oluşan yapay OOM gider | `run.log`'da **0** adet `no shared buffer`, `no host texture`, `LLVM ERROR`, `memory allocation of`. `ozet.txt`'de private tepesi 13-16 GB, WS tepesi ≤ 8 GB, en düşük boş RAM > 0,3 GB. Çökme olursa sebep "boş RAM" diye yazılmalı. **Bu tek adım tahmini de doğrular:** WS ≤ 8 GB iken private > 13 GB ise VRAM commit'e sayılıyor demektir. |
| A1 | **Diğer programları kapat** (tarayıcı, T3/ajanlar, Discord vb.) | — | Koşu başında boş RAM 9,3 → ≥ 11 GB; fiziksel pay +1,5-2 GB | `olcum.csv` ilk satırında `BosRAM_GB ≥ 11`; tüm koşu boyunca en düşük boş RAM ≥ 1 GB |
| A2 | `VICTORY_TEXTURE_BUDGET_MB=2048` | `resource.rs:1371` (unset → kart bütçesinin %90'ı, en az 2 GiB) | 4096'ya göre ~−2 GB VRAM **ve** −2 GB commit. Fiziksel RAM kazancı küçük (staging). Bedeli: dokular geç gelir | `memory of textures … host copies the title fills: N alive ≤ ~2100 MB`; `memory: X MB private` tepesi ≤ ~11.000. "they wait" uyarısı erken gelir (beklenen). |
| A3 | `VICTORY_CARD_BUFFERS_MIN_MB=16` (eski varsayılan) | `shadow.rs:158` (vars. **1**) | Koşu 9'da 656 MB'ın yalnız ~195 MB'ı ≥16 MiB. Kart tarafı −460 MB (VRAM ve commit), ikizler orantılı. PC staging aynı kalır (`shared_buffer` her durumda gerekli). Bedeli: küçük tamponların shader yazıları kartta hash'lenmez; yorum "first room's walls" sorununu anıyor (`shadow.rs:154-157`) | `a buffer's host copy is in the card's memory` satırları 79 → ~7; `len` toplamı ≤ ~400 MB (açılıştaki 174 MB dahil) |
| A3' | Daha sert: `VICTORY_CARD_BUFFERS=0` | `shadow.rs:139` | Kart tarafı ve ikizler tamamen gider (−0,66-1,3 GB commit, −0,66 GB VRAM). Bedeli: #413'teki FPS kazancı gider | Hiç "card's memory" satırı yok; private tepesi A3'ten de düşük |
| A4 | `VICTORY_ACTIVATE_RAYS=0` + `VICTORY_RT_RESOLVE=0` | `make.rs:77, 94` (vars. açık; koşu 9'da `<kaldırıldı>`) | Derleyici zirvesi −0,3-1,3 GB (TAHMİN; ActivateRays tek başına 1,3 GB, `make.rs:68-73`). BVH kapalıyken görsel kayıp yok | `LLVM ERROR: out of memory` yok; son 10 sn'deki `shader translated` sayısı < 143 |
| A5 | `VICTORY_DISPLAY_MODE=1600x900` | `dxgi.rs:230` (3840x2160; şu an 1920x1080) | Render/yazma kopyaları 383 MB → ~270 MB; küçük kazanç | `host copies the host renders to or writes: … MB` ~%30 düşer |
| — | `WINDOW_KB=512`, `WINDOW_TAILS=1` | `shadow.rs:2973, 3012` | Bu evrede etkisiz (pencereler 9 MB) | Değişken sayısını azaltmak için çıkarılabilir |
| — | `CARD_BUFFERS_MB=1024` | `shadow.rs:158` | **Zaten varsayılan, ayar etkisiz** | — |
| — | `TEXTURE_CATCHUP=0` | `draw.rs:1780` | Koşu 8'den beri set; staging zirvesini kısar. Tutulmalı | — |

Önerilen ilk deneme: **A0 + A1**, ardından A2, ardından A3 (ya da A3'). A4 en ucuz güvenlik payı; A0 ile aynı koşuya eklenebilir.

### B. Küçük kod yamaları (alt ajana verilebilir; risk düşükten yükseğe)

| # | Ne | Nerede | Risk | Başarı ölçütü |
|---|---|---|---|---|
| B1 | **Doku bütçesine commit/RAM kapısı:** `room()` ayrıca `GetProcessMemoryInfo.PrivateUsage > VICTORY_COMMIT_SOFT_MB` ya da sistem Available < `VICTORY_MIN_FREE_RAM_MB` olduğunda `false` dönsün. Bütçe yalnız title-fill'i değil, tüm host doku baytını (`KINDS[0]+KINDS[1]`) saysın | `resource.rs:1365-1385` | Düşük: dokular bekler, siyah/eksik malzeme görülebilir | Kapı açıkken yeni bir satır ("commit kapısı kapalı") görünür; private tepesi ≤ soft cap + ~1 GB |
| B2 | **Kart kopyasına aynı kapı** ve "before" ikizlerinin `most` sınırına sayılması | `shadow.rs:197-229` (`card_for`), `shadow.rs:2569-2582` | Düşük | Kapı kapalıyken "card's memory" satırı yok; `CARD_BYTES[0]+[1]` ≤ sınır |
| B3 | **Pipeline işçi sayısı için env** (`VICTORY_PIPELINE_WORKERS`, ör. 2-4) ve boş RAM < 1,5 GB iken yeni derleme başlatmayan kapı (five-star-recomp fikri) | `make.rs:547` | Düşük-orta: shader dalgası uzar, ilk saniyelerde eksik çizim | Son 10 sn'deki `shader translated` dağılımı düzleşir; LLVM OOM yok |
| B4 | **Başlatıcı:** Job'ta commit tavanı = RAM + VRAM bütçesi − 2 GB (bu makinede ~21 GB); asıl durdurma Available RAM < 0,5 GB × 3 sn ve WS > 9,5 GB olsun | `guvenli_baslat.ps1:16-36, 228, 589-598` | Düşük | Yapay OOM kalmaz; gerçek RAM baskısında temiz durdurma ve gerekçe satırı |
| B5 | Hazır dış adaylar (güvenlik incelemesi `ram_kaynaklari.md`'de): `vf-465`'te `VICTORY_LEVELS_ON_DEMAND=1` (mip'leri talep üzerine kurar → doku baytı düşer); soldsawn PR #432 (koşu 9 18:02:19'da 236 MB "not mapped any more"); PR #610 `TEXTURE_DROP_MIPS` (kayıplı) | — | Orta (port) | `host textures … MB` aynı sahnede düşer |

`D3D12_HEAP_FLAG` ile VRAM yedekleme commit'ini kapatmanın standart bir yolu yok. Placed heap'e geçmek de commit'i azaltmaz. Kazanç ya daha az bayt ayırmaktan ya da doğru tavan metriğinden gelecek.

## 4. Log'da gözden kaçan uyarılar

1. **`host pipeline refused 0x80070057 shaders=[compute 02384a60fda6f2d0]`**
   - Dört koşunun hepsinde, OOM'dan 1-2 sn önce görülüyor: 6 → 17:50:49,51; 7 → 17:57:01,08; 8 → 17:59:41,74; 9 → 18:02:31,91.
   - Ardından `a dispatch without a host pipeline dropped` geliyor.
   - Deterministik bir geçersiz parametre hatası, bellekle ilgisi yok. Dünya geçişindeki bir compute pass'i düşüyor. Ayrı bir hata kaydı açmaya değer (Belobug PR #45 / soldsawn PR #922 gibi PSO doğrulama düzeltmeleri aday).
2. **`host view heap capped at 1000000; views past it read a null slot count=1245184`** her koşunun 2.-3. saniyesinde görülüyor (ör. koşu 9 erken). ~245 bin görünüm null okuyor; eksik doku ya da çizim olarak görünebilir.
3. **29× `a pipeline creation method that is not implemented: the title goes without this pipeline`** her koşuda var.
4. **`buffer views left null: raw, refused by the host … width=182452224`** ve **`unordered buffer view the host would refuse left null format=39 … count=0`**: koşu 9'da 18:02:32,07, bir RAW UAV'nin uzunluğu 0. Not: 174 MB, açılıştaki tek kart tamponunun boyutu.
5. **OOM zinciri, ilk kırılma:**
   - Koşu 6: `no upload buffer … size=2802192` (17:50:50,76) ve `no host texture` (8 kez).
   - Koşu 7 ve 8: `no host texture` (1 ve 4 kez).
   - Koşu 9'da ilk kırılma `no shared buffer` (18:02:33,82).

   Hepsi aynı commit tavanı. Çıkış kodu 0xC0000409, sonradan gelen Rust/LLVM abort'u.
6. **Çökme sonrası sistem commit çöküşü:** victory 13,0 GB'ta donmuşken sistemin boş commit'i birden düşüyor:
   - Koşu 8, 20:59:45: 25,5 → 9,6 GB.
   - Koşu 9, 21:02:38: 25,6 → 12,7 GB.

   Başka bir süreç 13-16 GB commit alıyor. TAHMİN: 0xC0000409 için WER/crash dump yazımı. 16 GB makinede bu ikinci bir tehlike; tam döküm kapatılabilir ya da sınırlanabilir.
7. Küçük notlar:
   - `placed textures … 74 asked for again with other filler words` (18:02:29): her biri ayrı kopya; `VICTORY_PLACED_FILLER` ile ilgili.
   - 18:02:19'da `host copies of placed textures whose title memory is not mapped any more (estimate): 236 MB`: PR #432'nin hedefi.
   - `the pipeline library's mapped file let go … 146 MB`: normal.

## Kanıtlanmamış / açık kalanlar

- Guest section commit'inin (`xmem.rs`) Job `ProcessMemory` sınırına sayılıp sayılmadığı doğrulanmadı. Dökümdeki sabit taban bu pencerede büyümediği için sıçramadaki payı ~0 kabul edildi.
- Derleyici payı (0,2-0,8 GB) dolaylı bir tahmin. Ölçmek için A4'lü ve A4'süz iki koşu karşılaştırılmalı.
- "before" ikizlerinden kaçının bu pencerede oluştuğu log'da yok. Her 5 sn'lik döküme `CARD_BYTES[1]` (ikiz hash baytı) değil, ikizlerin gerçek bayt toplamı eklenmeli.
- Son döküm 18:02:29'da kaldı, sıçramanın kendisi dökümler arasında oldu. Döküm aralığını 1 sn'ye indirmek ya da OOM anında son bir döküm almak, bir sonraki analizi kesinleştirir.
