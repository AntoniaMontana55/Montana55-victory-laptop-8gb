# victory-16gb — çalışma defteri

the title (Xbox Series sürümü) "victory" çalışma zamanını **RTX 3070 Ti Laptop (8 GB) + 16 GB RAM** gibi NVIDIA ve 16 GB makinelerde çalıştırma işi.
Kod: bu repo, `nvidia-16gb` dalı. Taban `avatarmanz/VIctory-7900XTX @0a435665` (fxceless/victory-fork soyundan, GPL-3.0); geçmişi taşınmadı, ilk commit tabanın birebir ağacıdır.

## Nerede ne var
| Klasör | İçerik | Asıl yeri (çalışılan kopya) |
|---|---|---|
| `belgeler/STATUS.md` | Hedef, sıradaki adımlar, kanıtlı/kanıtsız, her koşunun sonucu (en yeni üstte) | `C:\CLLL\.calisma\victory-fork\STATUS.md` |
| `belgeler/ADAYLAR.md` | Diğer repolardan alınabilecek her şeyin tek listesi ve durumu | aynı klasör |
| `belgeler/KOSULAR.md` | Koşu defteri (olcum_profili otomatik yazar) | aynı klasör |
| `raporlar/` | Alt ajan araştırma ve güvenlik raporları | `C:\CLLL\.calisma\repo_durum\` |
| `araclar/` | Koşu, ölçüm, erken teşhis, güvenlik betikleri | `C:\CLLL\.calisma\repo_durum\` |
| `yamalar/` | Elle taşınan / reddedilen dış yamalar | `C:\CLLL\.calisma\yamalar_dis\` |

`araclar/kaydet.ps1 -Mesaj "..." [-Gonder]` bu kopyaları günceller, gizli bilgi kapısından geçirir ve commit eder.

## Çalışma yöntemi
1. **Tek değişken:** her deney tek bir şeyi değiştirir (kod commit'i ya da tek env).
2. **Ölçüm:** kısa koşu 4.5 dk, oyuna dokunulmaz, Enter basılı tutma deseni; bellek dünya+90 sn'de, hız dünya karelerinin medyanı. Gürültü ~%2.
3. **Karar:** kazanan ayar 2 tekrarla ve aynı oturumdaki eski exe'yle (A/B/A) doğrulanır; 10 dk dayanıklılık yalnız kazanana.
4. **Kayıt:** sonuç STATUS'a, deney commit'i bu dala; reddedilen değişiklik geri alınır ve nedeni yazılır.
5. **Güvenlik:** dış kod önce salt okunur incelemeden geçer; hazır exe/script indirilmez; oyun yalnız `guvenli_baslat.ps1` ile, yönetici olmadan, güvenlik duvarı kuralıyla çalışır.
