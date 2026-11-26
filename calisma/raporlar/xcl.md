# xcl-dev/xcl — Proje Durum Özeti (25 Kasım 2026)

## Şu an nerede
- XCL, Xbox Series oyunlarını CPU öykünmesi olmadan doğrudan Windows'ta çalıştıran uyumluluk katmanı (7.8k yıldız, 1.090 açık PR, 200 açık issue).
- Tempo aşırı yüksek: son 7 günde **244**, yalnızca son 2 günde (23–25 Kasım) **206 PR** birleşti; son 80 commit son ~4 saatte geldi.
- Birleştirmeleri fiilen 3 bakımcı yürütüyor (son 100 PR: `theaustinayers` 43, `ivall` 32, `raylitebull` 20; `Mavixot` 5). PR selini ("PRmageddon", #1673) yönetmek için tek hedef kondu (#1833): **yeni ticari oyunu dünya yüklemesinden geçirmek** (`goal:world-load`). İlgisiz PR'lar `off-goal` bekliyor, çakışanlar #1607'de ayıklanıyor, 3 gün sessiz kalan PR'lar otomatik bayatlayıp kapanıyor (`1d027e5`).

## Son 2 günde bitenler (206 PR)
- **D3D12.X / GPU (67):** Komut listesi `ExecuteIndirect` argüman sırası ve çevrilemeyen işleri atlama (#1791, #1195), çoklu yığın kilitlenmesini kaldırma (#1747), PSO (ekran kartı iş hattı) bekleme sınırı `XCL_PSO_WAIT_MS` (#891 — bazı makinelerde dünya sahnesi çizilmeye başladı), MSAA/kaynak hizalama düzeltmeleri (#1265, #1354).
- **Shader derleyici, Direct DXIL ve Wave/Lane (43):** En büyük sıçrama **doğrudan RDNA2→DXIL** (ara HLSL metni ve DXC derleyicisini atlayan) arka ucu `#1426` birleşti; peşinden şerit bazlı yürütme (#1745), bellek pencereleri (#1693), geometri/MSAA (#1679, #1806) ve dev shader şişmesini azaltma (#1748) geldi. NVIDIA 32-şeritli kartlarda Wave64 uyumu (#67, #133, #68) ve AMD sürücü (`amdxc64`) MSAA çökme yaması (#1900) kapandı.
- **Sistem — kernelx, VFS, XGameRuntime, Loader (28):** Dosya taşıma/kopyalama yönlendirmesi (#58–#60, #96), asenkron görev kuyruğu kilitlenmeleri (#53–#57, #98), Wine uyumu (#93, #213, #270), hızlı açılış için Bink giriş videosunu atlama (#1053).
- **Bellek / VRAM / UMA (11):** Coherent (CPU-GPU ortak) pencerelerde `VirtualQuery` maliyetini kaldırma (#1005); GPU-görünür bellek takibiyle menü yazıları ve giriş videosu düzeldi (#252, #283).
- **DirectStorage / BCPack (5):** Konsol doku sıkıştırması BCPack (BC1, BC3–BC7) çözücüsü tamamlandı (#905, #947–#950; reddedilen doku 512→0), durum/hata kayıtları düzeltildi (#1223, #1224).
- **Diğer (52):** Ses/XAudio2 (#89, #90), giriş (3), ışın izleme/BVH (3), ağ sertifika kontrolü (#77), test/CI ve PR kalite kapıları (#1037, #1133).

## Şu an üzerinde çalışılanlar (açık PR'lar)
- En güncel 500 açık PR'ın dağılımı (436 hazır, 64 taslak): **D3D12.X/GPU (141)**, **Shader/PSO (130)**, **Test/CI (45)**, **Sistem (40)**, **Direct DXIL (34)**, **Wave/Lane/EXEC (30)**, **Bellek/VRAM (23)**, **Ses (14)**, **DirectStorage (9)**, **Giriş (6)**, **Ray Tracing/BVH (6)**, **Ağ (2)**.
- Oyun bazlı ayrı bringup yok; eforun neredeyse tamamı tek yeni ticari oyuna odaklı (#1604 klavye/fare ikonları, #1655 README VRAM notu). Yan akışta Wine/Linux ve Rosetta 2 uyumu sürüyor (#1395, #1344).
- **En kritik açık PR'lar:**
  - `#1836`, `#1887`, `#1936`, `#1937`, `#1942`: Mesh (ağ) shader önbelleği, dolaylı `DispatchMesh` ve çift PSO kalıcılığı (reddedilen minimap ve geometri hatları için).
  - `#1801`, `#1630`: Direct DXIL'de gereksiz şerit maskelerini (`EXEC`) ve ölü okumaları kaldırarak derlemeyi hızlandırma.
  - `#1800`, `#1815`: Işın izleme BVH döngüsünü küçültme ve donanım DXR `RayQuery` ile kıyaslama (#1751).
  - `#1638`, `#1618`: DXC'yi bellek sınırlı alt süreçte çalıştırma ve `dxil.dll` doğrulamasını atlayıp doğrudan imzalama.
  - `#1798`, `#1931`: Çizimin derlenen PSO'yu beklememesi ve kare başına PSO bekleme bütçesi.
  - `#1329`, `#1916`: VRAM kopya yolu ve özel fiziksel sayfalarda GPU karo tahsisini erteleme.
  - `#1819`, `#1323`: Kuyruk başına `cpu_writes` işçisi ve donan GPU kuyruğundaki suçlu işi raporlama.
- **Çakışan PR kümeleri:** PSO derleme/önbellek/bekleme bütçesi (127 PR), VRAM/bellek bütçesi (76 PR), Direct DXIL optimizasyonları (49 PR) ve Mesh/Geometry PSO (12 PR — örn. `#1936/#1937/#1942/#1836`) yoğun çakışma yaşıyor; #1607'de eşleştirilip eleniyor.

## Oyun bazında durum
- **Oynanabilir (✅):** *Minecraft Preview* (NVIDIA titreme ve logo kareleri #84, #86 ile düzeltildi), *Unicorn Overlord* (tam oynanabilir).
- **Sunucu sınırına kadar çalışan (✅):** *Fortnite* ve *XDefiant* (çevrimiçi sunucuları olmadan oyunun izin verdiği noktaya kadar tam çalışıyor).
- **Geliştirme aşamasında (🚧) — "A newer retail game":** ~4 sn'de açılıyor, menülerde sabit 30 fps ve sesli çalışıyor, ayarlar ekranını geçiyor; oynanışa henüz ulaşılamadı, minimap mesh hatları reddediliyor.

## the title'ya ne kadar yakın
- README'deki "newer retail game" doğrudan **the title** Xbox Series sürümü (`game_scarlett_master.exe` 1.0.2.1, TitleID `7952987A` — #505, #846, #1585, #1773).
- **VIctory (`Marvisak/VIctory`) bağı:** XCL'in BCPack doku çözücüsü (`bcpack/victory.rs`, #1337, #1440) VIctory tablolarını kullanıyor; VRAM şişmesi de VIctory mimarisiyle kıyaslanarak inceleniyor (#1773).
- **Oynanışa engel 4 duvar (#1833):** (1) Dev shader/PSO derleme süreleri (#1269, #1035), (2) Aşırı VRAM/RAM tüketimi — 8–12 GB kartlarda 15+ GB VRAM ve 36–94 GB sanal bellek (#1337, #1566, #1773), (3) Dünya yükleme sonunda deterministik null yazma çökmesi (`+0x23cf683`, #846, #662, #1401) ile GPU bekleme/cihaz kaybı (`0x887a0005`, #505, #1452), (4) Reddedilen minimap mesh PSO'ları ve AMD'de doku akış takılması (#1268).
- **Konum:** Motorun iç durumuna göre yayalar/araçlar doğup kamera dünyaya geçebiliyor (#505), ancak yükleme ekranı kalkmadan çökme/donma yaşanıyor. Yani "menü→dünya geçişi" eşiğinde; oynanabilir ilk kareye oldukça yakın, ama kararlı oynanış ve 8–12 GB kartlar için hâlâ birkaç temel engel var.

## Sırada ne var (tahmin)
- **Kısa vade (günler):** Direct DXIL (`#1426`) varsayılan/olgun hale getirilip `#1801/#1630` benzeri optimizasyonlarla derleme süreleri aşağı çekilecek; `#1887/#1936/#1937/#1836` ile mesh/minimap PSO reddi kapanacak.
- **Orta vade (1–2 hafta):** `+0x23cf683` null yazma çökmesi (#846) ve `Wait64BitValueX` / `0x887a0005` GPU cihaz kaybı (#1452, #505) çözülerek the title'da ilk gerçek oynanış karelerine ulaşılması en olası sıçrama.
- **Devamı:** VIctory'de olduğu gibi isteğe bağlı VRAM tahsisi ve bütçe kısıtı (#1773, #1916, #1337) ile 8–12 GB GPU'larda çökmeden oynanış + isteğe bağlı donanım DXR ışın izleme yolu (#1751).
