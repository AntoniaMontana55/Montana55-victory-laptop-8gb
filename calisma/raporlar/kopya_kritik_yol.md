# Kopya kritik yolu — "preparing: copies" analizi (25.11, salt okunur)

Kod: `C:\CLLL\.calisma\vw-pr10\crates\victory-gpu\src` (aşağıda yollar buna göre). Log: `kosular\20261125-224546_kosu23-kayma-kontrol\run.log`, son pencere (satır 19950-20071, 20 kare, ~5,3 sn). Hiçbir şey çalıştırılmadı; yalnız okuma.

## 0. En önemli bulgu (kısa)

- Kare (≈272 ms) gönderim iş parçacığında (log'da `thread 14388`, karede 30 gönderim) sırayla şunlardan oluşuyor: **hazırlık ~183 ms** (bunun **125 ms'i kopya**) + **röle geri kopyasını bekleme ~40-60 ms** + başlığın iki gönderim arası işi ~34 ms.
- Kopyanın 125 ms'i kabaca: **ReadProcessMemory ile taşıma 61 ms** (~310 MB) + **write-watch/guard sorguları 43 ms** (karede 6.483 sorgu; bunun 27 ms'i guard) + ~20 ms defter tutma (journal, Batch, kirli liste).
- Gönderim iş parçacığı **70 ms user + 113 ms kernel** çalışıyor: hazırlık çekirdek çağrılarıyla sınırlı (RPM, GetWriteWatch, VirtualProtect). Bellek bant genişliği tek başına sınır değil.
- **Düzeltme:** "geri kopya kritik yolda değil" sonucu yalnız kısmen doğru. Karede ~1 kez "upload belleği doldurma (Fill)", uçuştaki gönderimlerin yazdığı aralığı okuyor. Bu yüzden gönderim iş parçacığı `relay_wait` ile rölenin o uçuşları indirmesini **39-62 ms** bekliyor. Bu süre frame satırında "GPU" sütununa yazılıyor. Kanıt: aşağıda §2.

## 1. Ölçüm nerede, içine ne giriyor

**Satır:** `host.rs:6512` (`"{fps} fps … preparing: copies {copy_ms} ms {copy_mb} MB …"`). Değerler `host.rs:6371-6374` içinde `PREPARING[0..1]` sayaçlarından (`host.rs:451`) kare başına okunuyor.

**`PREPARING[0]` (ms) ve `PREPARING[1]` (MB) sayaçlarına yazılan yerler:**
| Yer | Ne |
|---|---|
| `host.rs:5729-5730` | `run_deferred` (`host.rs:5652`) içinde `Step::Copy` (başlığın buffer→buffer kopyası) ve `Step::Fill` (upload belleği doldurma). Log'da karede 11.506 adet ama yalnız **6 MB**. |
| `host.rs:5809-5810` | `Batch::update`: gölge kopya **refresh**'i, yani `Shadow::update`. **MB'nin neredeyse hepsi buradan geliyor.** |
| `host.rs:5840` | `Batch::flush` içindeki `watch::ask_ahead` süresi (write-watch toplu sorgusu). |
| `host.rs:5847-5855` | `WIDE_REFRESH=1` açıkken toplanan süre duvar saatine düzeltiliyor. |

