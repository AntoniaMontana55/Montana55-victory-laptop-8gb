# AvatarManz tabanı (VIctory-7900XTX @0a435665): 8 GB VRAM ve 16 GB RAM planı

- **Tarih:** 26.11.2026. Salt okunur analiz. Uzak kod `gh api` ile okundu, hiçbir şey derlenmedi ve çalıştırılmadı.
- **Hedef makine:** RTX 3070 Ti Laptop (8 GB VRAM), 16 GB RAM, Windows, 38 GB sayfa dosyası.
- **Kanıt yolları:** Repo köküne göredir. `gpu/` = `crates/victory-gpu/src/`, `hle/` = `crates/victory-hle/src/`.
- **İşaretler:** Kodla ya da AvatarManz'ın ölçümüyle doğrulanmamış her şey **TAHMİN** olarak işaretli. Bu tabanda bizim ölçümümüz yok.

---

## 0. Kısa sonuç

1. **40-46 GB'ın kaynağı:** AvatarManz'ın kendi dökümü şöyle: liste ayırıcıları 9 GB, bellek pencereleri 7 GB, upload belleği 4 GB (`docs/findings.md:95`). Bunlara başlığın kendi heap'leri (~4.6 GB) ve process heap (~2.6 GB) ekleniyor (`gpu/copystat.rs:561-566`). Kalan ~15-20 GB büyük olasılıkla 24 GB kartta tutulan VRAM nesneleri. Bunlar WDDM'de private commit'e sayılıyor; bunu biz ölçtük (`ram_sicrama_analizi.md` §1). **TAHMİN:** pay oranı.
2. **VRAM'e bağlı payın çoğu 8 GB kartta kendiliğinden küçülüyor.** Kart kopyası sınırları, doku kuyruğu, staging ve doku tutma kuralları kartın bütçesine göre ölçekleniyor. **9+7+4 GB'lık CPU tarafı ise ölçeklenmiyor.** 16 GB'ı zorlayan bu kısım.
3. **#1117 (INPLACE_SECTIONS) bu tabanda yok.** Benzer bir `VICTORY_INPLACE` var ama başlığın sayfa havuzunu reddediyor: Windows'ta section `SEC_RESERVE` açılıyor (`hle/modules/xmem.rs:660`). Ayrıca açtığı sayfaları RAM'e kilitliyor. Kolay bir taşıma değil.
4. **NVIDIA'da en büyük doğruluk riski:** 65.535'i aşan dolaylı dispatch'ler yalnız AMD'de tam koşuyor (`gpu/indirect.rs:272-282`). 1440p'de ekranın alt şeridi bloklu kalır. **1080p'de sınırın altında kalıyor (TAHMİN, hesap §3'te).** Bu yüzden 1080p çalıştır.
5. **Dürüst beklenti:** Series X profili ve tam hızlı ayarlarla 16 GB'a **sığmaz**. Series S, `-Safe` ve pencere/doku sınırlarıyla sınırda kalır: yoğun sahnelerde 2-5 GB swap beklenir (**TAHMİN**). Kalıcı çözüm için §4.3'teki 3-4 küçük kod değişikliği gerekiyor.

---

## 1. 40-46 GB ne için kullanılıyor?

### 1.1 AvatarManz'ın kendi ölçümleri

| Kaynak | Ne diyor |
|---|---|
| `README.md:26` | "40-46 GB of private memory: **64 GB of RAM recommended**" |
| `docs/findings.md:95` | "Memory, 40-46 GB private: lists' allocators 9 GB, windows 7, uploads 4 (`VICTORY_MEMORY_MAP=1` lists it)" |
| `gpu/copystat.rs:561-566` (eski koşu 30, 18 GB private) | Başlık heap'leri 4.6 GB, process heap 2.6 GB (162 segment), 707 × 4 MiB pencere 2.7 GB, pipeline kütüphanesi 0.46 GB. Private sayılmayan eşlemeler: başlığın sayfa havuzu 6.2 GB, sürücünün shader önbellek dosyası 2.0 GB. |
| `gpu/copystat.rs:528-533` | Bellek haritası satırı diag build'de varsayılan açık (`VICTORY_MEMORY_MAP=0` kapatır). **Bizim ölçüm aracımız bu satır olacak.** |
| `docs/findings.md:71` | Series S ve küçük dokular yalnız **hız** açısından "değmez" bulunmuş. RAM için denenmemiş. |

### 1.2 Kalem kalem (kod kanıtı, hangi env ile büyüdüğü)

| # | Kalem | Kod | Büyüklük / büyüten | 8 GB kartta kendiliğinden küçülür mü? | Küçülten env |
|---|---|---|---|---|---|
| A | **Host liste havuzu ve komut ayırıcıları (9 GB)** | `gpu/list.rs:218-243` (`IDLE`, `IDLE_MOST=(512, 256 MiB)` :235), `gpu/host.rs:3954-4010` (liste başına 1 DIRECT allocator, ayrıca prelude) | Liste başına ~0.68 MB private (koşu 69, `host.rs:751-753`). Ayırıcı belleği en yüksek kayıt boyutunda kalıyor; havuz 512 boşta listeyi tutuyor. Büyütenler: `VICTORY_THIN=1` (her başlık listesine ek "back" liste, `gpu/list/thin.rs:1-28`), `VICTORY_JOIN_PARTS=8`, `VICTORY_SUBMIT_DEPTH` (64, `gpu/queue/behind.rs:612-616`), `VICTORY_BURY_LAG` (64 gönderim geç bırakma, `host.rs:479-488`) | **Hayır.** Sabit sayılar. Ayırıcı boyu sürücüye bağlı; NVIDIA'da farklı olabilir (**TAHMİN**). | `VICTORY_LIST_POOL=0` (her liste atılır; bedeli kare başına ~21 ms, `list.rs:222-223`), `VICTORY_THIN` kapalı (`-Safe`), `VICTORY_SUBMIT_DEPTH=16..32` (**TAHMİN** etkisi) |
| B | **Başlık belleği pencereleri (7 GB)** | `gpu/shadow.rs:5025-5060` (`WINDOW=4 MiB`, `VICTORY_WINDOW_KB`), PC belleğinde write-back custom heap: `gpu/host.rs:9133-9163` (`shared_buffer`) | 4 MiB ızgarada, ne kadarı okunursa okunsun pencerenin tamamı tutuluyor. Koşu 60: 803 pencere 3.3 GB tutuyordu, istenen kısım yalnız 1.6 GB'tı (`shadow.rs:5037-5045`). | **Hayır.** | `VICTORY_WINDOW_KB=1024` ya da `256` (64..4096, 2'nin kuvveti); `VICTORY_WINDOWS_OWN` kapalı kalmalı (varsayılan kapalı; açılırsa +3.3 GB, `shadow.rs:519-528`) |
| C | **Upload belleği (4 GB)** | Scratch havuzu: `host.rs:1830-1850` (`SPARE_KEPT = 1 GiB`, `keep_freed` ile koşulsuz); doku staging blokları: `gpu/pool.rs:30-37` (64 MiB blok, `STAGING_KEPT=8` → 512 MB); boşta liste upload'ı 256 MB (`list.rs:235`); upload arenaları: `gpu/upload.rs:1-60` (`VICTORY_UPLOAD_LOW`) | `VICTORY_KEEP_FREED` (varsayılan açık, `host.rs:350-364`) 1 GiB scratch tutuyor. PC tamponları 5 dk tutma kuralı yalnız ≥12 GiB boş RAM'de devreye giriyor (`host.rs:374-378`), 16 GB'ta girmez. | Kısmen: `ram_idle_ms` boş RAM'e göre 2-10 sn'ye iniyor (`host.rs:379-392`). Scratch 1 GiB **inmiyor**. | `VICTORY_KEEP_FREED=0` (scratch tavanı 128 MB, ≤ ~0.9 GB kazanç). **Risk:** AvatarManz'ın iki cihaz kaybı serbest bırakılmış belleğe okumadan çıkmış (`host.rs:350-360`). |
| D | **Kart kopyaları ve "before" ikizleri** | `gpu/shadow.rs:130-245` | 24 GB kartta: min 1 MiB, tavan bütçe/4 (~5.8 GB), yazılanlar ×2 ikiz (`shadow.rs:179-195`). **≤12 GiB bütçede kendiliğinden 16 MiB / 512 MiB.** | **Evet** (≈5.8 GB → 0.5-1 GB) | Dokunma. `VICTORY_CARD_BUFFERS_MIN_KB=64` 8 GB'ta **açılmamalı** (fxceless #1163 gerilemesi). |
| E | **Dokuların kart kopyaları** | `gpu/resource.rs:1660-1666` (taban = bütçe/4, en çok 2 GiB), `:1676-1705` (bütçenin %90'ı, `VICTORY_TEXTURE_ROOM_PERCENT`), `:1725-1740` (hedeflere 3 GB yer ayırma, `VICTORY_CARD_ROOM_FOR_TARGETS_MB`) | Kartın bütçesine bağlı | **Evet** (bütçe ~7.2 GB) | `VICTORY_TEXTURE_BUDGET_MB=2048` (eski tabanda −3 GB RAM ölçtük) |
| F | **Doku tutma (10 dk)** | `gpu/host.rs:136-160` (`vram_keep`: kart <%88 iken 10 dk), `:165-200` | VRAM'de tutulan dokular commit'e sayılıyor | **Evet** (%88 eşiği bütçeye göre) | `VICTORY_TEXTURE_KEEP=0` (eski 5 sn kuralı; daha çok yeniden yükleme) |
| G | **Boş doku blokları** | `gpu/pool.rs:30-50` (`SPARE_KEPT=64` → kart <%85 iken 4 GiB), `VICTORY_RIPE_BLOCKS=12` (`pool.rs:84-87`, 12 × 64 MiB = 768 MB önden) | VRAM; commit'e sayılıyor | Kısmen (%85 eşiği) | `VICTORY_RIPE_BLOCKS=4` (**TAHMİN** −0.5 GB commit; blok kaynaklı cihaz kaybı koruması azalır) |
| H | **Adresle okunan bellek (BVH vb.)** | `gpu/shadow.rs:5183-5215` (`VICTORY_MEMORY_CARD_MB`, 4096) | Kart tarafı ve PC'de CPU tarafı (`shared_buffer`) | Kart tarafı %97 kuralıyla; **CPU tarafı hayır** | `-RayTracing` kapalı kalsın |
| I | **Snapshot havuzu** | `gpu/shadow.rs:1277-1310` (`VICTORY_SNAPSHOT_POOL=1024` adet, toplamı `KEPT=512 MiB` ile sınırlı) | ≤512 MB | Hayır | Gerek yok |
| J | **Başlığın kendi belleği** | `gpu/console.rs:24-25` (Series X 10 + 3.5 GiB; Series S 7.6 GiB), `hle/modules/xmem.rs:653-660` | 4.6 GB private heap ve 6.2 GB eşlenmiş havuz (koşu 30) | Hayır | `victory run <exe> --console series-s --execute` (`apps/victory/src/main.rs:142-144`). `play.ps1` bu bayrağı geçirmiyor. |
| K | **Shader / pipeline önbelleği** | Sürücü dosyası 2.0 GB, kütüphane 0.46 GB (`copystat.rs:563-566`) | Eşlenmiş; çoğu private değil | — | — |

### 1.3 24 GB VRAM'e göre ayarlanmış yerler (8 GB'ta davranış)

- **`default_card_limits_mb`** (`shadow.rs:179-195`): Üç bütçe kademesi var. 8 GB kart en alt kademeye düşer. Kodda doğru ele alınmış.
- **`staging_budget_per_frame`** (`gpu/draw.rs:4400-4409`): 24 GB'ta kare başına 384 MB, 8 GB'ta 192 MB.
- **`targets_room_kept` 3072 MB** (`resource.rs:1725-1740`): 7.2 GB bütçenin ~%42'si. Doku kuyruğu erken durur, dokular geç gelir. Bu bilinçli bir koruma; ilk koşuda dokunma.
- **`vram_keep` / `pool::room`:** Yüzdeye bağlı, ölçekleniyor.
- **`MEMORY_CARD_MB` 4096 ve `INPLACE_MB` 4096:** Sabit, 8 GB'a göre büyük. Ray tracing ve INPLACE kapalıyken etkisiz.
- **`KEEP_FREED`:** 64 GB RAM düşünülerek yazılmış (12 GiB boş RAM şartı). 16 GB'ta PC tarafı devreye girmez; yalnız 1 GiB scratch kalır.

---

## 2. 8 GB VRAM ve 16 GB RAM için ayar planı

| Env | Değer | Tahmini RAM etkisi | Tahmini fps etkisi | Not |
|---|---|---|---|---|
| `-Safe` profili (`VICTORY_THIN`, `JOIN_PARTS`, `UPLOAD_AHEAD` kapalı) | `scripts/play.ps1:23-27` | **TAHMİN −2..−5 GB** (A kalemi: back listeler ve parçalar) | **Ciddi kayıp.** THIN 75→68 ms getirmişti (`findings.md:41`). | İlk koşu güvenli olsun diye |
| `VICTORY_WINDOW_KB` | `1024` | **TAHMİN −2..−3.5 GB** (B'nin ~yarısı; koşu 60 oranı 3.3→1.6) | Küçük CPU maliyeti, daha çok pencere. TAHMİN ±%3 | Kod "varsayılan 4 MiB, ince ızgara ölçülmedi" diyor (`shadow.rs:5043-5045`) |
| `VICTORY_TEXTURE_BUDGET_MB` | `2048` | −2..−3 GB commit (eski tabanda ölçüldü) | Dokular geç gelir | Eski taban ölçümü |
| `--console series-s` | CLI | **TAHMİN −3..−6 GB** (başlık 13.5→7.6 GiB, dokular küçük) | AvatarManz'a göre hızda fark yok (`findings.md:71`) | Bu tabanda Series S sunumu doğrulanmadı. Eski tabanda `PRESENT_PLANES` gerekmişti. **İlk denemede bakılacak.** |
| `VICTORY_CARD_AWAKE` | `0` | 0 | Masaüstünde +; laptopta ısı ve throttle riski | `play.ps1` 1 yapıyor (`awake.rs:1-13`) |
| `VICTORY_LIST_POOL` | `0` (deney) | **TAHMİN 0..−6 GB** (NVIDIA ayırıcı boyuna bağlı) | Kare başına ~+21 ms (koşu 61) | Önce ölç, sonra karar ver |
| `VICTORY_SUBMIT_DEPTH` | `24` (deney) | TAHMİN −0.5..−2 GB | TAHMİN −%0..5 | Uçuştaki liste sayısını kısar |
| `VICTORY_KEEP_FREED` | varsayılan (1) | `0` ile ≤ −0.9 GB | — | Cihaz kaybı koruması; ilk koşuda **açık kalsın** |
| `VICTORY_LEVELS_ON_DEMAND` | `1` (deney) | VRAM −2..−3 GB (koşu 189: 2.2-3.0 / 4.3-5.5 GB), dolayısıyla commit de düşer | `UpdateTileMappings` maliyeti; dokular daha erken gelebilir | Ampere tiled tier 3 > gereken 2 (`gpu/tiles.rs:75-104`). `HELD_LEVELS` ile ikisinden biri seçilir (`resource.rs:1240-1245`). |
| `VICTORY_WINDOWS_BAR_MB` | `512` (fps deneyi) | PC RAM'i ≈ verilen MB kadar azalır (CPU tarafı yok, `shadow.rs:476-490`) | Kart listeleri 29→26 ms (AvatarManz) | fxceless `WINDOW_UPLOAD` karşılığı. ReBAR / GPU upload heap şart: laptopta BIOS'a bağlı (**TAHMİN**). |
| `VICTORY_CARD_WINDOWS_MB` | — | RAM kazancı yok (CPU tarafı kalıyor, `shadow.rs:430-447`) | + | RAM öncelikli planda gereksiz |
| `VICTORY_CARD_BUFFERS_MIN_KB` | **açma** | — | 8 GB'ta gerileme (#1163) | — |
| `VICTORY_INPLACE` | `1` ile `INPLACE_MB=1024` (geç deney) | Pencere kopyası azalır **ama** açılan sayfalar RAM'e kilitlenir (`gpu/inplace.rs:13-17, 91-98`) | ? | Deneysel; NVIDIA'da ölçülmedi. Önce `VICTORY_INPLACE_PROBE=1`. Sayfa havuzu kapsam dışı. |

**Doku azaltıcılar:** Bu tabanda `TEXTURE_DROP_MIPS` **yok** (env listesinde bulunmadı). Var olanlar:

- `HELD_LEVELS`: Varsayılan açık. Görünen ilk seviyeden tutuyor; 6.3 GB'ın 1.2-1.6 GB'ı (`resource.rs:1232-1245`).
- `LEVELS_ON_DEMAND`: Kapalı.
- `LAZY_TEXTURES`: Varsayılan açık (`resource.rs:1482-1488`).
- `TEXTURE_BUDGET_MB`.

**#1117 durumu:** Yok. Kolay da değil: Windows'ta section `SEC_RESERVE` ile açılıyor (`xmem.rs:660`, yalnız Wine'da `SEC_COMMIT`). `OpenExistingHeapFromFileMapping` yalnız `SEC_COMMIT` kabul ediyor (`inplace.rs:22-27`). Taşıma için üç şey gerekir:

1. xmem'de Windows için `SEC_COMMIT`. Bütün havuz baştan commit edilir: Series X'te +10-13 GB commit. 54 GB commit sınırı bunu kaldırır ama Job sınırını aşar.
2. Pencereler için section-heap yerleşimi.
3. fxceless'in kilitsiz `HostHeap::mirror` değişikliği.

Tahmini iş: orta-zor, 1-2 gün ve ayrı bir güvenlik incelemesi (**TAHMİN**).

---

## 3. NVIDIA'ya özgü riskler

| Risk | Kod | NVIDIA'da ne olur | Öneri |
|---|---|---|---|
| **65.535 grup sınırı** | `gpu/indirect.rs:272-282` (`widest`: yalnız AMD'de 2^20), `:121` | Başlığın 8×4 karo listeleri 1440p'de 67.440 grup istiyor; NVIDIA'da kesilir ve alt 1.905 karo kaybolur ("blocky patches", `findings.md:61`). | **1080p çalıştır.** 1920×1080/32 = 64.800 karonun aynı oranı (~%58.5) ≈ 37.900 grup, sınırın altında (**TAHMİN**: oran sahneye göre değişebilir). Çözünürlüğü yükseltmek için dispatch'i bölen bir kod değişikliği gerekir. |
| **Wave genişliği** | `gpu/host.rs:915-927` (NVIDIA en çok 32 → wave64 programları 32'lik yarılarla), `crates/victory-shader/src/emit/mod.rs:509-514` (`WAVE_WIDE` bariyerli form varsayılan kapalı), `:600-606` | AvatarManz bu yolu hiç koşturmadı. Yanlış piksel ya da TDR riski var. | Varsayılanı koru. TDR olursa `VICTORY_GROUP_SYNC=1` dene (`emit/mod.rs:160-175`: NVIDIA bariyerde tüm thread'leri bekler). |
| **RDNA kayan nokta** | `emit/mod.rs:225-246` | "Nothing on other hosts": kendiliğinden kapanıyor (`host.rs:926`) | Bir şey yapma |
| **Helper lane** | `emit/mod.rs:152-158` (`HELPER_OWN` varsayılan açık; AMD helper lane düzeltmesi) | NVIDIA'da helper lane'ler wave okumalarına katılıyor; sonuç farklı çıkabilir (**TAHMİN**) | Gölgede delik ya da kesik kenar görülürse `VICTORY_HELPER_OWN=0`. Shader önbelleği bu ayara göre yeniden üretilir. |
| **Görünümü aşan ham okuma** | `host.rs:350-360` (AMD'de sınırsız, zararsız) | NVIDIA'da sayfa hatası ve cihaz kaybı olasılığı daha yüksek (**TAHMİN**) | `KEEP_FREED` ve `BURY_LAG` açık kalsın |
| **Görünüm sınırı 1.000.000** | `gpu/descriptor/host.rs:1362, 1410`, `gpu/draw.rs:3163` | Kodda ele alınmış | — |
| **CARD_AWAKE** | `gpu/host/awake.rs` | Laptopta sürekli tam saat: ısı, throttle, pil | `0` |
| **GPU upload heap / ReBAR** | `shadow.rs:476-490` | Laptop BIOS'una ve sürücüye bağlı; yoksa açılmaz | Log'da `GPUUploadHeapSupported` veya "windows ... bus" satırına bak |
| **Sürücü shader önbelleği** | `copystat.rs:566` (2 GB) | NVIDIA Denetim Masası'ndaki "Shader Cache Size" sınırı varsayılan olarak düşük olabilir | NVCP'de 10 GB veya "Sınırsız". İlk koşudan önce `victory warm --used` ve `victory shaders`. |
| **Job bellek sınırı (bizim başlatıcı)** | `.calisma/repo_durum/guvenli_baslat.ps1:228-230` (`ProcessMemory`) | VRAM commit'e sayıldığı için 13 GB sınırı bu tabanda kesin `E_OUTOFMEMORY` (eski koşu 6-8 böyle bitti) | Sınırı **28-32 GB**'a çıkar ya da kaldırıp WS'yi izle. Commit sınırı 16+38 = 54 GB. |

---

## 4. Sıralı plan

### 4.1 İlk koşu: güvenli 16 GB / 8 GB profili

`play.ps1` yerine doğrudan çalıştır (`--console` bayrağı için). Ortam:

```
# play.ps1 ortak (play.ps1:16-20), CARD_AWAKE hariç
VICTORY_TEXTURE_NO_WAIT=1  VICTORY_RELAY=2  VICTORY_WIDE_BACK=1  VICTORY_CARD_REHASH_RUNS=1
VICTORY_JOIN_COPIES=1  VICTORY_INDIRECT_WINDOW=1  VICTORY_RELAY_COPY=1  VICTORY_CARD_STEPS=2
VICTORY_INDIRECT_BATCH=2  VICTORY_MAP_THREADS=1  VICTORY_TITLE_PRIORITIES=0
VICTORY_SHOW_AFTER=1  VICTORY_AMBIENT=0.8  VICTORY_D3D12_DRED=light  VICTORY_SKIP_PIPELINES=none
# -Safe (play.ps1:22-23); THIN / JOIN_PARTS / UPLOAD_AHEAD ayarlanmaz
VICTORY_UNDER_WIDE=0  VICTORY_LAND_ECHO=0
# 16 GB / 8 GB / laptop
VICTORY_CARD_AWAKE=0
VICTORY_TEXTURE_BUDGET_MB=2048
VICTORY_WINDOW_KB=1024
# ölçüm (diag build'de zaten açık; açıkça yaz)
VICTORY_MEMORY_MAP=1
# DOKUNMA: CARD_BUFFERS_*, KEEP_FREED, RIPE_BLOCKS, THIN, INPLACE, LEVELS_ON_DEMAND
```

Komut: `target\diag\victory.exe run <exe> --console series-s --execute`. Oyun içinde 1080p seç.

**Neden Series S:** RAM'deki en büyük tek kaldıraç bu. Series S görüntüsü bozuksa ikinci koşu Series X ile yapılır, diğer her şey aynı kalır.

**Ölçütler:**

- 10 dk boyunca çökme yok.
- `memory: X MB private` ve bellek haritası satırı: pencereler, process heap, 4 MiB ayırmalar, write-combined.
- WS tepesi ≤ 11 GB (OS'e ~4 GB kalsın).
- Sistemde "Hard faults/sn" düşük kalmalı.
- fps not edilir ama karar ölçütü değil.

### 4.2 Tek değişkenli deneyler (önce RAM, sonra fps)

Her deney aynı sahnede yapılır (prolog iç mekan ve tekne), 10 dakika sürer, ilk koşuya göre tek fark içerir.

| # | Değişken | Beklenen | Ölçüt / karar |
|---|---|---|---|
| D1 | `VICTORY_WINDOW_KB=256` (1024'e karşı) | Pencere baytı bir kez daha −%20-40 (**TAHMİN**) | Haritadaki "windows of title memory" MB düşer, kare süresi %5'ten fazla uzamaz → kalır |
| D2 | `VICTORY_LIST_POOL=0` | NVIDIA ayırıcıları büyükse private −2..−6 GB; kare +10-20 ms (**TAHMİN**) | Önce ilk koşunun haritasında "other private allocations" ve host list satırı >3 GB mı, ona bak. Değilse deneyi atla. Kazanç ≥2 GB ve kare kaybı ≤%15 → kalır. Yoksa D2b: `VICTORY_SUBMIT_DEPTH=24`. |
| D3 | `VICTORY_LEVELS_ON_DEMAND=1` | VRAM −2..−3 GB ve commit de düşer; doku bekleme ("they wait") azalır | VRAM tepesi ve "host copies ... level by level" satırı; siyah doku yok → kalır |
| D4 | `VICTORY_THIN=1` (fps deneyinin ilki; RAM'i yükseltir) | Kare −%5-10 (AvatarManz 75→68 ms); RAM +1..+4 GB (**TAHMİN**) | WS tepesi hâlâ ≤ 11.5 GB ise kalır; sonra `VICTORY_JOIN_PARTS=8` ayrıca denenir |
| D5 | `VICTORY_WINDOWS_BAR_MB=512` (ReBAR varsa) | PC RAM −0.3..−0.5 GB, kart listeleri −%10 | Log'da pencerelerin BAR'a gittiği görülmeli; görülmüyorsa destek yok, deney biter |
| D6 | `VICTORY_KEEP_FREED=0` (yalnız RAM hâlâ taşıyorsa) | −0.5..−0.9 GB | **30 dk** çökmesiz koşu şart; tek bir DEVICE_REMOVED → geri al |

Sıra: D1 → D2 → D3 → D4 → D5 → D6. Bir deney RAM'i ilk koşudan kötüleştirirse geri alınır ve sonraki deneye geçilir.

### 4.3 16 GB'a sığmazsa: açık ve kod değişikliği

**Dürüst tahmin (TAHMİN, ölçülmedi):**

- **Series X ve tam hızlı profil:** private 22-30 GB, WS 14-18 GB. 16 GB fiziksel RAM'de sürekli swap, takılmalar, kare 2-3 kat uzar.
- **İlk koşu profili (Series S, Safe, sınırlar):** private 14-20 GB, WS 9-12 GB. Sınırda: yükleme sıçramasında 2-5 GB sayfa dosyasına taşar, oynanış sırasında büyük olasılıkla sığar.
- **D4 (THIN) ile:** WS +1-4 GB; büyük olasılıkla yine sınırın üstü.

**Gereken küçük kod değişiklikleri** (Sol'a verilebilir; her biri tek dosya):

1. `gpu/list.rs:235`: `IDLE_MOST` sabit 512 → env (`VICTORY_LIST_IDLE`, örneğin 64). Ayrıca belirli bir ayırıcı boyunu aşan listeleri havuza geri koymadan atmak gerekiyor; ayırıcı belleği reset ile küçülmüyor. Beklenen etki: A kaleminden GB'lar, `LIST_POOL=0`'ın 21 ms bedeli olmadan (**TAHMİN**).
2. `gpu/host.rs:1837`: Scratch `SPARE_KEPT` 1 GiB'ı boş RAM'e bağla (`ram_idle_for` gibi; <8 GiB boşsa 128 MB). `KEEP_FREED` korumasını kaybetmeden ~0.9 GB kazanılır.
3. `gpu/shadow.rs:5047`: `WINDOW_KB` varsayılanını sistem RAM'i 32 GB'ın altındaysa 1024 yap. Kolay iş.
4. `.calisma/repo_durum/guvenli_baslat.ps1:228-230`: Job `ProcessMemory` sınırını yükselt ya da kaldır; WS ve hard fault izlemesi ekle. Kod değil, bizim betik.
5. Büyük iş: #1117 benzeri section-heap pencereleri (§2 sonu). Ancak 1-4'ten sonra hâlâ taşıyorsa.
6. NVIDIA ve 1440p için: `indirect.rs` 65.535 üstü dispatch'i X ekseninde bölmek. Kolay değil; dolaylı dispatch `ExecuteIndirect` ile yapılıyor.

---

## 5. Kanıtlanmamış ve açık kalanlar

- "Lists' allocators 9 GB" NVIDIA sürücüsünde aynı büyüklükte mi? **Bilinmiyor.** İlk koşunun bellek haritası söyleyecek.
- VRAM'in private'a sayılması eski tabanda NVIDIA'da ölçüldü. Kalan 15-20 GB'ın bu yüzden şiştiği AMD için bir **TAHMİN**.
- Bu tabanda Series S görüntü/sunum doğruluğu test edilmedi.
- `WINDOW_KB` ince ızgarasının kazancını AvatarManz da ölçmemiş (`shadow.rs:5043-5045`).
- `INPLACE` NVIDIA sürücüsünde açılıyor mu? `VICTORY_INPLACE_PROBE=1` söyler.
- 1080p'de en büyük karo dispatch'inin 65.535 altında kalması oran varsayımına dayanıyor. Log'da `asked` / boş kayıt sayısına bakılmalı (`host.rs:1715-1730` çevresindeki kare satırı).
