# Write Marks (Sayfa Düzeyi Yazma İşareti) Kapsam ve Taşıma Analizi

- **Tarih:** 2026-11-26
- **Hedef ağaç:** `C:\CLLL\.calisma\vw-avatar` (dal `nvidia-16gb`, taban `avatarmanz/VIctory-7900XTX @0a435665`)
- **Referans:** `Marvisak/VIctory @main`
- **Problem (run.log kanıtı):** Dünya karelerinde `shader-written ranges: 509 steps, snapshots 14.6 ms, 240.4 MB compared back` + `what shaders wrote: 16.7 ms finding 0 pieces that differ` (~31 ms/kare boşa CPU işi); en büyük aralıklar 8–12 MB, geri yazılan ~0 MB, kaynak `"a view in a table"` (heap tablosundaki UAV görünümleri).

---

## 1. Marvisak `main` Commit / PR Zinciri (Sıralı)

### Ön Koşul Temeller (bizim ağaçta yok)
| Sıra | SHA / PR | Tarih | +/- Satır | Rol | Ana Dosyalar |
|---|---|---|---|---|---|
| 0a | `27c3dc9d` | 2026-11-24 | +2090/-99 | **Temel (page-table)** — `MEMORY_SPACE`, iki seviyeli sayfa tablosu, `HostShader::memory_writes` + raw-address dirty bitmap | `memory.rs`, `emit/{lower,mod,shader}.rs`, `draw.rs`, `shadow.rs`, `shader_cache.rs` (21 dosya) |
| 0b | `c75fdba3` | 2026-11-24 | +2180/-435 | **Temel (bindless + watch)** — `bindless.rs`, birleşik 64 KiB sayfa koruma/değişiklik günlüğü (`watch.rs`), `HostShader::{heap, heap_writes}` | `bindless.rs`, `watch.rs`, `shadow.rs`, `descriptor/host.rs`, `emit/{lower,mod}.rs` (32 dosya) |
| 0c | `ed0dbfd1` | 2026-11-24 | +76/-1 | **Ön koşul** — `Shadow::update_busy`: gönderim içi meşgul aralıklardaki title yazmalarını kaçırmama | `shadow.rs` |

