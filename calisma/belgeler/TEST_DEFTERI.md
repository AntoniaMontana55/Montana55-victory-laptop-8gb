# Test defteri: ne denendi, bizde ne oldu, başkasında ne olmuştu

Makine: RTX 3070 Ti Laptop 8 GB, 16 GB RAM, i7-12700H, Series S profili, hapishane sahnesi.
Kural: her A/B sonucu buraya **aynı gün** yazılır ve repoya gönderilir. Sayı = dünya karesi medyanı (ms, düşük iyi).
"Başkasında" sütunu o değişikliği getirenin kendi iddiasıdır; biz tekrarlamadıysak öyle yazılır.
Ham koşu satırları: KOSULAR.md. Ayrıntılı günlük: STATUS.md. Aday listesi: ADAYLAR.md.

En önemli gerçek (26.11 09:55, kanıtlı): darboğaz **ekran kartı** (%91-100 dolu, kritik yolun ~119/145 ms'si).
Bu yüzden CPU/RAM tarafında kazandıran ayarlar başkasında fps artırsa da bizde kareyi kısaltmıyor.

## Kabul edilenler (tabanda)

| Ne | Kaynak | Başkasında | Bizde | Not |
|---|---|---|---|---|
| RT paketi: BVH_MISS + RT_SKIP_RESOLVE + RT_SKIP_ACTIVATE | cool_fox 8cc31ab3 + bizim BVH_MISS | cool_fox: iddia var, sayı yok | **184 → 150 ms (−%18)**, görüntü aynı | 26.11 11:50 |
| DYNAMIC_SLOTS=524288 | trustno1 taraması, bizim düzeltme | — | reddedilen tablo 562-1357/kare → 0, düşen indirect çizim 229-272 → 0; kare aynı | 09:25, görüntü düzeldi (avlu dolu) |
| CARD_HASH_WIDE (fxceless #1014) | fxceless | 3060 12 GB: hash 35 → 10 ms | geri kopya −%38, kart işi −%43; kare aynı | 25.11 23:14 (o zaman darboğaz gönderimdeydi) |
| İki düzlemli present (siyah ekran düzeltmesi) | ZSlayer PR #10 fikri, Sol portu | Series S siyah ekran düzeliyor | düzeldi, yükleme ekranı görünüyor | 26.11 03:35 |
| warm --used (pipeline ön hazırlığı) | upstream komutu | Nozy: fps değişmedi | fps aynı (172.5), **dünyaya giriş 177 → 105 sn, takılma p90 640 → 309** | Takılma için kabul, fps için değil |
| WINDOWS_BAR_PERCENT=88, WINDOW_KB=256, TAILS, SEND_KEPT_MB=128 | bizim RAM işi | — | kilitli RAM 4726 → 3252 MB, 10 dk sızıntısız | RAM kabulü, fps aynı |
| Windows 10 Agility SDK desteği | bizim | BlatWolf (4060, Win10) için | Win11'de etkisiz, Win10'da çökmeyi önler | 90d3c4f |

## Reddedilenler

| Ne | Kaynak | Başkasında | Bizde | Neden bizde olmadı (kanıt / TAHMİN) |
|---|---|---|---|---|
| SEND_BAR=1 | Neon GPU taraması (7900XTX forku) | 7900XTX: kart ~5 ms az | **152 → 180 ms** (temiz çift, +%18); ikinci çift 184 (tabanda takılma, p90 1607) → 174, belirsiz | Kopyalar kartta biter ama CPU kart belleğine veri yolu üzerinden yazıyor: "hazırlık" 95 → 145-160 ms'ye çıktı (log kanıtı) ve kritik yol oldu. 7900XTX masaüstünde tam Resizable BAR var; laptopta bu yazma yavaş (TAHMİN). Kilitli RAM ~230 MB az. 26.11 13:00 |
| INDIRECT_GATHER=1 (fxceless #1171, Sol portu 50ac222) | fxceless #1171 | **3090 Ti: GPU listeleri 56 → 43-45 ms, kare 116 → 105 ms** | **182 → 204 ms (p90 335 → 2625), 151 → 371 ms** (ikinci çift n=4, kare satırları 172-182 ms) | Bizde clamp'ler zaten toplu (batch) çalışıyor, toplayacak ~1000 ayrı geçiş yok; prelude ek bekleme/takılma getirdi (TAHMİN). Düşen çizim sayısı tabanla aynı (271 + ~690), yani bozulma yok, sadece yavaş. 26.11 12:45 |
| TEXTURE_DROP_MIPS=1 | fxceless #610 / soldsawn | soldsawn 4060L 8 GB: DROP_MIPS=2 ile 8-10 fps (A/B yok); #1022 5060L 16 GB RAM: 2-4 fps (A/B yok) | **153 → 168, 160 → 170 ms (+%6-10)** | VRAM'imiz taşmıyor (6.3/7.1 GB), yani kurtaracağı bir şey yok; ek kopya/eşleme işi kaldı (TAHMİN). Kimse temiz A/B yayınlamamış. |
| DISPLAY_MODE=1280x720 | Neon önerisi | — | 177 → 229, 154 → 278 ms | Oyun modu yok sayıyor, düzlemler yine 1920x1080 / 3840x2160 (log kanıtı). Çözünürlük için tek yol RENDER_SCALE. |
| KEEP_FREED=0 | Neon RAM taraması | — | 172 → 168, 159 → 175 ms | Fark gürültü içinde, RAM de düzelmedi. |
| RAM_TRIM (#1240) | fxceless #1240 | RAM rahatlaması iddiası | **155 → 223, 149 → 234 ms (+%45-55)** | Havuzlar hiç dolmadı, her snapshot yeniden ayrılıyor (profil: NtAllocateVirtualMemory %11, kanıt). |
| MEMORY_MAP=0 | bizim şüphe | — | 161 → 172, 156 → 167 ms | Kazanç yok; uzun kareler yalnız dünyaya girişin ilk 10-20 sn'si. |
| JOIN_PARTS=8 | fxceless | 4070: hız iddiası | 128-149 ms (hızlı) ama **görüntü bozuk** (bulanık, yıkanmış) | Hız ne olursa olsun ret. JOIN_PARTS=2 park. |
| CARD_BUFFERS_MIN_MB=1 | fxceless #413 reçetesi | **3070 Ti 8 GB: 2.05 → 4.62 fps** | **150 → 307 ms (2 kat yavaş)** | Karşılaştırma işi karta taşınıyor, kart işi 42 → 201 ms (log kanıtı). #413'te darboğaz CPU karşılaştırmasıydı, bizde kart. |
| RAW_VIEWS=0 | profil bulgusu | — | 149.7 vs 149.9 ms | Fark yok. |
| LTO thin | soldsawn Cargo.toml | soldsawn: ayrı ölçüm yok | 173 → 199 ms (+%15) | Gürültüden büyük; geri alındı. |
| Shader tablo yaması (e04c0781 elle) | fxceless | — | görüntü yok, RAM kazancı yok | Geri alındı. |
| TEXTURE_BUDGET_MB=768 | bizim | — | 149 ms ama **görüntü bozuk** (doku eksik, oda kararmış) | 1024+ kalır. |
| INPLACE=1 ("sans copie") | Nozy yayını | **Nozy: 16-18 fps** | 490. sn'de **cihaz sıfırlandı** (0x887A0005), yalnız 137 pencere yerinde | Nozy'nin 64 GB'ı var; bizde 16 GB'ta kilitli RAM 5.3 GB'a çıktı (kanıt). |
| COPY_MODE=plain | fxceless | — | 60. sn'de **çöktü** (0xC0000005) | Ham kopya eşlenmemiş sayfaya çarpıyor (Opus uyarısı doğrulandı). |
| WIDE_MERGE=1 (#1066) | fxceless | — | +18 ms (+%7) | Kopya işi arttı. |
| WINDOW_KB=128 | bizim | — | 263-272 → 296 ms | Daha çok parça; 256 kalır. |
| WIDE_REFRESH=1 | fxceless | — | fark yok | Kapalı. |
| cool_fox derlemesinin tamamı | codec-d/cool_fox | "çok hızlı" iddiası, ölçüm yok | **~2.5 kat yavaş + siyah ekran** | Siyah ekran: premultiplied alfa varsayımı (bizimki düz alfa). Repo silindi; iyi parçaları tek tek alınıyor. |

## Belirsiz / tekrar gerek

| Ne | Bizde | Not |
|---|---|---|
| BVH_MISS tek başına | 176/162 vs 208 ms | Bir taban bozuktu; paketle birlikte kabul edildi. |
| ASYNC_SUBMIT=1 | 154 → 148 ms (1 temiz çift) | Küçük; ikinci temiz çift gerek. |
| CBV_DIRECT=1 | 267 → 255 ms (eski taban) | Gürültü eşiğinin altında; tekrar gerek. |

## Sırada (STATUS.md "TEST LİSTESİ")

fxceless #1171 indirect gruplama (Sol portluyor; 3090 Ti: kart 56 → 43 ms) → #1227 CARD_SYNC_BATCH (kartta 29 → 6 ms) → SEND_BAR=1 → GPU_TIMING teşhisi → RENDER_SCALE 480/540/720p.
Ölçüm serisinden önce `shaders` + `warm --used` tazelenir.
