# GTA / victory-fork — güvenlik düzeni (25.11, Yiğit: "güvenlik çok önemli")

Her çalıştırmadan önce bu liste uygulanır. Atlanan madde olursa Yiğit'e söylenir.

## 1. Kod (fork güncellemesi, PR, başka fork'tan port)
- Yalnız incelenmiş commit derlenir. Son incelenen: `c96b4f2` (25.11; 2119b0e Neon+Sol+Claude, 2119b0e..c96b4f2 Neon+Claude: GÜVENLİ).
- YENİ TABAN (25.11 karar): ZSlayerHQ/victory-world-runtime `next` @ `99f8834` — c96b4f2..99f8834 Neon+Claude: GÜVENLİ (yeni crate yok, build.rs/net.rs/CI değişmedi, tek Command::new kernel.rs:704 testte). Harness ZSlayerHQ/victory-world @ `566d5be`: GÜVENLİ; kurallar: run-queue.sh yalnız kendi kuyruk dosyamızla (eval), `--presentmon` yönetici ister → kullanılmaz, set-volume yalnız oyunun PID'ine dokunur. Rapor: guvenlik_zslayer.md.
- GÜNCEL TABAN (25.11 13:10): runtime `next` @ `02ea29c7eeb22da87156676c332dbb157cb9b681` (99f8834..02ea29c Neon+Claude GÜVENLİ: lock/toml/build.rs/net.rs aynı; host.rs = test için yazılım adaptör algılama + başlık). Harness @ `7b3a5d62d4af8b9c498ad653fdf88007d1feac08` GÜVENLİ (kurallı). Ek harness kuralları: `wpr-capture.ps1` yönetici ister → kullanılmaz; `save-snapshot.mjs --save` mevcut save klasörünü siler → yalnız kendi düz isimli snapshot'larımız; `gpu-trace.mjs`/`renderdoc.mjs` dış araç → gerekmedikçe yok. Rapor: guvenlik_zslayer2.md.
- Yeni commit/PR alınmadan önce: yalnız `<son incelenen>..<yeni>` farkı Neon'a salt okunur güvenlik incelemesi; Claude şüpheli yerleri kendisi açar.
- Kırmızı bayrak (görülürse durdur, Yiğit'e sor): yeni `build.rs`, `Cargo.lock`'ta yeni paket (özellikle ağ/http/tokio), `git=` bağımlılık, ağ API'si (reqwest, socket, WinHTTP...), `Command::new`/powershell/curl, registry yazma, kullanıcı klasörü okuma, base64/gömülü ikili, `net.rs` loopback kilidinin gevşetilmesi, CI'da secret/`pull_request_target`.
- Derleme her zaman `--locked`. Kimsenin hazır exe'si, script'i, Discord/Mega/Drive linki çalıştırılmaz; yalnız kaynaktan biz derleriz.

## 2. Oyun dosyaları (en büyük risk)
- `--execute` oyunun kodunu doğrudan işlemcide, Yiğit'in kullanıcı yetkisiyle çalıştırır. Programın ağ kilidi yalnız oyunun Windows'tan istediği ağ çağrılarını keser; kurcalanmış bir dosyadaki kod bunu atlayabilir. Bu yüzden dosyanın kaynağı kritik.
- Çalıştırmadan önce:
  1. Kaynak soruldu ve not edildi.
  2. Windows Defender ile klasörün tamamı taranır (`MpCmdRun -Scan -ScanType 3 -File <klasör>`).
  3. Dosya listesi incelenir: konsol dump'ına ait olmayan şeyler (`.bat`, `.ps1`, `.cmd`, `.vbs`, `.lnk`, `.scr`, `setup.exe`, "crack/loader/launcher" exe'si, `.url`) varsa durulur.
  4. Ana exe ve DLL'lerin SHA-256'ları alınır; topluluk issue'larında paylaşılan sürüm/hash bilgisiyle karşılaştırılır (ör. `game_scarlett_master.exe` 1.0.2.1).
- Oyun klasörü repo dışında; hiçbir zaman yönetici olarak çalıştırılmaz.

## 3. Çalıştırma ortamı (savunma katmanları)
- Windows Güvenlik Duvarı'nda iki `victory.exe` (release + diag) için GİDEN bağlantı engeli (yönetici onayı gerekir, Yiğit bir kez onaylar). Etkin ağ profili Public; Public'te güvenlik duvarı açık.
- Defender gerçek zamanlı koruma açık kalır (25.11: açık, imzalar güncel).
- Windows 11 Home'da Windows Sandbox yok. Ek izolasyon istenirse: ayrı standart (yönetici olmayan) Windows kullanıcısı. Tarayıcı oturumları/token'lar o kullanıcıda olmaz.
- Çalışırken tarayıcıda hassas oturumlar kapalı tutulabilir (isteğe bağlı).

- Çalıştırma HER ZAMAN `guvenli_baslat.ps1` ile (25.11, Sol yazdı, Claude inceledi + SelfTest 2/2 kendisi koştu): Job Object commit tavanı (vars. 13 GB), kapanınca oyunu da öldürür, BelowNormal; boş RAM <1.5 GB / boş commit <6 GB / GPU >87°C 5 sn → durdur; 10 dk süre; DURDUR dosyası; ağ/registry/admin yok. Önce PC yeniden başlat, Brave/Discord/Steam/Broadcast/qBittorrent kapalı. VRAM'e tavan konamaz → victory doku/kart bütçesi.

## 4. Kayıt
- Her çalıştırma: hangi commit, hangi dosya kaynağı, hangi ayarlar → STATUS.md tablosu.
- 25.11 20:15 İKİNCİ EXE: fxceless/victory-fork 310c6eb..465b94f5 (8 commit) Neon GÜVENLİ (guvenlik_vf3.md); Claude doğruladı: Cargo.lock/toolchain/CI aynı, yalnız victory-gpu/Cargo.toml windows-sys'e EventLog+ToolHelp feature; yeni crates/victory-shader/build.rs yalnız src okur + cargo direktifi; diff'te Command::new/ağ/registry yok. Yeni varsayılan-açık: DRIVER_WATCH (System olay günlüğünü salt okur), FAST_CORES (kendi thread'leri P-core). Worktree C:\CLLL\.calisma\vf-465 @465b94f5, güvenlik duvarı betiğine + GpuPreference=2'ye eklendi.
- 25.11 20:45 ÜÇÜNCÜ EXE: ZSlayerHQ/zslayer-pc-port-runtime PR #10 (Cleverdee) e7c05277 Neon GÜVENLİ (guvenlik_pr10.md); Claude doğruladı: kernel.rs +17 yalnız VICTORY_NO_FSR=1'de FSR DLL'ini REDDEDEN kapı (gevşetme değil), diff'te ağ/Command/registry/dosya yazma yok, URL'ler yalnız README; shader'lar düz metin HLSL (D3DCompile). Varsayılan açık: SUBMIT_EACH, CUT_DISPATCHES=32 (her modda). Worktree C:\CLLL\.calisma\vw-pr10, GpuPreference=2; güvenlik duvarı: guvenlik_duvari_ek_pr10.ps1 (kural 7, Yiğit yönetici).
