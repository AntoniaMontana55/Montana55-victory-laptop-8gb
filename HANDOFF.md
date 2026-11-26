# HANDOFF — NVIDIA 8 GB + 16 GB RAM dalı (`nvidia-16gb`)

Amaç: Xbox Series sürümünü VIctory ile **RTX 3070 Ti Laptop (8 GB VRAM) + 16 GB RAM**'de kararlı ve olabildiğince hızlı çalıştırmak. Bu repo kod + bütün çalışma kayıtlarıdır. **Oyun dosyası, exe, kayıt dosyası yoktur**; kendi kopyanla çalışırsın.

## Şu an nerede (26.11.2026)

- Hapishane başlangıç sahnesi (prologue) **görüntülü, ışıklı, 10 dk çökmeden** (koşu B13b).
- Dünyada **~157–173 ms/kare = 5.8–6.4 fps**, takılma p90 ~190–270 ms. İlk dünya karesi 95–120. sn.
- Bellek: kilitli PC→kart belleği **3.97 → 3.27 GB**, sızıntı yok; tepe VRAM ~6.8 GB / 8 GB.
- Darboğaz artık **hız**: CPU hazırlık 80–101 ms, geri kopya (copy-back) 53–60 ms, GPU 24–55 ms (run.log `copystat` satırları).

Test makinesi: i7-12700H, RTX 3070 Ti Laptop 8 GB, 16 GB RAM, Windows 11, NVIDIA 616.64, ReBAR açık. Profil: **Series S**, 1920x1080.

## Taban ve bu daldaki değişiklikler

- `98e44cb` taban: [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX) @ `0a435665` (o da fxceless/victory-fork tabanlı, GPL-3.0).
- `2706abd` bizim değişiklikler:
  - **Series S iki düzlemli Present** (`queue.rs`, `queue/behind.rs`, `host.rs`, yeni `host/present_planes.rs`): taban yalnız 1. düzlemi gösteriyordu → Series S'te yükleme/UI siyahtı. Kaçış: `VICTORY_PRESENT_PLANES=0`.
  - `gameinput.rs`: `VICTORY_KEYS` release derlemede de çalışır (otomatik Enter için).
  - `shadow.rs`: `VICTORY_WINDOWS_BAR_PERCENT` (50–88, varsayılan 80) — pencereleri kart belleğine (BAR) daha çok koyar → kilitli RAM düşer.
  - `host/send.rs`: `VICTORY_SEND_KEPT_MB` (16–1024) — gönderim belleği tavanı.
  - `book.rs` + `copystat.rs`: `VICTORY_BOOK=1` ile yığın türüne göre bellek sahibi muhasebesi (kim ne kadar tutuyor).
  - `inplace.rs`: pencere boşluk sayımı (census, yalnız ölçüm).
- `2d2fc48` + `96a3fdd`: lto=thin denendi ve **geri alındı** (aşağıda).
- `calisma/`: belgeler (STATUS, KOSULAR, ADAYLAR, reçeteler), raporlar (fork incelemeleri, güvenlik), araçlar (PowerShell), yamalar. Kişisel yollar `%USERPROFILE%` / `<OYUN_KLASORU>` ile değiştirildi.

## Derleme

```powershell
cargo build --profile diag -p victory --features diagnostics --offline --locked   # target\diag\victory.exe (+ .pdb)
```
Rust 1.98.1 (rust-toolchain.toml), MSVC Build Tools. `diag` = release kodu + tanı özellikleri; ölçümler bununla.

## Çalıştırma (bizim ölçüm düzenimiz)

Araçlar `calisma/araclar/`. Başlamadan oku: `calisma/belgeler/KOSU_RECETESI.md`.

1. Güvenlik: exe için Windows Güvenlik Duvarı'nda giden+gelen **engel kuralı** (`guvenlik_duvari.ps1`). Oyun internete çıkmaz.
2. Laptop'ta **NVIDIA ataması şart**: DXGI adaptör 0 = Intel iGPU; `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` → exe yolu = `GpuPreference=2;`. Yoksa oyun Intel'de açılır.
3. `kosu.ps1` içindeki oyun yolunu (`<OYUN_KLASORU>\game_scarlett_master.exe`) kendi yoluna çevir, sonra:
   ```powershell
   .\kosu.ps1 -Tag taban                      # güncel en iyi ayarlar, 4.5 dk
   .\kosu.ps1 -Tag deneme -Ek @{ VICTORY_X = '1' }   # TEK değişken ekle
   ```
   `kosu.ps1` = sabit en iyi ayar seti + otomatik Enter (20–150 sn, 2.5 sn basılı / 0.5 sn bırak; intro basılı tutunca geçer) + koruma (boş RAM < 0.2 GB 30 sn sürerse durdur, commit tavanı 28 GB) + `erken_teshis.ps1` (siyah/donuk ekranda erken durdurur).
4. Özet: `kisa_ozet.ps1 <koşu klasörü>` → ilk dünya sn, dünya kare medyanı/p90, ~200. sn bellek.

