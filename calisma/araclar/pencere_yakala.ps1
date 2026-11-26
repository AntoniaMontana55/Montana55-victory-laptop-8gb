# Oyun koşusu sırasında YALNIZ victory.exe penceresinin içeriğini aralıklarla PNG olarak kaydeder.
# Windows Graphics Capture (ffmpeg gfxcapture): pencere arkada kalsa bile sadece o pencere alınır,
# ekranın geri kalanı (tarayıcı vb.) asla kayda girmez.
# Kullanım: pencere_yakala.ps1 [-OutDir <klasör>] [-AralikSn 15] [-BeklemeDk 3]
# OutDir verilmezse en yeni %LOCALAPPDATA%\Victory\kosular\<koşu>\kareler klasörü kullanılır.
param(
    [string]$OutDir,
    [ValidateRange(5, 300)][int]$AralikSn = 15,
    [ValidateRange(1, 30)][int]$BeklemeDk = 3
)
$ErrorActionPreference = 'Stop'

# victory.exe başlayana kadar bekle
$son = (Get-Date).AddMinutes($BeklemeDk)
while (-not (Get-Process victory -ErrorAction SilentlyContinue)) {
    if ((Get-Date) -gt $son) { Write-Output 'victory.exe başlamadı, çıkılıyor.'; exit 1 }
    Start-Sleep -Seconds 2
}
if (-not $OutDir) {
    $kok = Join-Path $env:LOCALAPPDATA 'Victory\kosular'
    $kosu = Get-ChildItem $kok -Directory | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $OutDir = Join-Path $kosu.FullName 'kareler'
}
New-Item -ItemType Directory -Force $OutDir | Out-Null
Write-Output "Kareler: $OutDir"

$baslangic = Get-Date
while (Get-Process victory -ErrorAction SilentlyContinue) {
    $sn = [int]((Get-Date) - $baslangic).TotalSeconds
    $dosya = Join-Path $OutDir ('kare_{0:D4}s.png' -f $sn)
    # Yalnız victory.exe penceresi; imleç yok. Pencere yoksa ffmpeg hata verir, geçilir.
    # PS 5.1'de yerel komutun stderr'i Stop modunda hata sayılır; yakalama hatası koşuyu bitirmesin.
    $ErrorActionPreference = 'Continue'
    & ffmpeg -hide_banner -loglevel quiet -y -filter_complex "gfxcapture=window_exe='(?i)^victory\.exe$':capture_cursor=0:max_framerate=2,hwdownload,format=bgra,scale=960:-2" -frames:v 1 -update 1 $dosya 2>$null
    $ErrorActionPreference = 'Stop'
    Start-Sleep -Seconds $AralikSn
}
Write-Output "Bitti. $((Get-ChildItem $OutDir -Filter *.png).Count) kare."
exit 0  # son ffmpeg'in çıkış kodu (ör. pencere kapanırken 254) betiğin sonucu sayılmasın
