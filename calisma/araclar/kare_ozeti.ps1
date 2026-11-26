# Koşu karşılaştırması: run.log'daki diag "host: fps" satırlarından yalnız DÜNYA karelerini (draw >= MinDraw)
# alıp medyan ms/kare, kopya, geri kopya, GPU ms ve fps verir. Kullanım: kare_ozeti.ps1 <koşu klasörü> [...] [-MinDraw 2000]
[CmdletBinding(PositionalBinding = $false)]
param([Parameter(Mandatory, ValueFromRemainingArguments)][string[]]$Klasorler, [int]$MinDraw = 2000)

function Get-Medyan([double[]]$x) {
    if (-not $x -or $x.Count -eq 0) { return $null }
    $s = $x | Sort-Object; $n = $s.Count
    if ($n % 2) { return $s[[int](($n - 1) / 2)] } else { return ($s[$n / 2 - 1] + $s[$n / 2]) / 2 }
}
$desen = [regex]'host: ([\d.]+) fps, ([\d.]+) ms/frame \(([\d.]+) preparing: copies ([\d.]+) ms ([\d.]+) MB.*?; ([\d.]+) on the GPU; ([\d.]+) copying back ([\d.]+) MB\), (\d+) draws/frame'
foreach ($k in $Klasorler) {
    $log = Join-Path $k 'run.log'
    if (-not (Test-Path $log)) { Write-Output "$k : run.log yok"; continue }
    $fs = [IO.FileStream]::new($log, 'Open', 'Read', 'ReadWrite, Delete')
    $sr = [IO.StreamReader]::new($fs, [Text.Encoding]::UTF8)
    $r = @{ fps = @(); ms = @(); kopya = @(); kopyaMB = @(); gpu = @(); geri = @(); draw = @() }
    try {
        while ($null -ne ($l = $sr.ReadLine())) {
            $m = $desen.Match($l)
            if ($m.Success -and [int]$m.Groups[9].Value -ge $MinDraw) {
                $r.fps += [double]$m.Groups[1].Value; $r.ms += [double]$m.Groups[2].Value
                $r.kopya += [double]$m.Groups[4].Value; $r.kopyaMB += [double]$m.Groups[5].Value
                $r.gpu += [double]$m.Groups[6].Value; $r.geri += [double]$m.Groups[7].Value; $r.draw += [double]$m.Groups[9].Value
            }
        }
    } finally { $sr.Dispose() }
    $ad = Split-Path $k -Leaf
    if ($r.ms.Count -eq 0) { Write-Output "$ad : dünya karesi yok (draw >= $MinDraw)"; continue }
    '{0} | n={1} | medyan {2} ms/kare ({3} fps) | kopya {4} ms ({5} MB) | GPU {6} ms | geri kopya {7} ms | draw {8}' -f `
        $ad, $r.ms.Count, (Get-Medyan $r.ms), (Get-Medyan $r.fps), (Get-Medyan $r.kopya), (Get-Medyan $r.kopyaMB), (Get-Medyan $r.gpu), (Get-Medyan $r.geri), (Get-Medyan $r.draw)
}
