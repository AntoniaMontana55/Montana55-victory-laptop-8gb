# Neptune Indirect Clamp Yaması (`09222fa7`, `ce39c721`, `61f22060`) vs Bizim Ağaç (`vw-avatar`) Karşılaştırması

> **Kapsam:** `zm0kiemedia/Neptune_VI` commit'leri (`09222fa7`, `ce39c721`, `61f22060`) ile `C:\CLLL\.calisma\vw-avatar` (`nvidia-16gb`, taban `avatarmanz @ 0a435665`) ve son koşu `%LOCALAPPDATA%\Victory\kosular\20261126-050543_c0-ltosuz` (`ayarlar.txt`, `run.log`) karşılaştırması. Derleme/çalıştırma yapılmadı.

---

## 1. Neptune'de Yama Öncesi Durum ve Yama Mekanizması

- **Yama öncesi (`Neptune_VI @ 2c415c1d`):** Neptune tabanında (`347cac92`, `fxceless/victory-fork` kökenli) bizim ağaçtaki `indirect/batch.rs` (`VICTORY_INDIRECT_BATCH`) **hiç yoktur**. `VICTORY_INDIRECT_RAW=2` RTX 5070 Ti'de GPU hang yapınca `fb0c93c0` ile `VICTORY_INDIRECT_RAW=3` eklendi; bu mod yalnızca GPU shader'larının yazmadığı ve CPU'dan okunup doğrulanan tekil draw'ları raw geçirdi. Oyun içi cull/mesh indirect argümanları GPU shader'larınca yazıldığı için `RAW=3` bunları güvenli saymayıp tek tek per-draw `indirect::bound` (her draw öncesi 1 compute dispatch + 2 barrier) yoluna düşürdü (zm0kie ölçümü: **4.6 FPS**).
- **`61f22060` (`VICTORY_CLAMP_BATCH=1` + stale slot):** `VICTORY_CLAMP_BATCH=1` yeni bir batch shader değildir; mevcut `windows()` pencereleme yolunu açan bir alias'tır (`let windowed = (windows() || batch())`, `draw.rs`) ve `CLAMPS[3..5]` sayaçlarını ekler. Aynı commit'in `descriptor/host.rs` kısmı Neptune'e özgü 10 sn'lik `stale_keep_ms` zaman aşımını `VICTORY_STALE_KEEP_MAX_MS=120_000` (120 sn) tavanına uzatarak siyah gökyüzünü giderir.
- **`09222fa7` (`VICTORY_CLAMP_AFTER_WRITER=1`):** `draw.rs` tek-kayıt (`max == 1`, `count.is_none()`, `DRAW|INDEXED|MESH`) indirect çağrı adreslerini 2 kare boyunca `LEARNED` (`BTreeMap`) içinde öğrenir. `shadow.rs:gpu_writes` her shader yazım aralığını `HostList`'in `Deferred::san` (`San::wrote`) yapısına sıra numarasıyla işler. `list.gpu_written` olan bir kayıt okunurken temiz değilse, ilk okuyucuda tek bir `SANITIZE` compute shader'ı çalışır: son pass'ten beri yazılan aralıklarla çakışan tüm öğrenilmiş kayıtları (en fazla `MOST=32768`) **oyunun kendi tamponu üzerinde yerinde (in-place)** sınır dışıysa sıfırlar, öncesi/sonrası `uav_barrier` koyar ve `San::clean` işaretler. Aynı pass'in temizlediği sonraki tüm okuyucular artık clamp yapmadan **raw** `ExecuteIndirect` yürütür (zm0kie ölçümü: **4.6 → 8.4 FPS**).
- **`ce39c721`:** HLSL'de ayrılmış anahtar sözcük olan `cbuffer Pass` adını `cbuffer Sweep` yaparak `SANITIZE` derleme hatasını düzeltir ve `WHY[0..8]` fallback teşhis sayaçlarını ekler.

---

## 2. Bizim Ağaçta (`vw-avatar`) Aynı Yol ve Mevcut Anahtarlar

