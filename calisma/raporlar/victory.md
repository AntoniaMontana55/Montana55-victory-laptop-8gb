# VIctory (Marvisak/VIctory) Durum Özeti — 2026-11-25

## Şu an nerede
- **Oynanabilir değil**, ama önemli bir eşik geçildi (README, 24 Kas): oyun açılıyor, menüler 30 fps, giriş videosu/ses/kayıt çalışıyor ve dünya ~50 sn'de yüklenip ilk görevin açılışı (sokaklar, binalar, araçlar, karakterler, jenerik) çiziliyor.
- **Kusurlar:** Kare başına 1–2 sn sürüyor, yüzeyler ışıksız (karanlık), uzaktaki bir geometri şeridi eksik, bazı sahnelerde karakterler bozuk ve GPU sürücüsü hâlâ kilitlenebiliyor (test donanımı: AMD RX 6700 XT).
- **Shader kapsamı:** 24 Kas akşamı `70564ae` itişiyle doğrulanan shader sayısı 24.785'e (+3.061) fırladı (#1247); mesh %100 (8.765/8.765), vertex/pixel %99,7, compute %97,8.
- **Bakımcı düzeni (Marvisak):** Dış PR'ları kabul ediyor — 23–24 Kas'ta 89 PR'ı toplu dalgalarla birleştirdi (83'ü 24 Kas sabahı) — ama büyük mimari sıçramaları kendisi yazıp toplu itiyor (24 Kas 21:36'da 29 çekirdek commit). Kuyruk 1.072 açık PR / 434 issue'ya şişince dış PR açımı geçici olarak kısıtlandı (#712).

## Son 3 günde bitenler
- **Shader çevirici (DXIL — Windows GPU kodu):** Şerit başına tanımlayıcı (per-lane descriptor) ve waterfall döngüleri (#307, #566), 64-bit atomikler, lane maskeleri, f16/NaN/-0 doğruluk düzeltmeleri (#37, #140, #384, #448, #612, #634, #637, #706) ve ön-çeviri önbelleği (`translate --cache`) tamamlandı.
- **GPU / D3D12 ve Işın İzleme (BVH):** Ham adresli konsol belleği için sayfa tablosu, BVH (ışın izleme hızlandırma yapısı) kesişimi, FidelityFX ölçekleyici yolu, pipeline (çizim hattı) ön-derlemesi ve tanımlayıcı havuzu (heap) düzeltmeleri (#21, #336, #365, #671, #680, #681) eklendi.
- **Bellek, Yükleyici (Loader) ve İşlemci:** Intel CPU'larda AMD'ye özel komut ve `clwb` öykünmesi (#10, #38, #45), Wine/Proton bellek ve yazılım GPU (WARP) uyumu (#41, #53), çökme raporu/TLS düzeltmeleri (#4, #28, #51, #198, #204, #313) bitti.
- **Dosya, Ses, Kayıt, Giriş ve Araçlar:** BCPack/DirectStorage hızlandırması (#61, #93), XMA ses sonsuz döngü/taşma koruması (#14, #50, #88), kayıt ve eşzamansız görev (XAsync) hataları (#209, #239, #429, #520), Alt/F10 donması (#419) ve dünya kare/döngü teşhis bayrakları (#46, #70) birleştirildi.

## Şu an üzerinde çalışılanlar (açık PR'lar)
- **Alan dağılımı (son 500 / toplam 1.072 açık PR):** Shader/DXIL 251, GPU/D3D12/Doku 146, Bellek/Loader/OS 16, Araç/Test/CI 15, Kayıt/Runtime 11, Giriş 9, Işık/Gölge 8, Dosya 7, Ses 6, Performans 4, RT/BVH 3, Ağ 2, Diğer 22. (Onaylı açık PR yok; topluluk #1633'te iş bölümü yapıyor.)
- **Öne çıkan 9 PR:**
  1. `#439` (88 yorum) — LDS (yerel bellek) paylaşan 64 şeritli konsol shader'larını 32 şeritli (NVIDIA) kartlarda çalıştırır.
  2. `#189` (52) / `#233` (46) — Wave64 compute (hesaplama) shader'larını 32 şeritli GPU'larda öykünür; NVIDIA'da dünyayı açmak için kritik.
  3. `#704` (49) — Bilinmeyen adımlı (stride) tamponları ham görünümle okur; eksik/bozuk geometri ve veri okumasını düzeltir.
  4. `#578` (43) — 64 şeritsiz kartlarda wave64 mesh/compute'u reddederek NVIDIA TDR (sürücü sıfırlama) donmasını önler (#778).
  5. `#160` (36) — Doku örneklemede ofset ve gather (toplu örnekleme) formlarını ekleyerek sahne/gölge doğruluğunu artırır.
  6. `#334` (21) — Shader yazma-okuma arasına D3D12 durum geçişi (bariyer) ekler; bozuk veya sıfır okunan tamponları önler.
  7. `#595` (17) — GPU gönderimlerini zaman çizelgesinde eşzamansız (async) çalıştırır; kare süresini kısaltmayı hedefler.
  8. `#1065` (11) — Compute türevleri için DXIL 1.8 doğrulayıcısını açar; `70564ae` üstüne tek başına **+735 compute shader** ekler (#1247).
  9. `#1267` (6) & `#1399` (5) — `#1267` bellek kopya süresini sentetik testte 2,5–4,3 sn'den 0,8–1,2 sn'ye indirir (#400); `#1399` eksik 195 birleşik geometri (ES+GS) shader'ını açar.
- **Çakışan PR kümeleri:** Wave64/NVIDIA (#439, #233, #189, #578, #1281); kopyalama/kare hızı (#1267, #1351, #1472, #1555, #322); GPU sorguları (#159, #1731, #1740); derinlik sınırı (#158, #582, #925); 64-bit atomikler (#305, #539, #705, #1127, #1467 — çoğu `70564ae` ile zaten `main`'e girdi, #1247).

## Önündeki engeller ve kim çözüyor
- **Işıksız (karanlık) yüzeyler ve boş sahne (#255, #841, #850):** Typeless doku eşleme (#876, #970, #1406), R9G9B9E5 hedef dönüşümü (#492), ham tampon okuması (#1762) ve yazma-okuma bariyerleri (#334) çalışılıyor; ışığı tek başına açan kesin bir PR henüz yok.
- **1–2 sn/kare performans (#400, #776):** Kare süresinin çoğu bellek kopyalama ve yazma takibi (write-watch) taramasına gidiyor; #1267, #1351, #1555, #322, #595 ve tek adres alanı fikri (#407) bunu hedefliyor.
- **NVIDIA GPU donması / Wave64 (#71, #778, #1001):** 32 şeritli kartlar wave64 shader'larda kilitleniyor; #439, #233, #189 ve #578 çözüyor.
- **Eksik uzak geometri ve 195 birleşik geometri shader'ı (#293):** #1399 (ES+GS), #1065 (+735 compute) ve dolaylı çizim (ExecuteIndirect) sınırı (#242) bu açığı kapatıyor.
- **Bozuk karakterler ve kalan eksikler:** Paketli 16-bit/FMA/barycentric shader düzeltmeleri (#453, #592, #683, #1291, #1554); GPU sorguları (#1731, #1740), derinlik sınırı (#158), HDR (#212), reverb/yankı (#240), tetik titreşimi (#1761), doku metaverisi (#263).

## Sırada ne var (tahmin)
1. **Kuyruk temizliği + kolay shader kazanımları:** `70564ae` ile boşa düşen yüzlerce PR'ın kapatılması ve **#1065** (+735 compute) ile **#1399** (195 birleşik geometri) gibi yüksek getirili PR'ların alınması.
2. **Aydınlatma ve kararlılık:** Yüzeyleri karanlık bırakan bariyer/format/eksik compute zincirinin (#334, #492, #876, #1762) çözülmesi ve **Wave64 öykünmesiyle (#439/#189/#578)** NVIDIA kartlarda donmasız dünya görüntüsü alınması.
3. **Kare hızı sıçraması:** Bellek kopyalama (#1267, #1351) ve eşzamansız gönderim (#595) birleştiğinde 1–2 sn/kare darboğazının kırılıp oynanabilir FPS yolunun açılması.
