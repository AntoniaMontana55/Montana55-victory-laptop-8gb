# A1/A2 teşhisi: vw-avatar, Series S, 16 GB / 8 GB (26.11)

Salt okunur inceleme. Kaynaklar: `kosular\20261126-014709_avatar01-guvenli16` (A1), `…015539_avatar02-windowsbar` (A2), karşılaştırma için `…20261125-232803_kosu27-cbv-direct` (K27, eski taban vw-pr10). Kod: `C:\CLLL\.calisma\vw-avatar` @0a43566. Satırlar run.log'daki zamana göre verildi (A2 başlangıcı 22:55:41Z, K27 başlangıcı 20:28:06Z).

## 1. Neden 4,5 dk siyah ekran?

İki ayrı sorun var. Siyah ekranın asıl nedeni yükleme değil, Present yolu.

### 1a. Kanıtlı: Present yalnız ilk düzlemi gösteriyor (UI düzlemi atılıyor)
- Draw sayıları aynı, görüntüler farklı. K27'de 61. sn'de yükleme ekranı resmi görünüyor (`K27\kareler\kare_0061s.png`, 4-10 draw/kare). A2'de aynı aşamada aynı 8-10 draw/kare var ama kare siyah (`A2\kareler\kare_0062s.png`, `kare_0186s.png`). A2'nin 9 karesinin hepsi siyah.
- Eski taban iki düzlemi birleştiriyor. K27 run.log:135: `present planes changed planes=2 planes, stride 72: [1920x1080 f28 …] [3840x2160 f28 …]`. Oyun düzlemi 1080p, UI/yükleme düzlemi 4K.
- Yeni taban yalnız ilk düzlemin ilk kaynağını okuyor: `vw-avatar\crates\victory-gpu\src\queue.rs:928-950` (behind yolu) ve `queue.rs:959-972` (doğrudan yol). `Plane` yapısında stride/yerleşim/alfa yok (`queue.rs:647-653`). A2 log'unda "present planes" satırı hiç yok. Bunu geçiş raporu da söylüyor: `repo_durum\gecis_avatarmanz.md:9`, "Yok, gerekli, yüksek".
- Sonuç: yükleme ekranı ve büyük olasılıkla tüm menü/altyazı ikinci düzlemde ve hiç ekrana gelmiyor. Bu port yapılmadan hiçbir koşuda görüntü değerlendirilemez.
- Ek olarak FSR4 stand-in da yok (`gecis_avatarmanz.md:10`). Dünya çizilse bile Series S'in FSR yükseltme çıktısı boş kalabilir. **TAHMİN:** A2'de dünya kareleri ölçülecek kadar uzun sürmedi.

### 1b. Kanıtlı: dünya geç geldi, pipeline önbelleği soğuk
- Pipeline kütüphanesi yeni çevirmenle eşleşmiyor. K27 `pipeline library … loaded=2220` ile başlıyor ve hep "(0 being made)". A1 aynı 260 901 152 baytlık dosyayı açtı ama yalnız `loaded=92` (A1 22:47:38). A2'de `loaded=447 → 771`, `made` 20 sn'de yaklaşık 20 artıyor (A2 22:56:08 → 22:59:53).
- A2 fps satırlarında sürekli 1200-1760 pipeline "being made" var (22:55:56 → 22:59:51, en son 873).
- Dünyanın ilk karesi K27'de yaklaşık 71. sn'de (20:29:17: 8245 draw), A2'de yaklaşık 160. sn'de (22:58:21-33: 35 → 583 → 193 draw). Bu karelerde 857 / 2792 / 1207 draw pipeline olmadığı için atıldı, pipeline beklemesi 732 ms ve 501 ms.
- Doku kuyruğunda o anda 20 006 başlık dokusunun host kopyası yok (9,4 GB, A2 22:58:22). K27 aynı noktada 10 018 (6,5 GB) gösteriyordu.
- Patlamadan sonra A2 yine 2-3 draw/kareye düşüyor (22:58:39 → 22:59:51). Doku kuyruğunda "6167-9686 waiting for their memory" var. **TAHMİN:** başlık akış/yükleme durumuna döndü ya da RAM yetmediği için akış durdu. Ortalama sayfa hatası 88 178/sn (`A2\kaynak_ozet.txt`), bu thrash anlamına geliyor. Enter tuşu sahneyi atlatmış da olabilir. Elde kanıt yok.
- Yükleme ilerliyor, takılı değil: dstorage dosya açılışları K27 ile aynı. Doku "card has no room" ya da WAIT_RESIDENT beklemesi baskın değil: card wait medyanı 0 ms (`kosu_ozeti.md`).
- "host view heap capped at 1000000" uyarısı K27'de de var (K27 run.log:192). Neden bu değil.

