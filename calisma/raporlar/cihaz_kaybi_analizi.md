# GPU aygıt kaybı (0x887A0001) analizi: koşu 11 ve 12

Tarih: 2026-11-25. Kod: `C:\CLLL\.calisma\vw-pr10` (e7c0527 = PR #10, üstü next@02ea29c). Salt okunur inceleme: dosya değiştirilmedi, derleme ya da oyun çalıştırılmadı.

## Özet

1. **En olası neden (yüksek, yaklaşık %70):** Series S'te oyun 1920x1080 **R9G9B9E5_SHAREDEXP (format 67)** bir dokuyu RT+UAV bayrağıyla (flags 5) istiyor. D3D12 bu dokuyu reddediyor (`0x80070057`). Doku yokken render target olarak bağlandığında kod, format 67 ile **null RTV** oluşturuyor (`draw.rs:627`). Bu formatla RTV geçersiz. `CreateRenderTargetView` hata döndüremediği için cihaz `DXGI_ERROR_INVALID_CALL` ile kaldırılıyor.
2. Aynı çökme upstream'de birebir raporlanmış ve düzeltilmiş: **fxceless/victory-fork issue #13**. Debug katmanı orada şu mesajı veriyor: `CreateRenderTargetView: The format (0x43, R9G9B9E5_SHAREDEXP) cannot be used…`. Düzeltme **PR #32**: `target_format` içine `(67, 10)` stand-in'i, 10 satır. **vw-pr10'da bu düzeltme yok** (`resource.rs:253` yalnız `(86,28),(115,28)`).
3. Bir düzeltme: verilen özette "960x540 format 26 reddi" yazıyordu, ama bu satır **cihaz düştükten sonra** geliyor (`0x887A0005`). Asıl ret iki koşuda da **1920x1080 format 67 flags 5**: koşu 12'de 18:22:22.859, koşu 11'de 18:18:40.532.
4. Ölüm anı iki koşuda da aynı: oyun FSR sonrası post zincirini **ilk kez** bağladığı karede düşüyor (vertex `a52f36788c76bc03` → compute `4848dd3d7139514c` → vertex `5503f628fa7c8ac4`). Koşu 11'de frame 1685, koşu 12'de frame 2029. Düşüş, bu ilk bağlamadan sonraki 25-30 ms içinde. Koşu 11'de format 67 reddi ve FSR stand-in 3 kare boyunca sorunsuz sürdü (frame 1682-1684). Yani tetikleyici reddin kendisi değil, o dokuya hedef olarak ilk yazan post geçişi.
5. PR #799'daki CBV 64 KiB kırpması **bizde zaten var**, eşdeğer haliyle (`descriptor/host.rs:1999`, kullanıldığı yerler 1636 ve 2334). Root CBV'lerde boyut alanı yok. Bu aday çok düşük olasılıkta.
6. FSR stand-in (PR #10) tek başına düşük olasılıkta. Koşu 11'de 3 kare sorunsuz çalıştı, view'leri `fits()`/`stores()` ile doğrulanıyor. Ama stand-in format 67 çıkışının host'ta var olduğunu varsayıyor; varsayım tutmuyor.
7. DRED kapalı olduğu için "no breadcrumbs (0x887A0004)" görülüyor. DRED yalnız `VICTORY_D3D12_DRED=1` ile açılıyor (diag derleme). Debug katmanı ise SDKLayers olmadan hiçbir şey vermez.

### Önerilen sıradaki 3 koşu

1. **Koşu 13, PR #32 yamasıyla** (lead uygular: `resource.rs:253` → `[(86,28),(115,28),(67,10)]`, `[bool;3]`). Ayarlar koşu 12 ile aynı. Beklenen:
   - Logda `the host does not render to this format … format=67`
   - `no host texture … format: 67` satırı yok
   - frame ~2029 geçilir ve dünyaya girilir.
2. **Koşu 14, yamasız ama `--console series-x`** (kontrol). Beklenen: format 67 / FSR4 yolu hiç yok, dünyaya girişte 0x887A0001 yok (issue #13'te aynı derleme Series X'te çalışıyor). Daha yavaş ve RAM'e daha ağır olabilir.
3. **Koşu 15, yalnız 1 numara başarısız olursa:** yamalı derleme + `VICTORY_D3D12_DRED=1 VICTORY_RELAY=0`. Breadcrumb ve page fault verisi gelir. Senkron gönderimle düşüş, hatayı yapan çağrıya en yakın noktada yakalanır.

---

## 1. 0x887A0001 hangi çağrıdan?

### Zaman çizelgesi (koşu 12, `run.log`)

| Satır | Zaman (UTC) | Olay |
|---|---|---|
| 4761 | 18:22:22.792 | frame=2029, çok sayıda "first bind" compute; FSR dispatch düşüyor, stand-in giriş/çıkış bulamıyor |
| 4779-4780 | 22.828 | FSR stand-in: `1280x720 f10 -> 1920x1080 f10` |
| **4805** | **22.859** | **`no host texture error=0x80070057 desc={1920x1080, format: 67, flags: 5}`** |
| 4807-4810 | 22.860 | stand-in iki çıkışa yazıyor (f10 ve f28, Reinhard) |
| 4811-4818 | 22.861-22.866 | post zinciri **ilk kez** bağlanıyor: vertex a52f…/pixel bd66…, compute 4848…, 92c1…, vertex 7f99…, vertex 5503… |
| 4825/4922 | 22.895-22.896 | `device removed … 0x887A0001`, "no breadcrumbs (0x887A0004)", iletim thread'i "between submissions, nothing of ours was running on the card" |
| 4924, 4937-4939 | 22.896-22.903 | 960x540 f26, f10, 120x68 f10 → `0x887A0005` (cihaz zaten düşmüş; bunlar sonuç, neden değil) |

### Koşu 11 aynı deseni gösteriyor

- 3687, 18:18:40.532: aynı format 67 reddi (frame 1682).
- 3646-3693: stand-in 1682'de başlıyor, 5092-5093 (18:18:46) yine çalışıyor, cihaz düşmüyor.
- 5784-5790, 18:19:35.576-.582: frame 1685'te **aynı** post zinciri ilk kez bağlanıyor (a52f…, 4848…, 7f99…, 5503…).
- 5797 / 5923, 18:19:35.590 / .597: cihaz kaybı 0x887A0001.

Sonuç: format 67 reddi ve stand-in 3 kare boyunca zararsızdı. Düşüş, post zincirinin ilk kez çalıştığı karede, o zincirin ilk bağlamasından 8-30 ms sonra oluyor.

"Nothing of ours running on the card" ve INVALID_CALL birlikte, GPU'nun hata vermediğini (page fault ya da TDR olsaydı `0x887A0005`/`0x887A0006` görülürdü), hatanın **CPU tarafında geçersiz bir çağrı** olduğunu gösteriyor. Hata döndüremeyen `void` API'lerdeki (Create*View) geçersiz parametre, D3D12/NVIDIA'da cihazı INVALID_CALL ile kaldırır. Bu desen kodda zaten belgelenmiş: `descriptor/host.rs:1958-1964` ("It removes the device for one it does not (view creation returns nothing)… one such view ended the run (2026-11-24, run 10)").

### Adaylar

| Aday | Kanıt | Olasılık |
|---|---|---|
| **A. Format 67 ile null RTV** (`draw.rs:615-628` → `host.rs` `rtv()` → `CreateRenderTargetView(None, {Format: 67})`) | Doku reddedilince `host_resource` = None. `set_targets` başlıkta tanımlı bir desc varsa onunla null RTV yapıyor (`draw.rs:620-628`). Desc formatı `target_format(pc_format(..))` ile eşleniyor (`descriptor.rs:242`), ama `target_format`'ta 67 için stand-in yok (`resource.rs:253`). RTV için R9G9B9E5 geçersiz. Upstream'de debug katmanı tam bu çağrıyı adlandırdı (issue #13, ayrıca #616 madde 1: "without the stand-in the device is removed at world entry"). Zamanlama da uyuyor: dokuya hedef olarak yazan post zinciri ilk kez çalıştığında düşüyor. `set_targets`'ta RTV için `fits()` benzeri bir doğrulama da yok (SRV ve UAV'de var). | **Yüksek (~%70)** |
| B. Başka doğrulanmamış RTV/DSV (gerçek dokuda format ailesi uyuşmazlığı, RT bayrağı olmayan dokuya RTV) | `draw.rs:634`'te `host.rtv(Some(texture), record.desc())` format/bayrak denetimi yapmıyor. Post zinciri yeni hedefler bağlıyor. Doğrudan kanıt yok. | Orta-düşük (~%10) |
| C. FSR stand-in (PR #10, `standin.rs:341-405`, `draw.rs:2002-2097`) | View'ler `fits()` ile süzülüyor (`standin.rs:320-332`, UAV typed store sorgusu). Koşu 11'de 3 kare zararsız çalıştı. Ayrıca bir yan kusur var: stand-in descriptor halkası (`next % (VIEWS/2)`), uçuştaki listelerin slotlarının üstüne yazabiliyor. Bu yanlış görüntü üretir, INVALID_CALL üretmez. | Düşük (~%7) |
| D. Derinlik dokusu UAV'si | Twin'e yönlendiriliyor ve `fits()` ile bayrak denetimi var (`descriptor/host.rs:2408-2432`, 2193-2214). Koşu 11'de 74 kez loglandı, dakikalarca sorunsuz. | Düşük (~%4) |
| E. Null bırakılan buffer view'ler | "left null" ifadesi, geçersiz view'in **yaratılmadığı**, güvenli null'a çevrildiği anlamına geliyor (`buffer_view_ok`, 1965). Neden değil, korumanın kendisi. | Çok düşük |
| F. Reddedilen pipeline'ın bağlanması (compute 02384a60…, 2da4beb…, 369cbe…, 2cc4c1…) | Host pipeline yoksa dispatch atlanıyor (`draw.rs:2120-2135`, log 2768 "a dispatch without a host pipeline dropped"). Bağlanmıyor. | Çok düşük |
| G. CBV >64 KiB | Tablo CBV'leri kırpılıyor (`constant_size`, `descriptor/host.rs:1999`; kullanım 1636 ve 2334; null CBV 2628). Root CBV/SRV/UAV'lerin (`draw.rs:1128-1131`, 1144, 1227) boyut alanı yok, 64 KiB sınırı onlara uygulanmıyor. | Çok düşük (~%2) |
| H. Liste kapanma hatası (Close E_INVALIDARG → Execute) | `HostList::close` hatayı ayrı bir mesajla bitiriyor ("did not close", `host.rs:4826-4829`). Bu mesaj logda yok. | Çok düşük |
| I. "pipeline library: made … did not load it 0x80070057" | Bu sadece önbellek ıskası (`LoadPipeline` bulunamayan ad için E_INVALIDARG döndürür). Zararsız. | Yok |

## 2. "no host texture 0x80070057" neden?

- Reddedilen tanım: `dimension 3 (TEXTURE2D), 1920x1080, mip 1, format 67 = DXGI_FORMAT_R9G9B9E5_SHAREDEXP, flags 5 = ALLOW_RENDER_TARGET | ALLOW_UNORDERED_ACCESS`. R9G9B9E5'i hiçbir PC GPU render target ya da typed UAV olarak desteklemiyor. `CreateCommittedResource` E_INVALIDARG döndürüyor. Bu NVIDIA'ya özgü bir UAV typed store meselesi değil, formatın kendisi.
- Kod yolu: `resource.rs:1842-1923` `host_texture`.
  - Bayraklar PC'ye göre maskeleniyor (1856).
  - Derinlik+UAV kırpılıyor (1863-1867).
  - `target_format` yalnız 86 ve 115'i 28'e çeviriyor (253). **67 olduğu gibi kalıyor**, oluşturma başarısız oluyor ve uyarı loglanıyor (1917). `None` dönüyor, dokunun host kopyası olmuyor.
- Reddedilince ne oluyor:
  - SRV → `null_like` (yalnız okuma, zararsız).
  - UAV → `null_uav_desc` (R32_UINT, zararsız).
  - Clear → atlanıyor (`list.rs:1208-1212`).
  - FSR stand-in → `host_resource` None olduğu için doku atlanıyor (`draw.rs:2035-2037`), PR #10'un `stretch_shared_exponent` yolu bu dokuya hiç ulaşmıyor.
  - **Render target → başlıktaki desc formatıyla (67) null RTV oluşturuluyor (`draw.rs:620-628`).** Geçersiz çağrı bu.
- 960x540 f26 (R11G11B10_FLOAT) RT+UAV normalde geçerli. Logdaki reddi `0x887A0005`, yani cihaz kaybının sonucu.

## 3. Upstream düzeltmeleri: bizde var mı?

| Değişiklik | Ne yapıyor | vw-pr10'da? |
|---|---|---|
| **fxceless/victory-fork PR #32** (https://github.com/fxceless/victory-fork/pull/32, head 9ba864a, OPEN) | `target_format`'a `(67, 10)` ekliyor. R9G9B9E5 hedefleri RGBA16F olarak yaratılıyor. View'ler `fits()` içindeki `target_format(view) == own` kuralıyla otomatik eşleniyor (`descriptor/host.rs:2231`). Tek dosya, +10 satır. | **Yok**. Bu çökmenin doğrudan düzeltmesi. Issue #13'te RTX 4060 Ti, RTX 4080 ve RTX 5060'ta doğrulanmış. |
| fxceless/victory-fork issue #13 (https://github.com/fxceless/victory-fork/issues/13) | Aynı belirti: Series S, FSR4 "DeserializeComputePipelineStateX FAILED", ardından aygıt kaybı. Debug katmanı CreateCommittedResource ve CreateRenderTargetView'i R9G9B9E5 için adlandırıyor. FSR4 mesajları sadece oyunun geri düşüşü, neden değil. | (bilgi) |
| issue #616 (https://github.com/fxceless/victory-fork/issues/616) | RTX 3050 8 GB Series S için sıra: #32 (bu çökme), #188 (iki plane + stretch), #86 (PC FSR), #51 (integer blend). | #188'in karşılığı PR #10'da var. #32 ve #51 yok. |
| fxceless PR #799 / soldsawn 09b95ca (https://github.com/fxceless/victory-fork/pull/799) | `cbv_size = size.next_multiple_of(256).min(65536)`, iki CBV çağrı yerinde. | **Var, eşdeğer**: `constant_size = size.min(64<<10).next_multiple_of(256)` (`descriptor/host.rs:1999`, 1636, 2334). Eksik kısım yok. PR'daki shader çevirici (loose-LDS, s_pack, s_bfm) kısımları ayrı bir konu, çökmeyle ilgisi yok. |
| fxceless PR #936 (https://github.com/fxceless/victory-fork/pull/936) | `VICTORY_WIDE_BACK=1` ile `SNAPS` kuyruğunda, Shadow düşürülünce use-after-free. | **Düzeltme yok**: `shadow.rs:1063` SNAPS var, `Drop for Shadow` (1399-1422) flush etmiyor, wide_back varsayılan açık (`shadow.rs:970-971`) ve koşularda `=1`. CPU bellek bozulması riski taşıyor, 0x887A0001 açıklaması değil. Ayrı ele alınmalı. |
| fxceless PR #964 (https://github.com/fxceless/victory-fork/pull/964) | Kopya kuyruğu fence'inde cihaz kaybı sonrası `UINT64_MAX`'ı "bitti" saymama. | İlgisiz (kayıp sonrası davranış). `host.rs:2553` benzer bir `GetCompletedValue` kullanıyor, kontrol edilmeli. |
| Belobug 8629211 (= fxceless #51, ZSlayer PR #8'de de var) | Integer RT'lerde blend kapatma. Yoksa pipeline E_INVALIDARG ile reddediliyor. | **Yok** (`pipeline/host.rs`'ta karşılığı yok). Bu koşularda "mesh pipeline refused" 0 kez, aygıt kaybıyla ilgisi yok (ret dönüş değeriyle bildiriliyor). İleride duvar/oda çizimi için gerekli. |
| ZSlayer PR #10 sonrası | ZSlayer'da #11 ve #13 belge, #12 test. FSR/Series S için yeni bir düzeltme yok. PR #8 (Geckomacabre) integer blend ve hedef varyantlarını içeriyor, OPEN. | — |
| fxceless'te "887A0001" araması | `gh search issues 887A0001` sonuç vermedi. İlgili kayıtlar "R9G9B9E5" aramasıyla bulundu (#13, #616, #848). | — |

## 4. Teşhis araçları ve tek değişkenli deneyler

- **`VICTORY_D3D12_DEBUG`** (`host.rs:556-579`): yalnız `diagnostics` özellikli derlemede `D3D12GetDebugInterface` → `EnableDebugLayer` çağırıyor, `VICTORY_D3D12_GBV` ile GPU tabanlı doğrulamayı açıyor. Mesajlar InfoQueue'dan okunup `fail()` içinde "debug layer: …" olarak ekleniyor (`host.rs:511-528`). **d3d12SDKLayers.dll yoksa** arayüz alınamıyor, yalnız "no D3D12 debug layer" uyarısı çıkıyor, **kazanç yok**. Diskte hiçbir d3d12SDKLayers.dll bulunamadı (Windows Kits ve VS altında da arandı). Kurulum: yönetici ile "Grafik Araçları" isteğe bağlı özelliği (`Tools.Graphics.DirectX~~~~0.0.1.0`).
- **DRED** (`host.rs:583-593`, `dred_on` 1381-1388): yalnız `VICTORY_D3D12_DRED` tanımlıysa (diag derleme) `ID3D12DeviceRemovedExtendedDataSettings` ile breadcrumb ve page fault açılıyor. Açılmadığında `GetAutoBreadcrumbsOutput` `DXGI_ERROR_UNSUPPORTED (0x887A0004)` döndürüyor, "no breadcrumbs" bundan (`host.rs:4604-4611`). DRED ayarları SDKLayers gerektirmez (D3D12Core'da). Uyarı: INVALID_CALL CPU tarafı bir hata olduğu için breadcrumb'lar büyük olasılıkla "tüm listeler bitti" diyecek. Bu bile GPU hatası olmadığını teyit etmeye yarar. `name()` ile kaynak adları da DRED açıkken veriliyor.
- Eksik log: `draw.rs:620-628` null RTV yaratırken hiçbir şey loglamıyor. Yamada `once!(format, "typed null render target view")` eklemek, A hipotezini tek koşuda kanıtlar.

| Deney (tek değişken, koşu 12 temel) | Beklenen sonuç | Ne kanıtlar |
|---|---|---|
| PR #32 yaması (kod) | Format 67 reddi kaybolur, frame ~2029 geçilir | A doğrulanır (en güçlü test) |
| `--console series-x` | Format 67 ve FSR4 yolu yok, dünyaya girişte kayıp yok | Sorunun Series S'e özgü yolda olduğunu |
| `VICTORY_D3D12_DRED=1` | "breadcrumbs (done/count…)" çıkar, büyük olasılıkla tüm listeler bitmiş ve page fault yok | Hatanın GPU yürütmesinde değil, CPU çağrısında olduğunu |
| `VICTORY_RELAY=0` (`shadow.rs:482`) | Kayıp, bir sonraki senkron gönderimde yakalanır. İz (trail) daha kısa ve isabetli olur. Relay ve WIDE_BACK etkisini dışlar | Zamanlama daralır |
| `VICTORY_FSR_STANDIN=0` (`standin.rs:30-33`) | A doğruysa **yine** aynı karede kayıp (resim siyah). Sağ kalırsa suçlu stand-in'dir | C'yi eler ya da doğrular |
| `VICTORY_FSR_STANDIN_PACK=0` | R32_UINT paket yolu kapanır. A doğruysa değişiklik olmaz | Küçük ek eleme |
| `VICTORY_SUBMIT_EACH=0` (`host.rs:907`) | Değişiklik beklenmez (INVALID_CALL'u açıklamaz) | PR #10'un gönderim değişikliğini eler |
| `VICTORY_NO_FSR=1` | **Etkisiz**: DLL zaten host politikasınca reddediliyor (run.log 781, "refused dll=amd_fidelityfx_upscaler_xbox.dll") | Önerilmez |
| `VICTORY_D3D12_DEBUG=1` (SDKLayers kurulmadan) | "no D3D12 debug layer", bilgi yok | Önerilmez. Kurulursa çağrıyı adıyla verir |

## Kanıtlanmış / tahmin

- **Kanıtlı:**
  - Format 67 reddi iki koşuda da var (log satırları yukarıda).
  - 960x540 f26 reddi cihaz kaybından sonra geliyor.
  - Null RTV yolu desc formatıyla oluşturuluyor (`draw.rs:620-628`, `descriptor.rs:242`, `resource.rs:253`).
  - PR #32 bizde yok, CBV kırpması var, #936 yok, #51 yok.
  - Upstream'de debug katmanı aynı çağrıyı adlandırmış (issue #13).
- **Tahmin:**
  - Ölümcül çağrının bizdeki null RTV olduğu. Debug katmanı bu makinede yok. Post zincirinin bu dokuyu RT olarak bağladığı loglanmıyor.
  - PR #32 sonrası PR #10'un R9G9B9E5 paketleme yolunun (`stretch_shared_exponent`) çakışmadan devre dışı kalacağı. Host formatı artık 10 olacak, normal `stretch` yolu devreye girer. Koşuda doğrulanmalı.
