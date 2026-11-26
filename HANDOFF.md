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

`RELAY=2, RELAY_COPY=1, WIDE_BACK=1, JOIN_COPIES=1, INDIRECT_BATCH=2, INDIRECT_WINDOW=1, TEXTURE_BUDGET_MB=1024, TEXTURE_NO_WAIT=1, WINDOW_KB=256, WINDOW_TAILS=1, WINDOWS_BAR_MB=2048, WINDOWS_BAR_PERCENT=88, SEND_KEPT_MB=128, CARD_AWAKE=0, CARD_REHASH_RUNS=1, CARD_STEPS=2, UNDER_WIDE=0, LAND_ECHO=0, MAP_THREADS=1, TITLE_PRIORITIES=0, BOOK=0, PIPELINE_BOOK=1, MEMORY_MAP=1, SHOW_AFTER=1, AMBIENT=0.8, D3D12_DRED=light, DYNAMIC_SLOTS=524288` (hepsi `VICTORY_` önekli).

**NVIDIA'ya özel (26.11, D1):** 1 milyon view sınırlı heap'te dinamik tablo havuzu sabit 2^18'di ve `VICTORY_DYNAMIC_SLOTS` yok sayılıyordu; kare başına ~11 bin tablonun 560-1.360'ı reddedilip o çizimler **düşüyordu** (log'daki "dropped … indirect"). Kod artık 1M heap'te de bu ayarı dinliyor (en çok heap'in yarısı; trustno1-coder a9632bc8'den). `524288` ile reddedilen 0, düşen indirect 0, oturmuş kare süresi aynı (~145-180 ms), görüntü temiz. Pipeline önbelleği bir kez: `victory warm --used --jobs 4` (ilk dünya 177 → 105 sn, takılma p90 640 → 309 ms).

## Denendi, işe yaramadı (tekrar deneme)

| Deneme | Sonuç |
|---|---|
| `lto = "thin"` | 198.9 ms vs 172.7 ms → **yavaş** |
| `VICTORY_INPLACE=1` | yalnız 208 MB yerinde, kilitli RAM arttı, 490. sn'de cihaz sıfırlandı (0x887A0005) |
| `TEXTURE_BUDGET_MB=768` + `WINDOWS_BAR_MB=2560` | görüntü bozuk (oda karanlık, doku eksik) |
| Marvisak e04c0781 shader sabit tablosu (elle taşıma) | RAM kazancı yok, görüntü şüpheli |
| Neptune `CLAMP_AFTER_WRITER` | bizde `INDIRECT_BATCH=2` zaten var (~155 clamp/kare); onların 4.6→8.4 fps'i batch olmamasından |
| fxceless #2856 (geniş pencere kapsar) | bizde descriptor slotları isimle bulunmalı, bayat pencere riski |
| `VICTORY_RAW_VIEWS=0` | 149.7 vs 149.9 ms → fark yok |
| `CARD_BUFFERS_MIN_MB=1` (8 GB kart) | GPU 42 → 201 ms, kare 2× yavaş |
| `VICTORY_ASYNC_SUBMIT=1` | 147.9/155.6 vs 154.3 ms → nötr |
| `VICTORY_JOIN_PARTS=8` | görüntü bulanık/yıkanmış → **ret** |
| `VICTORY_MEMORY_MAP=0` | 172/167 vs 161/156 ms → fark yok |
| `VICTORY_RAM_TRIM_MB=2048` (bizim RT1) | +%45-55 yavaş (havuzlar hep boş); `=300` nötr, yükleme etkisi belirsiz |
| Kesin tablo-slotu yazmaları (bizim S1) | karşılaştırma 240.5 → 238.7 MB, etkisiz |
| Neptune_VI build (bizim makinede) | 175-281 ms + siyah ekran |

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

**Hazır exe/zip/installer kullanılmaz (26.11 uyarısı):** bir modding Discord'unun dağıttığı "exe"nin, kendi JDK'sını getiren bir Electron uygulaması olduğu ve gizli, karmaşıklaştırılmış bir Java yükü çalıştırarak tarayıcı girişlerini, kripto cüzdanlarını, Discord/Steam oturumlarını çalıp yüklediği bildirildi (statik analiz, X @Godly__BDS). Kural: yalnız kaynak koddan kendimiz derleriz; fork'lardan yalnız okunmuş diff alınır; repo içindeki başlatıcı/indirici `.ps1`/`.bat` çalıştırılmaz (DexNom, ad3lre, trustno1, Neptune'de var), içinden yalnız `VICTORY_*` değerleri okunur. "Hızlandırıcı", "launcher", "fps paketi" adıyla gelen ikili dosyaya dokunma.

