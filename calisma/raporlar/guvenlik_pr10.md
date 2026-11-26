# Güvenlik İncelemesi: `ZSlayerHQ/zslayer-pc-port-runtime` PR #10 (`02ea29c..e7c05277`)

Tarih: 2026-11-25  
Kapsam: Salt okunur kod, bağımlılık ve güvenlik incelemesi (derleme, test, script çalıştırma veya çalışma ağacı değişikliği yapılmadı; yalnız `git remote add`/`git fetch` + `git log/diff/show/merge-base` + `gh api` GET).  
PR: `ZSlayerHQ/zslayer-pc-port-runtime#10` (yazar: `Cleverdee`, dal: `Cleverdee/victory-world-runtime:series-s-display`)  
İncelenen commit: `e7c05277af65f9a10fcee6042dc1343fa48c543d` (tek commit, 8 dosya, +1566 / -51 satır).

---

## 1. Özet Hüküm

| Hedef | Commit / Aralık | Onaylanan Tam SHA | Karar |
| --- | --- | --- | --- |
| `ZSlayerHQ/zslayer-pc-port-runtime` PR #10 (`Cleverdee`) | `c760d68` → `e7c05277` (taban `02ea29c` / `0808b0c3` ile fark doğrulanmış) | `e7c05277af65f9a10fcee6042dc1343fa48c543d` | **GÜVENLİ** |

Hiçbir kırmızı bayrak (`Cargo.toml`/`Cargo.lock` değişimi, `build.rs`, ağ/telemetri, `Command::new`/shell, registry, dosya sistemine yazma, gömülü ikili/base64, host DLL politikasında gevşetme veya dış süreç erişimi) bulunmadı.  
**Operasyonel not (güvenlik değil, davranış/performans):** `VICTORY_SUBMIT_EACH` (açık) ve `VICTORY_CUT_DISPATCHES` (32) GPU üreticisi veya konsol modu ayrımı yapmadan **tüm modlarda (Series X ve NVIDIA dahil) varsayılan açık** geliyor; NVIDIA'da ek submission overhead'i görülürse `VICTORY_SUBMIT_EACH=0` ve `VICTORY_CUT_DISPATCHES=0` ile kapatılabilir.

---

## 2. Taban ve Git Topolojisi Doğrulaması

