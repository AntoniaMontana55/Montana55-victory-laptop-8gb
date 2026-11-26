# Oyun koşusu sırasında run.log'u canlı izler; ilk çökme/bellek işaretinde anında rapor yazar.
# Çıktı: <koşu klasörü>\cokme_raporu.txt (işaretten önceki 30 satır + yığın + son 15 sn ölçüm) ve konsola 1 satır.
# Kullanım: cokme_izle.ps1 [-KosuDir <klasör>] [-BeklemeDk 2]
param([string]$KosuDir, [ValidateRange(1, 30)][int]$BeklemeDk = 2)
$ErrorActionPreference = 'Stop'

$son = (Get-Date).AddMinutes($BeklemeDk)
while (-not (Get-Process victory -ErrorAction SilentlyContinue)) {
    if ((Get-Date) -gt $son) { Write-Output 'victory.exe başlamadı.'; exit 1 }
    Start-Sleep -Seconds 1
}
if (-not $KosuDir) {
    $KosuDir = (Get-ChildItem (Join-Path $env:LOCALAPPDATA 'Victory\kosular') -Directory | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
}
$log = Join-Path $KosuDir 'run.log'
$csv = Join-Path $KosuDir 'olcum.csv'
$rapor = Join-Path $KosuDir 'cokme_raporu.txt'

# İlk görülen her tür ayrı kaydedilir (aynı türün tekrarı atlanır).
$isaretler = [ordered]@{
    'bellek-red'   = 'no shared buffer|no upload buffer|no host texture'
    'istisna'      = 'first-chance exception'
    'panic'        = 'panicked at|memory allocation of \d+ bytes failed'
    'llvm'         = 'LLVM ERROR'
    'cihaz-kaybi'  = 'DEVICE_REMOVED|device removed|0x887A000[15]'
    'pipeline-red' = 'host pipeline refused'
}
$gorulen = @{}
$okunan = 0
Write-Output "İzleniyor: $log"
while ($true) {
    $calisiyor = [bool](Get-Process victory -ErrorAction SilentlyContinue)
    if (Test-Path $log) {
        $satirlar = @(Get-Content $log -Encoding UTF8 -ErrorAction SilentlyContinue)
        for ($i = $okunan; $i -lt $satirlar.Count; $i++) {
            foreach ($tur in $isaretler.Keys) {
                if (-not $gorulen.Contains($tur) -and $satirlar[$i] -match $isaretler[$tur]) {
                    $gorulen[$tur] = $true
                    $zaman = Get-Date -Format 'HH:mm:ss'
                    $bas = [math]::Max(0, $i - 30); $bit = [math]::Min($satirlar.Count - 1, $i + 40)
                    $olcum = if (Test-Path $csv) { Get-Content $csv -Tail 15 } else { @() }
                    $blok = @("===== $zaman  [$tur]  satır $($i + 1) =====", $satirlar[$i], '--- bağlam ---') +
                            ($satirlar[$bas..$bit] | ForEach-Object { if ($_.Length -gt 400) { $_.Substring(0, 400) } else { $_ } }) +
                            @('--- son 15 sn ölçüm (zaman,boşRAM,boşCommit,priv,ws,cpu,sıcaklık,vramKullanılan,vramToplam,util) ---') + $olcum + @('')
                    Add-Content -Path $rapor -Value $blok -Encoding UTF8
                    $kisa = if ($satirlar[$i].Length -gt 160) { $satirlar[$i].Substring(0, 160) } else { $satirlar[$i] }
                    Write-Output "$zaman ÇÖKME İŞARETİ [$tur]: $kisa"
                }
            }
        }
        $okunan = $satirlar.Count
    }
    if (-not $calisiyor) { break }
    Start-Sleep -Seconds 2
}
Write-Output "Oyun kapandı. Görülen işaretler: $(@($gorulen.Keys) -join ', ')"
