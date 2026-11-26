# Güvenlik ve Taşınabilirlik İncelemesi: `fxceless/victory-fork` PR #1066 ve PR #648

- **Hedef taban:** `C:\CLLL\.calisma\vw-pr10` (dal `f67-standin`, ZSlayerHQ/zslayer-pc-port-runtime `next@02ea29c` + PR #10 + yerel format-67 yaması)
- **Yerel baz çizgisi:** Kare 263–272 ms (kopya 119–127 ms, geri kopya 143–149 ms, GPU 48 ms)
- **İnceleme yöntemi:** Salt okunur (`gh pr view/diff` + yerel `host.rs`, `shadow.rs`, `indirect.rs` doğrudan karşılaştırması; klonlama/derleme/`git apply` yapılmadı).

---

## 1. `fxceless/victory-fork` PR #1066 — Relay Geri Kopya Birleştirme Paralelleştirme

- **Başlık:** `GPU: parallelize relay copy-back merges with VICTORY_WIDE_MERGE=1`
- **Yazar / Durum / Commit:** `UwUCammie` · `OPEN` · `9be34adc` · 2 dosya (+175 / -39)

### 1.1. GÜVENLİK

| Kontrol kalemi | Bulgu |
|---|---|
| Ağ erişimi | Yok. |
| Dosya sistemi yazma/silme | Yok. |
| Süreç başlatma | Yok. |
| `build.rs` / `Cargo.toml` bağımlılık değişikliği | Yok (yalnız `crates/victory-gpu/src/host.rs` ve `shadow.rs`). |
| Gizlenmiş/obfuskasyonlu kod, ikili dosya, script (`.ps1`/`.sh`/`.bat`) | Yok. |
| `unsafe` blok eklemeleri ve bellek güvenliği | Aşağıda listelenen 2 mevcut `unsafe` kalıbının paralelleştirilmesi + 1 **kritik eşzamanlılık/UB hatası** tespit edildi. |

**İncelenen `unsafe` ve şüpheli satırlar (PR diff'ine göre):**
- `crates/victory-gpu/src/host.rs:4708` (`relay_card_back`, yeni paralel kol) ve `host.rs:4895` (`relay_card_inflight`, yeni paralel kol):
  `unsafe { std::slice::from_raw_parts((data + at as usize) as *const u8, hi - lo) }` — Mevcut seri döngüdeki okumanın `shadow::wide_run` işçilerine taşınmış hali. `wide_run` tüm işçiler bitene kadar blokladığı için eşleme canlı kalır; salt okunur dilimdir.
- `crates/victory-gpu/src/shadow.rs:3062` (`merge_under_writes`) ve `shadow.rs:3544-3550` (`card_merge_relay`):
  `self.before` (`Mutex`) yerine `self.card_merge_lock` (`RwLock<()>`) getiriliyor. `merge_under_writes` yazma kilidi (`write()`), eşzamanlı `card_merge_relay` işçileri okuma kilidi (`read()`) alarak `unsafe { std::slice::from_raw_parts_mut(self.data.add(at), gpu.len()) }` ile `self.data`'ya yazıyor.
- **Bellek güvenliği riski (kötü amaçlı değil, Rust aliasing UB / veri yarışı hatası) — `crates/victory-gpu/src/host.rs:4693-4694` (`relay_card_back`):**
  ```rust
  let key = (std::sync::Arc::as_ptr(&shadow) as usize >> 6) ^ (lo >> 16);
  buckets[key % BUCKETS].push((shadow, lo, hi, at));
  ```
  `Shadow::card_piece` parçaları kopya ofseti `lo`'ya göre değil, **başlık adresi** `start + lo`'nun 64 KiB sınırlarına göre keser (`skew = start % 65536`). Eğer bir `Shadow` 64 KiB hizalı değilse (`skew != 0`) ve inişteki iki farklı `CardPlan` aynı 64 KiB başlık parçasının farklı alt aralıklarını bildirdiyse (ör. `[61440, 126976)` ve `[65536, 126976)`), `seen` bunu tekilleştiremez, `lo >> 16` farklı kova üretir ve iki işçi thread'i aynı anda `card_merge_lock.read()` tutup `self.data`'nın örtüşen baytları üzerinde `&mut [u8]` oluşturur. Yazar bu riski `relay_card_inflight` içinde `single_page` kontrolüyle (`host.rs:4870`) fark etmiş, ancak `relay_card_back`'e eklememiştir.

**Sonuç:** **GÜVENLİ (zararlı kod yok)** — ancak `host.rs:4693-4694`'teki `lo >> 16` kova anahtarı `skew != 0` durumunda veri yarışı/UB riski taşır; taşırken `(shadow_ptr, (start + lo) >> 16)` veya `single_page` korumasıyla düzeltilmelidir.

### 1.2. NE YAPIYOR

- `VICTORY_RELAY=2` açıkken tek iniş thread'inde seri çalışan geri kopya birleştirme döngülerini (`relay_hand_back`, `relay_card_back`, `relay_card_inflight`) mevcut `shadow::wide_run` işçi havuzuna (en fazla 8 kova) dağıtır.
- `Shadow` üzerine `card_merge_lock: RwLock<()>` ekleyerek `merge_under_writes` (write lock) ile farklı parçalardaki `card_merge_relay` işçilerinin (read lock) birbirini beklemesini kaldırır.
- Aynı kopya/sayfaya dokunan parçaları aynı kovaya koyarak plan sırasını korur.
- **Env bayrağı:** `VICTORY_WIDE_MERGE=1` (varsayılan: **kapalı / `0`**; yalnızca en az 8 parça/aralık olduğunda devreye girer).

### 1.3. TAŞINABİLİRLİK (`vw-pr10` karşılaştırması)

| Dosya / Fonksiyon | `vw-pr10` durumu | Eşleşme ve farklar |
|---|---|---|
| `host.rs` — `relay_hand_back` (`host.rs:2595`) | **Var** | Yapı birebir aynı (~775 satır ofset). Fark: PR `copy_back(1, written as u64)` çağırıyor; bizde `COPY_BACK[1].fetch_add(written as u64, Ordering::Relaxed)` (`host.rs:2625`). |
| `host.rs` — `relay_card_back` (`host.rs:3637`) | **Var** | Hunk çevresi birebir aynı (`CARD_STATS[5..6]`, `card_merge_relay`, `walls_merged`). Fark: yine `copy_back(...)` → `COPY_BACK[1].fetch_add(...)` (`host.rs:3747`) + `lo >> 16` UB düzeltmesi eklenmeli. |
| `host.rs` — `relay_card_inflight` | **Yok** | PR #648'e bağlı 3. hunk; `vw-pr10`'da yok, tamamen atlanır. |
| `host.rs` — `wide_merge()` (`host.rs:1698`) | **Var** | Birebir aynı. |
| `shadow.rs` — `Shadow`, `Shadow::new`, `merge_under_writes`, `card_merge_relay`, `wide_run` | **Tümü var** | (`shadow.rs:28, 1710, 2247, 2710, 1054`). Kod bağlamı birebir aynı, yalnız satır ofsetleri farklı. |

- **Hunk'lar doğrudan uygulanır mı?** **Hayır.** `relay_card_inflight` yokluğu ve `copy_back()` yardımcı fonksiyonu farkı nedeniyle doğrudan `git apply` reddeder.
- **Başka PR/commit bağımlılığı:** `relay_card_inflight` hunk'ı PR #648'e, `copy_back()` çağrısı fxceless'teki ufak bir helper commit'ine bağımlı; ancak bunlar olmadan mevcut 2 relay yolu bağımsız ve eksiksiz taşınabilir.
- **Uyarlama tahmini:** 2 dosya, **~115 ekleme / ~30 silme (toplam ~145 satır)**; 0 MB ek VRAM.

### 1.4. ÖLÇÜM

- **Donanım / Ortam:** AMD RX 7900 XT, Linux + Wine 11.22 (RADV), `VICTORY_RELAY=2`, `VICTORY_WIDE_MERGE=1`, `VICTORY_BACK_OWN_VIEW=1`.
- **Yöntem:** Hücre sahnesine kadar 480 saniyelik koşu.
- **İddia edilen kazanç:** Geri kopya süresi **28,2–32,8 ms → 17,2–17,6 ms/kare** (~%45 hızlanma); 0 GPUVM fault, 0 reddedilen bekleme, görüntü değişmedi.
- **Not:** Windows veya NVIDIA/8 GB üzerinde PR içinde bağımsız ölçüm yok; kazanç CPU tarafındaki birleştirme (merge) yüküne bağlıdır.

---

## 2. `fxceless/victory-fork` PR #648 — Uçuş İçi Kart Fetch (`VICTORY_CARD_FETCH_INFLIGHT`)

- **Başlık:** `Card copies' pieces come down inside their own flights: relay fetch wait 139 ? 0 ms/frame (VICTORY_CARD_FETCH_INFLIGHT=1, off by default)`
- **Yazar / Durum / Commit'ler:** `Jonirulah` · `OPEN` · `ca2513c0` (ilk sürüm) + `4d284865` (DMA + copy queue revizyonu) · 3 dosya (+1087 / -7)

### 2.1. GÜVENLİK

| Kontrol kalemi | Bulgu |
|---|---|
| Ağ erişimi | Yok. |
| Dosya sistemi yazma/silme | Yok. |
| Süreç başlatma | Yok. |
| `build.rs` / `Cargo.toml` bağımlılık değişikliği | Yok (yalnız `host.rs`, `indirect.rs`, `shadow.rs`). |
| Gizlenmiş/obfuskasyonlu kod, ikili dosya, script (`.ps1`/`.sh`/`.bat`) | Yok. |
| `unsafe` blok eklemeleri ve bellek güvenliği | Aşağıdaki tüm bloklar incelendi; hepsi mevcut `host.rs` D3D12 desenleriyle uyumlu ve sınır kontrollü. |

**İncelenen `unsafe` satırları (PR diff'ine göre):**
- `crates/victory-gpu/src/host.rs:3415-3435` (`make_card_inflight`): `CreateRootSignature`, `CreateComputePipelineState`, `std::mem::transmute_copy(&root)` — yerel `make_card_work` (`host.rs:3332-3342`) ile aynı D3D12 oluşturma kalıbı.
- `crates/victory-gpu/src/host.rs:3640` (`card_before`): `(inflight.table.1 as *mut u32).add(base as usize).write_volatile(0)` — `take_region` `base < slots` garantiler, `table` boyutu `slots * 4 + results`.
- `crates/victory-gpu/src/host.rs:3679-3731` (`card_before` mevcut `unsafe` bloğu içi): UAV bariyeri + `CARD_FETCH` HLSL compute shader 2 aşamalı dispatch (stage 0 ticket, UAV bariyer, stage 1 kopya). HLSL kodu (`host.rs:1765-1813`) açık ve okunur; `at < hi` ve `k < count` kontrolleriyle `staging` sınırları içinde kalır.
- `crates/victory-gpu/src/host.rs:3791-3813` (`make_inflight_down`): `D3D12_COMMAND_LIST_TYPE_COPY` kuyruğu, allocator, list, fence ve `CreateEventW`. (`Drop` yok ama `relay_worker` süreç boyunca yaşar; `RelayFetch` ile aynı.)
- `crates/victory-gpu/src/host.rs:4056-4062` (`relay_card_inflight`): `CARD_WORK` kilidi altında `differs`, `counters`, `entries` için `from_raw_parts` (salt okunur, boyutlar `dwords` ve `slots` ile eşleşiyor).
- `crates/victory-gpu/src/host.rs:4178-4195` (`relay_card_inflight`): Ayrı COPY kuyruğunda `CopyBufferRegion` + fence sinyali + `WaitForSingleObject`.
- `crates/victory-gpu/src/host.rs:4209` (`relay_card_inflight`): `(data + region as usize + at) as *const u8` için `from_raw_parts` (`into` ve `hi - from <= INFLIGHT_SLOT` ile sınır kontrollü).
- `crates/victory-gpu/src/indirect.rs:960-1022` ve `crates/victory-gpu/src/shadow.rs:1178`: Yalnız `#[test]` altında WARP birim testleri.

**Sonuç:** **GÜVENLİ** (zararlı kod, ağ/dosya/süreç erişimi veya bağımlılık değişikliği yok; `unsafe` kullanımı mevcut D3D12 mimarisiyle tutarlı). *Not:* Aşağıda ölçüm bölümünde belirtildiği üzere güvenlik değil **görüntü doğruluğu / taşma (overflow) semantiği** açısından topluluk testlerinde açık sorular var.

### 2.2. NE YAPIYOR

- Relay açıkken, compare listesinin hemen arkasına `CARD_FETCH` adlı 2 aşamalı compute geçişi ekler: değişen 64 KiB parçalar için bilet alır (stage 0) ve bunları kart belleğindeki 64 MiB'lık `staging` tamponunda uçuşun kendi bölgesine kopyalar (stage 1).
- Uçuş indiğinde `relay_card_inflight`, ana DIRECT kuyruğundaki yeni uçuşların arkasına takılmadan ayrı bir D3D12 `COPY` kuyruğundan tek `CopyBufferRegion` ile bu slotları indirip `card_merge_relay` ile birleştirir.
- Bölgesine sığmayan veya karşılaştırılmayan parçalar eski fetch yoluna düşer; `fallback_fetched_mark` ve `brought` ile karışık iniş sırası korunur.
- **Env bayrakları:** `VICTORY_CARD_FETCH_INFLIGHT=1` (varsayılan: **kapalı / `0`**, yalnızca `VICTORY_RELAY` etkinken çalışır), `VICTORY_CARD_FETCH_INFLIGHT_MB` (varsayılan: **64 MiB** VRAM), `VICTORY_CARD_FETCH_INFLIGHT_MAX_KB` (varsayılan: **sınırsız / tamponun 1/4'ü**).

### 2.3. TAŞINABİLİRLİK (`vw-pr10` karşılaştırması)

| Dosya / Fonksiyon | `vw-pr10` durumu | Eşleşme ve farklar |
|---|---|---|
| `host.rs` — `CardWork`, `CARD_COMPARE`, `CardPlan`, `QUEUE_ORDER`, `RELAY_STATS`, `RelayFetch`, `late_line` | **Tümü var** | (`host.rs:1589, 1626, 1716, 1980, 2511, 2683`). Bağlam satırları birebir aynı (~100–125 satır ofset). |
| `host.rs` — `compile`, `shared_buffer`, `uav_buffer`, `make_card_work`, `card_before`, `make_relay_fetch`, `relay_card_back` | **Tümü var** | (`host.rs:1073, 7203, 7255, 3302, 3367, 3611, 3637`). İmzalar, `COPY_BACK[1]`, `walls_merged`, `card_sides`, `CARD_PASSES[2]` birebir aynı (~290 satır ofset). |
| `indirect.rs` — `WarpHarness`, `compile_cs`, `uav_barrier`, `shared_buffer`, `default_uav`, `argument_clamp_builds` | **Tümü var** | (`indirect.rs:455, 488, 545, 588, 663, 769`). İmzalar birebir aynı (~45 satır ofset). |
| `shadow.rs` — `card_tests`, `outside`, `window_tests`, `near`, `Card`, `Shadow::new` | **Tümü var** | (`shadow.rs:395, 579, 3160, 3267, 84, 1710`). Birebir aynı (~3 satır ofset). |

- **Hunk'lar doğrudan uygulanır mı?** **Evet, neredeyse temiz uygulanır.** PR'ın taban commit'i (`492ae72ac`) `vw-pr10` ile neredeyse birebir örtüşüyor; yalnızca satır numarası ofsetleri var (en fazla 1–2 satır bağlam ayarı gerekebilir).
- **Başka PR/commit bağımlılığı:** **Yok.** (Not: Linux/Wine altında `D3DCompile` reddettiği için `dxc` gerekir, ancak Windows yerel koşusunda `host.rs:1073` `compile()` doğrudan çalışır.)
- **Uyarlama tahmini:** 3 dosya, **~1.087 ekleme / ~7 silme** (~310 satırı birim/WARP testi); **+64 MiB VRAM** (`staging`) + dinamik indirime tamponu maliyeti.

### 2.4. ÖLÇÜM

- **PR başlığındaki ölçüm (ilk sürüm `ca2513c`, DMA öncesi):**
  - Donanım/Ortam: **RTX 3070 Laptop 8 GB**, Ryzen 7 5800H, Windows 11, **Series S**, diagnostics build, dünya açıldıktan 2 dk sonra 45–55 örnek.
  - Sonuç: Kare **353 → 312 ms** (ilk deneme 305 ms), relay fetch beklemesi **138,7 → 0 ms**, geri kopya **236,5 → 119 ms**, GPU **89,6 → 55,6 ms** (62–79 parça / 3,2–3,9 MB, 0 taşma).
- **Yorumlardaki bağımsız ve son revizyon (`4d28486` — DMA + copy queue) ölçümleri:**
  - `VibeClauding` (RTX 3070 8 GB masaüstü, 5700X3D, Win11, Series S, `ca2513c`): 164–174 → 126–153 ms (+%24 FPS), geri kopya 88–98 → 32–36 ms.
  - `CuteBooty` (RTX 3070 8 GB, R5 3600, Win11, **Series S**, `4d28486`): Geri kopya **102 → 24 ms** düştü, ama GPU **77 → 122 ms**, hazırlık **92 → 110 ms**, kare **221 → 250 ms** (kare uzadı; aynı sahnede yalnızca copy queue kullanan PR #665 GPU'yu şişirmeden 212,7 → 201,4 ms verdi).
  - `oeirughrepuiohg` (RTX 4070 Ti, Linux/Proton, Series X, `4d28486`): **161–163 → 148–154 ms** (+%6–9), GPU 42,8 → 30,6–34,5 ms.
  - `iGoldenNinja` (RTX 5080, Win11, Series X, `4d28486`): 3,87 → 4,12 FPS, fetch wait 139,1 → 46,8 ms; **ancak duvar boşlukları, saç çizgileri ve tavan eksikliği gibi görsel bozulmalar** raporlandı.
  - `AaronMacArthur` (RTX 4080 SUPER, Series S, `ca2513c`): 5,35 → 5,69 FPS (+%6,25), **ancak avlu tel örgüsü kayboldu / duvar dokusu bozuldu**.
  - `Pin0d1` (RTX 2060 SUPER & WARP, `4d28486`): Taşma (overflow) durumunda ilk inişin daha sonraki flight'ın verisini (17 yerine 29) okuduğunu doğruladı.

---

## 3. Öneri: Önce Hangisi Taşınmalı?

**Tek öneri: Önce PR #1066 taşınmalı** (yalnız mevcut `relay_hand_back` ve `relay_card_back` yolları + `shadow.rs` `card_merge_lock`, `lo >> 16` kova anahtarı `(start + lo) >> 16` veya `single_page` ile düzeltilerek).

### 5 Satır Özet

1. Her iki PR de zararlı kod, ağ/dosya/süreç erişimi veya `Cargo.toml`/`build.rs` bağımlılığı içermiyor (**GÜVENLİ**); ancak PR #1066'nın `relay_card_back` kovalama anahtarında (`lo >> 16`) `skew != 0` için düzeltilmesi gereken bir Rust aliasing/veri yarışı riski var.
2. PR #1066 çok daha küçük (~145 satır, `relay_card_inflight` hariç), **0 MB ek VRAM** harcar, görsel bozulma raporu yoktur ve bizde zaten açık olan `RELAY=2` yolunda `VICTORY_WIDE_MERGE=1`'i işler hale getirir.
3. PR #648 `vw-pr10` ile neredeyse %100 bağlam uyumlu olsa da ~1.090 satır, **+64 MiB VRAM** ister ve son `4d28486` sürümünde 8 GB RTX 3070 Series S'te GPU süresini artırıp kareyi uzatan rapor (`CuteBooty`) ile görsel bozulma raporları (`iGoldenNinja`, `AaronMacArthur`) mevcuttur.
4. PR #648'in asıl hedeflediği `relay waiting for fetches` metriği yerel logda sürekli yüksek (ör. ≥25–40 ms) çıkmadıkça 64 MiB VRAM ve görsel regresyon riskine girmeye gerek yoktur.
5. Önce PR #1066'yı kova düzeltmesiyle taşıyıp 143–149 ms geri kopya üzerindeki etkisini ölçün; ardından logdaki `waiting for fetches` yüksek kalırsa PR #648'i ikinci adımda (sıkı görsel kontrolle) değerlendirin.
