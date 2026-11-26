# Yerinde Bellek (In-Place) Port İncelemesi — Güvenlik ve Taşınabilirlik Raporu

**İncelenen hedefler:** `Marvisak/VIctory` PR `#2088`, `#2112`, `#2921` ve `Belobug/VIctory-Belobug-AMD` commit `0434b3ac`.  
**Hedef taban (salt okunur):** `C:\CLLL\.calisma\vw-avatar` (`avatarmanz/VIctory-7900XTX @0a435665`, `fxceless` tabanlı).  
**Tarih:** 2026-11-26.

---

## Karar Özeti (10 Satır)

1. **Güvenlik:** Dört değişikliğin tamamı kötü amaçlı kod açısından **temiz** (`build.rs`, script, ağ, süreç başlatma, dosya yazma, indirme yok; yalnızca Rust + README).
2. **Kritik bulgu 1 (Mevcut altyapı):** `vw-avatar` tabanında `crates/victory-gpu/src/inplace.rs` (`VICTORY_INPLACE=1|2`, `OpenExistingHeapFromAddress1`, `VIEWS`, `Cover::Run`, `beside()`) **zaten var**; `#2112`'yi bütün olarak taşımak gereksiz ve çakışmalıdır.
3. **Kritik bulgu 2 (`#2088` tuzağı):** `#2088`'in `WINDOW=256 KiB` ayarı bizde zaten `VICTORY_WINDOW_KB=256` ile mevcut; üstelik Windows RTX 5070 açık dünya A/B'sinde RAM düşürürken kare süresini 1.22 s → 1.86 s'ye (+428 ms) bozduğu **kanıtlandı**. Taşınmamalı.
4. **Kritik bulgu 3 (Asıl engel):** Bizim `inplace.rs` şu an sayfa havuzu (`Cover::Run`) görünümlerini dışlıyor (`inplace.rs:141`) ve Windows pin kilidi bırakma / `ERROR_INVALID_PARAMETER (87)` retry mekanizmasına sahip değil.
5. **Taşınabilirlik:** `#2921` doğrudan taşınabilir (mevcut `xmem.rs:1888` `VirtualFree` export'uyla birleştirilerek); `Belobug 0434b3ac` mantığı ise bizim `inplace.rs::beside()` üstüne **~90–130 satırla** (sızıntısız) uyarlanabilir.
6. **Gizli ölümcül risk (Descriptor slotları):** Bizde 2.000+ pencere `HostHeap::kept[slot]` tarafından tutuluyor (`descriptor/host.rs:157`). Slotlar temizlenmeden `#2921`'in `release()` çağrısı `ID3D12Heap`'i öldüremez ve Windows pin kilidi açılmaz (`err=87`).
7. **Diğer riskler:** Her kare yazılan skinning buffer'ları yerinde açılırsa karakter geometrisi bozuluyor (`#2112` kanıtı); `VICTORY_LATE_WAIT` + `INPLACE=2` uyumsuz (`inplace.rs:76-80`); `Belobug 0434b3ac` ikinci `MapViewOfFile`'ı hiç unmap etmiyor (bizim `beside()` bunu çözüyor).
8. **Windows NVIDIA kanıtı:** `#2921` RTX 5070 Ti & RTX 5090 Windows'ta sentetik testlerle **kanıtlı**; `#2112` ve `0434b3ac` için Windows NVIDIA oyun içi RAM/FPS A/B'si **yok (`TAHMİN`)**.
9. **Önerilen en küçük güvenli sıra:** (0) Mevcut `VICTORY_INPLACE_PROBE=1` / `VICTORY_INPLACE=1` ölçümü → (1) `#2921` + `HostHeap::kept` slot temizliği + GPU settle (~580–670 satır) → (2) `inplace.rs::beside()` üzerinden `Cover::Run` yerinde açma (~90–130 satır, opt-in) → (3) gerekirse `#2112`'den `committed_part` ve `landed` (~60–90 satır).
10. **Sonuç:** Güvenli ve taşınabilir; ancak `#2088` ve `#2112`'nin gövdesini değil, **`#2921` (slot temizliği eklenerek) + `beside()` üstünden `0434b3ac` fikrini** almak en kısa ve en güvenli yoldur.

---

## A) Güvenlik İncelemesi (Tam Diff Denetimi)

Dört kaynaktaki tüm değişen dosyalar `gh pr diff` ve `gh api` ile satır satır incelendi.

### Genel Kontrol Özeti

| Kaynak | Değişen Dosyalar | `build.rs` / Script | Ağ / İndirme | Süreç / Dosya Yazma | Zararlı Bulgu |
|---|---|---|---|---|---|
| `Marvisak #2088` | `crates/victory-gpu/src/shadow.rs` (+224/-12) | Yok | Yok | Yok | **Temiz** |
| `Marvisak #2112` | `README.md`, `host.rs`, `memory.rs`, `shadow.rs`, `watch.rs` (+1164/-15) | Yok | Yok | Yok | **Temiz** |
| `Marvisak #2921` | `lib.rs`, `pinned.rs`, `memoryapi.rs`, `mod.rs`, `xmem.rs` (+693/-3) | Yok | Yok | Yok | **Temiz** |
| `Belobug 0434b3ac` | `copystat.rs`, `shadow.rs`, `watch.rs` (+182/-3) | Yok | Yok | Yok | **Temiz** (bellek/adres sızıntısı var, aşağıda) |

### `unsafe` Blokları ve İşaretçi Aritmetiği Denetimi (Diff Satır Numaralarıyla)

#### 1. `Marvisak/VIctory` PR `#2088`
- **Üretim kodu:** Yeni `unsafe` bloğu **yok**.
- **Yalnız testler (`shadow.rs`):** `VirtualAlloc` / `VirtualFree` / `write_volatile` / `read_volatile` (diff satır 255, 260, 269, 279, 285, 297, 301). Tümü testin kendi ayırdığı `4 * WINDOW` blok içinde, güvenli.

#### 2. `Marvisak/VIctory` PR `#2112`
- **`crates/victory-gpu/src/shadow.rs` (üretim kodu):**
  - **Diff satır 388–390, 594–596, 647–649:** `std::mem::zeroed()` + `VirtualQuery(at as *const _, &mut info, size_of_val(&info))`. Standart Win32 sorgusu, güvenli.
  - **Diff satır 459, 464–475:** `K32QueryWorkingSetEx` ve `a.VirtualAttributes.Flags` union okuması.
    - *Kontrol:* `step = pages.div_ceil(1 << 16)` → `asked.len() <= 65_536`; `size_of::<PSAPI_WORKING_SET_EX_INFORMATION>() == 16` → byte boyutu en fazla `1 MiB`, `as u32` taşması yok. Adres yalnızca sorgu girdisi, dereference edilmiyor.
  - **Diff satır 645:** `while at < (start + len) as usize` — `start + len` taşmaya karşı `saturating_add` ile korunmamış (pratikte kullanıcı alanı `< 2^47` olduğu için tetiklenmez, ancak port edilecekse `checked_add` önerilir).
  - **Diff satır 674:** `dev13.OpenExistingHeapFromAddress1::<ID3D12Heap>(start as *const _, size)` — `size = len.next_multiple_of(4096) as usize`. Not: `committed_part` (satır 621) üst sınırı 4 KiB'a yuvarlar; 64 KiB katı gerektiren bir sürücüde `OpenExistingHeapFromAddress1` veya `CreatePlacedResource` hata verip kopya yoluna düşer (UB/OOB yok).
  - **Diff satır 709–711:** `dev3.OpenExistingHeapFromAddress(base as *const _)` ve `heap.GetDesc()`. Güvenli.
  - **Diff satır 762–775:** `device.CreatePlacedResource(...)` ve `buffer.GetGPUVirtualAddress()`. Güvenli.
  - **Diff satır 344 & 839 (dikkat):** `Shadow::new_with(buffer, s0 as *mut u8, gpu, s0, l0, Some(heap))` — `Shadow::data` alanına başlığın (title) kendi adresi konuyor. `self.in_place == true` iken `update`/`writing`/`copy_back` erken döndüğü için dereference edilmiyor, ancak gelecekte `data`'yı okuyan yeni bir kod `in_place` kontrolünü unutursa başlık belleğine doğrudan erişir.
- **Testler (`shadow.rs` diff satır 984–1397):** `VirtualAlloc`/`VirtualFree`, `write_volatile`, `IUnknown` vtable `AddRef`/`Release` sayacı (satır 1223–1226; test içi COM refcount kontrolü, +1/-1 dengeli).

#### 3. `Marvisak/VIctory` PR `#2921`
- **`crates/victory-gpu/src/pinned.rs`:**
  - **Diff satır 177:** `unsafe { GetLastError() }`. Güvenli.
  - **Testler (diff satır 325, 343, 396–462):** `SetLastError`, `VirtualAlloc`, `OpenExistingHeapFromAddress1`, `CreatePlacedResource`, `VirtualProtect`, `at.cast::<u8>().add(BYTES / 2)`, `VirtualFree`. Tümü testin kendi 4 MiB bloğu içinde, güvenli.
- **`crates/victory-hle/src/modules/memoryapi.rs`:**
  - **Diff satır 502, 511, 522:** `VirtualQuery` + `std::mem::zeroed()`.
  - **Diff satır 508, 515, 517 (küçük sertleştirme notu):** `end = info.BaseAddress as usize + info.RegionSize;` ve `end - at` taşma koruması (`saturating_add`/`saturating_sub`) olmadan yazılmış. `VirtualQuery` geçerli bölge döndürdüğü sürece güvenlidir, ancak port ederken `saturating_*` yapılmalı.
  - **Diff satır 528:** `GetCurrentProcess()`, `GetProcessId(process)`, `GetCurrentProcessId()`. Güvenli.
  - **Diff satır 558–630:** `VirtualFree`, `VirtualFreeEx`, `VirtualProtect`, `VirtualProtectEx`, `UnmapViewOfFile`, `UnmapViewOfFileEx`, `UnmapViewOfFile2` — başlığın kendi argümanlarını Win32'ye geçiriyor (standart HLE pass-through).
- **`crates/victory-hle/src/modules/xmem.rs`:**
  - **Diff satır 751, 763–772:** Mevcut `VirtualFree` ve `UnmapViewOfFile2` çağrıları `title_memory_change` içine sarılmış. Güvenli.

#### 4. `Belobug/VIctory-Belobug-AMD` Commit `0434b3ac`
- **`crates/victory-gpu/src/copystat.rs`:** Yalnızca mevcut `tracing::info!` satırına `crate::watch::layout(...)` eklenmiş; yeni `unsafe` yok.
- **`crates/victory-gpu/src/shadow.rs` (`through_another_view`):**
  - **Patch satır 1246–1248:** `unsafe { MapViewOfFile(section as _, FILE_MAP_WRITE, (offset >> 32) as u32, offset as u32, len) }`.
    - **TOCTOU riski:** `watch::stretch` kilidi bıraktıktan sonra `MapViewOfFile` çağrılıyor; bu arada `section` kapatılmış ve handle numarası başka bir nesneye verilmiş olabilir (bizim `vw-avatar` tabanında `watch::title_memory_reading()` kilidi bu yarışı önler).
  - **Patch satır 1261:** `unsafe { device.OpenExistingHeapFromAddress1(view.Value, len) }`.
  - **Patch satır 1271:** `unsafe { UnmapViewOfFile(...) }` — **yalnızca hata durumunda** çağrılıyor.
  - **Sızıntı bulgusu (kod yorumunda açıkça yazılı):** `shortcut: the view of ours is never unmapped.` Başarılı olan her stretch görünümü süreç bitene kadar unmap edilmiyor; uzun oturumlarda sanal adres alanı ve section referansı biriktirir.

---

## B) Taşınabilirlik Analizi (`vw-avatar @0a435665`)

### 1. Fonksiyon Eşleştirme Tablosu

| Yabancı Değişiklik | Yabancı Fonksiyon / Yapı | `vw-avatar` Karşılığı (Dosya:Satır) | Uyum / Fark |
|---|---|---|---|
| **#2088** | `WINDOW` (4 MiB → 256 KiB) | [`shadow.rs:5052-5080`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L5052-L5080) (`WINDOW`, `window()`, `VICTORY_WINDOW_KB`) | **Zaten var** (env ile 64 KiB–4 MiB + `VICTORY_WINDOW_TAILS=1` + `find_any`/`holder`). |
| **#2088** | `enclosing` (`MAX_SPAN`, `.take(64)` kaldırma) | [`shadow.rs:4942,4953,4973,5181`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L4942-L4975) | Bizde hâlâ `.take(64)` var; `#2088`'deki `MAX_SPAN` araması bizde yok. |
| **#2088** | `loose` (enclosing window reuse) | [`shadow.rs:5251-5353`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L5251-L5353) (`loose`, `holder` [`5129`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L5129), `let_go_idle` [`2823`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L2823)) | Bizim `loose` çok daha gelişmiş (`RwLock`, `find_any`, `inplace::place_for`, `window_mapped_for`, `window_card_for`, `let_go_idle`). |
| **#2112** | `in_place_enabled` / `in_place_buffer` / `place_in` | [`inplace.rs:87-168,225-260`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/inplace.rs#L87-L168) (`level`, `place_part`, `place_for`, `open`, `place`) | **Zaten var** (`VICTORY_INPLACE=1\|2`, `ID3D12Device13::OpenExistingHeapFromAddress1`). |
| **#2112** | `Shadow::in_place`, `Shadow::new_with` | [`shadow.rs:104,2936-2960`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L2936-L2960) (`Shadow::place`, `Shadow::placed`) | **Zaten var**; `update` (`3695`), `nothing_new` (`3735`), `gpu_writes` (`3034`), `writing` (`3051`) erken dönüyor. |
| **#2112** | `HostList::keep_once` | [`shadow.rs:2964-2967`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L2964-L2967) (`Shadow::refresh` → `list.defer_refresh`) | Bizde `refresh` erken dönmediği için `Arc<Shadow>` (ve `place` içindeki `ID3D12Heap`) liste resetlenene kadar zaten tutuluyor. |
| **#2112** | `watch::title_memory_remapped` → `in_place_remapped` | [`watch.rs:2529-2536`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/watch.rs#L2529-L2536) → [`inplace.rs:183`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/inplace.rs#L183) → [`shadow.rs:4869-4895`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L4869-L4895) (`in_place_let_go`) | **Zaten bağlı** (ancak yalnız `retire` ediyor, senkron pin bırakma + retry yok). |
| **#2112** | `memory.rs` (`withheld` + `landed`) | [`memory.rs:1-250`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/memory.rs#L1-L250) | Mimari tamamen farklı (bizde arena + directory); bu 4 satır taşınmaz. |
| **#2112** | `Shadow::landed` (UAV yazımı sonrası diğer kopyaları işaretleme) | [`shadow.rs:3031-3036`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L3031-L3036) | Bizde `shortcut` olarak eksik bırakılmış; seçilerek alınabilir. |
| **#2921** | `victory_gpu::pinned` (`pin`, `title_memory_change`, `title_memory_held`) | Yok (`lib.rs:109` yalnızca `title_memory_freeing` export ediyor) | Yeni modül olarak eklenmeli; `in_place_let_go` + GPU settle + slot temizliği ile bağlanmalı. |
| **#2921** | `victory_hle::modules::memoryapi` (7 Win32 bellek fonksiyonu) | [`xmem.rs:1873-1888`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-hle/src/modules/xmem.rs#L1873-L1888) (yalnız `VirtualFree` → `title_memory_freed`) | **Çakışma noktası:** Bizde `xmem` zaten `VirtualFree`'yi aynı DLL'ler için export ediyor! Birleştirilmeli; diğer 6 fonksiyon (`VirtualFreeEx`, `VirtualProtect*`, `UnmapViewOfFile*`) bizde yok. |
| **#2921** | `xmem::XMemFreeDefault` ve `xmem::unmap` sarmalama | [`xmem.rs:362-365`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-hle/src/modules/xmem.rs#L362-L365) ve [`xmem.rs:1761-1771`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-hle/src/modules/xmem.rs#L1761-L1771) | Birebir karşılığı var; doğrudan sarılabilir. |
| **0434b3ac** | `watch::stretch` (ardışık 64 KiB view serisi) | [`inplace.rs:266-526`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/inplace.rs#L266-L526) (`VIEWS`, `cover_in` → `Cover::Run(section, offset)`) | **Bizde zaten hazır!** `cover_in` bir aralığın tek bir section'da ardışık run olup olmadığını doğrudan söylüyor. |
| **0434b3ac** | `shadow::through_another_view` (`MapViewOfFile` + `OpenExistingHeapFromAddress1`) | [`inplace.rs:536-566`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/inplace.rs#L536-L566) (`beside`) + [`inplace.rs:314-326`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/inplace.rs#L314-L326) (`title_memory_section_gone`) | **%80'i zaten hazır!** `beside()` zaten tüm section'ı bir kez `MapViewOfFile` ile eşleyip `v.whole` içinde tutuyor ve section ölünce `UnmapViewOfFile` ediyor (Belobug'un sızıntısı bizde yok!). |
| **0434b3ac** | `copystat.rs` view düzeni logu | [`copystat.rs:1-220`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/copystat.rs#L1-L220) | İsteğe bağlı teşhis satırı. |

---

### 2. Bizim `inplace.rs` (`VICTORY_INPLACE`) vs `#2112` (`VICTORY_IN_PLACE`) Karşılaştırması

- **Aynı mekanizma mı, çakışıyor mu?** Evet, ikisi de `ID3D12Device13::OpenExistingHeapFromAddress1` + `CreatePlacedResource(ALLOW_CROSS_ADAPTER | ALLOW_UNORDERED_ACCESS)` ile `Shadow`'u başlık belleğinin kendisine bağlıyor. İkisini aynı koda koymak doğrudan çakışır (çift env anahtarı, çift in-place bayrağı, çift bırakma yolu).
- **Hangisi daha kapsamlı?**
  - **Bizim `inplace.rs` daha temiz ve Windows'a daha uygun:**
    - `VICTORY_INPLACE=1` (yalnız pencereler) ve `=2` (büyük `cards()` VRAM buffer'ları dışındaki buffer'lar) kademeli.
    - `VICTORY_INPLACE_MB` (varsayılan 4096 MB) pin bütçesi var.
    - `VICTORY_INPLACE_PROBE=1|2` ile donanım/sürücü canlı doğrulama + sayım aracı hazır.
    - `VIEWS` + `Cover::Run` + `beside()` + `title_memory_section_gone()` (section görünümü yönetimi) halihazırda içinde.
    - `Shadow::refresh` `defer_refresh` yaptığı için in-flight liste ömrü zaten korunuyor.
  - **`#2112`'nin bizde olmayan artıları:**
    - `committed_part` (parçalı commit'li bir 4 MiB pencerenin yalnızca commit'li 64 KiB alt aralığını açma; bizde `private_whole` tüm pencere tek commit'li özel blok değilse reddediyor).
    - `VICTORY_INPLACE_MIN_KB` (1024 KiB altı buffer'ları kopyada tutma — küçük buffer'larda PCIe üstünden atomics'in 19–67 ms'ye çıktığı `fxceless#772`'de ölçülmüş).
    - `Shadow::landed` (yerinde UAV yazımı sonrası aynı 64 KiB sayfadaki diğer kopyalara/dokulara `watch::writing` bildirimi).
    - Linux/vkd3d-proton whole-allocation fallback (`OpenExistingHeapFromAddress` + `K32QueryWorkingSetEx` `dense()`). *Windows D3D12'de `ID3D12Device13` mevcut olduğundan whole-allocation yolu bize gerekmez.*

---

### 3. Windows / NVIDIA D3D12'de `OpenExistingHeapFromAddress1` Riskleri ve `#2921`'in Çözümü

| Risk | Durum (Kanıt vs `TAHMİN`) | `#2921` Nasıl Çözüyor? | Bizdeki Ek Boşluk |
|---|---|---|---|
| **Pin kilidi & `ERROR_INVALID_PARAMETER (87)`** | **KANIT:** RTX 5070 Ti, RTX 5090, RX 7600, WARP (`#2921`, `#2112`, `inplace.rs:33-36`). Heap açıkken `VirtualProtect`, `VirtualFree`, `UnmapViewOfFile2` 87 ile başarısız olur. | `title_memory_change` çakışan tüm pin'lerin `Release` callback'ini Win32 çağrısından önce çalıştırır. | Bizim `title_memory_freeing` → `in_place_let_go` yalnız `retire` yapıyor; in-flight liste veya descriptor slot (`HostHeap::kept`) referans tutuyorsa heap ölmez ve 87 sürer. |
| **Geç çekirdek kilit açma (late unpin)** | **KANIT:** RTX 5070 Ti (`#2921`): 40 bırakmanın 32'si <0.01 ms, en yavaşı 5.9 ms; retry olmadan ~1/6 başarısız. | `Release` sonrası 87 gelirse `SETTLE = 100 ms` boyunca `yield_now` ile yeniden dener; başka thread'in süren release'ini `RELEASED` condvar'ı ile bekler. | Doğrudan `#2921`'den alınabilir. |
| **İkinci görünüm (`beside` / `0434b3ac`) pin davranışı** | **`TAHMİN` (kısmi kanıt `inplace.rs:24-27`):** Heap bizim ikinci `MapViewOfFile` görünümümüzde açıldığında, başlığın kendi view'ı farklı sanal adrestedir; başlığın `UnmapViewOfFile2`'si bizim view'ımızı değil kendi view'ını kaldırır. | Section'ın kendisi `title_memory_section_gone` ile unmap edilirken `#2921` pin'i serbest bırakır. | İkinci view üzerinde `OpenExistingHeapFromAddress1` açıkken başlığın ilk view'ını unmap/protect etmesinin 87 verip vermediği Windows NVIDIA'da kısa birim testiyle doğrulanmalı. |
| **Device Removed** | **KANIT (`inplace.rs:31-32`):** Bu heap'lere `UpdateTileMappings` çağrılırsa cihaz düşer; row-major dışı doku / `ALLOW_CROSS_ADAPTER` eksikliği reddedilir. | Etkilemez. | Bizim `inplace.rs` zaten yalnız `ALLOW_CROSS_ADAPTER \| ALLOW_UNORDERED_ACCESS` buffer yerleştiriyor; tile mapping yapmıyor. |
| **Sayfa koruma / VEH guard çakışması** | **KANIT (`#2112` & `#2921` incelemesi):** `watch::protect` pin'li sayfaya `PAGE_READONLY` koyamaz (87). `watch::open` VEH içinden veya `GUARDS` kilidi altında `title_memory_change` çağrılırsa GPU beklemesi deadlock yapar. | `#2921` `watch.rs`'yi bağlamaz ve VEH/`GUARDS` altında çağrılmaması kuralını koyar. | Bizim `watch.rs:2536` `title_memory_changed` içinde `GUARDS` kilidinden **önce** `title_memory_freeing` çağırıyor — doğru yer, ancak VEH (`on_write`) yolundan asla çağrılmamalı. |
| **Başlığın belleği canlı değiştirmesi (CPU/GPU yarışı)** | **KANIT (`#2112`):** Her kare yazılan skinning buffer'ları yerinde açılınca karakter kolları uzayıp bozuluyor (snapshot yok). Ayrıca `inplace.rs:76-80`: `VICTORY_LATE_WAIT` + `INPLACE=2` uyumsuz. | Çözmez (tasarım sınırı). | Önce yalnız `INPLACE=1` (veya yalnızca salt-okunur / seyrek yazılan pencereler) ile başlanmalı; `INPLACE=2` `VICTORY_LATE_WAIT` ile birleştirilmemeli. |

---

### 4. Windows'ta Ölçülmüş Kanıt Durumu (Kanıt vs `TAHMİN`)

| Kaynak | Windows Kanıtı | Linux / vkd3d Kanıtı | Bizim 8 GB NVIDIA + 16 GB RAM Win 11 İçin Değerlendirme |
|---|---|---|---|
| **`Marvisak #2088`** | **VAR (Oyun içi A/B, RTX 5070 12 GB, Win 11, `r4tedx`):** Gevşek pencere belleği 5,520 → 2,640 MB (~2.9 GB düştü), **AMA** kare süresi 1,218 → 1,860 ms (+428 ms medyan), kare başı kopya 0.55 → 6 GB, sayfa hatası 27.6k → 52.7k. Bizim `shadow.rs:4927-4929`'da da benzer regresyon notlu. | RTX 5080 açık dünya: 11.4 → 6.5 GB kopya, 29.2 → 15.8 GB RSS (kare süresi ölçülmedi). | **KANITLI REGRESYON:** Windows açık dünyada kopya fırtınası yapıyor; ayrıca bizde zaten `VICTORY_WINDOW_KB=256` var. |
| **`Marvisak #2112`** | **Yalnız sentetik + tek yükleme örneği:** `yualk` (RTX 5070 Ti) & `nathanbzrr` (RX 7600) birim testleri geçti; `RKouchoo` tek yükleme örneğinde 243 MiB in-place / 1.11 GiB kopya (baseline yok). **Windows açık dünya A/B'si YOK.** | RTX 5080 (prison yard): 933 → 817 ms, kopyalar 438 → 331 ms, ama skinning bozuldu. RTX 3060 8 GB / 16 GB RAM (`0xKaji`, Series S): 15.67 → 11.25 GiB RSS. | Windows NVIDIA oyun içi RAM/FPS kazancı **`TAHMİN`**. |
| **`Marvisak #2921`** | **VAR (Sentetik, RTX 5070 Ti & RTX 5090, Win 11, `VICTORY_D3D12_DEBUG=1`):** 25/25 ve 45/45 GPU + 154/154 HLE testi geçti; unpin gecikmesi 32/40 <0.01 ms, maks 5.9 ms. | Yok (Windows-özgü). | Pin/87 altyapısı Windows NVIDIA'da **kanıtlı**; canlı oyun içi pin yükü **`TAHMİN`**. |
| **`Belobug 0434b3ac`** | **VAR (Oyun içi, Windows AMD, `beloooo`):** 33 stretch penceresi yerinde, copy-back 36 → 12 ms/kare, kare süresi 227–266 → 202–216 ms. **NVIDIA ölçümü ve RAM/NON_LOCAL ölçümü YOK.** | Yok. | Windows NVIDIA'da çalışırlık ve `NON_LOCAL` RAM düşüşü **`TAHMİN`** (ancak bizim ~760 MB çift pencere + reddedilen view belleği için en doğrudan aday). |

---

### 5. En Küçük Güvenli Taşıma Sırası ve Tahmini Satır Sayısı

**Taşımadan bırakılacaklar:**
- **`#2088` (`WINDOW=256 KiB`):** Taşıma; bizde `VICTORY_WINDOW_KB` zaten var ve Windows açık dünyada ağır kare süresi regresyonu kanıtlı. (İstenirse yalnız `MAX_SPAN`/`enclosing` ~30 satır ayrı alınabilir.)
- **`#2112` gövdesi (+1.164 satır):** Taşıma; bizim `inplace.rs` zaten `OpenExistingHeapFromAddress1` + `Shadow::placed` altyapısına sahip ve Linux whole-allocation (`dense()`) yolu Windows'ta gereksiz.

**Önerilen en küçük güvenli sıra:**

1. **Adım 0 — Mevcut Kodu Ölç (0 satır):**
   - `vw-avatar`'da zaten bulunan `VICTORY_INPLACE_PROBE=1` ve ardından `VICTORY_INPLACE=1` (`VICTORY_INPLACE_MB=1024`) ile kendi RTX 3070 Ti Windows makinemizde kaç pencerenin `view memory` nedeniyle reddedildiğini (`inplace.rs:201-202` log satırından) doğrudan ölç.
2. **Adım 1 — `#2921` Pin Güvenliği + Slot/Flight Bırakma (~580–670 satır; testler hariç ~360 satır):**
   - `crates/victory-gpu/src/pinned.rs` (+431 satır, `#2921`'den neredeyse aynen).
   - `crates/victory-hle/src/modules/xmem.rs` + `memoryapi.rs` (~150 satır): `#2921`'deki 7 bellek fonksiyonunu mevcut `xmem.rs:1888` `VirtualFree` export'u ile tek noktada birleştir (`title_memory_freed` + `title_memory_change` birlikte); `XMemFreeDefault` (`xmem.rs:363`) ve `unmap` (`xmem.rs:1763`) çağrılarını sar.
   - **Zorunlu yerel ek (~80–120 satır):** `Shadow::placed` oluşturulduğunda `pinned::pin(at, bytes, release)` kaydet; `release` callback'i (a) `LOOSE`/`BUFFERS`'dan çıkarıp, (b) `HostHeap::kept` içindeki ilgili `ID3D12Resource` slotlarını null'a çeksin (`unmake` / `keep(i, None)`), (c) in-flight GPU submission'ları settle etsin ki `ID3D12Heap` son referansı gerçekten düşsün.
3. **Adım 2 — `Belobug 0434b3ac` Mantığının `inplace.rs::beside()` Üstüne Uyarlanması (~90–130 satır):**
   - `inplace.rs:141`'de `Cover::Run(section, offset)` durumunu reddetmek yerine (yeni opt-in anahtar: örn. `VICTORY_INPLACE_VIEWS=1` ve `len >= 4 MiB` veya tüm pencereler), `watch::title_memory_reading()` altında `beside(at, bytes)` adresini alıp `OpenExistingHeapFromAddress1(beside_ptr, bytes)` ile yerleştir.
   - `beside()` zaten section başına tek `MapViewOfFile` tutup `title_memory_section_gone`'da `UnmapViewOfFile` yaptığı için **Belobug'un "never unmapped" sızıntısı hiç oluşmaz**.
   - `title_memory_section_gone` (`inplace.rs:314`) `UnmapViewOfFile` çağırmadan önce `title_memory_change(base, section_bytes, ...)` ile o section'daki pin'leri serbest bırakmalı.
4. **Adım 3 — `#2112`'den Seçme İyileştirmeler (İsteğe bağlı, ~60–90 satır):**
   - `committed_part` (parçalı commit'li özel pencerelerde tüm 4 MiB yerine commit'li 64 KiB alt aralığı açmak için).
   - `Shadow::landed` (`INPLACE=2` kullanılacaksa UAV yazımı sonrası `watch::writing` ile diğer kopyaları kirli işaretleme) ve `VICTORY_INPLACE_MIN_KB` (küçük buffer'ları kopyada tutma).

---

### 6. Descriptor Slot'larının Pencere Kimliğine Bağlılığı ve Yerinde Pencerelerle Etkileşimi

Bizim `vw-avatar` kodundaki yapısal gerçekler:
- `shadow::find()` ([`shadow.rs:4910-4922`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L4910-L4922)) pencereyi yalnızca `(start, len)` ismiyle arar; çünkü descriptor slot'u oluşturulduğunda ([`descriptor/host.rs:3526,3618,3633,3742`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/descriptor/host.rs#L3526-L3619)) `shadow.buffer.clone()`'u `HostHeap::kept[slot]` ([`descriptor/host.rs:157`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/descriptor/host.rs#L157)) içine yazar ve başlık o slotu üzerine yazana kadar bırakmaz.
- `let_go_idle()` ([`shadow.rs:2823-2843`](file:///C:/CLLL/.calisma/vw-avatar/crates/victory-gpu/src/shadow.rs#L2823-L2843)) `crate::host::references(&s.buffer) == 1` şartını aradığı için, slotlarca tutulan 2.000+ pencere `let_go_idle` ile **serbest bırakılamaz**.

Bu durumun yerinde pencerelerle **üç kritik etkileşimi** vardır:

1. **RAM tarafında büyük avantaj:** 2.000+ slotun tuttuğu pencereler `shared_buffer` kopyası yerine `Shadow::placed` olursa, slot `ID3D12Resource`'u ne kadar uzun tutarsa tutsun ortada kopyalanmış `NON_LOCAL` bellek yoktur; bellek başlığın kendi sayfasıdır.
2. **Pin kilidi tuzağı (`#2921` için belirleyici):** D3D12'de bir placed `ID3D12Resource`, bağlı olduğu `ID3D12Heap`'e dahili referans tutar. `HostHeap::kept[slot]` o `ID3D12Resource`'u tuttuğu sürece, `LOOSE`'tan pencereyi silmek `ID3D12Heap`'i **öldürmez**. Başlık o adresi `VirtualFree`/`VirtualProtect` etmeye kalktığında `#2921` 100 ms bekleyip `ERROR_INVALID_PARAMETER (87)` ile başarısız olur. **Çözüm:** `pinned::Release` callback'i, o `ID3D12Resource`'u tutan tüm `HostHeap::kept` slotlarını da tarayıp null'a çekmek (`unmake` veya `kept[slot] = None`) ve in-flight GPU listelerini settle etmek zorundadır.
3. **İsim kimliği ve değişken boyutlu `stretch` uyumu:** `Belobug 0434b3ac` değişken boyutlu (`4 MiB..75 MiB`) stretch pencereleri ürettiği için bizim sabit `(start, len)` anahtarlı `LOOSE` + `find()` düzenimizle doğrudan uyuşmaz (aynı adresi isteyen descriptor slot ile refresh geçişi farklı pencere görebilir). Buna karşılık Adım 2'de önerilen **`inplace::place_for(start, len, true)` içinde `Cover::Run` → `beside(at, bytes)`** yöntemi, her pencereyi tam `find()`'ın beklediği `(start, len)` sınırıyla açtığı için **pencere kimliğini ve `Shadow::offset` hizalamasını %100 korur**.
