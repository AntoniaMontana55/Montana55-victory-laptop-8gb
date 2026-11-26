# Fill bekleme kökü — 480 baytlık "fill of upload memory" neden 40-60 ms bekletiyor (25.11, salt okunur)

Kod: `C:\CLLL\.calisma\vw-pr10\crates\victory-gpu\src` (yollar buna göre). Log: `kosular\20261125-230921_kosu25-fill-teshis\run.log` (20.782 satır), ayarlar `ayarlar.txt`. Hiçbir şey çalıştırılmadı/derlenmedi; yalnız okuma. Önceki rapor: `kopya_kritik_yol.md` §0/§2.

## 0. Kısa sonuç

- `line.written`, uçuştaki gönderimlerin **bildirdiği** (`Step::GpuWrite`) aralıkların birleşimi; gerçekte yazılan baytlar değil.
- 174 MB'ın tamamını tek seferde bildiren yer büyük olasılıkla **`memory.rs:301-308`**: "adrese göre bellek" okuyan/yazan shader'lar (konsolun ışın izleme yapı kurucusu, BVH) her listede **numaralanmış her kopyayı baştan sona** yazılmış sayıyor. Log'da bu mod 20:10:36'da açılıyor (satır 6922), ilk fill beklemesi 2 sn sonra geliyor (20:10:38, satır 8038). Önceki satırlarda "fill of upload memory" sayısı 0. Bu zamanlama ilişkisi güçlü ama doğrudan kanıt değil (**TAHMİN-güçlü**).
- Bekleten 480 baytlık Fill'ler büyük olasılıkla **descriptor tablosundaki CBV'ler** (`draw.rs:2711-2718`): her `View::Constant` her zaman CPU'da scratch'e kopyalanıyor. 128 KiB aralıklı `…fd00` deseni, 174 MB yığında bir sabit tampon halkası ya da nesne başına bloklar gibi duruyor (**TAHMİN**).
- Bu baytları gerçekten bir shader yazıyor mu? Kodda ve log'da bunu gösteren **kanıt yok**. Bekleme tamamen bildirimin kaba olmasından geliyor.
- **En güvenli ilk yama:** tablodaki CBV'yi, kaynak kopyada 256 hizalıysa **doğrudan kopyanın GPU adresine** bağlamak. Bu, root CBV için zaten kullanılan yol (`draw.rs:483-486`). Böylece Fill hiç oluşmaz, `blocked_by` beklemez. Yama env arkasında kalır, kapalıyken davranış birebir aynı. ~15 satır.

## 1. `line.written` nereden doluyor, kim "yazabilir" diyor

**Zincir:**
1. `Line.written` alanı: `host.rs:2199-2203` ("the title memory the flights' shaders may write … in order and joined").
2. Gönderim uçuşa çıkınca `join_ranges(written, sent.back…)`: `host.rs:4202` (`execute_late`) ve `host.rs:4246` (`execute_own`). Hepsi gidince temizleniyor: `host.rs:4302`.
3. `sent.back` = `declared_writes(runs)`: `host.rs:2931-2949` (`submit` içinde `host.rs:4561` / `4684`). Listelerdeki tüm `Step::GpuWrite(shadow, address, len)` adımlarını topluyor.
4. `Step::GpuWrite`'ı ekleyen tek yer `HostList::defer_gpu_write` (`host.rs:5488-5497`). Onu çağıran tek yer `Shadow::gpu_writes` (`shadow.rs:1758-1769`); kopya yerindeyse (`place`) eklemiyor, uzunluğu kopyanın sonuna kırpıyor.

**`Shadow::gpu_writes` çağıranlar (aralığın nasıl belirlendiği):**

