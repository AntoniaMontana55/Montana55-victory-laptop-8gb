# Geri Kopya Adayları — Güvenlik ve Uygulanabilirlik İncelemesi (3 Aday)

**İnceleme kapsamı:** Salt okunur diff + GitHub PR/yorum incelemesi ve yerel `C:\CLLL\.calisma\vw-pr10` tabanıyla fonksiyon eşleştirmesi. Derleme, script çalıştırma veya repo klonlama yapılmadı.  
**Yerel taban:** ZSlayer `02ea29c` + PR #10 + fxceless PR #32 eşdeğeri + yerel PR #1066 taşıması (`crates/victory-gpu/src/{host.rs,shadow.rs,draw.rs,indirect.rs}`).

---

## Özet Karşılaştırma

| Aday | Güvenlik Kararı | Env Bayrağı (Varsayılan) | Taşıma Zorluğu | Doğruluk / Saha Riski |
|---|---|---|---|---|
| 1. `CSTCryst/VIctory-Cryst-NVIDIA` (`c271aff` + `dcdc980`) | **GÜVENLİ** | `VICTORY_RELAY_COPY_QUEUE=1`, `VICTORY_RELAY_COPY_STRICT=1` (kapalı) | **Zor** | Orta-yüksek (iki kuyruk arası fence sırası, 50 ms fallback spin, beyan dışı shader yazısı) |
| 2. `Belobug/VIctory-Belobug-AMD` PR #123 (`1bc559c`) | **GÜVENLİ** | `1bc559c`'de yok (`cad3975`'te `VICTORY_MERGE=legacy\|scalar`, varsayılan AVX2 açık) | **Orta-Zor** | **Yüksek** (Win11/RX 9070 XT'de koridor/hücre duvarlarının kaybolması raporu; kök sebep açık değil) |
| 3. `fxceless/victory-fork` PR #1014 (`5557224`) | **GÜVENLİ** | `VICTORY_CARD_HASH_WIDE=0` eski yol (varsayılan açık) | **Kolay** | **Düşük** (3 NVIDIA koşusunda doğrulandı, görsel bozulma yok, yerel tabanla birebir uyumlu) |

---

## 1. `CSTCryst/VIctory-Cryst-NVIDIA` — `c271aff` + `dcdc980`

### (a) Güvenlik
- **`build.rs` / `Cargo.toml` / `Cargo.lock` / yeni bağımlılık:** Yok.
- **Ağ, soket, HTTP, dosya yazma/silme, süreç başlatma, kayıt defteri, gizlenmiş/base64/ikili blob:** Yok.
- **`unsafe` / FFI:** Yalnız standart D3D12 COM ve Win32 bekleme çağrıları:
  - `c271aff` — `crates/victory-gpu/src/host.rs`: `Host::make_copy_fetch` (+3932–3951: `CreateCommandQueue(D3D12_COMMAND_LIST_TYPE_COPY)`, `CreateCommandAllocator`, `CreateCommandList`, `Close`, `CreateFence`), `Host::relay_card_back` (+4071–4091: `copy.allocator.Reset`, `copy.list.Reset`, `copy.list.Close`, `copy.queue.ExecuteCommandLists`, `copy.queue.Signal`, `self.queue.Wait`, `copy.fence.SetEventOnCompletion`, `WaitForSingleObject`; +4110, +4141: `self.fence.GetCompletedValue`).
  - `dcdc980` — `crates/victory-gpu/src/host.rs`: `Host::relay_card_back` (+4228–4230: `copy.queue.Wait(&self.fence, v)?`).
- **Karar:** **GÜVENLİ**

