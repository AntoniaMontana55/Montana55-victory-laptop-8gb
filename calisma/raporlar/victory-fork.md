# fxceless/victory-fork Durum Raporu (2026-11-25)

## Şu an nerede
- `fxceless/victory-fork`, `Marvisak/VIctory` (`ffe310e`) üzerine 24 Kasım 2026'da açıldı; 1 günde 1.706 yıldız, 242 açık PR ve 84 açık issue topladı.
- Açılış menüsü (30 fps), açılış ara sahnesi, kayıt/yükleme ve ilk oynanış sahnesi (prolog) çalışıyor; ancak oyun 2.5–4 fps (hız anahtarlarıyla 240–350 ms/kare) ve ~24 GB RAM kullanıyor.
- Işın izlemeli (ray tracing) dolaylı ışık ve gölgeler kapalı olduğu için güneş almayan tüm alanlar siyah.

## Bitenler
- **Shader çeviricisi / DXIL (konsol grafik kodunu PC'ye çevirme):** Meshlet ayıklama, sis, saç, ateş ve 730 compute (hesaplama) shader hatası giderildi; komut sayısını %43 azaltan temizlik geçişleri (`1684989`), `wave64` sürücü çökmesi düzeltmesi (`e9765a7`) ve park edilmiş şerit maskesi (`#11`) eklendi.
- **Işın izleme / BVH (3B ışın çarpışma ağacı):** 52 yapı kurucu (builder) programdan 49'u ve 64 ışın izleme (trace) programından 59'u çevriliyor; donanım ışın/kutu-üçgen testi DXIL'de benzetildi (`6a2d9f9`), ancak henüz varsayılan olarak kapalı.
- **GPU / D3D12, bellek ve performans:** Descriptor taşması ve haritalı pipeline dosyası kaynaklı ekran kartı kilitlenmesi çözüldü; büyük tamponlara ekran kartı kopyası, bekleme azaltan `VICTORY_LATE_WAIT`/`VICTORY_RELAY` (GPU boşta beklemesi ~136→30 ms), erken pipeline ön-ısıtma (`#1`) ve test düzeltmesi (`#14`) eklendi.
- **Giriş, ses ve araçlar:** XInput oyun kolu + klavye eşlemesi, kayıt/yükleme, `VICTORY_GPU_TIMING` kare profilleme ve ses/XMA yakalama araçları çalışıyor. (Son 60 commit'in ilk 12'si bu fork'a, kalan 48'i taban alınan upstream'e ait.)

## Şu an üzerinde çalışılanlar (açık PR'lar)
Toplam **242 açık PR** dağılımı: Shader/DXIL **95**, GPU/D3D12 **48**, Bellek/VRAM **32**, Performans **20**, HLE/Sistem **15**, Araç/Test/Dok **15**, Işın izleme/BVH **7**, Platform (Linux/Win10) **5**, Giriş **5**.

**En kritik 9 açık PR:**
- **#156** (*Ray tracing: shadow-ray traces and the full structure builder translate on 32-lane GPUs*): 32 şeritli (NVIDIA) kartlarda 4 gölge-ışını shader'ı ve yapı kurucunun tamamını çevirir.
- **#294** (*Structure builder: the last three programs translate on 32-lane hosts*): Yapı kurucuyu 32 şeritli kartlarda kapalı tutan son 3 shader engelini kaldırır.
- **#346** (*GPU and shaders: stores through title memory by address mark their copy*): Işın izleme kurucusu açıkken kareyi 3 saniyeye çıkaran 4.3 GB'lık gereksiz bellek kopyasını keser (`#284`).
- **#220** (*GPU: cheaper indirect draws: no barrier per draw...*): Kare başına ~3.200 dolaylı çizimdeki bariyer darboğazını (~40/63 ms) kaldırır (`#107`).
- **#71** (*GPU: use D3D12 COPY queue for relay card-copy fetches...*): Ekran kartı kopyalarını ayrı kopyalama kuyruğuna (COPY queue) alıp karşılaştırmayı paralelleştirir.
- **#203** (*Shaders: sizes of unused descriptors, paired f16 interpolation halves...*): 46 eksik shader'ı daha çevirir; siyah blok/eksik nesne sorunlarını azaltır.
- **#265** (*GPU: VICTORY_LEAN_WINDOWS=1...*): Ekran kartı pencerelerinde CPU kopyasını kaldırarak ~2.1 GB RAM tasarrufu sağlar.
- **#206** (*GPU: include uncopied pages in Shadow stale/merge...*): `VICTORY_RELAY=2` açıkken kaybolan duvar ve karakter kaplamalarını (`#168`, `#89`) düzeltir.
- **#22** (*Windows 10 support — Agility SDK*): Windows 10 kullanıcılarının Shader Model 6.6 ile oyunu çalıştırmasını sağlar (Linux/Proton için de `#20` ve `#64`).

**Çakışan (aynı işi yapan) PR kümeleri:**
- BVH eksik çocuk sıralaması (`#173`): `#213`, `#230` (ve kapatılan `#217`).
- Eski kayıt göçü (`#242`): `#322`, `#327`, `#338`, `#339`, `#341`.
- FP16 dikdörtgen yuvarlama (`#245`): `#315`, `#316`, `#326`, `#335`, `#337`.
- Metin giriş limiti (`#248`): `#295`, `#323`, `#331`; `--jobs 0` (`#249`): `#297`, `#332`, `#334`; açılış titremesi (`#269`): `#159`, `#343`; kayıp duvarlar (`#168`): `#206`, `#207`, `#208`; Win10 SDK: `#22`, `#69`; paralel doku çözme: `#92`, `#271`.

## Siyah gölge ve FPS için ne yapılıyor
- **Siyah gölgeler (ışın izleme):** `#156` ve `#294` eksik builder/gölge-ışını shader'larını 32 şeritli kartlara açıyor; `#213`/`#230` BVH sıralama hatasını (`#173`), `#346` kurucu açıkken oluşan 3 saniyelik kare donmasını (`#284`), `#36`/`#205` ise f16 kutu sınırındaki çökmeyi (`#16`) gidermeyi hedefliyor. (Alternatif olarak `#17` doğrudan DXR/RayQuery önerse de fork yazılımsal emülasyon yolunda.)
- **FPS artışı:** `#220` dolaylı çizim bariyerlerini (`#107`), `#270`/`#95` doku yenileme kilit beklemesini (`#178`), `#71` ve `#253` ekran kartı kopya/hash yükünü, `#92`/`#271` çok çekirdekli doku çözmeyi, `#49`/`#38` CPU işçi ölçeklemeyi, `#86` ise deneysel FSR ölçeklemeyi hedefliyor.

## Bakımcı ve birleştirme durumu
- **242 açık PR** bekliyor; şimdiye kadar yalnız **3 PR birleştirildi** (`#1`, `#11`, `#14`) ve 12 PR kapatıldı.
- **Neden yavaş:** Repo 1 günde yüzlerce (çoğu birbiriyle çakışan) PR ile doldu; `fxceless` tek başına her PR'ı kendi RTX 4070'inde derleyip `dxil.dll` doğrulaması ve canlı oyun testiyle inceliyor (`#1`, `#9`, `#11` yorumları). Resmi `CONTRIBUTING.md` veya yazılı PR kabul kuralı yok (`#96` bunu önermiş).
- **Upstream (`Marvisak/VIctory`) ilişkisi:** `fxceless`'in upstream'e açtığı **0 PR** var; değişiklikler geri gönderilmiyor. Fork, upstream düzeltmelerini tek yönlü port ediyor (`#22`, `#51`, `#69`, `#218`; `#126` yeni farkları listeliyor).

## Sırada ne var (tahmin)
- **Açık issue engelleri → ilgili PR'lar:** (1) Işın izleme kurucusu ve 3 sn kare yükü (`#16`, `#284`, `#173` → `#156`, `#294`, `#346`, `#213`), (2) Dolaylı çizim ve kilit darboğazı (`#10`, `#107`, `#178` → `#220`, `#71`, `#270`), (3) Hız anahtarlarında kayıp duvar/titreme (`#168`, `#89`, `#269` → `#206`, `#159`/`#343`), (4) Kalan siyah bloklar ve birleşik geometri shader'ları (`#279`, `#310`, `#138` → `#81`, `#203`, `#309`), (5) 24 GB bellek ve AMD/Win10/Linux uyumu (`#120`, `#6` → `#265`, `#317`, `#22`, `#20`).
- **Kısa vadeli beklenti:** Önce `#156`/`#294` + `#346` + `#213` birleştiğinde ışın izlemeli ışıkların ilk kez canlı sahneyi aydınlatması (siyah gölgelerin kalkması); ardından `#220` + `#71` + `#270` + `#206` ile kare hızının 2.5–4 fps'den 8–15 fps bandına ve daha kararlı görüntüye taşınması bekleniyor.