| Yer | Kaynak | Bildirilen aralık |
|---|---|---|
| `draw.rs:3359-3372` (`readable`) | Tablo/descriptor UAV (`View::Unordered`, `View::Placed(true…)`) | **View'in alt aralığı.** `descriptor/host.rs:1899` `buffer_copy` → `buffer_range` (`descriptor/host.rs:1832`) → `unordered_range` (`descriptor/host.rs:1916-1931`): `base + FirstElement*elem`, `NumElements*elem` (elem = StructureByteStride / RAW ise 4 / formattan). `VICTORY_VIEW_EXTENT` yalnız **hangi kopyanın** bulunacağını değiştiriyor, aralığı değil (`descriptor/host.rs:1883-1912`). Shader buffer'a hiç yazamıyorsa bildirilmiyor (`VICTORY_WRITE_SKIP`, `draw.rs:116-120`). |
| `draw.rs:1206-1213` (`root_uav`) | Root UAV | **Adresten tamponun sonuna kadar**: `s.left(a).min(limit)`, limit `VICTORY_ROOT_UAV_MB` (varsayılan sınırsız, `draw.rs:1189-1196`). Root view'in boyutu yok (yorum `draw.rs:1178-1180`). |
| `memory.rs:301-308` (`ready`) | Ham adresle okuyan/yazan shader'lar (BVH kurucu; `draw.rs:821-842` `memory_ready`) | **Numaralanmış her kopyanın tamamı**: `shadow.span()` (`shadow.rs:1658`) = (start, len). Listedeki ilk yazmada ve sonradan numaralananlar için. Modül başındaki "shortcut" bunu açıkça söylüyor (`memory.rs:39-44`: "declares all of them written … a dirty-page mask written by the shader would bound both"). Bir kopya, içindeki bir sayfa adlanınca bütünüyle numaralanıyor (`memory.rs:155-180`). |

Placed (yığına yerleşik) buffer view'ler de `View::Placed(true, …)` üzerinden **view alt aralığıyla** geliyor (`buffer_range`, `descriptor/host.rs:1849-1852`).

**174 MB'ın tamamı neden giriyor:** Log'daki 40 olayın hepsinde `written_from=0x1b4e0000 written_len=182452224`. Bu, yığının kendisi: `0x1b4e0000`, 182.452.224 bayt, kart belleğinde kopya (log satır 202). İki durum var:
- `written_ranges=1` olan olaylar (5 tane) tek bir bildirimin tüm yığını kapladığını gösteriyor. View alt aralıklarının birleşimiyle tam `base…base+len` çıkması pek olası değil. Kalan iki aday var: `memory.rs` (tam span, kesin olarak `base`'den başlar) ya da `base`'deki bir root UAV (`left(base)` = tüm yığın).
- Zaman çizgisi `memory.rs`'yi gösteriyor: satır 6922 (20:10:36.35) "title memory by address: a shader reads or writes through raw addresses", ardından satır 8038 (20:10:38.20) ilk fill beklemesi. Kare satırlarında "a fill of upload memory" önce 0.0, sonra karede 1.0 (**TAHMİN-güçlü**; kesin ayırt etmek için §4'teki teşhis satırı).
- `written_ranges` 4-168 olan olaylarda yığın yine tek parça. Diğer aralıklar başka tamponlar.

**Bekleme nerede:** `submit` → `late_blocked` (`host.rs:2306-2314`) → `blocked_by` (`host.rs:2319-2369`, Fill dalı `2334-2355`) → `relay_exact()` (`host.rs:2043-2048`; `VICTORY_RELAY` ayarlı değil, yani aşama 2 açık) → `relay_wait(WHY_READ, …)` (`host.rs:4437-4443`). Yığını bildiren uçuşlar inene kadar bekleniyor. Bu uçuşlar BVH dispatch'leri; kartta uzun sürüyorlar ve iniş sırasında her uçuş için 174 MB'ın kartta hash'lenmesi/karşılaştırılması gerekiyor ("12875 pieces compared on the card", satır 20708).

## 2. Bu 480 baytlık Fill'ler ne

`Step::Fill` üreten üç yer var (`host.rs:5462` `defer_fill`):

