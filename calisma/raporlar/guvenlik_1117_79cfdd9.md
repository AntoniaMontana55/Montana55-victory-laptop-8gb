# Güvenlik ve Taşınabilirlik İncelemesi: `fxceless#1117` ve `fxceless@79cfdd9`

- **Tarih:** 26.11.2026
- **Kapsam:** Salt okunur güvenlik, doğruluk ve taşınabilirlik incelemesi (`C:\CLLL\.calisma\vw-pr10` tabanına göre; ZSlayer `02ea29c` + PR #10 + PR #32 eşdeğeri + `#1014`/`#1066` + `CBV_DIRECT`).
- **Hedef donanım:** RTX 3070 Ti Laptop (**8 GB VRAM**) + **16 GB RAM**, Windows, `--console series-s`.

---

## 1. `fxceless/victory-fork` PR #1117 (OPEN — `RealAzemed`, commit'ler `3c85889`, `ed0ac8c`)

Değişen dosyalar (5 dosya, `+383/-29`): `Cargo.toml`, `crates/victory-gpu/src/descriptor/host.rs`, `crates/victory-gpu/src/draw.rs`, `crates/victory-gpu/src/inplace.rs`, `crates/victory-gpu/src/shadow.rs`.

### 1.1 Parça 1: Zero-Copy Section-Heap Windows (`VICTORY_INPLACE_SECTIONS=1`)

#### (a) Güvenlik — **GÜVENLİ**
- **Derleme/bağımlılık:** `Cargo.lock` ve `build.rs` değişmiyor; yeni crate/bağımlılık yok.
- **Ağ / dosya / süreç / kayıt defteri / gizli blob:** Hiçbiri yok.
- **Yeni `unsafe` / FFI noktaları (PR diff satırları):**
  - `crates/victory-gpu/src/inplace.rs` (`section_aliasing_verified`, diff L205): `unsafe { section(&mut p, &mut rig) }` — mevcut `inplace.rs:775` probe fonksiyonunu kendi geçici `SEC_COMMIT` section + placeholder'ı üzerinde tek sefer çalıştırıp temizler (`CreateFileMappingW`, `VirtualAlloc2`, `VirtualFree`, `MapViewOfFile3`, `OpenExistingHeapFromFileMapping`, `CreatePlacedResource`, `VirtualProtect`, `UnmapViewOfFile2`, `MapViewOfFile`, `UnmapViewOfFile`, `CloseHandle`).
  - `crates/victory-gpu/src/inplace.rs` (`place_section_window`, diff L351, L380, L396): Mevcut `open(&host.device, From::Section(section))` (`OpenExistingHeapFromFileMapping`, `vw-pr10` `inplace.rs:233`), `place()` (`CreatePlacedResource`, `inplace.rs:259`) ve `unsafe { buffer.GetGPUVirtualAddress() }` çağrıları.
  - `crates/victory-gpu/src/shadow.rs`: Yeni `unsafe` yok.

#### (b) Doğruluk Riski — **YÜKSEK (Windows için henüz hazır değil)**
1. **Kritik Windows farkı (`SEC_RESERVE` vs `SEC_COMMIT`):** Hem `fxceless` `main` hem `vw-pr10` (`crates/victory-hle/src/modules/xmem.rs:422`, `:442`) Windows'ta fiziksel sayfa section'ını **`SEC_RESERVE`** ile açıp `title_memory_section(handle, size, on_wine())` çağrısında **`committed = false`** geçiriyor (`SEC_COMMIT` yalnız Wine/Proton'da açık; Windows'ta 6,1 GiB tek seferde commit edilince yoğun sistemde `error 1450` görüldüğü için kapalı). Bu nedenle PR #1117 **native Windows'ta olduğu hâliyle hiç devreye girmez** (`find_section_run` diff L258: `if !committed { return None; }`).
2. **`VICTORY_LATE_WAIT` / `VICTORY_RELAY` çelişkisi (PR yorumu `@UwUCammie` + `ed0ac8c`):** `Shadow::placed` (`vw-pr10` `shadow.rs:1764`) `Shadow::gpu_writes`'ta erken döner; shader yazımları uçuş aralığı olarak takip edilmez. `ed0ac8c` ile `section_windows_on()` (diff L180–L187) `crate::host::late() || crate::host::relay()` aktifken özelliği reddeder. Bizim performans profilimizdeki late-wait/relay ile birlikte kullanılamaz; senkron moda dönmek GPU bekleme süresini geri getirir.
3. **Sürücü / vkd3d-proton sessiz bozulma bulguları (PR yorumları):**
   - `@zocram4cc` (RADV + vkd3d-proton): `OpenExistingHeapFromFileMapping` view'ı map edip hemen unmap ediyor; `AMDGPU_GEM_USERPTR_ANONONLY` reddedince vkd3d-proton sessizce ayrı private bellek ayırıp `S_OK` dönüyor (CPU↔GPU birbirini görmüyor). `ed0ac8c` bunun için `section_aliasing_verified()` byte round-trip self-testini ekledi.
   - `@Filip722` (RTX 5060 Laptop 8 GB + `#830` file-backed section): Pagefile self-testi geçse de dosya tabanlı section'da `OpenExistingHeapFromFileMapping` yine `S_OK` dönüp sessizce alias etmiyor; PR'da `file_backed` filtresi henüz birleştirilmedi.
   - `@Proudnoob4` (native Win10 + RTX 4080): `LATE_WAIT=3 RELAY=2` ile test ettiği için Parça 1 **native Windows'ta hiç ölçülmedi**.
4. **Bellek ömrü ve 16 GB RAM:** `section_outlives_views=true` doğrulaması, `title_memory_section` içinde `SECTION_HEAPS.remove(&section)` temizliği ve `title_memory_freeing` → `in_place_let_go` emekliliği mevcut; canlı GPU listeleri `Arc<Shadow>` üzerinden `ID3D12Heap` + `ID3D12Resource`'u tutar. Linux/Proton'da sparse memfd sayesinde swap tepe noktası **-5,1 GB**, `copies` **76–127 → 10 ms** ölçüldü; ancak Windows'ta `xmem.rs` `SEC_COMMIT`'e çevrilirse 6,1 GiB pagefile commit charge 16 GB RAM'de yeni OOM/1450 riski yaratır.

#### (c) Taşınabilirlik (`vw-pr10`)
- **Mevcut semboller (`vw-pr10`):**
  - `crates/victory-gpu/src/inplace.rs`: `title_memory_section` (:288), `title_memory_freeing` (:184), `VIEWS` (:279), `open(From::Section)` (:233), `place` (:254), `section()` probe (:775) — **hepsi var**.
  - `crates/victory-gpu/src/shadow.rs`: `Shadow::placed` (:1692), `Shadow::gpu_writes` (:1758), `in_place_let_go` (:2806), `holder` (:3037), `longest_tail` (:3077), `loose` (:3088) — **hepsi var**.
  - `crates/victory-gpu/src/host.rs`: `late()` (:1936), `relay()` (:2031) — **var**.
  - `find_section_run`: `vw-pr10`'da yok (PR ekliyor). Diff'teki `write_own` hunk'ı (diff L508) `vw-pr10`'da olmadığı için atlanır.
- **Önkoşul commit:** Kod derlemesi için yok; ancak Windows'ta fiilen çalışması için `xmem.rs` commit stratejisi ve/veya `LATE_WAIT`/`RELAY` uyumu gerekir.
- **Zorluk:** Kod aktarımı **kolay**, Windows'ta çalışır ve güvenli kılma **orta-zor**.

#### (d) Öneri (Parça 1)
- **BEKLE (merge edilsin + native Windows/NVIDIA doğrulaması ve `file_backed` yaması gelsin).** Şu anki hâliyle Windows'ta ölü kod (`SEC_RESERVE`) ve `LATE_WAIT`/`RELAY` ile uyumsuz.

---

### 1.2 PR #1117 Diğer 3 Parça (Ayrı Değerlendirme)

| Parça | Dosya:Satır (PR / `vw-pr10`) | Güvenlik | Doğruluk & Ölçüm | Taşınabilirlik | Öneri |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **2. Lock-free `HostHeap::mirror` + cached `gpu_handle`** | `descriptor/host.rs` (PR L26–L2017 / `vw-pr10` `:16`, `:791`, `:1283`, `:1621`, `:1655`) | **GÜVENLİ** (yeni `unsafe` yok; `gpu_handle` içindeki COM `unsafe` çağrısını azaltır) | `made_fast` Release/Acquire sırası doğru (`keep` ve descriptor yazımı sonrası yayın, `unmake` ve null geçişte 0'lanır). Win10/RTX 4080'de 5×330–360 sn 0 çökme (`@Proudnoob4`). Not: `#1146` ile benzer desen. | **Çok kolay** (~22 satır, önkoşul yok) | **TAŞI** |
| **3. Stack `[0u32; 64]` root constants** | `draw.rs` (PR L1184 / `vw-pr10` `:1040`) | **GÜVENLİ** (0 `unsafe`) | `dwords > 64` için `Vec` fallback var; ~6–10k/kare küçük heap alloc kaldırır. | **Çok kolay** (8 satır, önkoşul yok) | **TAŞI** |
| **4. `VIEWS: RwLock<Views>` + Thin LTO** | `inplace.rs` (PR L435 / `vw-pr10` `:279`), `Cargo.toml` (PR L74 / `vw-pr10` `:72`) | **GÜVENLİ** (yeni bağımlılık yok; `lto = "thin"` `[profile.diag]`'a da kalıtılır) | Okuma ağırlıklı `VIEWS` eşzamanlılığını açar; Thin LTO çapraz crate inline sağlar (derleme süresi/RAM biraz artar). Win10/RTX 4080'de doğrulandı. | **Çok kolay** (`write_own` hunk'ı hariç ~15 satır) | **TAŞI** |

---

## 2. `fxceless/victory-fork` Commit `79cfdd9` (`VICTORY_WINDOW_UPLOAD` + `CARD_WINDOWS_MB`)

Toplu commit (`+1858/-48`, 12 dosya). Bizi ilgilendiren dilim yalnız `crates/victory-gpu/src/host.rs` (`GPU_UPLOAD`, `D3D12_OPTIONS16`, `gpu_upload_buffer`) ve `crates/victory-gpu/src/shadow.rs` (`Shadow.uploaded`, `window_upload`, `window_upload_for`, `window_card_room`, `loose`) içindeki ~120 satırdır. (Aynı commit'teki `shot.rs`, `renderdoc.rs`, `dstorage.rs`, `pipeline/library.rs`, `VICTORY_WIDE_REFRESH=alt` kapsam dışıdır.)

### (a) Güvenlik — **GÜVENLİ**
- **Derleme/bağımlılık:** `Cargo.toml`, `Cargo.lock`, `build.rs` değişmiyor; yeni bağımlılık yok.
- **Ağ / süreç / kayıt defteri / gizli blob:** Hiçbiri yok.
- **Dosya yazma/silme:** `VICTORY_WINDOW_UPLOAD` diliminde **yok**. (Kapsam dışı `renderdoc.rs:581` `LoadLibraryExW` tam-yol korumalı ve `diagnostics` feature arkasında; `shot.rs` `VICTORY_SHOT` klasörüne yazar ve `diagnostics` feature arkasında; `pipeline/library.rs:458` `fs::rename`.)
- **Yeni `unsafe` / FFI (`VICTORY_WINDOW_UPLOAD` dilimi):**
  - `crates/victory-gpu/src/host.rs` (commit diff L36–L41): `device.CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS16, ...)` — salt okunur özellik sorgusu.
  - `crates/victory-gpu/src/host.rs` (commit diff L338–L345): `CreateCommittedResource` (`D3D12_HEAP_TYPE_GPU_UPLOAD`, `D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS`, `D3D12_RESOURCE_STATE_COMMON`) + `Map(0, None, Some(&mut data))`.
  - `crates/victory-gpu/src/host.rs` (commit diff L361–L367): İlk tamponda tek seferlik 8 KiB self-test (`copy_nonoverlapping`, `read()`/`ReadProcessMemory`, `from_raw_parts` doğrulama, `write_bytes` sıfırlama).
  - `crates/victory-gpu/src/shadow.rs` (commit diff L4076): `unsafe { buffer.GetGPUVirtualAddress() }`.

### (b) Doğruluk Riski — **DÜŞÜK**
- **Sürücü / donanım desteği:** `D3D12_OPTIONS16.GPUUploadHeapSupported` gerektirir (ReBAR açık + destekleyen NVIDIA sürücüsü + Win11/Agility SDK). Desteklenmiyorsa veya ilk 8 KiB self-test başarısızsa otomatik olarak mevcut RAM yoluna geri düşer.
- **Write-Combined (WC) okuma riski:** Ayrık GPU'da `D3D12_HEAP_TYPE_GPU_UPLOAD` `WRITE_COMBINE + L1`'dir. Başlık pencerelerine shader yazımı gözlenmediği (`0.0 MB`) için normalde yalnız CPU yazımı yapılır; ancak gelecekte bir shader bu pencereye yazarsa `Shadow::gpu_writes` (`shadow.card == None`) WC VRAM'den CPU okuması yaparak takılma üretebilir.
- **8 GB VRAM + 16 GB RAM etkisi:**
  - `fxceless` README Run 273 (native Windows 11, RTX 4070 12 GB, 155 sn, 0 hata): 368 pencere (1.400 MB) doğrudan VRAM'e tek tampon olarak gitti; paylaşılan sistem RAM **5,7 → 3,8 GB (-1,9 GB)**, boş RAM **1,1–1,9 → 3,2–3,6 GB** iyileşti. **FPS artışı yok** (4,99/7,86 vs 5,42/8,16 fps; send-and-hash 13,6 → 12,3 ms).
  - `window_card_room` VRAM kullanımını `used + size <= budget * 0.8` ile sınırlar. 8 GB VRAM'de agresif `VICTORY_CARD_WINDOWS_MB` değerleri doku yeniden üretimini artırabilir (`fxceless` 12 GB'de 1280–2067 → 2504–3699 artışı bildirdi); 8 GB'de `256..512 MB` ile ölçülüp doğrulanmalıdır.

### (c) Taşınabilirlik (`vw-pr10`)
- **Mevcut altyapı (`vw-pr10`):** `VICTORY_CARD_WINDOWS_MB` / `window_card_most()` / `WINDOW_CARD_BYTES` / `window_card_for()` (`shadow.rs:251–286`), `loose()` (`shadow.rs:3088–3142`), `host.rs::read()` (`host.rs:6008`), `windows = "0.62.2"` (`Cargo.toml:36`) **hepsi mevcut**.
- **Önkoşul commit:** **Yok** (yalnız `host.rs` + `shadow.rs` dilimi seçilerek alınır; `shot.rs`/`renderdoc.rs` hariç).
- **Platform:** Native Windows 11 + NVIDIA'da doğrudan ölçülmüş (Proton farkı yok).
- **Tahmini zorluk:** **Çok kolay** (~120 satır, 2 dosya, varsayılan kapalı).

### (d) Öneri (`79cfdd9`)
- **TAŞI (yalnız `VICTORY_WINDOW_UPLOAD` + `window_card_room` dilimi; `shot.rs`/`renderdoc.rs` hariç).** Hız artışı değil **16 GB RAM / paylaşılan bellek tasarrufu** hedefli, varsayılan kapalı ve güvenli; RTX 3070 Ti Laptop'ta `GPU upload heaps: supported=true` log'u doğrulandıktan sonra düşük bütçeyle (`VICTORY_CARD_WINDOWS_MB=256..512`) A/B ölçülmelidir.