- **Per-draw clamp var mı?** Yalnız fallback, çoklu kayıt (`max > 1`) veya `count` buffer içeren çağrılarda (`crates/victory-gpu/src/draw.rs:3858` → `indirect::bound`, `crates/victory-gpu/src/indirect.rs:398-407`, `record_clamp_as` `indirect.rs:457-554`). Tek-kayıt (`max == 1 && count.is_none()`) çağrılarda per-draw clamp **çalışmaz**.
- **Bizde zaten bulunan ve son koşuda AÇIK olan anahtarlar (`ayarlar.txt`):**
  - **`VICTORY_INDIRECT_BATCH=2`** (`ayarlar.txt`, `draw.rs:223-282`, `draw.rs:3634-3695`, `draw.rs:3789-3797`, `indirect.rs:579-611`, `crates/victory-gpu/src/indirect/batch.rs:1-297`): Neptune'de **olmayan** üstün bir toplu clamp mekanizmasıdır. Yazım/sync sonrası ilk tek-kayıt indirect çağrıda **tek bir** `open_batch` compute dispatch'i (`JOBS=256`, `SOURCES=4`, `l.Dispatch(4, 1, 1)`) açılır; sonraki çağrılar CPU'dan upload tablosuna (`batch::Table::place`, `batch.rs:226-257`) yazılıp barrier olmadan scratch slot'tan okur. `=2` modunda compute dispatch'ler bile bir barrier/sync (`synced()`) veya draw gelene kadar batch'i kapatmaz (`draw.rs:3634-3639`, `3656-3660`, `3679-3681`). Çıkışta yalnızca `dst` için tek bir `UAV→INDIRECT_ARGUMENT` transition konur (`batch.rs:272-295`), ikinci `uav_barrier` yoktur.
  - **`VICTORY_INDIRECT_WINDOW=1`** (`ayarlar.txt`, `draw.rs:176-221`, `3804-3877`): Neptune'ün `VICTORY_CLAMP_BATCH=1` ile açtığı pencereleme yolunun birebir aynısıdır. Bizde `INDIRECT_BATCH=2` aktifken (`batch.is_some()`, `draw.rs:3795-3797`) `self.window = None` olduğu için yalnız batch dışı kalan çağrılarda yedek olarak devreye girer.
- **Bizde olmayan anahtarlar:** `VICTORY_INDIRECT_RAW`, `VICTORY_HOIST_CLAMPS`, `VICTORY_CLAMP_AHEAD`, `VICTORY_CLAMP_BATCH`, `VICTORY_CLAMP_AFTER_WRITER`, `VICTORY_STALE_KEEP_MS` bizim ağaçta **yoktur** (0 eşleşme). `descriptor/host.rs:2295-2321` doğrudan `unmake`/`unmake_stale` kullanır; Neptune'deki `stale_line`/`STALE_KEPT` zaman aşımı mekanizması bizde yoktur.

---

## 3. Maliyet Kanıtı (`20261126-050543_c0-ltosuz\run.log`)

- **Ölçülü olan (dispatch / barrier sayısı):** `diag` derlemesinde `copystat::C::IndirectBounded` her `record_clamp_as` (`indirect.rs:520`) ve `open_batch` (`indirect.rs:606`) çağrısında artar ve `copystat.rs:1357-1361` ile loglanır.
  - Dünya karelerinde (`run.log:13262–16618`, son 13 pencere): **kare başına `149–162` indirect clamp dispatch'i** (son 6 pencere: `156, 149, 159, 155, 151, 152`, ortalama **~154 dispatch/kare**).
  - Aynı karelerde **`3.786–4.421 draw/frame`** (`run.log:15885–16592`) var; yani `VICTORY_INDIRECT_BATCH=2` ~4.000+ indirect çağrıyı zaten **~154 batch dispatch'e** (~25–30 kayıt/dispatch) indirmiş durumda.
  - Karşılaştırma: Oyunun kendi barrier grupları **`672–724/kare`** (+ `9` buffer barrier).
  - Aynı pencerelerde kare bütçesi (`run.log:16119–16592`): **`166.6–172.7 ms/kare`** = CPU hazırlık **`80.1–101.2 ms`** + copy-back **`53.2–60.2 ms`** + GPU **`24.1–55.9 ms`** (son pencerelerde `24.1–35.6 ms`). Ayrıca `0.00 indirect records a frame ran empty for want of their list's budget` (`run.log:16592+`).