## Güçlü sistemde (12–16 GB VRAM, 32+ GB RAM) denenecekler

Bizim 8 GB / 16 GB makinede bellek yüzünden deneyemediklerimiz. Her biri ayrı koşu, tek değişken; önce bizim ayar setiyle (`kosu.ps1 -Tag taban`) kendi taban ölçümünü al.

1. **Series X profili** (`--console series-x`, 4K iç çözünürlük): bizde RAM yetmedi. Taban ile fps/görüntü farkı.
2. **Neptune ayar seti** (README'de 6.500 draw / ~90 ms açık dünya; RTX 5070 Ti'de 20+ fps): `INDIRECT_RAW=3`, `CLAMP_AFTER_WRITER=1`, `CLAMP_BATCH=1`, `LEVELS_ON_DEMAND=1`, `DROP_DXIL=1`, `DESC_DATA=1`, `PIPELINE_WAIT_MS=2500`, `TEXTURE_HASH_EVERY=32` ve 32 GB için `LIST_POOL_MB=512`, `LIST_POOL_MOST=1024`, `TEXTURE_STAGE_MB=256`, `CARD_WINDOWS_MB=3072`. Bunların bir kısmı yalnız Neptune kodunda var (`calisma/raporlar/neptune_traffic_inceleme.md` §2.2); bizim ağaçta olmayanlar sessizce yok sayılır — run.log'da hangilerinin tanındığına bak.
3. **Yerinde (kopyasız) pencereler** `VICTORY_INPLACE=1` + `INPLACE_MB=4096`: 16 GB'ta cihaz sıfırlandı; bol RAM'de sonuç farklı olabilir.
4. **Kart tamponları** `VICTORY_CARD_BUFFERS_MIN_MB=1` + `CARD_BUFFERS_MB=1024` (trustno1 önerisi) ve `WINDOWS_BAR_MB=4096`.
5. **trustno1 hız seti** (`a9632bc8` DYNAMIC_SLOTS=524288, `24a479d3` BVH_MISS=1, `afd84de1` RENDER_SCALE=0.5 + OWN_UPSCALER=1): kod portu gerektirir; Smooth Motion (sürücü ara karesi) **kapalı** ölçülmeli, yoksa fps ikiye katlanmış görünür.

**Bize ne lazım (her koşu için):** donanım (CPU, GPU+VRAM, RAM, sürücü), profil (S/X), çözünürlük, `-Ek` ayarları, `kisa_ozet.ps1` satırı (ilk dünya sn, dünya medyan/p90 ms, ~200. sn bellek), bir ekran görüntüsü, çöktüyse run.log'un son 100 satırı. Bunları issue olarak açabilir ya da `calisma/belgeler/KOSULAR.md` biçiminde gönderebilirsin.

## Takip ettiğimiz repolar

Ayrıntı ve commit bazında değerlendirme: `calisma/raporlar/HASAT_MATRISI.md` (bizde var/yok, 8/16 GB değeri, risk), `calisma/belgeler/ADAYLAR.md`, `calisma/raporlar/watcher.md`.

