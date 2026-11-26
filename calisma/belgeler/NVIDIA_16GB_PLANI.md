# NVIDIA + 16 GB planı (25.11, Yiğit'in isteği: sistemli, rastgele deneme yok)

Hedef makine: RTX 3070 Ti Laptop 8 GB, i7-12700H, **16 GB RAM**. Hedef kitle aynı: 16 GB'lı NVIDIA makineler.
Temel: ZSlayer runtime `02ea29c` + PR #10 (vw-pr10). Diğer repolar parça mağazası.

## 1. Kilometre taşları (sırayla; biri bitmeden sonrakine geçilmez)

| # | Taş | "Bitti" kanıtı | Durum |
|---|-----|----------------|-------|
| K1 | 16 GB'ta yüklemeyi geçmek | Koruma durdurmadan oyun menüsü/ilk sahne ekranda, koşu klasöründe ekran görüntüsü + olcum.csv | **TAMAM 25.11 21:39** (Koşu 16: açılış sinematiği, kare_0116s.png; anahtar: Series S + PR #10 + PR #32 f67 yaması + TEXTURE_BUDGET_MB=2048 + Job tavanı 20) |
| K2 | Kararlı 10 dk | 10 dk çökme/koruma durdurması yok; kendi `hapishane-basi` kaydımız alındı | **10 dk kısmı TAMAM** (Koşu 17+18, WINDOW_KB=256); kendi kayıt (snapshot) alınmadı — Enter otomasyonu her koşuda prologa getiriyor, şimdilik yeterli |
| K3 | Baz ölçüm | Sabit sahnede 2 koşu: ort. ms/kare, RAM, VRAM, CPU/GPU meşgul | **TAMAM** — 263/272 ms (3.7-3.8 fps), kopya 119-127, geri kopya 143-149, GPU 48 ms; gürültü %3 |
| K4 | Hızlandırma | Aday hattından geçen her değişiklik için A/B satırı (KOSULAR.md) | **ŞİMDİ** — sıra: WIDE_REFRESH=1 → WINDOW_KB=128 → fxceless #1066/#648 |
| K5 | 30 fps'e yaklaşmak | Dürüst not: aynı kartta bilinen en iyi 4.6 fps (#413). 30 fps upstream ilerlemeye bağlı uzun yol | uzun vade |

## 2. K1 adımları (RAM duvarı)
1. Koşu 6: boş RAM eşiği 0.3 GB / 10 sn, Brave kapalı. Windows sayfa dosyasına itip yer açabiliyor mu? → sonucu yaz.
2. Neon `repo_durum/ram_kaynaklari.md`: kodda RAM'i kim ayırıyor (dosya:satır), hangi ayar küçültür.
3. Bu rapordan **tek değişkenli** RAM ayarları, en olası kazanç önce; her biri 1 koşu, sonuç KOSULAR.md'ye.
4. Ayar yetmezse: upstream'de RAM azaltan kod değişikliği aranır (aday hattı, §3) ya da kendi yamamız (Opus/Sol, ayrı dal).
5. Yan yol: sayfa dosyası boyutu/konumu kontrolü (yönetici gerekirse Yiğit onayı).

## 2b. K3-K4 için not: CPU (Yiğit, 25.11)
Yiğit'in gözlemi: CPU kullanımı çok düşük görünüyor. Ölçümümüz tüm makinenin yüzdesi (20 iş parçacığı): tek çekirdek %100 = toplamda ~%5. #413 de "CPU-bound" diyordu → büyük olasılıkla tek bir iş parçacığı (GPU gönderim/çeviri ana döngüsü) darboğaz, diğerleri boşta. K3'te ölçülecek: iş parçacığı başına CPU (en yoğun 5 thread, %), kare süresiyle birlikte. Doğrulanırsa K4 adayları: iş yükünü bölen/paralelleştiren commit'ler (vf-465 submit thread + copy queue, Belobug a0a4591 paralel copy-back, JoshuaO22 003beef2 kilit inceltme). Önce K1 (RAM).

## 3. Aday hattı (her değişiklik bu sırayla, atlanmaz)
`ADAYLAR.md` tek tablo; her aday bir satır, durum sütunu ilerler:

1. **Tespit** — günlük 09:00 watcher (Neon) yeni commit/PR/issue'yu bulur, ADAYLAR.md'ye `bulundu` yazar.
2. **Ön eleme** (Claude) — bizim soruna değiyor mu: RAM / NVIDIA siyah ekran-TDR / fps / çökme. Değmiyorsa `ret: kapsam dışı`.
3. **Güvenlik** — Neon salt okunur inceleme (ağ, dosya, süreç, build script) → Claude şüpheli yerleri kendisi doğrular → `incelendi`.
4. **Derleme** — Sol, ayrı klasör/dal, `cargo build --locked` → SHA kaydı → güvenlik duvarı kuralı (Yiğit yönetici) + GpuPreference=2 → `derlendi`.
5. **A/B** — olcum_profili.ps1, TEK değişken, 2 koşu, aynı sahne → `denendi`.
6. **Karar** — fark > gürültü ve görüntü bozulmadıysa `kabul` (temele alınır), değilse `ret: <neden>`.

## 4. Ritim
- Her gün 09:00: watcher → ADAYLAR.md yeni satırlar + 4-6 satır özet.
- Her koşu sonrası: KOSULAR.md satırı + STATUS.md.
- Aynı anda tek ağır iş (oyun koşusu veya derleme).
- Taş bitince Yiğit'e rapor; plan gerekirse burada güncellenir.

## 5. Değişmeyen güvenlik kuralları
GUVENLIK.md geçerli: yalnız incelenmiş commit, `--locked`, hazır exe/script yok, yönetici yok, oyun her zaman guvenli_baslat üzerinden, ekran görüntüsü yalnız oyun penceresi öndeyken.
