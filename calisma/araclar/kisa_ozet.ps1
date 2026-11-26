# Kısa koşu özeti: dünya kare medyanı + belirli saniyedeki bellek durumu.
# Kullanım: kisa_ozet.ps1 <koşu klasörü> [-Saniye 200]
param(
    [Parameter(Mandatory)][string]$Kosu,
    [int]$Saniye = 200,
    [int]$DunyaSonra = 0
)
$ErrorActionPreference = 'Stop'
$log = Join-Path $Kosu 'run.log'
$ad = Split-Path $Kosu -Leaf

# Koşunun başlangıcı (ilk log satırının zamanı).
$ilk = Get-Content $log -TotalCount 1
$bas = [datetime]::Parse($ilk.Substring(0, 27), [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AdjustToUniversal)
$hedef = $bas.AddSeconds($Saniye)
$zaman = { param($satir) [datetime]::Parse($satir.Substring(0, 27), [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AdjustToUniversal) }

# İlk dünya karesi (draw >= 2000) ve dünya kare medyanı.
$kareler = Select-String -Path $log -Pattern 'host: ([\d.]+) fps, ([\d.]+) ms/frame.*?(\d+) draws/frame' | ForEach-Object {
    [pscustomobject]@{ T = & $zaman $_.Line; Ms = [double]$_.Matches[0].Groups[2].Value; Draw = [int]$_.Matches[0].Groups[3].Value }
}
$dunya = @($kareler | Where-Object { $_.Draw -ge 2000 })
$ilkDunya = if ($dunya.Count) { [int]($dunya[0].T - $bas).TotalSeconds } else { -1 }
$medyan = if ($dunya.Count) { ($dunya.Ms | Sort-Object)[[int][math]::Floor($dunya.Count / 2)] } else { 0 }
$p90 = if ($dunya.Count) { ($dunya.Ms | Sort-Object)[[int][math]::Floor($dunya.Count * 0.9)] } else { 0 }

# -DunyaSonra verilirse hedef = ilk dünya karesi + o kadar saniye (Enter deseni farklı koşular için).
if ($DunyaSonra -gt 0 -and $dunya.Count) { $hedef = $dunya[0].T.AddSeconds($DunyaSonra) }
# Hedef saniyeye en yakın bellek satırları.
function En-Yakin($desen) {
    $s = Select-String -Path $log -Pattern $desen | ForEach-Object { [pscustomobject]@{ T = & $zaman $_.Line; L = $_.Line } }
    $s | Sort-Object { [math]::Abs(($_.T - $hedef).TotalSeconds) } | Select-Object -First 1
}
$vm = En-Yakin 'video memory as the system counts it'
$mem = En-Yakin 'copystat: memory: '
$kart = if ($vm.L -match "card's own: budget (\d+) MB, in use (\d+) MB") { "$($matches[2])/$($matches[1])" } else { '?' }
$kilitli = if ($vm.L -match "shared with the card: budget \d+ MB, in use (\d+) MB") { $matches[1] } else { '?' }
$priv = if ($mem.L -match 'memory: (\d+) MB private \((\d+) MB of it in memory; the PC has (\d+) MB free') { "priv $($matches[1]) / bellekte $($matches[2]) / PC boş $($matches[3])" } else { '?' }
$pen = if ($mem.L -match '(\d+) windows outside buffers (\d+) MB') { "$($matches[1]) pencere $($matches[2]) MB" } else { '?' }
$an = if ($vm) { [int]($vm.T - $bas).TotalSeconds } else { -1 }

# Koşu başındaki boş RAM (Brave/T3 vb. o anki yükü; koşular arası adil karşılaştırma için).
$csv = Join-Path $Kosu 'olcum.csv'
$basBos = if (Test-Path $csv) { $r = @(Import-Csv $csv | Select-Object -First 2); if ($r.Count) { '{0:N1}' -f [double]$r[0].BosRAM_GB } else { '?' } } else { '?' }
$enAz = if (Test-Path $csv) { '{0:N2}' -f ((Import-Csv $csv | Measure-Object BosRAM_GB -Minimum).Minimum) } else { '?' }
"başta boş RAM $basBos GB, en düşük $enAz GB"
# PC kullanımı (kullanim_izle.ps1): oyun önde değilse ya da başka programlar CPU yediyse koşu KİRLİ sayılır.
$etiket = ($ad -split '_', 2)[1]
$kul = Join-Path (Split-Path $Kosu) "kullanim_$etiket.csv"
if (Test-Path $kul) {
    $k = @(Import-Csv $kul | Select-Object -Skip 1)
    if ($k.Count) {
        $onde = 100.0 * @($k | Where-Object { $_.on_surec -match '^(victory|game_scarlett_master)$' }).Count / $k.Count
        $cpu = $k | ForEach-Object { [double]::Parse($_.diger_cpu_cekirdek, [Globalization.CultureInfo]::InvariantCulture) } | Measure-Object -Average -Maximum
        $digerOn = ($k | Where-Object { $_.on_surec -notmatch '^(victory|game_scarlett_master)$' } | Group-Object on_surec | Sort-Object Count -Descending | Select-Object -First 3 | ForEach-Object { "$($_.Name) $([int](2 * $_.Count)) sn" }) -join ', '
        $kirli = $onde -lt 90 -or $cpu.Average -gt 1.5
        "kullanım: oyun önde %{0:N0}, diğer programlar CPU ort {1:N1} / tepe {2:N1} çekirdek{3}{4}" -f $onde, $cpu.Average, $cpu.Maximum, $(if ($digerOn) { " | önde: $digerOn" } else { '' }), $(if ($kirli) { ' | ⚠ KİRLİ' } else { ' | temiz' })
    }
}
"{0} | ilk dünya {1} sn | dünya n={2} medyan {3} ms ({4:N1} fps) p90 {5} | @{6} sn: kilitli paylaşım {7} MB, kart {8} MB, {9}, {10}" -f $ad, $ilkDunya, $dunya.Count, $medyan, (1000 / [math]::Max($medyan, 1)), $p90, $an, $kilitli, $kart, $pen, $priv
