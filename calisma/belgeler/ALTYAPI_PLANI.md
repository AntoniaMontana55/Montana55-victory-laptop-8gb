# the title PC — Altyapı ve 30 fps planı (25.11, Yiğit: "aşırı planlı, adım adım")

Hedef: RTX 3070 Ti Laptop 8 GB / i7-12700H / 16 GB RAM'de oyun ÇÖKMEDEN, sonra kademeli olarak 30 fps (kare ≤ 33 ms).
Taban: ZSlayerHQ/victory-world-runtime `next` + ZSlayerHQ/victory-world (harness). Diğer repolar = parça mağazası.
Her adımın bir "bitti kanıtı" var; kanıt yoksa adım bitmiş sayılmaz. Her adım STATUS.md'ye işlenir.

## FAZ 0 — Altyapı (oyun dosyası GEREKMEZ) — şimdi
| # | Adım | Kim | Bitti kanıtı |
|---|---|---|---|
| 0.1 | Güvenlik: runtime 99f8834..02ea29c + harness 566d5be..son farkı incelemesi | Neon (salt okunur) + Claude şüpheli yerleri açar | guvenlik_zslayer2.md GÜVENLİ + GUVENLIK.md satırı |
| 0.2 | Ayrı çalışma kopyaları: runtime worktree `C:\CLLL\.calisma\vw-runtime` (zslayer/next @ onaylı SHA), harness klonu `C:\CLLL\.calisma\vw-harness` | Claude | `git rev-parse` = onaylı SHA |
| 0.3 | Derleme: release + diag, `--locked`, tek ağır iş | Sol (komut) / Claude doğrular | iki exe var, `--version`/`--help` çalışıyor, derleme logu |
| 0.4 | Testler: `cargo test --locked` (GPU gerektirmeyenler) | Sol | test özeti (geçen/atlanan/kalan) |
| 0.5 | Güvenlik duvarı: iki victory.exe için GİDEN engel | Yiğit (bir kez yönetici onayı) | `Get-NetFirewallRule` çıktısı |
| 0.6 | Harness okuma: başlatma, kuyruk dosyası, ilerleme akışı, yakalama tarifleri, koşu tabloları → kendi koşu reçetemiz | Neon okur, Claude reçeteyi yazar | KOSU_RECETESI.md |
| 0.7 | İzleme düzeni: guvenli_baslat.ps1 (RAM/commit/VRAM/sıcaklık, saniyelik) + victory'nin kare zamanlama/log/yakalama değişkenleri tek bir "ölçüm profili"nde | Sol script, Claude inceler | olcum_profili.ps1 + sahte süreçle deneme logu |
| 0.8 | Ayar envanteri: tüm VICTORY_* değişkenleri (Series S modu, doku bütçesi, kart alanı yüzdesi, wave genişliği, late wait/relay, GPU timing, capture) koddan çıkarılır | Neon (kod okuma) | AYARLAR.md (değişken, varsayılan, etkisi, dosya:satır) |
| 0.9 | Bellek hazırlığı: yeniden başlatma, arka plan kapatma listesi, NVIDIA GPU atanması (Windows Grafik ayarı) | Yiğit + Claude kontrol | kontrol listesi işaretli |

## FAZ 1 — Dosyalar inince: güvenli ilk açılış
1.1 Dosya kontrolü (Defender tam tarama, dosya listesi, SHA-256; masaüstü exe hash'iyle karşılaştır).
1.2 `victory warm` (shader ön ısıtma, ~4 dk+; bellek izlenerek).
1.3 İlk koşu: guvenli_baslat + Series S modu + 10 dk sınırı + pencere modu. Hedef yalnız "çökmeden menü".
1.4 Kanıt: ozet.txt, olcum.csv, crashes.log, ekran görüntüsü.

## FAZ 2 — Ölçüm tabanı (baseline)
2.1 Sabit test sahnesi seç (menü, hapishane başı) + tekrarlanabilir girdi (harness kuyruğu).
2.2 Her koşuda: kare süresi (ort/p95), CPU %, GPU %, kartı bekleme %, draw/kare, RAM, VRAM, kopya MB/kare.
2.3 Kayıt: KOSULAR.md tablosu (commit, ayarlar, sonuç). Aynı ayarla 2 koşu → gürültü payı.

## FAZ 3 — Darboğaz avı (her seferinde TEK değişiklik)
Sıra (beklenen kazanca göre):
3.1 Bellek/VRAM sığdırma (8 GB): doku bütçesi, kart alanı yüzdesi (%80 kuralı → %90-92 dene), Series S.
3.2 CPU tarafı çizim başı maliyet: Hyphen 3 commit (descriptor parmak izi), JoshuaO22 memory-gap (kilit), five-star içerik deposu fikri.
3.3 Bekleme: late wait/relay, geri okuma kuyruğu, kare bindirme.
3.4 Shader/NVIDIA: wave64→32 yolları, reddedilen programlar.
3.5 Çok çekirdek: tek iş parçacığı darboğazını bölme.
Her deneme: önce/sonra ölçüm + görüntü kontrolü (duvar/ışık bozulması = geri al).

## FAZ 4 — 30 fps'e yaklaşma
Kare bütçesi 33 ms: hedef dağılım CPU çeviri ≤ 15 ms, bekleme ≤ 5 ms, GPU ≤ 25 ms (paralel). Ölçümle güncellenir.

## Kurallar (değişmez)
- GUVENLIK.md: yalnız incelenmiş commit, `--locked`, admin yok, oyun hep guvenli_baslat ile.
- Aynı anda 1 ağır iş (derleme/oyun/render).
- GitHub'a hiçbir şey gönderilmez.
