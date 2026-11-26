# Port projelerinin log / teşhis yöntemleri (25.11, Neon araştırması; Claude kısmen doğruladı)

Doğrulanan: `VICTORY_CAPTURE` bizim klonda var (crates/victory-gpu/src/capture.rs, host.rs).

## Bizim tabanda (victory / victory-fork) hazır olanlar
- `VICTORY_LOG` (debug/trace yalnız `--features diagnostics` = diag derlemesi)
- Çökme: `%LOCALAPPDATA%\victory\data\crashes.log`; `VICTORY_CRASH_BREAKPOINTS=1` (yığın + oyunun son çıktısı)
- Kare yakalama: `VICTORY_CAPTURE=<klasör>`, `VICTORY_CAPTURE_FRAMES`, `VICTORY_CAPTURE_WORLD` → PNG + `frame-<n>-ops.txt` (draw/dispatch/copy sırası), `before-world-ops.txt`
- `VICTORY_D3D12_DRED=1` (cihaz kaybı izi, ~2.5x yavaş → yalnız gerektiğinde), `VICTORY_D3D12_WARP=1`
- `VICTORY_KEYS` (betikli tuş), `VICTORY_SHADER_LOOP_LIMIT`, `victory shader-info --disasm`, `victory translate --validate`

## Başka projelerden alınabilecek fikirler
- AnyXboxX: `VICTORY_PRODUCERS=1` (çevrilmeyen shader'ın çıktısını okuyan pass), `VICTORY_SHOW_TARGET=busiest`, `VICTORY_PROFILE=<ms>` (en yoğun yığınlar), `VICTORY_RT_MISS=1`, `VICTORY_D3D12_DEBUG=1` (NVIDIA'da reddedilen 16 pipeline'ın 15'i wave64), tek kare PIX.
- victory-amd: `VICTORY_NULL_MAGENTA` (boş görünüm macenta), `VICTORY_TEXTURE_SWEEP`, `VICTORY_VIEW_GUARD=0`, `VICTORY_HEAP_RECHECK=0`.
- five-star-recomp: süreç dışı debugger (minidump, `--watch-hang` 5/15/25 sn yığınlar, `--memory-map` VRAM/commit, `--max-commit`), `--probe-pipeline`, ekran tanıyan girdi script'i.
- xcl: büyük PSO derlemesini tek şerit + 3 GB boş commit kapısı (16 GB'ta 1455 commit hatasına karşı), `report.json` + önceki koşu `.prev`.
- TuxBox: shader ret sebeplerini kümeleme, kare zamanı CSV + p95/p99 özeti.
- ScarlettPort: `PRESS_FILE`, `FAST_BOOT`, `NO_WATCHDOG`, çökme zip paketi; issue şablonu RAM + page file soruyor.

## Bizim 16 GB / 3070 Ti 8 GB için 5 teknik
1. Bellek bekçisi: commit + VRAM'i saniyede bir kaydet; büyük shader derlemesi sırasında commit tükenmesi (hata 1455) en olası çökme.
2. Release temiz; diag + DRED yalnız cihaz kaybı incelemesinde. Eksik görüntüde `frame-ops` + PRODUCERS.
3. Series S profili (720p iç çözünürlük) + watchdog kapatma + tekrarlanabilir girdi script'i.
4. Tam oturum yakalama yok; şüpheli tek pipeline probe + tek kare PIX.
5. Süreç dışı minidump yakalayıcı + her koşuya otomatik rapor/log klasörü.

Not: ZSlayerHQ'nun GTA portu açık değil (GitHub'da yalnız dokunulmamış leonida fork'u). ubiquitarianism/leonida (1962 yıldız, Vulkan tabanlı ayrı proje) README'sine göre oyun henüz açılmıyor (25.11).
