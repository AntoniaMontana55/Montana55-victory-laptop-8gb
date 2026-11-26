# `soldsawn/victory-nvidia-8gb` İnceleme Raporu

- **İnceleme tarihi:** 2026-11-25
- **Hedef repo:** `soldsawn/victory-nvidia-8gb` (`https://github.com/soldsawn/victory-nvidia-8gb`)
- **Durum:** Bulundu (public fork, `parent: fxceless/victory-fork`, 14 star, 2 fork, GPL-3.0)
- **HEAD:** `a14d8c88c12dfb00e0c0361b99d3d44addc620de` (`main`, son push: `2026-11-25T14:15:37Z`, oluşturulma: `2026-11-25T13:52:45Z`)
- **Yerel inceleme klonu:** `C:\CLLL\.calisma\_inceleme\victory-nvidia-8gb` (salt okunur, derleme/çalıştırma yapılmadı)
- **Güvenlik ön hükmü:** **TEMİZ (zararlı sinyal yok)** — hazır `.exe`/`.zip` release yok, `Cargo.toml`/`Cargo.lock`/`build.rs` değişmemiş, ağ kodu yok.

---

## 1. Keşif ve Kaynak Taraması

- **Neden standart GitHub repo aramasında çıkmadı?** Repo bir **fork** (`fork: true`, parent: `fxceless/victory-fork`) olduğu için GitHub Search API'nin varsayılan `search/repositories` sorgusu bunu gizliyor. GitHub kod aramasında (`search/code?q=victory-nvidia-8gb`) `MaDeInCCCP2/victory-fork :: docs/soldsawn-fork-analysis.md` üzerinden yakalandı ve doğrudan `gh api repos/soldsawn/victory-nvidia-8gb` ile doğrulandı.
- **X (`@h4nkick`) ve Reddit:** Arama motoru indeksinde `@h4nkick`'in 25.11 tarihli gönderisi doğrudan yakalanamadı; ancak gönderideki tarif (*"RTX 4060", "4 to 8 fps", "faces, walls, floors & night lighting", "victory-nvidia-8gb"*) `soldsawn/victory-nvidia-8gb` README'siyle birebir örtüşüyor.
- **PortWatch (`https://portwatch-vi.xyz/data/tracker.json`):** `soldsawn/victory-nvidia-8gb` henüz `efforts`, `candidates`, `flagged` veya `ignored` listelerinde yer almıyor. (Not: Benzer 8 GB NVIDIA fork'ları olan `Itsbatt/victory-unified` `candidates` içinde; `edsuuu/VI-BR-PORT` ve `Joaco9020/mangrove` ise `ignored` içinde kayıtlı.)
- **Hesap profili:** `soldsawn` hesabı `2024-12-05` tarihinde açılmış, 7 public reposu var (ör. `ExcaliburReimagined`). Yeni açılmış tek-kullanımlık/tuzak hesap profili göstermiyor.

---

## 2. Köken (`merge-base`) ve Commit Özeti

| Karşılaştırılan Hedef | Ortak Ata (`merge-base`) | `soldsawn` Önde | Karşı Taraf Önde |
| :--- | :--- | :---: | :---: |
| `fxceless/victory-fork` (`main` = `465b94f`) | `0b0074e` (*Copies in place measured and fixed...*) | **54** (37 non-merge, 14 first-parent) | 2 |
| `Marvisak/VIctory` (`main` = `67e8ae1`) | `ffe310e` (*Shaders: the console's waterfall loop...*) | 77 | 401 |
| `ZSlayerHQ/victory-world-runtime` (`zslayer/next`) | `310c6eb` (*Packed textures stay in the upload queue...*) | 60 | 22 |
| `ZSlayerHQ/victory-world-runtime` (`zslayer/world`) | `c96b4f2` (*The depth bounds test works (#82)...*) | 61 | 0 |
| **Yerel tabanımız (`02ea29c`, `vw-runtime`)** | `310c6eb` (*Packed textures stay in the upload queue...*) | **60** | **22** |

- **Önemli yapısal not:** `soldsawn/victory-nvidia-8gb`, sıfırdan yazılmış bir fork değil; `fxceless/victory-fork@0b0074e` üzerine 15 topluluk PR'ını birleştiren (README'de AI-destekli entegrasyon/test edildiği açıkça belirtilmiş) bir **entegrasyon fork'udur**.
- **`0b0074e..HEAD` toplam farkı:** 39 dosya, `+7569 / -1588` satır.
  - `crates/victory-shader/`: Wave64 paired compute lowering + >32 KiB mesh-pixel linking (PR #854)
  - `crates/victory-gpu/`: 8 GB VRAM / mip drop, CBV 64 KiB clamp, unmapped texture release, WC compare skip, view parking, descriptor/pipeline/queue2 kararlılık düzeltmeleri
  - `crates/victory-hle/src/modules/xmem.rs`: >32 GB RAM Windows 11 için low-4GiB guard (VIctory #1009)
  - `scripts/play-nvidia-8gb.ps1` ve `README.md`

---

## 3. README İddiaları ve Kanıt Değerlendirmesi

- **README iddiası:** Windows 11, **NVIDIA RTX 4060 Laptop GPU (8 GB VRAM, Wave32)**, 48 GB RAM, Series X profilinde ilk odada **~6 FPS** (`172.7 ms/frame` @ 5,181 draw; `175.2 ms/frame` @ 6,271 draw; `~7.2 / 8.0 GB VRAM`, `%38 GPU`), bir ön-ısınma (shader cache) çalıştırması sonrasında tam duvarlar, kapılar, zemin ve doğal dolaylı ışık.
- **Kanıt durumu:**
  - Repoda ekran görüntüsü, video, ham `framelog` çıktısı veya bağımsız doğrulama kanıtı **bulunmuyor**; sayılar yalnızca README beyanına dayanıyor.
  - **Ölçüm temiz/kayıpsız değil:** Launcher betiği (`scripts/play-nvidia-8gb.ps1`) hem `VICTORY_TEXTURE_DROP_MIPS=2` (2D dokuların en büyük 2 mip seviyesini atarak görsel kalite/veri düşürür) hem de `VICTORY_SKIP_WC_COMPARE=1` (write-combining belleğe shader yazımlarının copy-back karşılaştırmasını atlayan deneysel bayrak) kullanıyor. Bu nedenle "~6 FPS" değeri, tam doğruluklu bir referans ölçümle doğrudan eşdeğer değildir.

---

## 4. 8 GB NVIDIA İçin Ne Değiştiriyor?

### A. Kod içinde varsayılanı açılan (`default-flip`) `VICTORY_*` bayrakları
`0b0074e..HEAD` farkında aşağıdaki bayrakların kod içi varsayılanı açık hale getirilmiş:
- `VICTORY_INDIRECT_WINDOW` (`52d5f3f`): varsayılan açık
- `VICTORY_QUEUE2` (`ce304e5`): varsayılan `1` (down lane açık, `0` kapatır)
- `VICTORY_TEXTURE_NO_WAIT` (`1b77aed`): varsayılan açık
- `VICTORY_WIDE_TEXTURES` / çok iş parçacıklı host yenileme (`5b7c878`): varsayılan açık
- `VICTORY_CARD_ROOM_FOR_BUFFERS`: varsayılan açık (`0` kapatır)
- `VICTORY_VIEW_SET`: varsayılan açık (`VICTORY_PARK_EPOCH=32`)
- `VICTORY_WAVE_PAIRS` ve `VICTORY_MESH_LINK` (PR #854): varsayılan açık (`0` kapatır)

### B. `scripts/play-nvidia-8gb.ps1` başlatıcı ayarları
Açıkça set edilenler:
```text
VICTORY_ASYNC_SUBMIT=1
VICTORY_QUEUE2=1
VICTORY_INDIRECT_WINDOW=1
VICTORY_TEXTURE_NO_WAIT=1
VICTORY_COPY_PLAIN=1
VICTORY_SKIP_WC_COMPARE=1
VICTORY_WIDE_TEXTURES=1
VICTORY_TEXTURE_DROP_MIPS=2   (-DropMips parametresi, varsayılan 2)
```
Açıkça temizlenenler (`Remove-Item Env:...`):
`VICTORY_SKIP_PIPELINES`, `VICTORY_BAKE_TEXTURES_MS`, `VICTORY_CARD_FETCH_INFLIGHT`, `VICTORY_LEVELS_ON_DEMAND`, `VICTORY_REFRESH_ONCE`, `VICTORY_AMBIENT`.

### C. Doku / VRAM bütçesi yaklaşımı
1. **`VICTORY_TEXTURE_DROP_MIPS=2`** (PR #610, `b4ff304`–`68e6f6a`): 2D dokuların en büyük 2 mip seviyesini host'ta hiç ayırmayarak VRAM tüketimini ciddi biçimde kısıyor. README'ye göre `VICTORY_LEVELS_ON_DEMAND=1` kapalı bırakılıp bunun yerine `DROP_MIPS=2` seçildiğinde 8 GB taşması önleniyor ve diffuse + indirect lighting probe'ları ilk açılışta yükleniyor.
2. **Unmapped placed texture serbest bırakma** (PR #432, `4ae2d11` + `8f73867`): Belleği unmap edilen placed texture'ların host kopyaları kare başına tek geçişte derhal serbest bırakılıyor (kayıpsız VRAM kazanımı).

---

## 5. Hızlı Güvenlik Ön Taraması

| Kontrol | Bulgular | Durum |
| :--- | :--- | :---: |
| **GitHub Releases / Hazır exe-zip** | `releases: []`, `tags: []` — hazır derlenmiş ikili/arşiv **yok**. | Temiz |
| **Repo içi binary / script dosyaları** | `.exe`, `.dll`, `.zip`, `.bat`, `.py` **yok**. Tek script `scripts/play-nvidia-8gb.ps1` (26 satır: env var ayarlayıp yerel `target\release\victory.exe run --execute $Exe` çalıştırıyor). | Temiz |
| **`Cargo.toml` / `Cargo.lock` / `build.rs`** | `0b0074e..HEAD` arasında **sıfır değişiklik**. Yeni crate veya derleme betiği eklenmemiş. | Temiz |
| **Ağ kodu (`reqwest`, `TcpStream`, `http`, vb.)** | Kod diff'inde ağ çağrısı **yok** (yalnızca `README.md` içinde GitHub bağlantıları). | Temiz |
| **Süreç çalıştırma (`Command::new`)** | Tek eşleşme: `crates/victory-gpu/src/queue2.rs:848` (`0fdeee8`) — yalnızca `#[test]` içindeki `device_removal_does_not_complete_a_copy_job` birim testinin kendi test binary'sini izole süreçte çalıştırması. | Temiz |

**Ön hüküm:** Kötü amaçlı yazılım / tuzak sinyali yok. X gönderisindeki *"verified on GitHub"* ifadesi pazarlama/abartı dili olabilir; ancak reponun kendisi saf kaynak kodu (Rust), GPL-3.0 ve temiz bir PR entegrasyon fork'udur.

---

## 6. Bizim Taban `02ea29c`'de Olmayan ve 8 GB İçin Alınmaya Aday Commit'ler

> **Ön koşul notu:** `02ea29c` ile `soldsawn`'ın tabanı `0b0074e` arasında zaten **6 upstream `fxceless/victory-fork` commit'i** (`8dd30ab`, `a26859f`, `9376fd6`, `f85101b`, `a253706`, `0b0074e`) bulunuyor. Bu 6 commit'in `soldsawn`'dan değil doğrudan `fxceless/victory-fork` üzerinden değerlendirilmesi gerekir.

### Öncelik 1 — Temiz / Kayıpsız 8 GB NVIDIA Adayları (Doğrudan Alınabilir)
1. **`09b95ca`** *(PR #799)* — NVIDIA Wave32 CBV 64 KiB clamp (`cbv_size`) ve shader translator host düzeltmeleri. NVIDIA'da geçersiz >64 KiB CBV yüzünden oluşan `0x887A0001` device-removed çökmesini engeller.
2. **`4ae2d11` + `8f73867`** *(PR #432)* — Unmap edilen placed texture host kopyalarını kare başına tek yürüyüşle serbest bırakma. Kayıpsız doğrudan 8 GB VRAM rahatlaması.
3. **`26600c6` + `0751197`** *(PR #854)* — 32-lane (NVIDIA Wave32) GPU'larda paired Wave64 compute lowering ve >32 KiB mesh-pixel shader linking (230 ek shader derlenebilir hale geliyor).
4. **`b212918` + `31cb370`** *(PR #936)* — `Shadow` drop edilmeden önce `SNAPS` flush (use-after-free düzeltmesi).
5. **`8654c67` + `0fdeee8`** *(PR #964)* — Copy queue fence'lerinden device-loss durumunun temiz yayılımı.
6. **`da4e362`** *(VIctory #1009)* — >32 GB RAM Windows 11 sistemlerde `xmem::low_guards`.

### Öncelik 2 — Kararlılık ve CPU/GPU Verimlilik Düzeltmeleri (Kayıpsız)
- **`e5028d8` + `a0e6f89`** *(PR #922)* — Integer render target'larda blend'i kapatma (host pipeline derleme hatası düzeltmesi).
- **`34a7ebd` + `bfbc2af`** *(PR #932)* — Dinamik descriptor aramasında arama imlecini sararak boş blok bulma.
- **`20908e6` + `b1c3be1` + `ed1274a`** *(PR #896)* — Streaming patlamalarında doku yenileme bildirimlerinin kaybolmasını önleme.
- **`fb5c67a`** *(PR #864)* — Indirect window kayıtlarını lane sırasında clamp etme.
- **`a544813`** *(PR #860)* — `fill_ahead` latch yanlış tetiklenmelerini durdurma.
- **`e277b98`** *(PR #812)* — Unmapped view bloklarını park ederek `VirtualQuery` yükünü azaltma.
- **`19546dd`** *(PR #768)* — `VICTORY_WIDE_TEXTURES=1` paralel doku kontrolleri.

### Öncelik 3 — Opsiyonel / Kayıplı (Varsayılan Kapalı Tutulmalı)
- **`b4ff304`, `fc4b3dd`, `7d0e398`, `bd2ccd8`, `68e6f6a`** *(PR #610, `VICTORY_TEXTURE_DROP_MIPS=N`)* — 8 GB kartlarda VRAM taşmasını engellemek için pratik, ancak üst mip seviyelerini atarak doku detayından ödün verir. Alınırsa varsayılan `0` kalmalı.

### Alınmaması Önerilenler
- **`4e34eb0`** *(PR #868, `VICTORY_SKIP_WC_COMPARE=1`)* — Write-combining belleğine shader yazımlarının copy-back karşılaştırmasını atlayan deneysel kısayol; doğruluk riski taşır.
- **Default-flip commit'leri (`52d5f3f`, `5b7c878`, `1b77aed`, `ce304e5`)** — Kendi donanım/test ölçümlerimiz olmadan varsayılan davranış değiştirilmemeli.
