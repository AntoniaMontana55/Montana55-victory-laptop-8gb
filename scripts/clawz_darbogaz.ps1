# clawz gta VI — Canli Darbogaz & Kasma Analiz Ekrani
$Host.UI.RawUI.WindowTitle = 'clawz gta VI - Kasma & Darbogaz Analizi'
try {
    $buf = $Host.UI.RawUI.BufferSize
    $win = $Host.UI.RawUI.WindowSize
    if ($win.Width -lt 80) {
        $buf.Width = 82; $Host.UI.RawUI.BufferSize = $buf
        $win.Width = 82; $win.Height = 28; $Host.UI.RawUI.WindowSize = $win
    }
} catch {}

Clear-Host
$totalRamMB = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1MB, 0)
$fpsHist = New-Object System.Collections.Generic.Queue[double]
$msHist  = New-Object System.Collections.Generic.Queue[double]
$stutterCount = 0

while ($true) {
    $counters = @(
        '\Processor(_Total)\% Processor Time',
        '\Memory\Available MBytes',
        '\Memory\Pages/sec',
        '\PhysicalDisk(_Total)\Disk Read Bytes/sec',
        '\PhysicalDisk(_Total)\Disk Write Bytes/sec',
        '\GPU Engine(*engtype_3D)\Utilization Percentage',
        '\GPU Adapter Memory(*)\Dedicated Usage',
        '\GPU Adapter Memory(*)\Shared Usage'
    )
    $cpuPct = 0.0; $availMB = 0.0; $pagesSec = 0.0; $diskR = 0.0; $diskW = 0.0
    $gpuPct = 0.0; $vramDedMB = 0.0; $vramShrMB = 0.0
    try {
        $samples = (Get-Counter -Counter $counters -ErrorAction SilentlyContinue).CounterSamples
        foreach ($s in $samples) {
            $p = $s.Path.ToLowerInvariant()
            if ($p -like '*processor(_total)*') { $cpuPct = $s.CookedValue }
            elseif ($p -like '*available mbytes*') { $availMB = $s.CookedValue }
            elseif ($p -like '*pages/sec*') { $pagesSec = $s.CookedValue }
            elseif ($p -like '*disk read bytes/sec*') { $diskR = $s.CookedValue / 1MB }
            elseif ($p -like '*disk write bytes/sec*') { $diskW = $s.CookedValue / 1MB }
            elseif ($p -like '*gpu engine*utilization*') { $gpuPct += $s.CookedValue }
            elseif ($p -like '*gpu adapter memory*dedicated usage*') { $vramDedMB += ($s.CookedValue / 1MB) }
            elseif ($p -like '*gpu adapter memory*shared usage*') { $vramShrMB += ($s.CookedValue / 1MB) }
        }
    } catch {}
    $gpuPct = [math]::Min(100.0, $gpuPct)

    $vp = Get-Process -Name 'victory' -ErrorAction SilentlyContinue | Select-Object -First 1
    $fps = $null; $ms = $null; $draws = $null; $vWsMB = 0; $vPmMB = 0
    if ($vp) {
        $vWsMB = [math]::Round($vp.WorkingSet64 / 1MB, 0)
        $vPmMB = [math]::Round($vp.PrivateMemorySize64 / 1MB, 0)
        $title = $vp.MainWindowTitle
        if ($title -match '(\d+)\s+fps\s+([\d\.]+)\s+ms/frame\s+(\d+)\s+draws/frame') {
            $fps = [double]$Matches[1]
            $ms = [double]$Matches[2]
            $draws = [int]$Matches[3]
            $fpsHist.Enqueue($fps); if ($fpsHist.Count -gt 15) { $null = $fpsHist.Dequeue() }
            $msHist.Enqueue($ms);   if ($msHist.Count -gt 15)  { $null = $msHist.Dequeue() }
            if ($ms -ge 100.0) { $stutterCount++ }
        }
    }

    # En cok RAM yiyen ilk 3 arka plan sureci (victory disinda)
    $topBg = Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.ProcessName -ne 'victory' -and $_.ProcessName -ne 'Idle' } |
        Sort-Object WorkingSet64 -Descending | Select-Object -First 3

    # === CANLI DARBOGAZ TESHISI ===
    $verdict = "DENGELI / AKICI"
    $verdictColor = "Green"
    $reason = "Belirgin bir donanim tikanmasi gorunmuyor."
    $advice = "Mevcut ayarlar dengeli; goruntu keskinligi icin RENDER_SCALE=0.67 denenebilir."

    if (-not $vp) {
        $verdict = "OYUN CALISMIYOR"
        $verdictColor = "DarkGray"
        $reason = "victory.exe su an acik degil."
        $advice = "Masaustundeki GTA6_Baslat.bat dosyasina cift tikla."
    } elseif ($availMB -lt 1000 -or $pagesSec -gt 500) {
        $verdict = "KRITIK: RAM / PAGEFILE (DISK PAGING) DARBOGAZI!"
        $verdictColor = "Red"
        $reason = ("Bos RAM cok dusuk ({0:N0} MB) veya agir paging var ({1:N0} sayfa/sn)." -f $availMB, $pagesSec)
        $advice = "Tarayici/Discord/OBS kapat! 16 GB RAM dolunca Windows diske yazip FPS dusurur."
    } elseif ($vramDedMB -gt 7350 -or $vramShrMB -gt 3500) {
        $verdict = "VRAM TASMASI (RX 7600 8 GB DOLU -> RAM'E TASIYOR)"
        $verdictColor = "Red"
        $reason = ("Dedicated VRAM {0:N0}/8192 MB, Shared {1:N0} MB (PCIe uzerinden yavas okuma)." -f $vramDedMB, $vramShrMB)
        $advice = "play_rx7600_8gb.ps1 icinde TEXTURE_BUDGET_MB=768 veya TEXTURE_DROP_MIPS=1 ac."
    } elseif ($gpuPct -ge 90) {
        $verdict = "GPU DARBOGAZI (RX 7600 %100 YUKTE)"
        $verdictColor = "Yellow"
        $reason = ("Ekran karti tam kapasitede ({0:N0}%), CPU beklemede ({1:N0}%)." -f $gpuPct, $cpuPct)
        $advice = "Daha yuksek FPS icin play_rx7600_8gb.ps1'de VICTORY_RENDER_SCALE=0.4 dene."
    } elseif ($diskR -gt 60 -or $diskW -gt 40) {
        $verdict = "DISK STREAMING / BOLUM YUKLEMESI"
        $verdictColor = "Cyan"
        $reason = ("Disk yogun: Okuma {0:N1} MB/s, Yazma {1:N1} MB/s (RPF / bolum akisi)." -f $diskR, $diskW)
        $advice = "Bolum/doku yuklenmesi bitince takilma kendiliginden gecmelidir."
    } elseif ($cpuPct -ge 80 -or ($ms -ne $null -and $ms -gt 70 -and $gpuPct -lt 70)) {
        $verdict = "CPU / KOMUT HAZIRLAMA DARBOGAZI (R5 5600X)"
        $verdictColor = "Yellow"
        $reason = ("GPU %{0:N0} bos beklerken kare suresi yuksek (CPU %{1:N0}, draw={2})." -f $gpuPct, $cpuPct, $draws)
        $advice = "Arka plan CPU kullanan uygulamalari kapat; PORT_CONSTANTS + PORT_DESCRIPTORS aktif."
    }

    $avgFps = if ($fpsHist.Count -gt 0) { ($fpsHist | Measure-Object -Average).Average } else { 0 }
    $maxMs  = if ($msHist.Count -gt 0)  { ($msHist  | Measure-Object -Maximum).Maximum } else { 0 }
    $now = Get-Date -Format 'HH:mm:ss'

    try { [Console]::SetCursorPosition(0, 0) } catch { Clear-Host }
    Write-Host "================================================================================" -ForegroundColor Magenta
    Write-Host "                  clawz gta VI  |  CANLI DARBOGAZ & KASMA ANALIZI               " -ForegroundColor Cyan
    Write-Host "================================================================================" -ForegroundColor Magenta
    Write-Host ("  Saat: {0}  |  Anlik: {1} FPS ({2} ms)  |  Draw: {3}  |  Ort15s: {4:N1} FPS" -f $now, $(if($fps -ne $null){$fps}else{'--'}), $(if($ms -ne $null){$ms}else{'--'}), $(if($draws -ne $null){$draws}else{'--'}), $avgFps).PadRight(80) -ForegroundColor White
    Write-Host ("  En Kotu Kare (15s): {0:N1} ms  |  Agir Takilma (>100ms): {1}" -f $maxMs, $stutterCount).PadRight(80) -ForegroundColor Gray
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host ("  TESHIS : {0}" -f $verdict).PadRight(80) -ForegroundColor $verdictColor
    Write-Host ("  SEBEP  : {0}" -f $reason).PadRight(80) -ForegroundColor White
    Write-Host ("  ONERI  : {0}" -f $advice).PadRight(80) -ForegroundColor Cyan
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host ("  GPU (RX 7600) : {0,5:N1}%  |  VRAM: {1,4:N0}/8192 MB  |  Shared: {2,4:N0} MB" -f $gpuPct, $vramDedMB, $vramShrMB).PadRight(80) -ForegroundColor Magenta
    Write-Host ("  CPU (5600X)   : {0,5:N1}%  |  Oyun WS: {1,5:N0} MB   |  Commit: {2,5:N0} MB" -f $cpuPct, $vWsMB, $vPmMB).PadRight(80) -ForegroundColor Green
    Write-Host ("  RAM / PAGING  : Bos {0,5:N0}/{1} MB  |  Pages/sn: {2,5:N0}  |  Disk R/W: {3:N1}/{4:N1} MB/s" -f $availMB, $totalRamMB, $pagesSec, $diskR, $diskW).PadRight(80) -ForegroundColor Yellow
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor DarkGray
    Write-Host "  Arka Planda En Cok RAM Yiyenler (kasma varsa kapat):" -ForegroundColor Gray
    foreach ($bg in $topBg) {
        Write-Host ("    - {0,-28} : {1,6:N0} MB  (PID {2})" -f $bg.ProcessName, ($bg.WorkingSet64/1MB), $bg.Id).PadRight(80) -ForegroundColor DarkYellow
    }
    Write-Host "================================================================================" -ForegroundColor Magenta
}