### Ana Write Marks Commit'i ve Zorunlu Main Düzeltmeleri
| Sıra | SHA / PR | Tarih | +/- Satır | Rol | Ana Dosyalar |
|---|---|---|---|---|---|
| **1** | **`cd94d353`** | **2026-11-25** | **+1058/-78** | **TEMEL (`view_marks`)** — `Options::view_marks`, `HostShader::view_marks` (`<<5`), `Lower::mark_view_write`, `t[VIEW_MARKS]`/`u[VIEW_WRITES]`, `copy_back_marks` / `copy_back_marked`, `Tracked::saved` | `shadow.rs` (+362/-15), `emit/lower.rs` (+118/-8), `emit/shader.rs` (+80/-3), `watch.rs` (+76/-13), `descriptor/host.rs` (+68/-4), `emit/mod.rs` (+56/-4), `draw.rs` (+53/-17), `pipeline/host.rs` (+31), `host.rs` (+23/-9), `shader_cache.rs` (+9/-2) (20 dosya) |
| 2 | `940d32f6` | 2026-11-25 | +8/-0 | **Düzeltme (zorunlu)** — İşaretli copy-back kendi kopyası için sayfayı tekrar `saved` yapmaz | `shadow.rs`, `watch.rs` |
| 3 | `19533398` | 2026-11-25 | +41/-5 | **İyileştirme** — Title'ın yazmadığı temiz kopyaları gönderimde tek bayrakla atlama | `watch.rs`, `shadow.rs`, `host.rs` |
| 4 | `9337b35e` | 2026-11-25 | +131/-38 | **Düzeltme** — Title-memory miss sırasında yazmaları ayrı bitmap'te (`view_writes(true)`) tutup geri yazmayı durdurma | `shadow.rs`, `memory.rs`, `host.rs`, `draw.rs` |
| 5 | `80d8cf53` | 2026-11-25 | +47/-10 | **Düzeltme** — `take_reads`'in iki bitmap'i birden okuması (yalnız read-marks varsa gerekir) | `shadow.rs`, `host.rs` |
| 6 | `2bba7921` (#1906) | 2026-11-25 | +140/-7 | **Düzeltme (NVIDIA kritik)** — 1M heap sınırında dinamik slotlarda view-address tablosu temizleme/yazma | `descriptor/host.rs`, `draw.rs` |
| 7 | `4106550f` (#2054) | 2026-11-25 | +41/-18 | **İyileştirme** — Shadow writeback birleştirme hızlandırması | `shadow.rs` |
| 8 | `e9763443` (#2155) | 2026-11-25 | +212/-2 | **Düzeltme (zorunlu)** — Depth-twin compute dönüşümü sonrası `VIEW_MARKS`/`VIEW_WRITES` root descriptor'larını yeniden bağlama | `draw.rs`, `pipeline.rs` |
| 9 | `53193663` (#2172) | 2026-11-25 | +473/-66 | **Düzeltme (zorunlu)** — İşaretli UAV yenilemelerini tam 64 KiB sayfalara genişletme (`Shadow::page_range`); yoksa `Tracked::take` sayfayı korumaz ve GPU yazmaları düşer | `draw.rs` (+387/-3), `shadow.rs` (+79/-61), `bindless.rs` |
| 10 | `106377d1` (#2377) | 2026-11-25 | +71/-2 | **Düzeltme (zorunlu)** — Kısmi yenilemede `Tracked::saved` güncelleme (`Tracked::refreshed`) | `shadow.rs`, `watch.rs` |
| 11 | `e815cef2` (#2484) | 2026-11-25 | +241/-37 | **Düzeltme (page-table)** — 256 MiB / 4 GiB sınırını aşan ham adres yazmalarında son bayt düzeltmesi | `emit/lower.rs`, `emit/memory_tests.rs` |

### Açık İyileştirme PR'ları (`main`'de değil)
- **#2242** (+214/-60) → **#2280** (+274) → **#2409** (+332/-62, `shadow.rs`): İşaretli sayfalarda 1 KiB/32 B blok karşılaştırma, değişmeyen sayfalarda `take_others` ve ikinci `take` atlama, zero-word atlama.
- **#2465** (-48, 4 dosya): `VIEW_WRITES` bitmap'ini sistem RAM'den VRAM'a (`DEFAULT` UAV + readback + zero-upload) taşıma.
- **#2914** (+91/-4) & **#2918** (+612/-45): Çok iş parçacıklı marked-page copy-back + düz store ile yalnızca değişen baytları yazma.
- **#2948** (+84/-9, 15 dosya): `HostShader::written_root_uavs` (DexNom@`bc3cd4b`'den) — salt okunan root UAV'ları geri yazmama (`VSH2`).
- **#2958** (+69/-0, `lower.rs` + `lower/marks.rs`): Wave başına tek lane (`waveReadLaneFirst`/`waveIsFirstLane`, PS'de non-helper min lane) mark atomiği; sistem RAM PCIe bus trafiğini kırar.

---

## 2. Mekanizma (6–10 Cümle)

Çevirici (`Options::view_marks = true`), heap tabanlı herhangi bir unordered buffer görünümü üzerinden yapılan her store ve atomic işlemine `Lower::mark_view_write` ekler (`cd94d353`, `emit/lower.rs`, `emit/mod.rs`). Her host view heap'i slot başına iki dword tutan `t[VIEW_MARKS]` (kök SRV, `MEMORY_SPACE`) tablosu barındırır: `base` (kopyanın bitmap başlangıç biti; `VIEW_MARK_NONE` ise işaretleme yok) ve `phase` (görünümün ilk baytının 64 KiB sayfa içi ofseti, 0–65535). Shader, ofset `x`'teki erişimin ilk ve son baytı için `bit = min(base + ((phase + x) >> 16), VIEW_WRITE_BITS - 1)` hesabını yapar; `u[VIEW_WRITES]` bitmap sözcüğünü okur ve bit 0 ise `InterlockedOr` ile 1'ler. Host tarafında her izlenen (`Tracked`) kopya, dokunduğu her 64 KiB title sayfası için bu bitmap'ten ardışık bir bit aralığı alır; bu sayede gönderim öncesi tüm aralıkların dev snapshot'ı alınmaz. Gönderim bitince `copy_back_marks` → `Shadow::copy_back_marked` yalnızca biti 1 olan 64 KiB sayfaları tarayıp title belleğiyle karşılaştırır ve geri yazar. Gönderim sürerken title CPU'su da aynı sayfaya yazarsa, sayfa koruması (`watch.rs`, `Tracked::saved`) ilk title yazmasından önceki kopya baytlarını saklar; böylece hem GPU hem CPU yazmaları birbirini ezmeden birleştirilir. **Bizim sorunumuzu (heap tablosundaki UAV görünümleri — `"a view in a table"`) doğrudan kapsar:** `cd94d353` açıklaması gereği tablo görünümleri, heap-doğrudan erişimler (`heap_writes`) ve heap slotu üzerinden açılan root UAV'lar aynı `t[VIEW_MARKS]`/`u[VIEW_WRITES]` yolundan işaretlenir. Ham adres (page-table) yazmaları ayrı olarak `memory_writes` bitmap'ini kullanır.

---

## 3. Bizim Ağaçta Karşılıkları ve Ortak Ata

### Ortak Ata (Doğrulanmış)
- `avatarmanz/VIctory-7900XTX @0a435665` (ve `fxceless/victory-fork`) içinde `28494774`, `8c26061f` ve **`35b054ec`** (2026-11-24, *"GPU: pipelines off the render thread and cached; indirect execution on"*) **mevcut**.
- Hemen sonrasındaki `27c3dc9d` (page-table) ve `c75fdba3` (`bindless.rs` + yeni `watch.rs`) **yok**.
- Sonuç: İki ağaç **tam olarak `35b054ec` (2026-11-24)** noktasında ayrılmış; `fxceless` kendi yolunda (`picked.rs`, `SLOT_MEMORY` dizin/arena, VRAM `card` kopyaları + GPU 128-bit hash/compare, `relay`, `inplace`) ilerlemiş.

### Dosya / Sembol Eşleştirmesi (`C:\CLLL\.calisma\vw-avatar`)
| Bileşen | Bizde VAR | Bizde YOK (Marvisak `main`) |
|---|---|---|
| Shader bayrakları | `HostShader` (`crates/victory-shader/src/emit/mod.rs:335-357`): `root_cbvs`, `root_srvs`, `root_uavs`, `root_indexed_views` | `Options::view_marks`, `translate_with`, `HostShader::{memory, memory_writes, view_addresses, heap, heap_writes, view_marks}` |
| Shader uzayları | `ROOT_CBV_SPACE`, `SLOT_MEMORY=254`, `SLOT_STRIDES`, `SLOT_FAULTS` (`crates/victory-shader/src/emit/shader.rs:45-87`) | `MEMORY_SPACE`, `VIEW_MARKS`, `VIEW_WRITES`, `VIEW_MARK_NONE`, `VIEW_WRITE_BITS`, `HEAP_VIEWS` |
| Shader cache | `VSH1` (`crates/victory-gpu/src/shader_cache.rs:179, 216-340`) | `view_marks << 5` / `heap_writes << 4` alanları, `VSH2` |
| Yazma eleme | Shader-düzeyi `write_skip()` (`draw.rs:151`), `STORES` thread-local (`draw.rs:68, 1492, 4133`), `pipeline::stores()` (`pipeline.rs:368-387`) | Slot-düzeyi veya sayfa-düzeyi (`Lower::mark_view_write`) işaretleme |
| Tablo UAV yolu | `State::tables()` (`draw.rs:3235-3305`) → `readable()` (`draw.rs:4122-4145`) → `shadow.gpu_writes()` (`shadow.rs:3028`) | `VIEW_MARKS` / `VIEW_WRITES` root bağlama + `Shadow::page_range` |
| Kopya/Karşılaştırma | CPU snapshot (`shadow.rs:3050-3100`) + **VRAM kart kopyası & GPU 128-bit hash/compare** (`shadow.rs:217-248, 3068`; `host.rs:2104-2135, 4295-4345`, `VICTORY_CARD_BUFFERS*`, `VICTORY_CARD_REHASH_RUNS`) | `copy_back_marks`, `copy_back_marked`, `shadow::view_writes()` bitmap |
| Sayfa izleme | `Tracked` (`watch.rs:101`, `take` :762) | `Tracked::saved`, `Tracked::refreshed` (armed-page koruması) |
| Bindless / Heap | `picked.rs` (388L), `descriptor/host.rs` (5542L, `HostHeap::strides_gpu` :1716) | `bindless.rs`, `HostHeap` per-slot view-mark tablosu |

---

## 4. Taşıma Tahmini (Write Marks)

- **Toplam satır (TAHMİN):** Çekirdek `cd94d353` (+1058/-78) + zorunlu düzeltmeler (`940d32f6`, `2bba7921`, `e9763443`, `53193663`, `106377d1`, toplam +904/-77) + NVIDIA için pratikte şart olan `#2958` (+69) ≈ **~1.800–2.300 satır** uyarlama (kendi `ROOT_CBV_SPACE`/`SLOT_STRIDES` düzenimize oturtarak `27c3dc9d`/`c75fdba3` taşınmadan yazılırsa).
- **Çakışma riski ÇOK YÜKSEK dosyalar:** `crates/victory-gpu/src/shadow.rs` (5606L — `card`/`relay`/`inplace` ile ağır farklı), `host.rs` (10434L), `draw.rs` (4471L), `watch.rs` (4385L), `descriptor/host.rs` (5542L), `crates/victory-shader/src/emit/lower.rs` (13374L).
- **Bağımlı değişiklikler:**
  1. `watch.rs`'ye `Tracked::saved` + `refreshed` (title CPU ile GPU eşzamanlı sayfa yazması bozulmasın diye zorunlu).
  2. `descriptor/host.rs` `HostHeap`'e `strides_buffer` benzeri per-slot `view_marks` SRV tamponu (dinamik slotlar dahil, `#1906`).
  3. `shader_cache.rs` sürüm artışı (eski önbellek geçersizleşir).
  4. NVIDIA'da sistem RAM bitmap atomikleri çok pahalı (`#2465`: 1M `InterlockedOr` 18 ms vs VRAM 0.03 ms; `#2958`: RTX 4070 SUPER'da 411→283 ms); en azından `#2958` wave-coalescing birlikte taşınmalı.
- **Kademeli mi?** **Evet.** `Options::view_marks` (örn. `VICTORY_VIEW_MARKS=1`, varsayılan kapalı) + ayrı cache anahtarı ile güvenle A/B test edilebilir.

---

## 5. Ölçülmüş Kazanç Kanıtları (Marvisak PR / Commit Metinlerinden)

| Kaynak | Donanım / Sahne | Ölçülen Etki |
|---|---|---|
| `cd94d353` (temel `view_marks`) | Intro-cutscene / dünya | Öncesi: 2.8–4.5 GB/kare snapshot+compare (%98 değişmeyen), 1.5–1.7 s karenin **0.45–0.70 s**'si; `view_marks` ile yalnızca işaretli 64 KiB sayfalar karşılaştırılır (sonrası kesin ms commit metninde yok — TAHMİN: bizim 31 ms CPU payının ~%70–90'ı). |
| `#2948` (DexNom@`bc3cd4b` referansı) | Radeon build | `written_root_uavs` + page-table write marks birlikte: **3,469 → 969 ms/kare**, karşılaştırılan bellek **12.8 GB → 2.7 GB/kare**. |
| `#2409` (marked-page gereksiz iş atlama) | Win11, RTX 5090, 300 s dünya (n=26) | Copy-back **142.3 → 116.2 ms** (marked pages 133→109 ms), kare **466.7 → 455.6 ms** (2.1→2.2 fps). 4 KiB alt-işaret denemesi daha kötü (124.7 ms, GPU +7 ms). |
| `#2958` (wave başına tek lane mark) | Win11, RTX 4070 SUPER, 15 dk prolog | Kare **411–414 → 281–283 ms** (2.4→3.5 fps), GPU **290–292 → 151 ms** (pixel shader'lar dahil). |
| `#2465` (VRAM bitmap) | Win11, RTX 5090 (mikro + 300 s dünya) | Mikrobenchmark: 1M `InterlockedOr` **18 ms → 0.03 ms**; 5090 dünya sahnesinde GPU ~85.7→85.2 ms (gürültü içi). |

---

## 6. Daha Küçük Ara Adımlar (Bizim Ağaca Özel)

1. **Ara Adım 1 — Mevcut VRAM GPU-Hash Yolunu Doğrulama (0 satır kod, anında deney):**
   - Bizim ağaçta **zaten kartta 64 KiB parça başına 128-bit GPU hash/compare** (`shadow.rs:217-248, 3068`, `host.rs:4295-4335`) var. 16 GB kartta varsayılan `VICTORY_CARD_BUFFERS_MIN_MB = 4` (2048 MB bütçe, `shadow.rs:188-190`), `VICTORY_CARD_WINDOWS_MB = 0` (`shadow.rs:431-451`) ve `VICTORY_CARD_REHASH_RUNS = 0` (`host.rs:2104`).
   - 8–12 MB'lık aralıkların hâlâ CPU'da (`snapshots 14.6 ms` + `what shaders wrote 16.7 ms`) karşılaştırılması ya bunların **buffer dışı window** olmasından (`shadow.rs:1784-1795` `back_line` bunu loglar) ya da 2048 MB kart bütçesi dolduğu için CPU'ya taşmasından kaynaklanır.
   - Önce log'daki `shader writes looked for on the CPU per frame` (`shadow.rs:1793`) satırına bakıp `VICTORY_CARD_BUFFERS_MIN_MB=1` (veya `VICTORY_CARD_WINDOWS_MB` / `VICTORY_CARD_REHASH_RUNS=1`) ile ölçüm almak sıfır kodla bedava teşhis ve potansiyel kazanç sağlar.
2. **Ara Adım 2 — Slot-Düzeyi `stores` Analizi (~100–180 satır, düşük risk):**
   - Şu an `pipeline.rs:368` (`fn stores`) tüm shader için tek `bool` üretiyor; shader tek bir UAV'a yazsa bile `draw.rs:3267-3271` → `readable:4133` o shader'ın tablosundaki **tüm** `ResourceType::Uav` slotlarını `gpu_writes` ilan ediyor.
   - PR `#2948` (+84/-9 satır) mantığını `crates/victory-shader/src/emit/lower.rs` / `resource.rs` içinde `UsedResource` (tablo UAV slotu) başına `written: bool` olarak genişletmek, shader'ın yalnız okuduğu 8–12 MB'lık tablo UAV'larını (`0 pieces that differ` kaynaklarını) snapshot+compare'dan tamamen çıkarır; `watch.rs`/`shadow.rs`/`descriptor/host.rs`'ye hiç dokunmaz.
   - Sınırı: Gerçekten yazılan ama 12 MB'ın sadece 1 sayfasını değiştiren UAV'ları çözmez (bunun için tam `view_marks` gerekir).

---

ARA ADIM: Önce mevcut VRAM GPU-hash (`back_line` + `VICTORY_CARD_BUFFERS_MIN_MB`/`VICTORY_CARD_WINDOWS_MB`/`VICTORY_CARD_REHASH_RUNS=1`) ile slot-düzeyi `stores` analizini (~100-180 satır) uygula; hâlâ yüksek CPU compare kalırsa `view_marks` + `#2958` (~2.000 satır) taşı. — İki ağaç `35b054ec`'de ayrıldığı için tam `view_marks` taşıması 6 çekirdek dosyada yüksek çakışma ve `watch.rs`/`Tracked::saved` + `#2958` wave-coalescing gerektirir; mevcut kart-içi hash ve slot-düzeyi `stores` ise çok daha düşük riskle 31 ms'lik "0 pieces that differ" yükünün büyük kısmını kesebilir.