| Yer | Ne | Boyut |
|---|---|---|
| `draw.rs:479-490` (`root_view`) | 256'ya hizalı olmayan root **CBV**'yi scratch'e kopyalamak | `min(left, 64 KiB)`: genelde 65.536, 480 değil |
| `draw.rs:2711-2718` | **Descriptor tablosundaki her `View::Constant` (CBV)**, hizasına bakılmadan scratch'e | View'in `size` alanı (16'nın katı) |
| `list.rs:960-975` | Buffer→doku kopyasının footprint kaynağı | Doku satırı boyutu |

480 (0x1e0), 112 ve 48 boyutları 16'nın katı ve küçük. Hepsi 256 hizalı `…fd00` / `…ff40` / `…ef80` adreslerinde. Bu yüzden bunlar büyük olasılıkla **tablo CBV'leri** (`draw.rs:2716`; **TAHMİN-güçlü**). 128 KiB adımlı `…fd00`, yığında her 128 KiB bloğun sonunda 0x300 baytlık bir başlık ya da sabit blok gibi görünüyor (**TAHMİN**; içerik bilinmiyor).

**Shader bu baytları yazıyor mu:**
- Koddaki bildirim kaba (yukarıdaki tablo). `line.written` içinde olmaları, yazıldıklarını göstermiyor.
- Kartın kendi karşılaştırması zaten gerçekten değişen 64 KiB parçaları buluyor ("550 of them fetched (35.1 MB)", satır 20708). Ama hangi parçaların değiştiği log'a yazılmıyor. 0x1b4e0000 yığını için parça listesi yok; `copystat compared` satırları kart kopyalarını göstermiyor (`host.rs:3201-3205`, `carded` → `copy_back_done`).
- Sonuç: Kanıt yok. Kodun bu baytları shader'ın yazdığını gösterdiği bir yer de yok, yazmadığını gösterdiği bir yer de. BVH kurucunun konsolda bu CB bloklarına yazması anlamsız görünüyor (**TAHMİN**).

## 3. Daraltma seçenekleri ve doğruluk riski

**Mevcut env anahtarları (late/relay/written ile ilgili):**

| Anahtar | Yer | Varsayılan | Ne yapar |
|---|---|---|---|
| `VICTORY_LATE_WAIT` 0/1/2/3 | `host.rs:1940-1955` | relay açıksa 3 (bu koşuda =3) | Uçuş aşaması |
| `VICTORY_RELAY` 1/2 | `shadow.rs:484`, `host.rs:2047` | açık (test dışı), aşama 2 | `=1` olursa okuma her şeyi bekler |
| `VICTORY_LATE_WAIT_LISTS` | `host.rs:2149` | 256 | Havadaki liste üst sınırı |
| `VICTORY_LATE_WAIT_WRITES` | `host.rs:2159` | açık | Bottom-of-pipe yazmalar uçabilir |
| `VICTORY_LATE_WAITS` | `frame.rs:138` | kapalı | (kare tarafı uyku/bekleme) |
| `VICTORY_WRITE_SKIP` | `draw.rs:119` | açık | Store'suz shader UAV'sini bildirmez |
| `VICTORY_ROOT_UAV_MB` | `draw.rs:1192` | sınırsız | Root UAV refresh/bildirim/kopya-geri sınırı (**doğruluk riskli**: ötesindeki yazmalar kaybolur) |
| `VICTORY_ROOT_VIEW_MB` | `draw.rs:474` | sınırsız | Root SRV sınırı |
| `VICTORY_VIEW_EXTENT` | `descriptor/host.rs:1885` | açık | Kopya arama biçimi (aralığı değiştirmez) |
| `VICTORY_CUT_WRITES` | `list.rs:2004` | kapalı | (liste tarafı, bu yolla ilgisiz görünüyor) |
| `VICTORY_DIRECT_WRITES` | `host.rs:6039` | açık | Eşlenmiş belleğe doğrudan yazma |
| `VICTORY_REFRESH_CHECK` | `shadow.rs:2443` | kapalı | Refresh doğrulama (aşağıda doğrulama için) |

`memory.rs` bildirimini daraltan bir anahtar **yok**. Fill'i başka yoldan yapan bir anahtar da **yok**.

**(a) written'ı view alt aralığına daraltmak:** Tablo UAV'leri zaten alt aralığa daraltılmış durumda. Kaba olan root UAV (tamponun sonuna kadar) ve `memory.rs` (tüm kopya). Bunları daraltmak **doğruluk açısından riskli**: BVH kurucu, bellekten okuduğu işaretçilerle yazıyor (`memory.rs:32-37`). Bildirilmeyen bir yazma geri kopyalanmaz ve başlığın gördüğü bellekte kaybolur. Ancak shader tarafında kirli sayfa maskesi tutulursa güvenli olur (modülün kendi önerisi); bu da büyük iş.

**(b) Fill yerine başka yol (önerilen):** Tablo CBV'si, kaynak bir kopyadaysa ve `shadow.gpu(address) % 256 == 0` ise doğrudan kopyanın GPU adresine CBV olarak bağlanır; bunun için önce `shadow.refresh(list, address, constant_size(size))` çağrılır. Bu, root CBV'nin zaten izlediği yol (`draw.rs:483-486`: `if srv || gpu % 256 == 0 { shadow.refresh(...); return Some(gpu) }`). Sonuçlar:
- CPU artık uçuşların yazabileceği belleği okumuyor, Fill adımı yok, `blocked_by` beklemiyor.
- Kart CBV'yi **kuyruk sırasıyla** okuyor, yani önceki uçuşların kart kopyasına yazdıkları da görünür. Bu, konsoldaki davranışa daha yakın ("the card reads title memory when it runs", `host.rs:1931-1934`).
- Başlığın aynı aralığa uçuş sırasında yazması, zaten tüm SRV okumalarını kapsayan nokta 3'e düşüyor (`stale_under_flights`, `host.rs:2377-2405`; `late_exact` merge). Yani yeni bir risk sınıfı yok.
- Kalan riskler:
  1. CBV boyutu 256'ya yuvarlanıyor (480 → 512). Bu yüzden `left(address) >= constant_size(size)` şartı gerekiyor; değilse eski yol.
  2. Pencere kopyaları (`VICTORY_WINDOW_KB=256`): adres pencere sınırını aşıyorsa `left` yetmez ve eski yola dönülür.
  3. Refresh aralık sayısı artar, karta gönderilen parça ve hash geçişi sayısı biraz artabilir (**TAHMİN**; 64 KiB sayfa birimi zaten refresh'lenmiş olabilir).
  4. Ömür: refresh kopyayı liste sıfırlanana kadar canlı tutuyor (`host.rs:5466-5468` yorumu). `constant_at` ise `keep(index, None)` (`descriptor/host.rs:1631-1643`). Bu yeterli görünüyor, ama doğrudan bağlanan slotun kopya serbest kalmadan önce yeniden yazıldığı doğrulanmalı (**TAHMİN**).

**(c) İnce taneli kontrol:** `blocked_by` içinde Fill aralığının gerçekten bir UAV view'ine düşüp düşmediğine bakmak, `memory.rs` bildirimi tüm kopyayı kapsadığı sürece işe yaramaz: aralık "yazılabilir" kalır. Bildirimin kaynağını işaretlemek mümkün (ör. `GpuWrite`'a "kaba/ham-adres" bayrağı eklemek). Ama böyle bir bayrakla beklemeyi atlamak, BVH kurucu o baytı gerçekten yazıyorsa **eski veri okumak** demek. Kanıt olmadan önerilmez.

## 4. Öneri

**Önce 1 küçük teşhis (isteğe bağlı, ~6 satır, diagnostics):** `memory.rs:306` içinde `len >= 64 MiB` olan kopya bildirildiğinde "ilk N kez" `tracing::info!(start, len, "memory: whole copy declared written")`; `draw.rs:1209` için de aynısı (root UAV, `len >= 64 MiB`). Hangisinin 0x1b4e0000'ı kapladığı böylece kesinleşir. Bu, §1'deki TAHMİN'i kanıta çevirir. Yamanın kendisi bu sonuçtan bağımsız çalışır.

**İlk yama (env arkasında, kapalıyken birebir aynı):** `draw.rs:2711-2718` (`Some(View::Constant(address, size))` dalı), ~15 satır.

```rust
Some(View::Constant(address, size)) => {
    // VICTORY_CBV_DIRECT=1: a constant view inside a host copy, on 256 bytes
    // there, is bound at the copy (as root_view does) — no fill, no CPU read
    // of memory flights may write.
    if cbv_direct()
        && let Some(s) = crate::shadow::find(address, u64::from(size))
        && let gpu = s.gpu(address)
        && gpu % 256 == 0
        && s.left(address) >= u64::from(constant_size(size))   // 480 -> 512
    {
        s.refresh(list, address, u64::from(constant_size(size)));
        host.constant_at(index, gpu, size);
        continue;   // (or fall through matching the surrounding loop shape)
    }
    // existing: scratch + defer_fill + constant_at
}
```

- `cbv_direct()`: `OnceLock`, `std::env::var_os("VICTORY_CBV_DIRECT").is_some_and(|v| v == "1")`, **varsayılan kapalı**. Kapalıyken kod yolu aynı kalır.
- `constant_size` `descriptor/host.rs:1999`'da private. Ya `pub(crate)` yapılır ya da `size.min(64<<10).next_multiple_of(256)` satır içinde yazılır.
- `s.left`, `s.gpu`, `s.refresh` imzaları `root_view`'deki kullanımla aynı (`draw.rs:479-486`). Uygulayıcı tipleri (`u64`/`usize`) oradan kopyalamalı.
- Dokunulmayacaklar: `blocked_by`, `memory.rs`, root UAV, list.rs footprint Fill.

**Beklenen kazanç (TAHMİN):** Fill kaynaklı `relay_wait` karede ~1 kez ve 39-62 ms sürüyordu (önceki rapor §0). Bu bekleme sıfıra iner. Gönderim iş parçacığı bu süre boyunca hazırlığa devam edebilir, kare ~272 ms → ~215-235 ms. "Title memory read" settle beklemesi (karede 14.3 ms, satır 20775) de büyük ölçüde düşer. Kazancın bir kısmı başka bir beklemeye kayabilir: `WHY_MOST`, nokta 3, ya da hazırlık zaten kartı bekliyorsa.

**Doğrulama ölçütü (aynı sahne, aynı `ayarlar.txt` + `VICTORY_CBV_DIRECT=1`):**
1. `grep -c "a fill of upload memory waits"` → 0 (ya da yalnız `size` hizasız/pencere sınırı olgular). Kare satırında "title memory read … a fill of upload memory" → 0.0.
2. "the card's clock per frame … the submitting thread stood waiting for the card X ms" düşmeli. "of the waiting for the card in settles … title memory read" 14.3 → ~0.
3. Kare süresi/fps (frame satırı) ve "preparing" ile "GPU" sütunları önceki koşuyla karşılaştırılmalı.
4. Görüntü: aynı karelerin ekran görüntüleri yan yana konmalı (`kareler/`). Bakılacaklar: patlayan üçgen, kaybolan ağ, yanlış ışık/malzeme (CBV yanlış veri belirtisi).
5. Ek güvence: Bir koşuda `VICTORY_REFRESH_CHECK=1` (`shadow.rs:2443`) açılır, refresh'in kaçırdığı yazma uyarısı çıkıp çıkmadığına bakılır. Bir koşuda da `VICTORY_RELAY=1` (her şeyi bekleyen eski yol) + yama ile görüntü karşılaştırılır.
6. Yan etki: "host copies in the card's memory … sent up in N ranges" ve "hash passes" sayılarındaki artış kabul edilebilir olmalı (ör. <%20).

**Sonraki adım (ayrı iş, kanıt sonrası):** Kök neden `memory.rs`'nin tüm kopyayı bildirmesi. Kalıcı çözüm, shader tarafında kirli sayfa maskesi ya da bildirimin BVH yapılarının gerçek ayırma sınırlarına daraltılması (`memory.rs:32-37`'deki açık not). Bu, ilk yamadan bağımsız ve daha büyük bir iş.