## 2. Fiziksel RAM'i kim yiyor?

### Başlangıçtaki boş RAM farkı (kanıtlı, `olcum.csv` 2. satır)
| koşu | başta boş RAM | sonuç |
|---|---|---|
| K27 | 8,77 GB | 10 dk sürdü, sonda 2,1 GB boş |
| A2 | 7,38 GB | 272. sn'de durdu |
| A1 | **4,23 GB** | 212. sn'de durdu |

A1 geçersiz bir karşılaştırma: oyun dışı yaklaşık 11,8 GB doluydu. "Oyun dışı yaklaşık 5 GB" varsayımı bu koşularda tutmuyor (A2'de yaklaşık 8,6 GB).

### Süreç içinde (A2'nin son copystat'ları ve memory map'i, yaklaşık 23:00)
- Private 17,8 GB, bunun yalnız 3,7 GB'ı bellekte. Commit sınırı sorun değil, hâlâ 17,7 GB commit edilebiliyor.
- Karta paylaşılan, kilitli bellek: "PC's shared with the card in use 3924 MB". A1'de 4484, K27'de 4014. Bu bellek working set'te görünmüyor ama fiziksel RAM'i tutuyor.
  - `shared buffers 2150 MB` (A1: 2942). Pencereler bunun büyük kısmı: `host.rs:9133-9175`, CUSTOM + L0 + `resident()`.
  - `upload buffers 1512 MB` (doku staging havuzu 512 MB, `pool.rs:352`).
  - Komut listelerinin scratch belleği 72 MB.
- Pencereler: 1520 adet, 2130 MB, 1535 MB başlık belleğinin üstünde. Yani %39'u aynı belleğin ikinci kopyası ("memory, windows" satırı).
- Kartın kendi belleği 6058/7249 MB (A1: 4379). Döküm:
  - Doku havuzu 2944 MB + staging 512 MB ("memory in blocks")
  - `card buffers 1931 MB`: A2'de WINDOWS_BAR ile karta giden pencereler. A1'de bu 550 MB.
  - Render hedefleri yaklaşık 1 GB ("host copies the host renders to… 1062 MB")
- WINDOWS_BAR işe yaradı ama yarım kaldı. Kilitli shared buffer 2942 → 2150 MB'a indi (yaklaşık −0,8 GB), kart da 1,4 GB doldu. Kart yaklaşık %83 doluydu. Bu `shadow.rs:503-506`'daki "used+size > budget·0,8" kapısının üstünde olduğu için kalan pencereler shared_buffer'a düştü (A2'de 1518 pencere hâlâ RAM'de).
- TEXTURE_BUDGET_MB çalışıyor ama kartı korumuyor. Limit verilince `resource.rs:1680-1688` yalnız doku baytını sayıyor; kart doluluğu (`room_percent`, `card_room_kept`, `targets_room_kept`) hiç bakılmıyor. Host dokuları 2732 MB'ta durdu (render hedefleri dahil), ama kartı asıl dolduran doku + pencere + hedef toplamı.
- Diğer kalemler (memory map, A2 11664): "other private ≥1 MB 4806 MB (1018 MB bellekte)", "4 of 1024 MB: 2462 MB". **TAHMİN:** pipeline derleme işçileri (13 işçi, A2 "pipeline workers workers=13") ve sürücü derleyicisi de RAM ve CPU'ya ortak. Kanıt yok, tek değişkenle ölçülmeli.

### En etkili 3 ayar
1. **Koşu öncesi ≥ 9 GB boş RAM** ve **pipeline önbelleğini önceden ısıtma.**
   - RAM: tarayıcı/Discord/T3 vb. kapatılmalı. Harness başta boş RAM < 8,5 GB ise başlamamalı.
   - Önbellek: `victory warm --jobs 6` (`apps\victory\src\main.rs:108-129`, `warm.rs`) bilinen 2338 erken pipeline'ı oyundan ayrı süreçlerde yapar, böylece koşu K27 gibi "0 being made" ile başlar. Ağır iş sayılır, tek başına çalıştırılmalı.
2. **`VICTORY_TEXTURE_BUDGET_MB=1536` + `VICTORY_WINDOWS_BAR_MB=2048`:** kartta yaklaşık 0,5 GB yer açar, böylece pencereler %80 kapısının altında karta sığar ve kilitli RAM'den çıkar. Beklenen etki yaklaşık −0,5…−1 GB kilitli RAM. **TAHMİN**, ölçülmedi.
3. **`VICTORY_PIPELINE_WORKERS=6`:** önbellek ısıtıldıysa derleme işçilerinin RAM/CPU payını azaltır. **TAHMİN.** Isıtma yapılmadıysa dünya daha da geç gelir.

WINDOW_KB: 1024 KB ile 2130/1535 MB (×1,39), K27'de 256 KB ile 2060/1402 MB (×1,47). Fark küçük, 1024'te kalınmalı.

### En küçük kod değişiklikleri (öncelik sırasıyla)
1. **Zorunlu: iki düzlemli Present'i port et.**
   - Kaynak: `vw-pr10\…\queue.rs:672`, `host.rs:7010`.
   - Hedef: `vw-avatar\…\queue.rs:647-653` (Plane'e 72 bayt stride, src/dest alanları) ve `queue.rs:927-972` (behind ve doğrudan yolda iki kaynağı ölçekleyip alfa ile birleştir).
   - Bu olmadan ekran siyah kalır.
2. **Ara teşhis (birkaç satır):** `queue.rs:959-965`'te `VICTORY_PRESENT_PLANE=1` gibi bir env ile `plane_parameters` 72 bayt ileri alınıp ikinci düzlem gösterilir. Yükleme resmi görünürse 1a kesinleşir.
3. **Pencere kapısı:** `shadow.rs:503-506`'daki %80 sınırı WINDOWS_BAR için ayrı bir env ile (ör. %90) ayarlanabilir hale getirilmeli.
4. **Sayfalanabilir pencereler:** daha büyük bir iş. `host.rs:9133-9175`'teki shared_buffer resident kalıyor. Boşta kalan pencerelerde `ID3D12Device::Evict`, kullanımdan önce `MakeResident` gerekir (bırakma `let_go_idle` yakınında). Kilitli RAM'i en çok bu düşürür ama ölçülmeden önerilmez.

## 3. Sıradaki tek koşu (A3)

Önkoşul: (a) Present portu ya da en az `PRESENT_PLANE` teşhisi derlenmiş olmalı, yoksa görüntü yine değerlendirilemez. (b) Ayrı adımda `victory warm --jobs 6` bitmiş olmalı. (c) Başta boş RAM ≥ 9 GB.

`--console series-s`, 1920x1080, Enter 0,4 sn aralıkla 20-300 sn. Env (hepsi `VICTORY_` önekli):
```
TEXTURE_NO_WAIT=1 RELAY=2 WIDE_BACK=1 CARD_REHASH_RUNS=1 JOIN_COPIES=1 INDIRECT_WINDOW=1
RELAY_COPY=1 CARD_STEPS=2 INDIRECT_BATCH=2 MAP_THREADS=1 TITLE_PRIORITIES=0 SHOW_AFTER=1
AMBIENT=0.8 D3D12_DRED=light UNDER_WIDE=0 LAND_ECHO=0 CARD_AWAKE=0 MEMORY_MAP=1
WINDOW_KB=1024
TEXTURE_BUDGET_MB=1536        (2048'den)
WINDOWS_BAR_MB=2048           (1536'dan)
PIPELINE_WORKERS=6            (yeni; warm yapıldıysa)
PIPELINE_BOOK=1               (yeni; çizilen pipeline'ları sonraki warm için yazar)
```
`SKIP_PIPELINES=none` çıkarılmalı: geçersiz değer uyarısı veriyor (A2 22:55:45, "expects 16-digit shader hashes").

Ölçülecekler:
- 61. sn'de yükleme resmi görünüyor mu?
- "being made" 0'a yakın mı?
- "PC's shared with the card" < 3,5 GB mi?
- `card buffers` MB artışı
- Boş RAM tabanı

## Kanıtlanmamış (TAHMİN) özet
- A2'deki patlama sonrası 2-3 draw: akış RAM yüzünden durmuş olabilir ya da Enter sahneyi atlatmış olabilir.
- Pipeline işçilerinin RAM payı.
- FSR4 stand-in eksikliğinin dünya görüntüsüne etkisi.
- Bütçe 1536 + bar 2048 ile kazanılacak RAM miktarı.