- **Ölçülemiyor (eksik sayaçlar):**
  1. Bu ~154 dispatch'in **kaçının `open_batch` vs `record_clamp_as` (tekil/window)** olduğu ve batch hit/miss oranı ayrı loglanmıyor → `crates/victory-gpu/src/indirect.rs:520` ve `indirect.rs:606` ayrı `copystat::C` sayaçlarına bölünmeli, `draw.rs:3791` için batch hit/open sayacı eklenmeli.
  2. Bu ~154 clamp dispatch'in **GPU ms maliyeti** logda yok → `indirect/batch.rs:280-296` ve `indirect.rs:538-552` etrafına D3D12 timestamp query (veya hızlı A/B için mevcut `VICTORY_BARRIER_TEST=indirect`, `copystat.rs:277`) ile GPU ms ölçülmeli.

---

## 4. Taşıma: Bizim Ağaca Uyum ve Bağımlılıklar

### Bağımlı Olduğu Neptune Commit'leri
1. **`347cac92` (Neptune tabanı) — `Deferred::gpu_ranges` & `HostList::gpu_written(addr, len)`:** Bizde `host.rs:2227` `Deferred::gpu_written` bir `HashSet<(usize, usize, usize)>` (dedup amaçlı) olup aralık sorgulayan `HostList::gpu_written(addr, len)` yoktur.
2. **`347cac92` — `STALE_KEPT` / `stale_line` / `stale_expire` (`descriptor/host.rs`):** `61f22060`'ın siyah gökyüzü düzeltmesi buna bağımlıdır; bizde bu altyapı yoktur.
3. **Bizde ek uyarlama gerektiren fark (`VICTORY_INDIRECT_CARD_BUDGET=1`):** `09222fa7` `budget` tamponunda 6. dword (`words.add(5)`, offset 20) kullanır; bizde `host.rs:6910`, `zero_card_budget` (`host.rs:6945`) ve `card_budget_out` (`host.rs:6986`, `CopyBufferRegion(..., 4, ..., 4, 16)`) yalnız 5 dword (20 byte) kapsar — güncellenmezse GPU-side budget'ta 6. dword sıfırlanmaz ve geri okunmaz.

### Hunk Bazlı Uyum Tablosu

| Commit | Dosya & Hunk | Uyum Durumu | Neden |
|---|---|---|---|
| `09222fa7` + `ce39c721` | `shadow.rs:3028-3039` (`gpu_writes` → `with_san`) | **Aynen uygulanır** | `self.place.is_some()` öncesi 5 satır eklenir. |
| `09222fa7` + `ce39c721` | `indirect.rs:308+` (`SANITIZE`, `San`, `LEARNED`, `sanitized`, `pass`) | **Uyarlama gerekir** | Bizdeki `make_from` (`indirect.rs:308`) `WIDEST`/`REST` makrosu ekler; `SANITIZE` için ayrı compile çağrısı gerekir. |
| `09222fa7` + `ce39c721` | `host.rs:2227, 6883-6993, 8570` (`Deferred::san`, `gpu_written`, `budget` word 5, `Meter`) | **Uyarlama gerekir** | `gpu_written(addr, len)` bizde yok; `zero_card_budget`/`card_budget_out` 6. dword'a genişletilmeli; `Meter` satırları farklı. |
| `09222fa7` + `ce39c721` | `draw.rs:3789-3909` (`learn` + `after_writer` → raw `ExecuteIndirect`) | **Çakışır / Uyarlama gerekir** | Bizde `raw_all()`/`RAW3` yok; `draw.rs:3789` doğrudan `batches()` yoluna girer. `after_writer` yalnız `DRAW\|INDEXED\|MESH` kapsarken `INDIRECT_BATCH=2` `DISPATCH`'i de toplar; iki yolun önceliği ve raw `ExecuteIndirect` (`draw.rs:3901`) yeniden yazılmalı. |
| `61f22060` | `draw.rs` (`VICTORY_CLAMP_BATCH=1`) | **Çakışır / Gereksiz** | Bizde `VICTORY_INDIRECT_WINDOW=1` (birebir aynı) ve `VICTORY_INDIRECT_BATCH=2` (daha üstün) zaten var ve açık. |
| `61f22060` | `descriptor/host.rs` (`stale_expire_ms`, `STALE_KEPT`) | **Çakışır / Uygulanamaz** | Neptune'e özgü `stale_line`/`stale_at` bizde yok (`descriptor/host.rs:2295-2321`). |

