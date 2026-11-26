# Koşu reçetesi (25.11) — her oyun koşusu bu sırayla

Kaynaklar: HARNESS_NOTLARI.md (Neon), AYARLAR.md, GUVENLIK.md, ALTYAPI_PLANI.md.
Taban: vw-runtime @ 02ea29c (diag + release derli), harness @ 7b3a5d6.
Araçlar bu PC'de var: Node v24.19, ffmpeg 9.0.1 (gfxcapture destekli), Git Bash.

Konumlar: config + cache + data `%LOCALAPPDATA%\Victory\data-root\`, koşu klasörleri `%LOCALAPPDATA%\Victory\kosular\<tarih>_<tag>\` (C:\CLLL git deposu/kasa dışında), tablo `KOSULAR.md`.
Başlatıcı: `C:\CLLL\.calisma\repo_durum\olcum_profili.ps1` (içeride guvenli_baslat.ps1; -DryRun ile önce bak).

## A. Her koşudan önce (kontrol listesi, 2 dk)
1. Bu açılışta PC yeniden başlatıldı mı? (kernel havuzu 3.4 GB'tı → temizlenmeli)
2. Kapalı: Brave, Discord, Steam, NVIDIA Broadcast, qBittorrent. (Görev Yöneticisi'nde RAM boşta ≥ 9 GB)
3. Şarj aleti takılı, Windows güç modu "En iyi performans", laptop sert zeminde.
4. NVIDIA ataması: HKCU UserGpuPreferences'ta 4 victory.exe = `GpuPreference=2;` (25.11'de yapıldı; exe yeniden derlense de yol aynı kaldıkça geçerli).
   NEDEN ŞART: bu laptopta DXGI adaptör 0 = Intel Iris Xe; victory varsayılan adaptörü alıyor (victory-gpu/src/host.rs:603). Ayar yoksa oyun Intel'de açılır.
5. Güvenlik duvarı kuralları var mı: `Get-NetFirewallRule -DisplayName "victory-fork engel*"` (4 exe × giden/gelen).
6. Başka ağır iş yok (derleme, render, başka oyun).

## B. Oyun klasörü ilk geldiğinde (bir kez)
1. GUVENLIK.md sırası: kaynak kaydı → Defender tam tarama → dosya listesi → exe/DLL SHA-256 (masaüstü exe hash'i ile).
2. Klasör repo DIŞINDA olmalı (ör. `D:\the title\`; C:\CLLL içine KOYMA).
3. `olcum_profili.ps1 -Mode warm -GameDir <klasör> -Tag warm` → shader ön ısıtma.
   - --jobs varsayılan (çekirdek − 4 = 16); RAM boşu 1.5 GB altına inerse guvenli_baslat keser → `-Set` yerine jobs 6 ile tekrar.
   - Kanıt: koşu klasöründe ozet.txt "normal çıkış", cache klasörü doldu.

## C. Koşu 1 — "çökmeden menü" (Faz 1)
`olcum_profili.ps1 -Game <oyun exe> -Tag menu -MaxMinutes 6`
- Series S, 1080p, AYARLAR.md başlangıç seti otomatik.
- Elle menüye kadar gir, menüde 60 sn bekle, sonra pencereyi kapat (ya da koşu klasörüne `DURDUR` dosyası koy).
- Kanıt: kosu_ozeti.md, olcum.csv, run.log, ekran görüntüsü (Win+Shift+S). crashes.log yoksa iyi.
- Durdurma ölçütleri (guvenli_baslat otomatik): commit tavanı 13 GB, boş RAM < 1.5 GB, boş commit < 6 GB, GPU > 87 °C (5 sn sürerse), süre sınırı.

## D. Koşu 2 — sabit sahne kaydı (Faz 2 başı, bir kez)
1. İlk oynanabilir sahneye (hapishane başı / ilk autosave) kadar oyna.
2. Oyundan çık; harness ile KENDİ düz isimli kaydını al:
   `node C:\CLLL\.calisma\vw-harness\harness\save-snapshot.mjs save-as hapishane-basi --config %LOCALAPPDATA%\Victory\data-root\victory.toml --snapshots %LOCALAPPDATA%\Victory\data-root\snapshots`
   (Kural: yalnız kendi snapshot'larımız; başkasının kaydı yüklenmez.)

## E. Ölçüm koşusu (Faz 2-3, her deneme için)
1. Snapshot'ı geri yükle: `node C:\CLLL\.calisma\vw-harness\harness\save-snapshot.mjs restore hapishane-basi --config %LOCALAPPDATA%\Victory\data-root\victory.toml --snapshots %LOCALAPPDATA%\Victory\data-root\snapshots` (mevcut kaydı önce yedekler).
2. `olcum_profili.ps1 -Game <exe> -Tag <deneme-adi> -MaxMinutes 5 [-Set @{...}]`
3. Sahneye girince 90 sn karakteri SABİT tut (dokunma) → kare süresi dağılımı temiz olur.
4. Aynı ayarla 2 koşu (gürültü payı). Fark < gürültü ise "etkisiz" say.
5. KOSULAR.md satırı otomatik; görüntü bozulması (duvar/ışık/doku kaybı) varsa satıra not düş ve değişikliği geri al.

## F. Bir değişiklik denemesi (Faz 3 döngüsü)
TEK değişken. Sıra ALTYAPI_PLANI.md §FAZ 3. Örnek:
- Doku bütçesi: `-Set @{VICTORY_TEXTURE_ROOM_PERCENT='90'}`
- Late wait kapalı karşılaştırma: `-Set @{VICTORY_LATE_WAIT='0'}`
- Kod yaması (Hyphen/JoshuaO22 commit'i): önce Neon güvenlik incelemesi → ayrı dal/worktree'de derle (`--locked`) → `-VictoryExe` ile o exe → GpuPreference kaydı o yol için de eklenmeli.

## Yasaklar (GUVENLIK.md)
- `--presentmon`, `wpr-capture.ps1` (yönetici ister) → p95/GPU median sütunları boş kalır; yerine run.log + olcum.csv.
- `run-queue.sh` yalnız kendi kuyruk dosyamızla. gpu-trace/renderdoc gerekmedikçe yok.
- `VICTORY_WAVE_WIDE` açılmaz (NVIDIA'da TDR). `VICTORY_GPU_TIMING=1` relay/late wait'i sıfırlar → yalnız bilinçli teşhis koşusunda.
- Tuş script'i (`-Keys`) yalnız diag ve yalnız `sn:Tuş` biçimi (`70:J:8` sessizce yok sayılır; `Back`, `Backspace` değil).

## Harness'ın kendisi (world-run.sh) ne zaman?
Şimdilik HAYIR. İki engel: run-world.sh:325 repo içinde `cd` yapıyor ve raise-priority.ps1 bizim BelowNormal kilidiyle çatışıyor.
Baz ölçüm olcum_profili.ps1 ile; harness'ın compare-runs/summarize-run'ı ileride log okuyucu olarak kullanılabilir.
