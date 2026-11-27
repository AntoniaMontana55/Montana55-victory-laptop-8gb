# Changelog

## 2026-11-27 — Tam doku hattı (RX 7600 8 GB), commit 519a38d

GTA 6 dokularının yarım/siyah/gri kalması sorunu için 6 katmanlı düzeltme:

- **Katman A — Dinamik TX bütçesi:** `VICTORY_TEXTURE_BUDGET_MB=dynamic` (veya `dxgi`): sabit
  1.4 GiB tavan yerine kartin kendi DXGI bütçesi (8 GB kartta 2.8–3.4 GiB'ye kadar).
  16 MiB'yi aşan tekil dokular committed kaynağa düşer, reddedilmez.
- **Katman B — Tam çözünürlük:** VRAM'in %90'ına kadar tüm dokular tam çözünürlükte ve
  `MipLODBias = 0.0`. Yalnızca gerçek baskıda (%91+) kademeli koruma: 1024/512px tavan ve
  en çok +0.5/+1.0 bias. Karakter/yüz/cilt (BAKING) dokuları hiçbir koşulda düşmez.
- **Katman C — Stale descriptor sonu:** `flip_now()` — bir dokunun upload'u bittiği karede
  `neutral_for` nötr stand-in'i gerçek SRV ile anında değişir; `critical_small` önceliği
  görünür dokuları (<= 8 MiB) kapsar, önce soğuk/ekran dışı tahliye edilir.
- **Katman D — Mip senkronizasyonu:** SRV `ResourceMinLODClamp` gerçekten yüklenmiş en
  detaylı mip'e kitli; mip 0 fence'i geçince 0.0 olur (siyah yerine önce temiz alt mip).
- **Katman E — Katmanlı tahliye:** `memory_safety_valve` Seviye 1 (LOCAL >= 6.2 GiB) yalnız
  scratch temizler; Seviye 2 (6.85 GiB/OOM) son 2 s / 90 karede kullanılan dokuları pin'ler,
  strict LRU ile en soğukları alır — pop-in döngüsü yok.
- **Katman F — Başlatıcı:** `scripts/play_rx7600_8gb.ps1` her iki profilde
  `BUDGET=dynamic`, `MAX=0`, `ROOM_PERCENT=93`.

OOM kırmızı çizgileri korunmuştur: `BURY_LAG=8`, `KEEP_FREED=0`, `SUBMIT_CHUNK=6`,
6.1 GiB hard cap, shader önbelleği ve `saves`/`settings.xml` dokunulmadı.

Testler: `texture_budget` / `room` / `emergency_eviction_pin` birim testleri eklendi.