**Ölçüm kuralları:** koşu sırasında klavye/fareye dokunma; tek değişken; karar veren koşu 2 tekrar (gürültü ~%2); bellek karşılaştırması sabit saniyede (~200. sn).

## Güncel en iyi ayar seti (kosu.ps1 içinde)

`RELAY=2, RELAY_COPY=1, WIDE_BACK=1, JOIN_COPIES=1, INDIRECT_BATCH=2, INDIRECT_WINDOW=1, TEXTURE_BUDGET_MB=1024, TEXTURE_NO_WAIT=1, WINDOW_KB=256, WINDOW_TAILS=1, WINDOWS_BAR_MB=2048, WINDOWS_BAR_PERCENT=88, SEND_KEPT_MB=128, CARD_AWAKE=0, CARD_REHASH_RUNS=1, CARD_STEPS=2, UNDER_WIDE=0, LAND_ECHO=0, MAP_THREADS=1, TITLE_PRIORITIES=0, BOOK=0, PIPELINE_BOOK=1, MEMORY_MAP=1, SHOW_AFTER=1, AMBIENT=0.8, D3D12_DRED=light` (hepsi `VICTORY_` önekli). Pipeline önbelleği bir kez: `victory warm --used --jobs 4` (ilk dünya 177 → 105 sn, takılma p90 640 → 309 ms).

## Denendi, işe yaramadı (tekrar deneme)

| Deneme | Sonuç |
|---|---|
| `lto = "thin"` | 198.9 ms vs 172.7 ms → **yavaş** |
| `VICTORY_INPLACE=1` | yalnız 208 MB yerinde, kilitli RAM arttı, 490. sn'de cihaz sıfırlandı (0x887A0005) |
| `TEXTURE_BUDGET_MB=768` + `WINDOWS_BAR_MB=2560` | görüntü bozuk (oda karanlık, doku eksik) |
| Marvisak e04c0781 shader sabit tablosu (elle taşıma) | RAM kazancı yok, görüntü şüpheli |
| Neptune `CLAMP_AFTER_WRITER` | bizde `INDIRECT_BATCH=2` zaten var (~155 clamp/kare); onların 4.6→8.4 fps'i batch olmamasından |
| fxceless #2856 (geniş pencere kapsar) | bizde descriptor slotları isimle bulunmalı, bayat pencere riski |

## Profil (karenin nereye gittiği)

`VICTORY_PROFILE=<from>,<to>,<ms>,<dosya>` diag'da örnekleyici profil yazar. **Dikkat:** uzun pencerede profilcinin kendisi çöktü (`unwind_into` → ntdll, 221. sn). Güvenli kullanım: 15 sn'lik parçalar, 20 ms, `;` ile:
```powershell
$p = (150,165,180,195 | % { "$_,$($_+15),20,$env:LOCALAPPDATA\Victory\kosular\profil_$_.txt" }) -join ';'
.\kosu.ps1 -Tag profil -Ek @{ VICTORY_PROFILE = $p }
python profil_oku.py <birleşik profil> --kisa [--ara shadow::lookup] [--thread victory-wide]
```
İlk bulgu (P1b): `shadow::lookup` örneklerin yalnız %0.18'i → fxceless #1237 (lookup cache) bizde küçük kazanç (TAHMİN). Sıcak yerler: başlığın kendi kodu, `shadow::workers` (~%17), `land::Pool::help` (~%11), `draw::State::bind`, `relay_worker`. Analiz yalnız çalışan iş parçacıklarına daraltılmalı (bekleyenler ntdll'de görünüyor).

## Sıradaki adımlar

1. Profili yalnız aktif iş parçacıklarına göre ayır → en büyük CPU kalemini bul.
2. Geri kopya (53–60 ms) için aday paket: Marvisak #2948 (readback azaltma), fxceless #1229–#1233, Neptune `QUIET_BACK/COPY_CHANGED/UNDER_WRITES_*` (bkz. `calisma/raporlar/HASAT_MATRISI.md`, `neptune_traffic_inceleme.md`).
3. Marvisak #2958 (mark atomikleri wave başına; NVIDIA'da −%25 iddiası).
4. Bellek ikinci sırada: soldsawn `TEXTURE_DROP_MIPS=2`.

Her yabancı commit için önce güvenlik incelemesi (`calisma/raporlar/guvenlik_*.md` örnekleri), sonra tek değişkenle 2 koşu.

## Nerede ne var

- `calisma/belgeler/STATUS.md` — günlük (en yeni üstte), kanıtlı / tahmin ayrımıyla.
- `calisma/belgeler/KOSULAR.md` — her koşunun ayar + sonuç satırı.
- `calisma/belgeler/ADAYLAR.md`, `calisma/raporlar/HASAT_MATRISI.md` — fork'lardan alınabilecekler.
- `calisma/raporlar/` — fork incelemeleri (Neptune, soldsawn, trustno1, ZSlayer, Belobug…) ve güvenlik raporları.