---

## 5. Beklenen Kazanç (`TAHMİN`), Risk ve Varsayılan Kapalı Taşınabilirlik

- **Beklenen kazanç (`TAHMİN`): ~0–3 ms/kare (<%2).**
  - *Gerekçe:* zm0kie'nin **4.6 → 8.4 FPS** kazancı Neptune'de `VICTORY_INDIRECT_BATCH` olmadığı için binlerce per-draw clamp'i kaldırmasından geldi. Bizde `VICTORY_INDIRECT_BATCH=2` zaten toplam clamp dispatch'ini **~154/kare**ye indirmiş; toplam GPU süresi yalnız `24–55 ms` iken kareyi belirleyen darboğaz **CPU hazırlık (`80–101 ms`)** ve **copy-back (`53–60 ms`)**. Üstelik `CLAMP_AFTER_WRITER` her yazıcı→okuyucu geçişinde yine 1 `SANITIZE` dispatch + 2 `uav_barrier` (bizim `batch.rs`'deki 1 `uav_barrier`'den fazla) üretir ve `DISPATCH` çağrılarını kapsamadığı için batch'leri bölüp kazancı sıfırlayabilir.
- **Riskler:**
  - **In-place mutasyon + Copy-back etkileşimi:** `SANITIZE`, oyunun kendi `Shadow::buffer`'ına doğrudan yazar (`RWByteAddressBuffer`). Sınır dışı bir kaydı sıfırladığında `Card::before` / CPU snapshot karşılaştırması bunu shader yazımı sanıp **oyun belleğine geri kopyalayabilir** veya `Shadow` state'iyle çakışabilir (bizim `batch.rs` oyunun tamponuna hiç dokunmaz, ayrı scratch'e yazar).
  - **TDR / GPU Hang:** `LEARNED` 2 karelik geçmişe güvenir; ilk kez görülen veya `gpu_writes` dışından yazılan kayıt kaçarsa raw `ExecuteIndirect` TDR tetikleyebilir.
  - **Siyah gökyüzü:** `61f22060`'daki düzeltme Neptune'ün kendi `stale_keep_ms` hatasını kapatır; bizde ilgili mekanizma yoktur.
- **Varsayılan kapalı env ile taşınabilir mi?** Teknik olarak `VICTORY_CLAMP_AFTER_WRITER=1` (varsayılan kapalı) ile taşınabilir, ancak mevcut ~154 dispatch/kare verisiyle getiri/karmaşıklık oranı çok düşüktür; eforu copy-back (`53–60 ms`) ve CPU hazırlık (`80–101 ms`) darboğazlarına yönlendirmek çok daha rasyoneldir.

---

**TAŞIMA** — Bizim ağaçta zaten açık olan `VICTORY_INDIRECT_BATCH=2` indirect clamp'leri kare başına ~154 dispatch'e indirmiş durumda; Neptune'ün 4.6→8.4 FPS kazancı bu batch mekanizmasının yokluğundan kaynaklandığı için yama bizde ihmal edilebilir kazanç (`TAHMİN` 0–3 ms) karşılığında in-place tampon mutasyonu ve copy-back/TDR riski getirir.