- `02ea29c7eeb22da87156676c332dbb157cb9b681` (bizim taban) ve `0808b0c3a1be70a9099b004c10c59484604e7f57` (PR #10 base) ikisi de `c760d6811457fac8c351e9c4db74d078f54e3e1c` üzerine kurulu ve içerik olarak özdeştir (`git diff --stat 02ea29c..0808b0c3` = **0 fark**).
- `0808b0c3` commit'inin **yalnız `README.md`** değiştirdiği doğrulandı (`1 file changed, 132 insertions(+), 146 deletions(-)`).
- `e7c05277`'nin doğrudan ebeveyni `c760d681` olduğundan `02ea29c..e7c05277` farkında `README.md` dışında tam olarak PR #10'un 8 kodu yer alır:
  - `crates/victory-gpu/src/draw.rs` (+109)
  - `crates/victory-gpu/src/host.rs` (+578 / -30)
  - `crates/victory-gpu/src/lib.rs` (+2)
  - `crates/victory-gpu/src/list.rs` (+84 / -18)
  - `crates/victory-gpu/src/pipeline.rs` (+26)
  - `crates/victory-gpu/src/queue.rs` (+150 / -2)
  - `crates/victory-gpu/src/standin.rs` (**YENİ**, +601)
  - `crates/victory-hle/src/modules/kernel.rs` (+17)

---

## 3. Kırmızı Bayrak Taraması (`GUVENLIK.md`)

- **`Cargo.toml` / `Cargo.lock`:** Değişmedi. Yeni crate, feature, `git=` veya `path=` bağımlılığı yok.
- **`build.rs` / `proc-macro` / `rust-toolchain` / `.github/workflows`:** Değişmedi.
- **Ağ API'si (`net.rs`, socket, WinHTTP, reqwest, URL) / telemetri:** **YOK.**
- **`Command::new` / PowerShell / cmd / `CreateProcess` / shell:** **YOK.**
- **Registry okuma/yazma:** **YOK.**
- **Dosya sistemine yazma / kullanıcı klasörü okuma:** **YOK.**
- **Base64 / gömülü ikili / blob:** **YOK.** (Tüm shader'lar düz metin HLSL; bkz. §4.2.)
- **`unsafe` ile keyfi bellek yazımı / dış süreç erişimi:** **YOK.** Tüm `unsafe` blokları süreç-içi D3D12 COM çağrıları (`GetDesc`, `Create*`, `ResourceBarrier`, `CopyTextureRegion`, `ExecuteCommandLists`, `DrawInstanced`, `Dispatch`, ödünç COM pointer için `transmute_copy`), `list.rs`'te mevcut desenle aynı `This`→`&mut List` dönüşümü ve `queue.rs`'te oyunun `present()` çağrısıyla geçirdiği `plane_parameters` yapısından salt-okunur `read_unaligned` okumalarıdır.

---

## 4. Özel İnceleme Noktaları (Dosya:Satır)

### 4.1. `crates/victory-hle/src/modules/kernel.rs` (+17 satır) — Host DLL Politikası
- **Ne yapıyor:** `find_module()` içine (`kernel.rs:352-359`) yalnız `VICTORY_NO_FSR=1` (varsayılan **KAPALI**, `kernel.rs:412-415`) verildiğinde `amd_fidelityfx_upscaler_xbox.dll` (`FSR_DLL`, `kernel.rs:409`) için `ERROR_MOD_NOT_FOUND` (126) ve `Some(null_mut())` döndüren bir erken ret kapısı ekler.
- **Host DLL gevşetmesi var mı?** **HAYIR.** `HostPolicy::default().allows(&file)` ve altındaki ret akışı (`kernel.rs:370-385`) değişmedi. Eklenen kod herhangi bir DLL'e izin vermiyor; aksine, kullanıcı açıkça `VICTORY_NO_FSR=1` isterse oyunun kendi FSR DLL'ini reddediyor.

### 4.2. `crates/victory-gpu/src/standin.rs` (+601) ve `host.rs` — HLSL/Shader Kaynakları
- **Derlenmiş ikili blob var mı?** **HAYIR.** İki shader da okunabilir düz metin HLSL kaynağı olarak gömülü ve çalışma zamanında mevcut `crate::host::compile()` (`D3DCompile`, `host.rs:1001`) ile derleniyor:
  - `standin.rs:191-231` (`const STRETCH: &str`): `stretch` (`cs_5_1`, 8×8 bilinear stretch + `mode == 1` Reinhard tone-mapping) ve `pack` (`cs_5_1`, 8×8 bilinear stretch + `pack_rgb9e5` R9G9B9E5_SHAREDEXP paketleme).
  - `host.rs:6464-6480` (`const COMPOSE: &str`): `corner` (`vs_5_1`, tam ekran üçgen) + `compose` (`ps_5_1`, bilinear örnekleme ile present plane birleştirme).
- `standin.rs` yalnız D3D12 kaynakları (`ID3D12Resource`, `ID3D12DescriptorHeap`, `ID3D12PipelineState`, `ID3D12RootSignature`) ve `tracing::info!/warn!` kullanır; dış I/O yoktur.

### 4.3. `crates/victory-gpu/src/host.rs` — DRED / Device-Removed ve Preemption Raporlaması
- `in_flight()` (`host.rs:7339-7365`): GPU bir `RESOURCEBARRIER` üzerinde durduğunda, ardışık bariyerlerden geriye sararak (`while back > 0 && barrier(&history[back - 1])`) bariyerin beklediği önceki komutları da mevcut bellek-içi `String` raporuna ekler.
- `create()` (`host.rs:661-670`): `IDXGIAdapter3::GetDesc2()` ile `GraphicsPreemptionGranularity` ve `ComputePreemptionGranularity` değerlerini okur ve tek satır `tracing::info!` ile loglar.
- **Dışarıya bir şey yazıyor mu?** **HAYIR.** Dosya, ağ, olay günlüğü veya başka bir dış hedefe yazma yok; yalnız mevcut `tracing` log akışı kullanılır.

### 4.4. `crates/victory-gpu/src/queue.rs` — Çoklu Present Plane Okuması
- `plane_stride()` (`queue.rs:384-406`) ve `plane_at()` (`queue.rs:413-439`): Yalnız `planes > 1` iken oyunun `plane_parameters` işaretçisinden `read_unaligned` ile salt okuma yapar (stride adayları `[72, 64, 56, 48, 40, 32, 24]`; ikinci plane'de token eşleşmesi + `count` 1..=8 + 8-bayt hizalı non-null liste kontrolü). `source_fractions()` ve `placement()` (`host.rs:6624-6655`) geçersiz/sınır-dışı/NaN değerleri güvenli varsayılana düşürür.

---

## 5. Yeni `VICTORY_*` Anahtarları ve Varsayılanları

| Anahtar | Varsayılan (`e7c05277`) | Değerler | Dosya | Ne Yapar |
| --- | --- | --- | --- | --- |
| `VICTORY_PRESENT_PLANES` | **AÇIK** (straight alpha) | `0` (eski tek-plane kopya), `premul` (premultiplied alpha), diğerleri (straight alpha) | `queue.rs:445-452` | `planes > 1` iken tüm present plane'lerini compose eder (plane 0 opak, sonrakiler blend). |
| `VICTORY_FSR_STANDIN` | **AÇIK** | `0` kapatır, diğerleri açık | `standin.rs:35-38` | DXIL içermeyen compute pipeline'larının (`pipeline::without_dxil`) düşen dispatch'lerinde renk girdisini çıktıya bilinear stretch eder. |
| `VICTORY_FSR_STANDIN_PACK` | **AÇIK** | `0` kapatır, diğerleri açık | `standin.rs:138-141` | FSR stand-in sırasında `R32_UINT` (f42) / `R9G9B9E5_SHAREDEXP` (f67) çıktılarını R9G9B9E5 olarak paketleyip yazar. |
| `VICTORY_NO_FSR` | **KAPALI** | `1` açar, diğerleri kapalı | `kernel.rs:409-415` | `amd_fidelityfx_upscaler_xbox.dll` yüklemesini `ERROR_MOD_NOT_FOUND` ile reddeder. |
| `VICTORY_SUBMIT_EACH` | **AÇIK** | `0` kapatır, diğerleri açık | `host.rs:905-908` | Bir gönderimdeki her host listesini (+ prelude'unu) ayrı `ExecuteCommandLists` çağrısıyla gönderir. |
| `VICTORY_CUT_DISPATCHES` | **32** | `0` kapatır, `<n>` üst sınır | `list.rs:245-253` | `queue::cuts()` açıkken bir host listesini her `<n>` direct/indirect dispatch'te yeni host listesine böler. |

---

## 6. Series X ve NVIDIA Üzerinde Varsayılan-Açık Etki Değerlendirmesi

1. **`VICTORY_SUBMIT_EACH` (Varsayılan AÇIK — her modda ve her GPU'da):**
   - `host.rs:905-908` içinde konsol (`series-s`/`series-x`) veya GPU üreticisi/preemption seviyesi kontrolü yoktur; **Series X'te ve NVIDIA'da da varsayılan açıktır**.
   - Etkisi: Tek submission içinde birden fazla host listesi varsa tek `ExecuteCommandLists(&lists)` yerine her host listesi (+ prelude'u) için ayrı `ExecuteCommandLists` çağrısı yapılır. AMD'de DMA-buffer preemption kaynaklı 2 sn TDR'yi önler; NVIDIA'da işlevsel bozulma beklenmez ancak daha fazla sürücü/CPU gönderim yükü oluşturabilir (`VICTORY_SUBMIT_EACH=0` ile eski davranışa dönülür).
2. **`VICTORY_CUT_DISPATCHES` (Varsayılan 32 — her modda ve her GPU'da):**
   - `list.rs:245-292` içinde de konsol veya GPU filtresi yoktur (`queue::cuts()` varsayılan açık olduğu için **Series X ve NVIDIA'da da aktiftir**).
   - Etkisi: 32'den fazla direct/indirect dispatch içeren oyun komut listeleri 32'lik parçalara bölünür; her parça için yeni host listesi alınır, state `replay` edilir ve `SUBMIT_EACH` ile ayrı gönderilir. NVIDIA'da ek CPU/driver maliyeti yaratırsa `VICTORY_CUT_DISPATCHES=0` ile kapatılabilir.
3. **`VICTORY_PRESENT_PLANES` (Varsayılan AÇIK — Series X'te etkisiz):**
   - Yalnız `planes > 1` iken devreye girer (`queue.rs:933`). Series X tek plane sunduğu için mevcut `CopyResource` yolunu değiştirmez.
4. **`VICTORY_FSR_STANDIN` / `VICTORY_FSR_STANDIN_PACK` (Varsayılan AÇIK — yalnız no-DXIL compute düşerse):**
   - Yalnız `pipeline::without_dxil` doğru olan bir compute dispatch düştüğünde çalışır (`draw.rs:2129`). Series X'te DXIL'siz compute programı yoksa tetiklenmez.
5. **`VICTORY_NO_FSR` (Varsayılan KAPALI):**
   - Açıkça `=1` verilmedikçe hiçbir etkisi yoktur.
