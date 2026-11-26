# Belobug/VIctory-Belobug-AMD İnceleme Raporu

- **Hedef:** `https://github.com/Belobug/VIctory-Belobug-AMD` @ `01aadd19` (`2026-11-25`, 1486 yıldız, 257 fork, `fork: false`, 713 toplam commit)
- **Yerel klon:** `C:\CLLL\.calisma\_inceleme\belobug`
- **Bizim taban:** `ZSlayerHQ/victory-world-runtime` `next` @ `02ea29c` (`C:\CLLL\.calisma\vw-runtime`)
- **Donanımımız:** RTX 3070 Ti Laptop 8 GB VRAM + 16 GB RAM (NVIDIA, Wave32)

---

## 1. Köken ve Tarihçe

- **Ortak ata (`git merge-base`):** `8e79c80e4b71e252bcd271e0587aa662c4770008` (`CI: clippy and tests on pull requests and main`, Marvis, 2026-11-24 09:55). `fxceless/victory-fork` (`vfork/main`), `ZSlayerHQ/victory-world-runtime` (`zslayer/next`) ve `JoshuuDev/victory-amd` (`joshuu/main`) ile **birebir aynı** ortak ata.
- **Türeme şekli:** [README.md:3-6](file:///C:/CLLL/.calisma/_inceleme/belobug/README.md#L3-L6) orijinal `Marvisak/VIctory` @ `8e79c80` üzerine inşa edildiğini söylüyor; GitHub'da fork bayrağı yok (`fork: false`) ama tüm orijinal git geçmişi (`f4504f6` Initial commit, Marvis) korunmuş.
- **Yazar ve katkı dağılımı (`8e79c80..01aadd19` = 512 commit, 97 dosya, +45.067 / -2.945):**
  - `Claude`: 189 commit (başta upstream PR #35..#414 birleştirmeleri ve shader/GPU düzeltmeleri)
  - `beloooo` (`Belobug`, GitHub hesabı 2024-05-10, 1 public repo, 28 takipçi): 119 commit (çoğu `Co-Authored-By: Claude Fable 5.2` / README: Claude Code + Gemini Antigravity ile geliştirilmiş)
  - Geri kalan ~204 commit upstream topluluk PR yazarları (`mich0206`, `r4.`, `Manuel Pérez`, `TG0000`, `lordon1a` vb.) ile Belobug reposunda birleştirilen 2 harici PR: **PR #18** (`PikPak`, `a0a4591`) ve **PR #45** (`pgm1207`/`Arvi23`, `8629211`).
- **Odak alanları:** AMD RX 9070 XT'de dünya görüntüsü düzeltmeleri (ambient ışık, quad read, integer RT blend, depth twins, driver compiler crash hayatta kalma), indirect clamp hoisting, multi-view page guard, paralel sparse copy.

---

## 2. README İddiaları vs. Kanıt

- **Test makinesi ([README.md:14-16](file:///C:/CLLL/.calisma/_inceleme/belobug/README.md#L14-L16)):** Tek makine — Windows 11, **AMD Radeon RX 9070 XT 16 GB**, Ryzen 7 9800X3D, 64 GB RAM, `series-x` profili (`VICTORY_PRESENT_TARGET=1x1:1`, `VICTORY_HAIR=1`, `diag` profili). NVIDIA veya Intel'de hiç çalıştırılmamış.
- **İddia edilen durum ([README.md:18-32](file:///C:/CLLL/.calisma/_inceleme/belobug/README.md#L18-L32), [README.md:108-138](file:///C:/CLLL/.calisma/_inceleme/belobug/README.md#L108-L138)):**
  - Açılış/menüler: **30 fps** (upstream ile aynı); kayıtlar yükleniyor/yazılıyor.
  - Dünya: ışıklandırılmış, dokular, insanlar, araçlar, bitki örtüsü, su, gökyüzü çiziliyor; **3–5 fps** (ağır sahnede sabit **4.4–4.9 fps**, sürüşte **~3 fps**).
  - Uzak ağaçlar çiziliyor; hareketli araçlar bütün (multi-view page guard sonrası kopma/uzama gitmiş).
  - **Oyun içi telefon BOZUK:** Telefon çıkarılınca dünya tamamen siyah (ışıksız), telefon boş çerçeve.
  - Bellek: ~22 GB RAM, 15 dk sürüşte **~33 GB commit**, **16 GB VRAM'i aşıyor**. Oturumda 1 kez Windows display driver timeout (TDR) yaşanmış.
  - Dünyada kare başına **~380 draw** eksik pipeline yüzünden düşüyor (çevrilemeyen shader'lar + AMD sürücüsünün reddettiği 6 pipeline). `VICTORY_RAYS=1` ~25 sn sonra `device removed` veriyor.
  - Performans rakamları (commit/README):
    - Hoisted clamps (`ccab973`, [README.md:60-63](file:///C:/CLLL/.calisma/_inceleme/belobug/README.md#L60-L63)): 9.000–9.300 draw sahnede GPU süresi **225 → 72–80 ms**, **2.6 → 3.9 fps**.
    - PR #18 paralel sparse copy (`a0a4591`, [README.md:64-66](file:///C:/CLLL/.calisma/_inceleme/belobug/README.md#L64-L66)): copy-back **43 → 33 ms** (Belobug RX 9070 XT) / **48 → 32 ms** (PikPak RX 9060 XT + 9700X); **FPS değişmedi**.
    - `VICTORY_ASYNC_SUBMIT=1` ([README.md:67-70](file:///C:/CLLL/.calisma/_inceleme/belobug/README.md#L67-L70)): kararlı ama **daha hızlı değil** (CPU sınırlı).
- **İddia / Kanıt ayrımı:**
  - Repoda **hiçbir ekran görüntüsü, video veya ham log dosyası yok** (`git ls-files` yalnızca `.md`/`.txt`). Yukarıdaki tüm FPS/ms ve görsel durumlar yazarın README/commit beyanıdır (tutarlı ve abartısız/dürüst, ancak bağımsız logla doğrulanmamış).
  - Tek harici donanım kanıtı: Issue **#137** (RTX 5070 Laptop 8 GB, Win11, `01aadd1` üzerinde `cargo test --workspace`) — oyun koşusu değil, birim/GPU test raporu; **30 test başarısız**.

---

## 3. AMD'ye Özel mi? NVIDIA (RTX 3070 Ti 8 GB) İçin Anlamı

- **Doğrudan NVIDIA'da sorun çıkaran / eksik kalan noktalar:**
  1. **Wave64 compute shader'ları düşüyor:** [host.rs:1045-1049](file:///C:/CLLL/.calisma/_inceleme/belobug/crates/victory-gpu/src/host.rs#L1045-L1049) 64-lane wave olmayan hostlarda (tüm NVIDIA kartlar Wave32) uyarı basıp lane-okuyan console Wave64 compute programlarını çevirmeden düşürüyor. (PR #128 hâlâ açık/birleştirilmemiş; bizim ZSlayer tabanımızda `wave64-on-wave32` yolları çok daha ileride.)
  2. **NVIDIA test suite kırık (Issue #137, RTX 5070 Laptop 8 GB @ `01aadd1`):** `victory-shader`'da **23 GPU emit testi**, `victory-hle::xmem`'de **5 test**, `victory-gpu`'da **1 test** (`dynamic_slots_are_held_until_released`) kalıyor.
  3. **Descriptor heap tavanı:** [descriptor/host.rs:511-514](file:///C:/CLLL/.calisma/_inceleme/belobug/crates/victory-gpu/src/descriptor/host.rs#L511-L514) `DYNAMIC = 1 << 20` (1.048.576) yapılmış ama NVIDIA'nın D3D12 shader-visible sınırı `PORTABLE_VIEWS = 1_000_000` olduğu için NVIDIA'da `DYNAMIC_CAPPED = 1 << 16` (65.536) devrede kalıyor — bu artıştan NVIDIA yararlanamıyor.
  4. **AMD'ye özel workaround'lar:** `03629f3` (tüm pixel quad okumalarını helper lane dahil quad read'e zorlama — AMD'de wave read 0 döndürdüğü için), `1503444` (AMD sürücü derleyicisi çökmesin diye buffer eleman sayısını host tablosundan okuma), `pipeline/library.rs:144-242` (AMD compiler crash/hang SEH yakalama). Bunlar NVIDIA'da çökme yapmaz ama ek yük veya gereksiz fark oluşturur.
- **Adı "Wave64" olup NVIDIA'da çalışanlar:** [card.rs:30-95](file:///C:/CLLL/.calisma/_inceleme/belobug/crates/victory-gpu/src/card.rs#L30-L95) (`CARD_COMPARE_HLSL`) standart `[numthreads(64, 1, 1)]` `cs_5_1` + `groupshared` kullanır (SM6.0 wave intrinsics yok), NVIDIA'da çalışır; ancak varsayılan bütçesi `VICTORY_CARD_BUFFERS_MB=2048` ([card.rs:129](file:///C:/CLLL/.calisma/_inceleme/belobug/crates/victory-gpu/src/card.rs#L129)) 8 GB VRAM'imiz için fazla yüksek (bizim tabanda varsayılan 1024 MB).
- **Özet:** Bu repo AMD RDNA4 16 GB VRAM / 64 GB RAM için ayarlanmış. Bizim 8 GB VRAM + 16 GB RAM + NVIDIA Wave32 makinemizde doğrudan çalıştırmak hem daha fazla shader düşüşü hem de bellek taşması riski taşır.

---

## 4. Performans ve Kararlılık Değişiklikleri vs. Bizim Taban (`02ea29c`)

| Özellik / Commit | Ne Yapıyor | Varsayılan | `02ea29c`'de Var mı? | Bizim Darboğaz (CPU/kopya/bekleme/8 GB VRAM) İçin Etkisi |
|---|---|---|---|---|
| **Integer RT blend kapatma** (`8629211`, PR #45) | Integer formatlı render target'larda (`R8_UINT` vb.) `BlendEnable` kapatır; D3D12 `E_INVALIDARG` reddetmesini önler | **Açık** | **YOK** (`pipeline/host.rs:637` blend'i aynen geçiriyor) | **Yüksek (doğruluk/kararlılık):** İlk oda duvarları gibi eksik çizilen mesh pipeline'ını kurtarır; NVIDIA'da da geçerli. |
| **Dinamik slot araması önbelleği** (`788ea46`) | Başarısız slot aramasını bir sonraki release'e kadar tekrarlamaz; kare başı max 8 başarısız tarama | **Açık** | **YOK** | **Çok yüksek (CPU):** NVIDIA'da `DYNAMIC_CAPPED=65_536` olduğu için slot tükenmesi daha olası; 7–11 sn'lik kare donmasını keser. |
| **Indirect clamp hoisting** (`ccab973`, `918ab03`, `76387c2`) | Indirect draw dizisinin clamp dispatch'lerini ayrı bir ön listeye alır (`VICTORY_HOIST_CLAMPS=7`) | **Açık (`7`)** | **Kısmi** (`indirect.rs:291` `bound_ahead` var, ayrı listeye hoist/mode 7 yok) | **Yüksek (GPU/barrier):** 9.300 draw sahnesinde GPU süresini 225→72–80 ms düşürdüğü raporlanıyor; NVIDIA'da barrier yükünü ciddi azaltır. |
| **Paralel sparse copy** (`a0a4591`, PR #18) | Çok çekirdekli tile snapshot/copy-back + aynı submission'daki tekrarlanan aralıkları atlama + tek kilitte tile bulma | **Açık** | **Kısmi** (`shadow.rs:935,2094`'te `thread::scope` var, tek kilit/skip farkı incelenmeli) | **Orta (CPU/kopya):** Kilit çekişmesini ve kopya süresini (43→33 ms) azaltır; `02ea29c` ile diff alıp yalnız eksik kısım alınabilir. |
| **Tile sweep** (`0a32e89`, `e11fb06`) | Artık geçersiz title belleğinin tile'larını bölge bazlı ve kilit dışında geri verir (`VICTORY_SWEEP_TILES=2`) | **Kapalı (`0`)** | **Kısmi** (`shadow.rs:1601` genel pencere sweep var) | **Orta-Yüksek (8 GB VRAM / 16 GB RAM):** Uzun koşuda commit/VRAM şişmesini durdurmaya yardımcı. |
| **Async submit** (`9b89ec2`, `bdecda3`, `cf57ca3`) | Title submission'ları GPU bitmeden döner (`VICTORY_ASYNC_SUBMIT=1`), sonraki wait'te tamamlanır | **Kapalı (`0`)** | **EVET, daha olgunu var** (`VICTORY_LATE_WAIT=1..3`, `host.rs:1783-1843`) | **Gereksiz:** ZSlayer `LATE_WAIT=3` + `LATE_WAIT_LISTS=256` aynı işi daha kapsamlı yapıyor. |
| **Multi-view page guard** (`ae90937`, `579a8d9`, `f2cc100`) | Aynı fiziksel sayfanın tüm sanal görünümlerini korur (`VICTORY_VIEW_GUARD=1`) | **Açık** | **EVET** (`watch.rs:559` `VICTORY_VIEW_GUARD`) | **Gereksiz:** `02ea29c`'de zaten mevcut (`579a8d9` kernel write 2 sn kuralı gerekirse ayrıca kıyaslanır). |
| **Ambient light** (`4f45ab8`, `4ffc476`, `cd9f78c`) | RT dolaylı ışık yerine sabit ortam ışığı + derinlik tabanlı karartma (`VICTORY_AMBIENT`, `_TINT`, `_SHADE`) | **Açık (`0.6`)** | **YOK** | **Görsel (performans değil):** Magenta yaprak/karanlık iç mekan sorununu geçici çözer; performans getirmez, isteğe bağlı. |

### "Parça Mağazası" Olarak Alınmaya En Değer 5 Commit
1. **`8629211`** (PR #45) — Integer render target'larda blend kapatma (`crates/victory-gpu/src/pipeline/host.rs`, +122/-2 satır): Temiz, bağımsız, D3D12 şartına uygun.
2. **`788ea46`** — Başarısız dinamik slot araması önbelleği + kare başı 8 yürüyüş sınırı (`crates/victory-gpu/src/descriptor/host.rs`, +26/-3 satır): Çok küçük, sıfır risk, NVIDIA 65K dinamik slot tavanında dev donma önleyici.
3. **`ccab973` + `76387c2`** — Indirect draw argument clamp'lerini ön listeye alma (`VICTORY_HOIST_CLAMPS=7`, ~405 satır): GPU tarafında en büyük kanıtlı kazanç (225→72–80 ms), tek değişkenle açılıp kapatılabilir.
4. **`a0a4591`** (PR #18) — `shadow.rs` içinde submission içi tekrarlanan aralıkları atlama ve tek kilitte tile bulma kısmı (bizdeki `thread::scope` ile birleştirilerek): CPU/kilit yükünü azaltır.
5. **`0a32e89` + `e11fb06`** — `VICTORY_SWEEP_TILES=2` bölge bazlı kilit dışı tile geri verme (~100 satır): 16 GB RAM / 8 GB VRAM sınırı için uzun koşu bellek güvenliği.

---

## 5. Güvenlik Denetimi (`8e79c80..01aadd19` tam farkı)

- **Bağımlılıklar (`Cargo.lock`, `Cargo.toml`):**
  - Yeni harici crate **YOK**. `Cargo.lock`'ta tek fark, dahili workspace paketi `victory-core`'un `victory-gpu`'ya eklenmesi.
  - `git=` / `path=` harici bağımlılık **YOK**.
  - `crates/victory-hle/Cargo.toml`: `windows-sys`'e DualSense HID için 3 Win32 feature (`DeviceAndDriverInstallation`, `HumanInterfaceDevice`, `System_IO`).
  - `crates/victory-loader/Cargo.toml`: `Wdk_System_SystemServices`, `Win32_System_SystemInformation`.
- **`build.rs` ve proc-macro:**
  - Yeni [apps/victory/build.rs:1-11](file:///C:/CLLL/.calisma/_inceleme/belobug/apps/victory/build.rs#L1-L11): Sadece MSVC için `cargo:rustc-link-arg-bins=/STACK:16777216` (16 MiB stack; büyük shader derlerken stack overflow önlemi). Yeni proc-macro yok.
- **Ağ kodu / telemetri / loopback kilidi:**
  - `crates/victory-hle/src/modules/net.rs`, `xsystem.rs`, `lifecycle.rs` **hiç değişmemiş** (`git diff 8e79c80..HEAD` boş).
  - `reqwest`, `ureq`, `TcpStream`, `UdpSocket`, `WinHttp`, `WSASocket`, telemetry, webhook, Discord/Mega/Drive linki **YOK**. Diff'teki tek URL'ler `README.md`'deki GitHub/Claude bağlantıları.
- **`Command::new` / süreç başlatma:**
  - `8e79c80..HEAD` farkında eklenen 3 `Command::new` çağrısının tamamı **test** kodunda:
    - [depth_bounds_test.rs:114](file:///C:/CLLL/.calisma/_inceleme/belobug/crates/victory-gpu/src/pipeline/depth_bounds_test.rs#L114) (`VICTORY_PC_DXC` ile test shader derleme)
    - [crash_log.rs:42](file:///C:/CLLL/.calisma/_inceleme/belobug/crates/victory-loader/tests/crash_log.rs#L42) (`current_exe()` alt süreç testi)
    - [opcode_audit.rs:239](file:///C:/CLLL/.calisma/_inceleme/belobug/crates/victory-shader/src/emit/opcode_audit.rs#L239) (`current_exe()` test işçisi)
- **Dosya sistemi yazmaları:** Tüm yeni `fs::write` / `OpenOptions` çağrıları yalnızca açıkça ortam değişkeniyle açılan teşhis/dump yollarına (`VICTORY_CAPTURE`, `VICTORY_PROFILE`, `VICTORY_SHADER_DUMP`, `VICTORY_EMIT_DUMP`) ve pipeline/shader cache dosyalarına yazıyor. Kullanıcı profili/tarayıcı/registry yazması **YOK**.
- **CI / İkili / Blob:** `.github/workflows` değişmedi. Diff'te yeni ikili dosya (`.exe`/`.dll`/`.so`/`.zip`/`.ps1`/`.bat`) veya base64/şifreli blob **YOK**.
- **Hüküm:** **GÜVENLİ** — `01aadd19` (ve `8e79c80..01aadd19` aralığındaki tüm commit'ler) için geçerlidir. (Not: repo içindeki açık/birleştirilmemiş PR'lar kapsam dışıdır, alınacak olursa ayrıca incelenmelidir.)

---

## 6. Bizim İçin Öneri

Taban değiştirmeye **kesinlikle değmez**; `ZSlayerHQ/victory-world-runtime` (`02ea29c`) tabanında kalmalıyız. Belobug reposu tek bir AMD RX 9070 XT 16 GB / 64 GB RAM makinesi için ayarlanmış, NVIDIA'da hiç çalıştırılmamış, 32-lane GPU'larda Wave64 compute shader'larını düşürüyor ve RTX 5070 Laptop'ta 30 test birden kırılıyor (Issue #137). Üstelik bizim tabanda `VICTORY_LATE_WAIT=3`, `VICTORY_VIEW_GUARD`, `VICTORY_CARD_BUFFERS` ve harness ölçüm altyapısı zaten daha olgun. Buna karşın kod tamamen **GÜVENLİ** ve çok iyi bir "parça mağazası": ilk faz ölçümlerinden sonra önce `8629211` (integer RT blend) ile `788ea46` (dinamik slot araması önbelleği), ardından GPU darboğazı görülürse `ccab973+76387c2` (hoisted clamps) seçilerek ZSlayer tabanımıza port edilmelidir.

---
## Claude doğrulaması (25.11 19:45)
- DÜZELTME: `VICTORY_ASYNC_SUBMIT` bizim taban 02ea29c'de YOK (`grep -rn ASYNC_SUBMIT crates` boş). `VICTORY_VIEW_GUARD` var (vw-runtime crates/victory-gpu/src/watch.rs:522-559). HOIST_CLAMPS ve SWEEP_TILES da bizde yok.
- DOĞRU, KRİTİK: Belobug host.rs:1032-1047 — 64-lane wave olmayan kartta (NVIDIA = 32) "diğer lane'leri okuyan wave64 compute programları çevrilmez, dispatch'leri düşürülür". Bizim taban host.rs:626-633 bunları 32-lane yarılara çevirerek ÇALIŞTIRIYOR (`set_host_max_wave`). → Belobug NVIDIA'da görüntü/simülasyon kaybı yaşar; taban değiştirmemek için belirleyici neden.
- DOĞRU: 86292110 (PR #45 integer RT blend kapatma) bizde yok → ilk aday.
