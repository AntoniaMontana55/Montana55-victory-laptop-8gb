# clawz gta VI — Canli Sistem & Oyun Kullanim Izleyici (RX 7600 8GB / R5 5600X / 16 GB RAM)
$Host.UI.RawUI.WindowTitle = 'clawz gta VI - Canli Kullanimlar'
try {
    $buf = $Host.UI.RawUI.BufferSize
    $win = $Host.UI.RawUI.WindowSize
    if ($win.Width -lt 74) {
        $buf.Width = 76; $Host.UI.RawUI.BufferSize = $buf
        $win.Width = 76; $win.Height = 24; $Host.UI.RawUI.WindowSize = $win
    }
} catch {}

Clear-Host
$cpuName = (Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty Name).Trim()
$gpuName = (Get-CimInstance Win32_VideoController | Select-Object -First 1 -ExpandProperty Name).Trim()
$totalRamGB = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 1)

function Get-Bar([double]$pct, [int]$width = 24) {
    $pct = [math]::Max(0, [math]::Min(100, $pct))
    $filled = [int][math]::Round(($pct / 100.0) * $width)
    return ('#' * $filled) + ('-' * ($width - $filled))
}

while ($true) {
    $counters = @(
        '\Processor(_Total)\% Processor Time',
        '\Memory\Available MBytes',
        '\PhysicalDisk(_Total)\Disk Read Bytes/sec',
        '\PhysicalDisk(_Total)\Disk Write Bytes/sec',
        '\GPU Engine(*engtype_3D)\Utilization Percentage',
        '\GPU Adapter Memory(*)\Dedicated Usage',
        '\GPU Adapter Memory(*)\Shared Usage'
    )
    $cpuPct = 0.0; $availMB = 0.0; $diskR = 0.0; $diskW = 0.0
    $gpuPct = 0.0; $vramDedMB = 0.0; $vramShrMB = 0.0
    try {
        $samples = (Get-Counter -Counter $counters -ErrorAction SilentlyContinue).CounterSamples
        foreach ($s in $samples) {
            $p = $s.Path.ToLowerInvariant()
            if ($p -like '*processor(_total)*') { $cpuPct = $s.CookedValue }
            elseif ($p -like '*available mbytes*') { $availMB = $s.CookedValue }
            elseif ($p -like '*disk read bytes/sec*') { $diskR = $s.CookedValue / 1MB }
            elseif ($p -like '*disk write bytes/sec*') { $diskW = $s.CookedValue / 1MB }
            elseif ($p -like '*gpu engine*utilization*') { $gpuPct += $s.CookedValue }
            elseif ($p -like '*gpu adapter memory*dedicated usage*') { $vramDedMB += ($s.CookedValue / 1MB) }
            elseif ($p -like '*gpu adapter memory*shared usage*') { $vramShrMB += ($s.CookedValue / 1MB) }
        }
    } catch {}
    $gpuPct = [math]::Min(100.0, $gpuPct)
    $usedRamGB = [math]::Max(0, $totalRamGB - ($availMB / 1024.0))
    $ramPct = if ($totalRamGB -gt 0) { ($usedRamGB / $totalRamGB) * 100.0 } else { 0 }
    $vramPct = ($vramDedMB / 8192.0) * 100.0

    $vp = Get-Process -Name 'victory' -ErrorAction SilentlyContinue | Select-Object -First 1
    $gameStatus = if ($vp) {
        $t = if ($vp.MainWindowTitle) { $vp.MainWindowTitle } else { "Calisiyor (PID $($vp.Id))" }
        "$t"
    } else {
        "Bekleniyor / Hazirlaniyor..."
    }
    $gameRam = if ($vp) {
        "WS: $([math]::Round($vp.WorkingSet64/1MB,0)) MB  |  Commit: $([math]::Round($vp.PrivateMemorySize64/1MB,0)) MB"
    } else {
        "WS: 0 MB  |  Commit: 0 MB"
    }

    try { [Console]::SetCursorPosition(0, 0) } catch { Clear-Host }
    $now = Get-Date -Format 'HH:mm:ss'
    Write-Host "==========================================================================" -ForegroundColor Magenta
    Write-Host "                             clawz gta VI                                 " -ForegroundColor Cyan
    Write-Host "==========================================================================" -ForegroundColor Magenta
    Write-Host ("  Saat  : {0,-20}  GPU : {1,-36}" -f $now, $gpuName).PadRight(74) -ForegroundColor Gray
    Write-Host ("  CPU   : {0,-62}" -f $cpuName).PadRight(74) -ForegroundColor Gray
    Write-Host "--------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host ("  Oyun  : {0}" -f $gameStatus).PadRight(74) -ForegroundColor Yellow
    Write-Host ("  Bellek: {0}" -f $gameRam).PadRight(74) -ForegroundColor White
    Write-Host "--------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host ("  CPU   : [{0}] {1,5:N1} %" -f (Get-Bar $cpuPct), $cpuPct).PadRight(74) -ForegroundColor Green
    Write-Host ("  GPU   : [{0}] {1,5:N1} %" -f (Get-Bar $gpuPct), $gpuPct).PadRight(74) -ForegroundColor Cyan
    Write-Host ("  VRAM  : [{0}] {1,5:N0} / 8192 MB  (Paylasilan: {2:N0} MB)" -f (Get-Bar $vramPct), $vramDedMB, $vramShrMB).PadRight(74) -ForegroundColor Magenta
    Write-Host ("  RAM   : [{0}] {1,4:N1} / {2:N1} GB  (Bos: {3:N1} GB)" -f (Get-Bar $ramPct), $usedRamGB, $totalRamGB, ($availMB/1024.0)).PadRight(74) -ForegroundColor Yellow
    Write-Host ("  DISK  : Okuma {0,5:N1} MB/s  |  Yazma {1,5:N1} MB/s" -f $diskR, $diskW).PadRight(74) -ForegroundColor White
    Write-Host "==========================================================================" -ForegroundColor Magenta
    Write-Host "  Kapatmak icin Ctrl+C veya pencereyi kapat.                              " -ForegroundColor DarkGray
}