`Step::GpuWrite` için alınan snapshot burada sayılmıyor (`SnapshotUs` copystat'ta ayrı). `WIDE_BACK` açıkken `take_snapshots` da (`shadow.rs:1070`) bu sayaca girmiyor (log: snapshots 2,7 ms). Doku refresh'i ayrı sütunda ("textures").

**Kare başına 320 MB neden kopyalanıyor:** Tüm başlık tamponları kopyalanmıyor; **yalnız kirli 64 KiB sayfalar** kopyalanıyor.
- `Shadow::update` (`shadow.rs:2366`) → `update_free` (`shadow.rs:2406`) → `watch::writes` → `copy_stale` (`shadow.rs:2492`). Bu yol, son kopyadan sonra yazılmış sayfaları ardışık parçalar halinde `copy` → `copy_mapped` → `read` (= `ReadProcessMemory`, `host.rs:5994`) ile kopyalıyor. Birim `PAGE = 64 KiB` (`watch.rs:32`).
- Log: "copies per frame … 26578 refreshes 25554.6 MB … 6483 watches 43.46 ms, **7701.1 MB checked, 7391.6 MB unchanged**". İstenen aralık 25,5 GB, Batch birleştirdikten sonra bakılan 7,7 GB, **gerçekten kopyalanan ~310 MB** (= 7701 − 7391). "0.0 MB copied whole (memory not watched)", yani her şey kirli takipte.
- 310 MB'ın 64 KiB sayfa parçalı olması, başlığın karede bu kadar sayfaya dokunduğu anlamına geliyor. Sayfa içinde 1 bayt yazılsa bile 64 KiB kopyalanıyor (`watch.rs:28-31` yorumu: daha ince birim denetimi pahalılaştırmış).

**Kirli takip: VAR ve AÇIK, iki mekanizma:**
1. **Özel bellek (`MEM_PRIVATE`, 0x20000):** `GetWriteWatch` + `WRITE_WATCH_FLAG_RESET` kullanılıyor (`watch.rs:306` `writes`, `watch.rs:250` `ask_kernel`). Sorgular gönderim başına `ask_once` (`host.rs:4486`, `watch.rs:174`; `VICTORY_ASK_ONCE` varsayılan açık, `watch.rs:168`) ve flush başına `ask_ahead` (`watch.rs:77`) ile toplanıyor.
2. **View belleği (`MEM_MAPPED`, 0x40000; xmem section'ları):** write-watch burada çalışmıyor, yerine koruma tabanlı takip var (`watch.rs:405` `guard`). Kopyalanan 64 KiB blok read-only yapılıyor; ilk yazmayı vectored handler yakalıyor (`watch.rs:8-15`). Log: **karede 349 guard fault, guards 27 ms**.
- **En büyük kopya aralıklarının çoğu view belleği:** `0x3c020000+0xae00000` (174 MB yığın, 12,5×/kare), `0xac6160000`, `0xaf5b70000` ve `0xac9360000`, hepsi `memory 0x40000`. View aralıkları `ask_once`/`ask_ahead` toplu sorgusuna **giremiyor** (`ask_kernel` başarısız olunca `false` dönüyor, `watch.rs:270`). Her refresh'te `writes` → `guard()` ayrı ayrı çağrılıyor.

## 2. İş parçacığı, paralellik ve sınırlayıcı

- **Refresh kopyaları gönderim iş parçacığında tek iş parçacığıyla yapılıyor** (`host.rs:5856-5859` döngüsü). `WIDE_REFRESH=1` olunca kopya başına bir iş olarak wide havuzuna dağılıyor (`host.rs:5843-5855`). Paralellik **kopya (Shadow) başına**, bayt başına değil: 174 MB'lık tek yığının kirli sayfaları yine tek işte kalıyor. Denenen `WIDE_REFRESH=1` etkisiz çıktıysa olası sebebi bu, ayrıca JOURNAL/GUARDS mutex'leri (TAHMİN).
- **Wide havuzu:** `available_parallelism()/4`, 2 ile 6 arasına sıkıştırılmış (`shadow.rs:993`), artı çağıran iş parçacığı. Örneğin 16-20 mantıksal çekirdekte 4-5 işçi var. İş birimi 1 MiB (`WIDE_JOB`, `shadow.rs:1063` civarı).
- **`WIDE_BACK` (varsayılan AÇIK, `shadow.rs:970-973`):** shader'ın yazacağı aralıkların snapshot'ını hazırlık sonunda topluca alıyor (`take_snapshots`). GPU sonrası karşılaştırmayı (`find_all`) 1 MiB parçalara bölüp havuzda çalıştırıyor. **Refresh kopyalarına dokunmuyor**, yalnız snapshot ve karşılaştırma tarafını hızlandırıyor (log: snapshots 2,7 ms, artık küçük).
- **Sınırlayıcı (koddaki ve log'daki kanıtla):**
  - **Çekirdek çağrıları:** `thread 14388 … in them running 70.3 ms user + 113.3 ms kernel`. 113 ms kernel ≈ moves 61 ms (RPM) + watches/guards 43 ms (GetWriteWatch, VirtualProtect) + diğerleri.
  - **RPM hızı:** 310 MB / 61 ms ≈ **5 GB/s**. Kod yorumunda düz `memcpy` "bir iş parçacığında ~8 GB/s" (`shadow.rs:922-924`). RPM'in ek yükü ~%40 (TAHMİN). Tek çekirdek bant genişliği ikincil sınır.
  - **Sorgu sayısı:** 6.483 sorgu × ~6,7 µs = 43 ms. Bu sayı `ask_once`'a rağmen yüksek, çünkü (a) view aralıkları her flush'ta `guard()`'a gidiyor; (b) her `Step::Run` (karede 78) `ask_once_end` çağırıyor (`host.rs:5696`), sonraki `GpuWrite → flush_one` (karede 5.191 adım, `host.rs:5711`) kendi sorgusunu yapıyor. `flush_one` `ask_ahead` çağırmıyor.
  - **Sayfa hatası kritik değil:** copystat "243 page faults"/kare (`copystat.rs:1059`, süreç toplamı) ≈ 900/sn. **Brifteki 17-19k/sn bu log'la uyuşmuyor**; başka bir ölçüm ya da başka pencere olmalı. Doğrulanmadı.
  - **Kilit:** "41.2 ms waiting for the submission lock" (`host.rs:4458` `lock = started - called`) aslında kilit değil. `started` ölçümü `relay_wait`'ten (`host.rs:4428`) sonra alınıyor, yani bu sayı **rölenin geri kopyasını bekleme süresini** içeriyor. Log: "settled for … title memory read 1.0 (… **a fill of upload memory 1.0** …)", "waiting … title memory read 43.7 / 49.2 / 62.4 ms", "the submitting thread stood waiting for the card 39.6-62.4 ms".

## 3. Bu tabandaki kapalı ya da ayarlanabilir anahtarlar

**`COPY_PLAIN`, `JOIN_PARTS`, `QUEUE2` bu tabanda YOK.** Kodda geçmiyorlar; fxceless'te var (bkz. `kopya_adaylari.md:33`). Mevcut olanlar:

| Anahtar | Yer | Varsayılan | Ne yapar | Kopyaya etkisi / risk |
|---|---|---|---|---|
| `VICTORY_COPY_MODE=plain` | `copystat.rs:139-146`, `host.rs:5880-5888` | kapalı (yalnız diag) | RPM yerine ham `ptr::copy`. | **Teşhis:** moves 61 ms'in ne kadarı RPM ek yükü, bunu ölçer. Eşlenmemiş sayfada **süreç çöker**; yalnız kısa ölçüm koşusu. `none` görüntüyü bozar. |
| `VICTORY_JOIN_COPIES=1` | `host.rs:2875-2894` | kapalı | Ardışık başlık buffer kopyalarını birleştirir. | **Etkisiz:** log "26 copies, 0 continue the one before". |
| `VICTORY_INPLACE=1` | `inplace.rs:68-91` | 0 | Buffer dışındaki pencerelerden **özel bellekte** olanlar kopyalanmaz; başlık belleği host heap olarak açılır. | View belleğini (en büyük aralıklar) **kapsamaz**. `0x960710000` ve `0x9891fdf00` gibi özel aralıklar kazanır. Sayfaları kilitler (`INPLACE_MB` 4096); 16 GB'ta **2,3 GB boş** RAM var, bellek baskısı riski. Seviye 1 late-wait ile uyumlu; **seviye 2 late-wait ile birlikte kullanılmamalı** (`inplace.rs:76-81`). |
| `VICTORY_INPLACE_PROBE` | `inplace.rs:60-63` | 0 | INPLACE uygunluk sayımı. | Zararsız teşhis; INPLACE'in kapsayacağı MB'ı önceden gösterir. |
| `VICTORY_CARD_BUFFERS` / `_MB` / `_MIN_MB` | `shadow.rs:139-161` | açık, 1024, 1 | Büyük buffer kopyalarını kartta tutar. GPU'nun yazdığını kart hash'leriyle bulur. | **CPU refresh'ini azaltmaz.** CPU tarafı başlık belleğini yine kopyalar, kirli olan GPU'ya gönderilir (`card_changed`, `shadow.rs:2525`). Kazancı geri kopya tarafında. Log: 79 kopya 657 MB, karede 64 MB gönderiliyor. MB'ı 1024'ün üstüne çıkarmak kopyayı düşürmez. |
| `VICTORY_CARD_WINDOWS_MB` | `shadow.rs:234-256` | 0 | Pencereleri kartta tutar. | Bus okumasını azaltır, **CPU kopyasını azaltmaz** (yorum açıkça: "the CPU's side takes title memory as before"). Kopya için ilgisiz. |
| `VICTORY_KEEP_SNAPSHOTS` | `shadow.rs:1448` | kapalı | Snapshot tamponlarını tutar. | Snapshot zaten 2,7 ms; kopyaya etkisi yok. |
| `VICTORY_ASK_ONCE` | `watch.rs:163-168` | **açık** | Gönderim başına tek write-watch sorgusu. | Zaten açık; kapatmak kötüleşir (run 81). |
| `VICTORY_GUARD_ASK=1` | `watch.rs:412-418` | kapalı (iyi) | Her guard'da VirtualQuery. | Kötüleştirir. |
| `VICTORY_WIDE_REFRESH=1` | `host.rs:5747` | kapalı | Kopya başına paralel refresh. | Denendi, etkisiz (bkz. §2). |
| `VICTORY_WINDOW_KB` / `WINDOW_TAILS` | `shadow.rs:2963-3015` | 4096 / kapalı | Pencere ızgarası. | 256 seçili; 128 kötüleşti. Log: 2014 MB pencere 1362 MB başlık belleğini örtüyor, yani **~650 MB çift kopya potansiyeli** ("copied once for each"). |

## 4. Log'daki kırılım (son 5 sn penceresi, kare başına)

- Frame satırı: 272,7 ms; preparing 168-187 (copies **116-126 ms, 313-321 MB**); textures 4-12 ms; GPU 30-62; copying back 135-167 ms (röle iş parçacığında).
- copystat (`copystat.rs:1067`): 30 gönderim; 11.506 başlık kopyası/doldurması 6,0 MB; 26.578 refresh (istenen 25,5 GB); 78 run; **moves 61,47 ms**; 243 sayfa hatası; **6.483 watch 43,46 ms**; bakılan 7.701 MB, değişmemiş 7.392 MB; 349 guard fault.
- Kayıt (`copystat.rs:1097`): 5.191 shader-write adımı; snapshots 2,7 ms; 32 MB geri karşılaştırılmış; **guards 27,0 ms**; whole 0 MB.
- Gönderim iş parçacığı: gönderimler arası 33,7 ms; gönderim içinde **70,3 ms user + 113,3 ms kernel**; "lock" 41,2 ms (= relay_wait, §2).
- Röle: karede 28-34 uçuş, kart kopyalarının parçalarını getirme **98-139 ms**, toplam iniş 132-174 ms. Kart: "nothing of ours queued 72-88 ms, **42-73 ms of that while the submitting thread was working**", yani kart, hazırlık sırasında boşta.
- En büyük refresh aralıkları (istenen bayt, süre): `0x3c020000+0xae00000` view, 12,5×, 6,07 ms; `0xac6160000+0x3200000` view, 20×, 5,30 ms; `0x960710000+0x2d00000` private, 21×, 1,10 ms. **İlk 10 aralığın toplamı yalnız ~15 ms.** Geri kalan ~110 ms binlerce küçük refresh ve sorgunun **çağrı başı ek yüküne** dağılmış.

## 5. Önerilen deneyler (tek değişken, beklenen kazanca göre sıralı)

1. **Fill-settle teşhisi + düzeltme (küçük yama).** `blocked_by` (`host.rs:2335-2341` `Step::Fill`) eşleşince Fill'in `src..src+len` aralığını ve kesişen `line.written` aralığını (kaynağıyla) bir kez logla. Yanlış pozitif olabilir: `join_ranges` yazılan aralıkları dokunanlarla birleştiriyor (`host.rs:2276`); Fill küçük olup birleşmiş geniş aralığa düşüyor olabilir (TAHMİN). Gerçekten kesişiyorsa o Fill'in kaynağını belirle.
   - **Beklenen:** yanlış pozitifse karede 40-60 ms (kare ~272 → ~215-230 ms). Gerçekse kazanç yok ama kritik yol kanıtlanmış olur.
   - **Risk:** yalnız log, risk yok. Düzeltme adımı doğruluk riski taşır (okunan bayt bayatlar).
   - **Ölçüt:** "title memory read … a fill of upload memory" ≈ 0 ve "stood waiting for the card" < 10 ms; görüntüde fark yok.
2. **`VICTORY_COPY_MODE=plain` ölçüm koşusu (env, yalnız teşhis).**
   - **Beklenen:** moves 61 → ~35-40 ms çıkarsa (TAHMİN) darboğaz RPM ek yükü demektir. Sonraki adım fxceless'in fault yönetimli `plaincopy.rs` uyarlaması (`kopya_adaylari.md:39`). Değişmezse bant genişliği sınırı demektir ve sıradaki deney #4 olur.
   - **Risk:** eşlenmemiş sayfada çökme. Kısa (1-2 dk) koşu, oyun durumu önemsiz bir noktada.
   - **Ölçüt:** copystat `moves` ve frame satırı `copies` ms'i, aynı sahnede.
3. **`ask_once`'ı `Step::Run`'dan sonra yeniden kur (küçük yama).** `host.rs:5696`'da `ask_once_end` çağrısından sonra, listenin kalan `Refresh` adımlarının aralıklarıyla `watch::ask_once`'ı yeniden çağır. Ya da `Batch::flush_one` (`host.rs:5784` civarı) `ask_ahead`'e girebilsin.
   - **Beklenen:** watches 6.483 → birkaç yüz, write-watch tarafında ~10-15 ms (TAHMİN). Guard'ın 27 ms'ine dokunmaz.
   - **Risk:** düşük. Mantık zaten `ask_once` kurallarıyla aynı; Run başlık belleğini yazdıktan *sonra* soruluyor.
   - **Ölçüt:** copystat `watches` sayısı ve ms; `VICTORY_REFRESH_CHECK=1` ile kısa koşuda "stale after its refresh" uyarısı çıkmamalı.
4. **Büyük `copy_stale` parçalarını wide havuzuna böl (orta yama).** `Shadow::copy` (`shadow.rs:2510` civarı): 4 MiB'tan büyük parçayı 1 MiB'lık işlere bölüp `wide()` ile paralel RPM yap. `copy_jobs` deseni (`shadow.rs:1080`) hazır.
   - **Beklenen:** moves 61 ms'in büyük parçalara düşen kısmı 2-4 kat hızlanır, toplamda ~15-30 ms (TAHMİN). Kazanç kirli parçaların boyut dağılımına bağlı; önce bunu loglamak lazım.
   - **Risk:** `changed`/`card_changed` sırası korunmalı (kopya bitince bir kez çağrılmalı).
   - **Ölçüt:** `moves` ms düşüşü, `copies` ms düşüşü; görüntü aynı.
5. **`VICTORY_INPLACE_PROBE=1`, ardından `VICTORY_INPLACE=1` (env).** Önce probe ile kapsanacak MB'ı gör. Özel bellekteki pencerelerin payı büyükse INPLACE=1'i dene.
   - **Beklenen:** yalnız özel aralıkların kopyası kalkar, birkaç ms ile 20 ms arası (TAHMİN; büyük aralıklar view olduğu için sınırlı).
   - **Risk:** sayfa kilitleme ile RAM baskısı (16 GB, 2,3 GB boş), başlığın decommit hatası (err 87).
   - **Ölçüt:** copies MB/ms düşüşü, `ram_mb` ve takılma yok, 5+ dk kararlılık.

**İlk deney:** #1. Yalnız log yaması, risksiz. 40-60 ms'lik tek kalem ve şu an "geri kopya kritik değil" varsayımını doğrudan test ediyor. Paralelde sıfır kod gerektiren #2 de (COPY_MODE=plain, kısa koşu) yapılabilir.