### (b) Diff Özeti
- **`c271aff`** (+313 / -32, toplam 345 satır):
  - `crates/victory-gpu/src/host.rs` (+292 / -32): `CardPlan.sent_as`, `relay_copy_queue()`, `CARD_IN_AIR`, `contested()`, `CopyFetch`, `RELAY_FETCH_STATS`, `RelayFetch.copy`, `Host::make_copy_fetch()`, `Host::card_before()`, `Host::relay_card_back()`, `Host::settle()`, `late_line()`, `Meter` logu, `relay_tests::a_fetch_is_contested_only_by_flights_in_the_air_that_declared_its_bytes`.
  - `crates/victory-gpu/src/draw.rs` (+16 / -0): `HEAP_BUFFER_PARTS` telemetri sayaçları (`heap_buffers`, `heap_textures`).
  - `crates/victory-gpu/src/shadow.rs` (+5 / -0): `Shadow::offsets(&self, address, end)`.
- **`dcdc980`** (+218 / -48, toplam 266 satır):
  - `crates/victory-gpu/src/host.rs` (+213 / -48): `merge_groups()` + `merge_group_tests` (çakışan title aralıklarını tek thread'e toplayan `VICTORY_WIDE_MERGE` düzeltmesi), `InAir` (fence değeri), `contested(..., whole: bool)`, `relay_copy_strict()`, `FLIGHT_SIGNALLED`, `CONTESTED_SIGNAL_WAIT` (50 ms), `RELAY_FETCH_STATS` (8→10), `Host::card_before(..., fence: u64)`, `Host::relay_card_back()` (çakışan fetch'te `copy.queue.Wait`), `Host::submit()` (`FLIGHT_SIGNALLED.fetch_max`).
  - `crates/victory-gpu/src/shadow.rs` (+5 / -0): `Shadow::title_start(&self)`.
- **Env bayrakları:**
  - `VICTORY_RELAY_COPY_QUEUE=1` (varsayılan **kapalı**, ayrıca `relay()` açık olmalı). Kapalıyken copy queue açılmaz ve `CARD_IN_AIR` kullanılmaz; yalnız yeni istatistik/log satırları eklenir, davranış değişmez.
  - `VICTORY_RELAY_COPY_STRICT=1` (varsayılan **kapalı**).
  - `merge_groups` yalnız `VICTORY_WIDE_MERGE=1` (varsayılan **kapalı**) iken çalışır.

