# `trustno1-coder/VIctory-NVIDIA` İnceleme ve Güvenlik Raporu

## 1. Repo Kimliği, Köken ve Release Durumu
- **Kim / Hesap:** `trustno1-coder` (GitHub hesabı `2026-11-25T23:34:23Z` — repodan 1 dk önce açılmış, 0 takipçi, 1 kamu reposu; README sonunda *"This fork's changes were made with Claude Code"* notu var).
- **Tarih & İstatistikler:** Oluşturulma `2026-11-25T23:35:29Z`, son push `2026-11-26T00:19:59Z` (~45 dk içinde 9 commit). **33★**, **3 fork** (API listesinde 4), **0 issue**, **0 PR**, **0 commit yorumu**, **0 tag**.
- **Lisans:** `GPL-3.0-or-later` (`Cargo.toml` ve `LICENSE`).
- **Parent / Taban:** GitHub'da `fork: false` (standalone repo), ancak git geçmişi korunmuş ve **birebir bizim çalışma tabanımız olan `avatarmanz/VIctory-7900XTX @ 0a435665`** üstüne **9 commit önde / 0 commit geride** (`HEAD = 3e5fb363`, toplam 12 dosya, `+903/-57`).
- **Release / Hazır Exe:** **Yok** (0 release, 0 tag; repoda `.exe`/`.dll`/`.zip` yok, tek `.bin` upstream'den gelen `crates/victory-formats/src/bcpack-tables.bin`).

---

## 2. "İnanılmaz Çalışıyor" İddiasının Kanıtı
- **Kanıt Durumu:** **Anekdot / tek-makineli yazar raporu.** Repoda issue/PR/yorum yok; GitHub genelinde bu repoyu bağımsız doğrulayan harici test raporu bulunmadı.
- **Test Donanımı & Sahne (README):** Windows 11, **Ryzen 7 9800X3D**, **64 GB RAM**, **RTX 5070 12 GB**, **Prologue (iç mekân)** — açık dünya ölçümü yok.
- **Yazarın Adım Adım Ölçümü (`scripts/play-nvidia.ps1`):**
  1. `7900XTX` ayarları (Series X): yüksek FPS görünüyor ama kare başına 1.100–4.800 draw atlanıyor (ekran donup atlıyor).
  2. `VICTORY_DYNAMIC_SLOTS=524288` + Series X tam ölçek: **6.0 fps** (GPU 110 ms/kare).
  3. Aynı + yarım ölçek (`RENDER_SCALE=0.5`): **7.2 fps** (GPU 82–88 ms/kare).
  4. Series S + yarım ölçek (**960×540** → 1080p) + `OWN_UPSCALER=1` + `BVH_MISS=1`: **11.0 fps** (GPU 33 ms/kare).
  5. Artı `CARD_BUFFERS_MIN_MB=1` + `CARD_BUFFERS_MB=1536` (bellek karşılaştırması GPU'da): **21.8 fps** (~46 ms/kare, copy-back 27→10 ms, GPU ~6 ms; hafif odalarda 24–31 fps).
- **Gerçeklik Payı (`TAHMİN`):** 20–22 fps'nin "inanılmaz akıcı" algılanmasında (a) 960×540 dahili çözünürlük + ışın izleme kapatma (`BVH_MISS=1`), (b) 9800X3D + 64 GB RAM + 12 GB VRAM üstünlüğü, (c) `play-nvidia.ps1`'in varsayılan `victory_sm.exe` kopyasıyla **NVIDIA Smooth Motion (2× sürücü kare üretimi)** kullanması etkili. Bizim **RTX 3070 Ti Laptop 8 GB + 16 GB RAM** açık dünya (~170 ms) senaryomuzda aynı 20–22 fps doğrudan çıkmaz (`TAHMİN`), fakat altta yatan teknik kazanımlar (özellikle descriptor düzeltmesi + GPU buffer karşılaştırması + `BVH_MISS` + yarım ölçek) gerçek ve doğrudan aktarılabilir.

---

## 3. Tabana (`0a435665`) Göre Bizde Olmayan Commit'ler
Toplam 9 commit'in **4'ü yalnız README** (`969843e0`, `10e87106`, `0cdad6b3`, `3e5fb363`), **5'i kod/betik** (`+733/-8`, 11 dosya). **Kod değişikliklerinin tamamı opt-in ENV anahtarlıdır** (varsayılan kapalı, `play-nvidia.ps1` açar):

| SHA | Satır | Dosyalar | Anahtar / Varsayılan | Ne Yapar? (Hız / NVIDIA / Bellek) |
|---|---|---|---|---|
| `a9632bc8` | `+14/-1` | `crates/victory-gpu/src/descriptor/host.rs` | `VICTORY_DYNAMIC_SLOTS` (opt-in, varsayılan `2^18`, tavan `null/2`) | **NVIDIA'ya özgü:** NVIDIA shader-visible view heap tavanı 1.000.000 (oyun 1.245.184 ister). `THIN=1` ile taşan dinamik havuzu `524288`'e çıkarıp kare başına 1.100–4.800 atlanan draw'ı 0–33'e indirir. |
| `7997a203` | `+363/-5` | `crates/victory-gpu/src/{compose.rs, queue.rs, host.rs, lib.rs, shader_cache.rs}` | `VICTORY_COMPOSE=1`, `VICTORY_PLANE_PREMULTIPLIED=1` (opt-in) | Series S'in 4K arayüz düzlemini 1080p oyun üstüne compute shader ile bindirir (`fxceless#1189` portu, FSR taklidi yok) → siyah menü/altyazı çözülür (~8 MB VRAM). Ayrıca `shader_cache.rs` anahtarına `VICTORY_BVH_MISS` ekler. |
| `afd84de1` | `+277/-0` | `crates/victory-loader/src/{hooks.rs, windows.rs, lib.rs}` | `VICTORY_RENDER_SCALE` (`0.2..=1.0`), `VICTORY_OWN_UPSCALER=1`, `VICTORY_NO_WATCHDOG` (opt-in) | Yalnız `1.0.2.1` exe (BLAKE3 `b99f876b74d4a77f`) + tam bayt eşleşmesiyle bellek yaması (`victory-perf`/`ScarlettPort`): dahili çözünürlüğü düşürür (Series S'te 960×540), FSR yerine oyunun kendi temporal upscaler'ını zorlar, 30 sn watchdog çökmesini susturur. |
| `24a479d3` | `+21/-2` | `crates/victory-shader/src/emit/intersect.rs` | `VICTORY_BVH_MISS=1` (opt-in) | BVH oluşturucu kapalıyken eski düğümleri gezmek yerine tüm kutu/üçgen testlerini `MISS` döndürür; GPU hang önler, ~10 ms/kare GPU süresi kazandırır (karanlık odalarda sekme ışığı azalır). |
| `a1849fac` | `+58/-0` | `scripts/play-nvidia.ps1` | Başlatıcı betik | `play.ps1` ayarları + yukarıdaki anahtarlar + bizde **zaten var olan** `CARD_BUFFERS_MIN_MB=1` / `CARD_BUFFERS_MB=1536` + `PIPELINE_WAIT_MS=10000` + `victory_sm.exe` kopyası (`-NoSmoothMotion` / `-SeriesX`). |

> **Eksikler (8 GB VRAM / 16 GB RAM notu):** Bu repoda `Cargo.toml` `lto = "thin"`, `VICTORY_TEXTURE_DROP_MIPS=2`, NVIDIA Wave32 CBV 64 KiB clamp (`09b95ca9c9`) ve 16 GB RAM bütçe kısıtları **yoktur**.

---

## 4. Güvenlik Denetimi (Salt Okunur)
| Kontrol | Durum | Kanıt (`dosya:satır`) |
|---|---|---|
| **Ağ erişimi** (`reqwest`/`TcpStream`/`WinHttp`/URL) | **TEMİZ** | 9 commit'lik diff'te ve tüm `Cargo.toml` dosyalarında ağ crate'i veya soket/HTTP çağrısı yok (`README.md` dış bağlantıları hariç). |
| **Telemetri / IPC** | **TEMİZ** | Yeni/değişen dosyalarda dış raporlama veya Discord IPC yok; yalnızca yerel `tracing::info!/warn!`. |
| **`build.rs` / `Cargo.toml`** (`git=`, özel registry) | **TEMİZ** | `Cargo.toml` ve `build.rs` dosyaları `0a435665` ile birebir aynı. `Cargo.toml:14-45` yalnız crates.io + yerel `crates/*`; `crates/victory-gpu/build.rs:18-45` yalnız yerel blake3 hash; `crates/victory-shader/build.rs:5-60` yalnız yerel `src` okur; `tests/fixtures/hle-probe/build.rs:1-4` yalnız linker bayrağı. |
| **Gömülü ikili / base64** | **TEMİZ** | Diff'te base64/ikili yok. `crates/victory-loader/src/hooks.rs:537-553,588-655` belgelenmiş 3–11 baytlık x86-64 yama imzaları; `crates/victory-gpu/src/compose.rs:25-42` düz metin HLSL. |
| **Repo dışına dosya yazma** | **TEMİZ** | Tek yazım `scripts/play-nvidia.ps1:44-47` (`Copy-Item $victory $sm -Force`): repo içi `target\<Build>\victory.exe` → aynı klasörde `victory_sm.exe`. |
| **Process çalıştırma / bellek yaması** | **TEMİZ** | Rust diff'inde `Command::new` yok; `scripts/play-nvidia.ps1:57` yalnız yerel `& $victory @runArgs` başlatır. `crates/victory-loader/src/hooks.rs:718-725` `VirtualProtect(PAGE_EXECUTE_READWRITE)` ile yalnız `BUILD_1_0_2_1` hash + beklenen bayt doğrulamalı opt-in bellek yaması uygular (emülatör loader'da normal). |

---

## 5. 8 GB VRAM + 16 GB RAM İçin İlk 5 Alınabilir Aday
1. **`VICTORY_CARD_BUFFERS_MIN_MB=1` + `VICTORY_CARD_BUFFERS_MB=512..1024` (Yalnız ENV):** Copy-back karşılaştırmasını GPU'ya alıp en büyük FPS sıçramasını verir (yazar 27→10 ms) | Risk: `1536` MB 8 GB VRAM/16 GB RAM'de taşabilir (`TAHMİN`), 512–1024 ile başlanmalı | Bizde karşılığı: **VAR** (`0a435665`'te hazır, hemen test edilebilir).
2. **`a9632bc8` — `VICTORY_DYNAMIC_SLOTS=524288` (`descriptor/host.rs`, +14/-1):** NVIDIA 1M descriptor tavanında `THIN=1` ile atlanan binlerce draw'ı ve ekran atlamasını keser | Risk: Çok düşük (14 satır, opt-in, `null/2` sınırı) | Bizde karşılığı: **YOK**.
3. **`24a479d3` + `shader_cache.rs` — `VICTORY_BVH_MISS=1` (`intersect.rs`, +22/-2):** Eski BVH gezmesini kapatıp ~10 ms/kare GPU süresi ve hang koruması sağlar | Risk: Düşük (opt-in, ayrı shader cache; loş odalar biraz kararır) | Bizde karşılığı: **YOK**.
4. **`afd84de1` — `VICTORY_RENDER_SCALE=0.5` + `VICTORY_OWN_UPSCALER=1` + `VICTORY_NO_WATCHDOG` (`hooks.rs`, +277/-0):** Series S'te 960×540 render ile GPU ve render-target VRAM yükünü ciddi düşürür, FSR çökmesini aşar | Risk: Orta (yalnız 1.0.2.1 exe hash'ine özel bellek yaması, görüntü yumuşar) | Bizde karşılığı: **YOK**.
5. **`7997a203` — `VICTORY_COMPOSE=1` (`compose.rs`+`queue.rs`+`host.rs`, +362/-5):** Series S 4K arayüz düzlemini bindirip siyah menü/altyazı/yükleme ekranlarını düzeltir (~8 MB VRAM) | Risk: Düşük (opt-in, yalnız 8-bit RGBA, `fxceless#1189` portu) | Bizde karşılığı: **YOK**.

**GÜVENLİ — Kesinlikle denemeye değer** (birebir bizim `0a435665` tabanımızın üstünde yalnız 5 temiz, küçük, opt-in commit; önce sıfır kodla `CARD_BUFFERS_MIN_MB=1` denenip ardından 5 commit doğrudan cherry-pick edilebilir; `soldsawn`'dan `lto = "thin"` + `DROP_MIPS=2` + Wave32 CBV ile birleştirildiğinde 8 GB / 16 GB için çok güçlü aday).