| Repo | Neden takipte | Bizdeki durum |
|---|---|---|
| [Marvisak/VIctory](https://github.com/Marvisak/VIctory) | upstream | #2948 (readback), #2958 (mark atomikleri) aday; #2088/#2112/#2921 incelendi |
| [fxceless/victory-fork](https://github.com/fxceless/victory-fork) | ana topluluk fork'u, en hareketli | #1237 (lookup cache, profilde bizde %0.18 → düşük), #1172, #1229–#1233 (geri kopya) aday; #2856 ret |
| [avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX) | **bizim taban** (@0a435665) | yeni commit gelirse taşınır |
| [zm0kiemedia/Neptune_VI](https://github.com/zm0kiemedia/Neptune_VI) | Neptune (Traffic & zm0kie), high-end'de 20+ fps | CLAMP ret; geri kopya paketi (QUIET_BACK, COPY_CHANGED, UNDER_WRITES_*) ve placement.rs aday — `neptune_traffic_inceleme.md` |
| [soldsawn/victory-nvidia-8gb](https://github.com/soldsawn/victory-nvidia-8gb) | en yakın donanım (RTX 5060 Laptop 8 GB) | lto ret; TEXTURE_DROP_MIPS=2, wave32 CBV aday — `soldsawn_inceleme.md` |
| [trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA) | bizim tabanla aynı + 9 commit, "20–22 fps" | iddia 960×540 + Smooth Motion (ara kare); DYNAMIC_SLOTS, BVH_MISS, RENDER_SCALE aday — `trustno1_nvidia_inceleme.md` |
| [ZSlayerHQ/zslayer-pc-port-runtime](https://github.com/ZSlayerHQ/zslayer-pc-port-runtime) | Series S present (PR #10) | present portu bizde (2706abd) |
| [Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD) | sayfa havuzu yerinde (0434b3ac), 788ea46 | aday — `belobug.md` |
| [CSTCryst/VIctory-Cryst-NVIDIA](https://github.com/CSTCryst/VIctory-Cryst-NVIDIA) | NVIDIA fork | izleniyor |
| [myokex/victory-amd](https://github.com/myokex/victory-amd) | JOURNAL_FREE (aa8399e5, 179→159 ms iddiası) | aday, önce güvenlik incelemesi |
| [triippywhale/victory-fork](https://github.com/triippywhale/victory-fork) | CBV_IN_PLACE, BACK_CURRENT | izleniyor |
| [timibloozinger/VIctory-compact](https://github.com/timibloozinger/VIctory-compact) | page mirror (RAM) | güven düşük (0★, yeni), park |
| [KomanderDG88/VIniVIdiVIci](https://github.com/KomanderDG88/VIniVIdiVIci) | TEXTURE_SPILL | izleniyor |
| [xcl-dev/xcl](https://github.com/xcl-dev/xcl) | başka katman, davranış referansı | yalnız referans — `xcl.md` |
| [LooperSalty/AnyXboxX](https://github.com/LooperSalty/AnyXboxX), [ThatRecompDev/five-star-recomp](https://github.com/ThatRecompDev/five-star-recomp) | tanı araçları fikirleri | referans — `tani_yontemleri.md` |

## Watcher (repoları sürekli izletmek)

Biz bir ajana 3 saatte bir yukarıdaki repoları taratıp `calisma/raporlar/watcher.md`'ye (en yeni üstte) yazdırıyoruz; örnek kayıtlar orada. Kendi ajanına (Claude Code, Codex, Gemini…) verebileceğin hazır görev:

```text
SALT OKUNUR. Kod indirme/derleme/çalıştırma yok; yalnız `gh api` ile oku. Tek dosyaya yaz: calisma/raporlar/watcher.md (en yeni kayıt en üstte, başlık "## <tarih saat> — <son kontrolden beri>").
İzlenen repolar: HANDOFF.md "Takip ettiğimiz repolar" tablosu + fxceless/victory-fork'un en çok yıldızlı 15 fork'u (`gh api repos/fxceless/victory-fork/forks?sort=stargazers`).
Her repo için son kontrolden beri: yeni commit'ler (SHA, 1 cümle ne yapıyor, env anahtarı mı varsayılan mı), yeni/merge edilen PR'lar, README'de fps/donanım iddiası.
Öncelik: HIZ (ms/kare, CPU hazırlık, geri kopya) ve 8 GB VRAM / 16 GB RAM'e uyan BELLEK değişiklikleri. Ölçümlü iddiayı (fps, ms, donanım, sahne) TAHMİN'den ayır; ara kare (Smooth Motion/frame gen) ile ölçülmüşse belirt.
Sonda "ÖZET" satırı: büyük gelişme var/yok + bizim dal için ilk 3 aday (bkz. calisma/raporlar/HASAT_MATRISI.md biçimi).
```

## Nerede ne var

- `calisma/belgeler/STATUS.md` — günlük (en yeni üstte), kanıtlı / tahmin ayrımıyla.
- `calisma/belgeler/KOSULAR.md` — her koşunun ayar + sonuç satırı.
- `calisma/belgeler/ADAYLAR.md`, `calisma/raporlar/HASAT_MATRISI.md` — fork'lardan alınabilecekler.
- `calisma/raporlar/` — fork incelemeleri (Neptune, soldsawn, trustno1, ZSlayer, Belobug…) ve güvenlik raporları.