### (c) Uygulanabilirlik (`vw-pr10` Doğrulaması)
- **Yerel tabanda mevcut olanlar:**
  - [host.rs:1698](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L1698) `wide_merge`, [host.rs:1716](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L1716) `CardPlan`, [host.rs:1980](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L1980) `QUEUE_ORDER`, [host.rs:2511](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L2511) `RelayFetch`, [host.rs:2664](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L2664) `late_line`, [host.rs:3397](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3397) `card_before`, [host.rs:3641](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3641) `make_relay_fetch`, [host.rs:3667](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3667) `relay_card_back`, [host.rs:4179](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L4179) `settle`, [host.rs:4323](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L4323) `submit`.
  - [draw.rs:1380](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/draw.rs#L1380) `heap_buffers`, [draw.rs:1685](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/draw.rs#L1685) `heap_textures`.
  - [shadow.rs:2591](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/shadow.rs#L2591) `card_span` (`self.inside` ve `self.start` mevcut; `offsets` ve `title_start` kolayca eklenebilir).
- **Yerel tabanda farklı / eksik olanlar:**
  - `copy_fetch_runs` bizde yok; [host.rs:3736-3738](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3736-L3738) içinde doğrudan `CopyBufferRegion` döngüsü var.
  - `relay_hand_back` ([host.rs:2595](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L2595)) bizde tek parametreli (`owners` parametresi yok).
  - Bizdeki `wide_merge` ([host.rs:3768-3799](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3768-L3799)) zaten 8 sayfa kovası kullanıyor; `dcdc980`'deki `merge_groups` parçası ayrı değerlendirilebilir.
- **Bağımlı olduğu önceki commit'ler:** `c271aff` ebeveyni `921d8d3`, `dcdc980` ebeveyni `ceff7c6` (`ae69659` üzerinden `c271aff` üstüne geliyor). Doğrudan cherry-pick çakışır; copy queue + `CARD_IN_AIR` + `FLIGHT_SIGNALLED` mekanizması elle uyarlanmalı.
- **Tahmini taşıma zorluğu:** **Zor**.

### (d) Doğruluk Riski
- **İki kuyruk deadlock / fence sırası riski:** `copy.queue.Wait(&self.fence, v)` ve ardından `self.queue.Wait(&copy.fence, value)` sıralamasını `FLIGHT_SIGNALLED` + 50 ms spin-yield koruyor. Bir uçuş `card_before` sonrası `submit` içinde sinyal atmadan çıkarsa 50 ms spin-wait fallback'e düşer (veri bozulmaz ama kare takılır).
- **Beyan dışı shader yazısı (data race):** `VICTORY_RELAY_COPY_STRICT=0` iken shader beyan edilen aralık dışına yazarsa copy queue yarı yazılmış veri okuyabilir (`STRICT=1` bunu önler ama copy queue paralelliğini düşürür).
- **Görsel bozulma raporu / AMD varsayımı:** Bilinen görsel bozulma raporu yok; saf D3D12, AMD'ye özgü varsayım yok.

---

## 2. `Belobug/VIctory-Belobug-AMD` — PR #123 (`1bc559c`)

### (a) Güvenlik
- **`build.rs` / `Cargo.toml` / `Cargo.lock` / yeni bağımlılık:** Yok (yalnız `std::arch::x86_64::*`).
- **Ağ, soket, HTTP, dosya yazma/silme, süreç başlatma, kayıt defteri, gizlenmiş/base64/ikili blob:** Yok.
- **`unsafe` / FFI:** Yalnızca `is_x86_feature_detected!("avx2")` ile korunan `#[target_feature(enable = "avx2")]` fonksiyonları ve 32 baytlık unaligned AVX2 load/store intrinsics:
  - `1bc559c` — `crates/victory-gpu/src/shadow.rs`:
    - `merge_gpu_writes` (+2087) → `merge_gpu_writes_avx2` (+2132–2186)
    - `merge_gpu_writes_mut` (+2198) → `merge_gpu_writes_mut_avx2` (+2253–2309)
    - `merge_gpu_writes_card` (+2324) → `merge_gpu_writes_card_avx2` (+2387–2445)
  - Tüm döngüler `while i + 32 <= n` (`n = len1.min(len2).min(len3)`) ile sınır kontrolü yapıyor; taşma yok.
- **Karar:** **GÜVENLİ**

### (b) Diff Özeti
- **`1bc559c`** (+287 / -6, toplam 293 satır; PR #123 tamamı 3 commit — `1bc559c`, `0d862a5`, `cad3975` — ile +428 / -6):
  - Tek dosya: `crates/victory-gpu/src/shadow.rs`.
  - Fonksiyonlar: `merge_gpu_writes{,_scalar,_avx2}`, `merge_gpu_writes_mut{,_scalar,_avx2}`, `merge_gpu_writes_card{,_scalar,_avx2}`, birim testi `merge_gpu_writes_matches_scalar_across_lengths_and_patterns`.
  - İşlev: `hw == ow` durumunda `tw != hw` kontrolüyle değişmemiş 8/32 baytlık blokların mapped `host`/`old` arabelleğine geri yazılmasını atlar; x86_64'te 32 baytlık AVX2 döngüsü kullanır.
- **Env bayrağı:** `1bc559c`'de **yok** (her zaman aktif). Takip commit'i `cad3975` teşhis için `VICTORY_MERGE=legacy|scalar` ekler (unset = AVX2, yani hâlâ varsayılan açık).

### (c) Uygulanabilirlik (`vw-pr10` Doğrulaması)
- **Yerel tabanda mevcut olanlar / olmayanlar:**
  - `merge_gpu_writes`, `merge_gpu_writes_mut`, `merge_gpu_writes_card` fonksiyonlarının **hiçbiri** `vw-pr10`'da yok.
  - Bizdeki karşılıklar:
    - [shadow.rs:568](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/shadow.rs#L568) `merge_piece(now, held, title)` (`relay_merge` [shadow.rs:2018](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/shadow.rs#L2018) ve `card_merge_relay` [shadow.rs:2714](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/shadow.rs#L2714) tarafından çağrılır): **mapped `host` belleğine zaten hiç yazmaz** (bkz. [shadow.rs:564](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/shadow.rs#L564)). Dolayısıyla PR #123'ün asıl kazancı olan "mapped D3D12 belleğine gereksiz yazıyı atlama" bizim aktif `relay 2` yolunda geçerli değil.
    - [shadow.rs:1960-1966](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/shadow.rs#L1960-L1966) `copy_back_with` içindeki senkron döngü `host` üzerine `*h = merged` yazıyor; ancak bu relay-dışı/senkron yol.
- **Bağımlı olduğu önceki commit'ler:** Belobug `01aadd1` tabanındaki `merge_gpu_writes*` yapısına bağlı.
- **Tahmini taşıma zorluğu:** **Orta-Zor** (sıfırdan yeniden uyarlama gerekir; ayrıca önceki taramada belirtildiği gibi yalnız `merge_piece` AVX2 portu %50 değişen bloklarda ~%10 gerilediği için kapatılmış).

### (d) Doğruluk Riski
- **Görsel bozulma raporu (KRİTİK):** PR #123 yorumlarında `jmal423` (Windows 11, AMD Radeon RX 9070 XT, Series X) *"all prison/corridor walls disappear (likely due to a skipped/mis-merged GPU culling buffer write). Reverting this PR immediately restores all walls."* bildirdi. Yazar (`altuux`) 200k rastgele testte AVX2 ile scalar arasında fark olmadığını, sorunun **mapped kopyaya yazının atlanmasından** kaynaklanabileceğini belirtip `cad3975` ile araştırma başlattı; kök sebep henüz çözülmedi.
- **Veri yarışı / bellek modeli:** Write-Combined mapped arabellekte yazının atlanması veya eşzamanlı GPU yazısı sırasında üç ayrı 32-bayt yüklemenin tutarsız anlık görüntü görmesi olası.
- **AMD / Linux varsayımı:** Pozitif oyun ölçümü (624→350 ms) Arch Linux / RADV / Proton / RX 7900 XT üzerinde; Windows 11 D3D12'de duvar kaybı görüldü.

---

## 3. `fxceless/victory-fork` — PR #1014 (`5557224`)

### (a) Güvenlik
- **`build.rs` / `Cargo.toml` / `Cargo.lock` / yeni bağımlılık:** Yok.
- **Ağ, soket, HTTP, dosya yazma/silme, süreç başlatma, kayıt defteri, gizlenmiş/base64/ikili blob:** Yok.
- **`unsafe` / FFI:**
  - Üretim kodunda yeni `unsafe` blok **yok**; yalnızca `crates/victory-gpu/src/host.rs` (+3988–3991) içinde mevcut `D3DCompile` çağrısına `CARD_COMPARE` yerine `card_compare_source()` veriliyor.
  - Test kodunda (`crates/victory-gpu/src/indirect.rs` +993–1026): `#[test] fn warp_card_compare_finds_every_byte_with_skew` içinde WARP test arabelleği okuma/yazma (`copy_nonoverlapping`, `write_volatile`, `read_volatile`, D3D12 compute dispatch).
- **Karar:** **GÜVENLİ**

### (b) Diff Özeti
- **`5557224`** (+160 / -6, toplam 166 satır):
  - `crates/victory-gpu/src/host.rs` (+90 / -4):
    - `CARD_COMPARE_WIDE: &str` (`cs_5_1`, `[numthreads(64, 1, 1)]`, parça başına 1 thread group, `groupshared uint4 lanes[64]`, thread `t` 16-byte adımları `lo + t*16` ile `+= 1024u` tarar, `GroupMemoryBarrierWithGroupSync()` sonrası thread 0 64 lane hash'ini sırayla katlayıp `differs`/`hashes` yazar).
    - `card_hash_wide()`, `card_compare_source()`, `card_compare_groups(pieces)`.
    - `make_card_work` (`D3DCompile` kaynağı) ve `card_before` içindeki iki `Dispatch` çağrısı (`run.pieces.div_ceil(64)` → `card_compare_groups(...)`).
  - `crates/victory-gpu/src/indirect.rs` (+70 / -2):
    - `warp_card_compare_stores_the_hash_it_made_when_told` iki shader'ı da test eder.
    - Yeni `warp_card_compare_finds_every_byte_with_skew` testi (`skew = 0x3450`, `first = 1`, tek bayt değişiklik doğrulaması).
- **Env bayrağı:** `VICTORY_CARD_HASH_WIDE` — **varsayılan açık** (`!= "0"`). `VICTORY_CARD_HASH_WIDE=0` verildiğinde birebir eski `CARD_COMPARE` + `pieces.div_ceil(64)` kullanılır, davranış değişmez.

### (c) Uygulanabilirlik (`vw-pr10` Doğrulaması)
- **Yerel tabanda birebir eşleşen noktalar:**
  - [host.rs:1626-1666](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L1626-L1666): `pub(crate) const CARD_COMPARE: &str` — root signature, 6 elemanlı `Span` cbuffer, `t0`/`u0`/`u1` register'ları ve `compare` (0/1/2) semantiği **birebir aynı**.
  - [host.rs:3332-3352](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3332-L3352): `make_card_work` → `D3DCompile(CARD_COMPARE...)` — **birebir aynı**.
  - [host.rs:3507](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3507) ve [host.rs:3592](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/host.rs#L3592): `card_before` içindeki her iki `list.Dispatch(..., 1, 1)` — **birebir aynı**.
  - [indirect.rs:707-765](file:///C:/CLLL/.calisma/vw-pr10/crates/victory-gpu/src/indirect.rs#L707-L765): `warp_card_compare_stores_the_hash_it_made_when_told` — **birebir aynı**.
- **Bağımlı olduğu önceki commit'ler:** **Yok** (tamamen bağımsız; `QUEUE2`, `plaincopy.rs` veya `async_submit` gerektirmez).
- **Tahmini taşıma zorluğu:** **Kolay** (~160 satır, doğrudan ve temiz uyarlama).

### (d) Doğruluk Riski
- **Veri yarışı:** Yok (`groupshared` erişimi `GroupMemoryBarrierWithGroupSync()` ile senkronize; yalnız `t == 0` yazıyor; her group ayrı `piece` indeksine yazıyor).
- **Dispatch üst sınırı:** D3D12 X-dispatch limiti 65.535; `shadow::card_for` kopyaları `< 1 GiB` (≤ 16.385 parça) olduğu için limit içinde kalıyor.
- **Hash tutarlılığı:** `OnceLock` ile süreç başına tek kez seçildiği için saklanan ve karşılaştırılan hash'ler daima aynı algoritmayı kullanır.
- **Görsel bozulma raporu / AMD varsayımı:** Yok. PR #1014 yorumlarında 3 bağımsız NVIDIA doğrulaması mevcut (2× RTX 3060 12 GB + RTX 4080 16 GB; send-and-hash + compare süresi ~35→10 ms ve 37,4→7,0 ms, yeni görsel bozulma ve device loss yok). Saf HLSL `cs_5_1`, AMD'ye özgü varsayım yok.

---

## Önerilen İlk Aday

**Öneri: Aday 3 — `fxceless/victory-fork` PR #1014 (`5557224`).**  
Güvenli, dış bağımlılığı yok, `vw-pr10` ile satır satır uyumlu (`host.rs` + `indirect.rs`), tek bayrakla (`VICTORY_CARD_HASH_WIDE=0`) geri alınabiliyor, WARP birim testiyle birlikte geliyor ve 3 farklı NVIDIA kartta görsel bozulmasız doğrulanmış. İkinci sırada `CSTCryst` (`c271aff` + `dcdc980`) kontrollü port olarak değerlendirilebilir; `Belobug #123` ise Win11 duvar kaybı raporu çözülmeden ve bizim `relay 2` yolunda mapped geri yazma olmadığı için şu aşamada önerilmez.
