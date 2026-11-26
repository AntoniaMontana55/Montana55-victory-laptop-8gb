# clawz GTA VI — Kararli & Temiz Shader Baslaticisi (RX 7600 8 GB)
param(
    [ValidateSet('', 'ultralow', 'low', 'medium', 'high', 'ultra')]
    [string]$Preset = '',
    [switch]$Turbo,
    [switch]$EnableRT,
    [switch]$FsrFixed720p
)
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent $PSScriptRoot
$victory = Join-Path $repo 'target\release\victory.exe'
$gameExe = 'D:\GTA 6 Leek\game_scarlett_master.exe'

if (-not (Test-Path $victory)) {
    Write-Host "HATA: $victory bulunamadi." -ForegroundColor Red
    Read-Host "Cikmak icin Enter'a bas"
    exit 1
}
if (-not (Test-Path $gameExe)) {
    Write-Host "HATA: $gameExe bulunamadi." -ForegroundColor Red
    Read-Host "Cikmak icin Enter'a bas"
    exit 1
}

$turboMode = [bool]$Turbo
$rtEnabled = [bool]$EnableRT
$useFsr    = [bool]$FsrFixed720p
$Host.UI.RawUI.WindowTitle = 'clawz - GTA VI (Temiz Shader + Kararli Mod Menusu)'

if (-not $Preset) {
    while (-not $Preset) {
        Clear-Host
        $modeTxt = if ($turboMode) { "ULTRA-SAFE SENKRON (RELAY=2 korunur, yavas test modu)" } else { "HIZLI & TEMIZ SHADER (ONERILEN - RELAY=2 + Port #6/#7, cokmesiz)" }
        $rtTxt   = if ($rtEnabled) { "ACIK  (UYARI: 8GB VRAM'de +3GB yer & cokme riski!)" } else { "KAPALI + SABIT AYDINLATMA 0.18 (Yuzler & gece sahneleri NET AYDINLIK!)" }
        $fsrTxt  = if ($useFsr)    { "FSR4 (Yalniz sabit 1280x720)" } else { "OYUNUN TEMPORAL UPSCALER'I (Gecerli olcekler 0.5-1.0)" }
        $modeCol = if ($turboMode) { "Yellow" } else { "Green" }
        $rtCol   = if ($rtEnabled) { "Red" } else { "Green" }
        $fsrCol  = if ($useFsr)    { "Yellow" } else { "Cyan" }

        Write-Host "============================================================================" -ForegroundColor Magenta
        Write-Host "                           C L A W Z   G T A   V I                          " -ForegroundColor Cyan
        Write-Host "         RX 7600 8GB | Aydinlik Yuzler + Hizli & Temiz DX12 Motor           " -ForegroundColor Cyan
        Write-Host "============================================================================" -ForegroundColor Magenta
        Write-Host ""
        Write-Host ("  [T] Motor Modu       : {0}" -f $modeTxt) -ForegroundColor $modeCol
        Write-Host ("  [R] Isin Izleme (RT) : {0}" -f $rtTxt) -ForegroundColor $rtCol
        Write-Host ("  [F] Upscaler         : {0}" -f $fsrTxt) -ForegroundColor $fsrCol
        Write-Host ""
        Write-Host "  Grafik / Performans Kalitesi Sec (Hepsi gecerli 0.5-1.0 olcekte):" -ForegroundColor White
        Write-Host "   [1] ULTRA LOW  (Maks FPS)     - 720p Cikis  x0.50 (640x360)  | 1280 MB" -ForegroundColor Green
        Write-Host "   [2] LOW        (ONERILEN 8GB) - 720p Cikis  x0.67 (854x480)  | 1536 MB" -ForegroundColor Cyan
        Write-Host "   [3] MEDIUM     (Dengeli)      - 1080p Cikis x0.50 (960x540)  | 1536 MB" -ForegroundColor Yellow
        Write-Host "   [4] HIGH       (Kalite)       - 1080p Cikis x0.75 (1440x810) | 1536 MB" -ForegroundColor DarkYellow
        Write-Host "   [5] VERY HIGH / ULTRA         - Series X    x1.00 (1080p)    | 1536 MB" -ForegroundColor Red
        Write-Host ""
        Write-Host "   [M] Canli Kullanim + Darbogaz Analiz Ekranlarini Ac" -ForegroundColor DarkGray
        Write-Host "============================================================================" -ForegroundColor Magenta
        Write-Host ""

        $choice = (Read-Host "Secimin (1-5, Enter=2, T=Guvenli/Turbo, R=RT, F=Upscaler, M=Monitor)").Trim().ToLowerInvariant()
        switch ($choice) {
            ''  { $Preset = 'low' }
            '1' { $Preset = 'ultralow' }
            '2' { $Preset = 'low' }
            '3' { $Preset = 'medium' }
            '4' { $Preset = 'high' }
            '5' { $Preset = 'ultra' }
            't' { $turboMode = -not $turboMode }
            'r' { $rtEnabled = -not $rtEnabled }
            'f' { $useFsr = -not $useFsr }
            'm' {
                Start-Process "explorer.exe" -ArgumentList '"C:\Users\clawz\Desktop\clawz_Monitor.bat"'
                Start-Process "explorer.exe" -ArgumentList '"C:\Users\clawz\Desktop\clawz_Darbogaz_Analiz.bat"'
            }
            default {
                Write-Host "  Gecersiz secim! 1-5, T, R, F veya M gir." -ForegroundColor Red
                Start-Sleep -Milliseconds 700
            }
        }
    }
}

# Arka planda acik kalmis eski victory surecini kapat
Stop-Process -Name 'victory' -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

# Eski VICTORY_* degiskenlerini temizle
Get-ChildItem Env:VICTORY_* -ErrorAction SilentlyContinue | ForEach-Object { Remove-Item "Env:$($_.Name)" }

# === TEMIZ & KARARLI ORTAK TEMEL (Upstream -Safe uyumlu, sifir shader bozulmasi) ===
# PORT_CBV=0: Standart DXIL shader kullanir (shader bozulmalarini engeller).
# CLAMP_WAITS=1 (varsayilan): ExecuteIndirect oncesi tam UAV bariyeri (0x887A0007 onler).
$env:VICTORY_NO_WATCHDOG              = '1'
$env:VICTORY_PLANES                   = '1'
$env:VICTORY_PORT_CONSTANTS           = '0'
$env:VICTORY_PORT_CBV                 = '0'
$env:VICTORY_UP_QUEUE                 = '0'
$env:VICTORY_LEVELS_ON_DEMAND         = '1'
$env:VICTORY_RIPE_BLOCKS              = '6'
$env:VICTORY_CARD_ROOM_FOR_TARGETS_MB = '1280'
$env:VICTORY_FRAME_INTERVAL_MS        = '16.667'
$env:VICTORY_PIPELINE_WAIT_MS         = '120'
$env:VICTORY_SCREEN_WAIT_MS           = '10000'
$env:VICTORY_SHOW_AFTER               = '1'
$env:VICTORY_WIDE_BACK                = '1'
$env:VICTORY_JOIN_COPIES              = '1'
$env:VICTORY_INDIRECT_BATCH           = '2'
$env:VICTORY_INDIRECT_WINDOW          = '1'
$env:VICTORY_CARD_REHASH_RUNS         = '1'
$env:VICTORY_CARD_STEPS               = '2'
$env:VICTORY_CARD_AWAKE               = '0'
$env:VICTORY_MAP_THREADS              = '1'
$env:VICTORY_TITLE_PRIORITIES         = '0'
$env:VICTORY_TEXTURE_NO_WAIT          = '1'
$env:VICTORY_SEND_KEPT_MB             = '128'
$env:VICTORY_WINDOW_KB                = '256'
$env:VICTORY_WINDOW_TAILS             = '1'
$env:VICTORY_MEMORY_MAP               = '0'
$env:VICTORY_PIPELINE_BOOK            = '0'
$env:VICTORY_VIEW_BYTES               = '1'
$env:RUST_LOG                         = 'error'

if ($turboMode) {
    # ULTRA-SAFE SENKRON: RELAY=2 korunur, diger asenkron yollar kapanir
    $env:VICTORY_RELAY             = '2'
    $env:VICTORY_THIN              = '0'
    $env:VICTORY_ASYNC_SUBMIT      = '0'
    $env:VICTORY_UPLOAD_AHEAD      = '0'
    $env:VICTORY_PORT_DESCRIPTORS  = '0'
    $env:VICTORY_PORT_GUARD        = '0'
    $env:VICTORY_PORT_JOURNAL      = '0'
    $env:VICTORY_JOIN_BARRIERS     = '0'
    $env:VICTORY_UNDER_WIDE        = '0'
    $env:VICTORY_LAND_ECHO         = '0'
} else {
    # HIZLI & TEMIZ MOD: Asenkron CPU/GPU (RELAY=2 + THIN + Port #6/#7) ama shader'a dokunmaz (PORT_CBV=0, PORT_DESCRIPTORS=0, CLAMP_WAITS=ON)
    $env:VICTORY_RELAY             = '2'
    $env:VICTORY_RELAY_COPY        = '1'
    $env:VICTORY_THIN              = '1'
    $env:VICTORY_THIN_THREADS      = '6'
    $env:VICTORY_ASYNC_SUBMIT      = '1'
    $env:VICTORY_UPLOAD_AHEAD      = '1'
    $env:VICTORY_PORT_DESCRIPTORS  = '0'
    $env:VICTORY_PORT_GUARD        = '1'
    $env:VICTORY_PORT_JOURNAL      = '1'
    $env:VICTORY_JOIN_BARRIERS     = '1'
    $env:VICTORY_UNDER_WIDE        = '0'
    $env:VICTORY_LAND_ECHO         = '1'
    $env:VICTORY_WIDE_SPIN         = '0'
    $env:VICTORY_QUICK_FIND        = '1'
}

# === ISIN IZLEME (RAY TRACING) & AYDINLATMA ===
# constant:0.18 + max(was, found) -> gece, araba ici ve yuzler icin korunan taban isik
if ($rtEnabled) {
    $env:VICTORY_BVH_BUILDER       = '1'
    $env:VICTORY_BVH_MISS          = '0'
    $env:VICTORY_RT_SKIP_RESOLVE   = '0'
    $env:VICTORY_RT_SKIP_ACTIVATE  = '0'
    $env:VICTORY_RT_SKIP_EXTRA     = '0'
    $env:VICTORY_AMBIENT           = 'constant:0.18'
} else {
    $env:VICTORY_BVH_BUILDER       = '0'
    $env:VICTORY_BVH_MISS          = '1'
    $env:VICTORY_RT_SKIP_RESOLVE   = '1'
    $env:VICTORY_RT_SKIP_ACTIVATE  = '1'
    $env:VICTORY_RT_SKIP_EXTRA     = '0'
    $env:VICTORY_AMBIENT           = 'constant:0.18'
    $env:VICTORY_AMBIENT_SHADE     = '0.65,0.5'
}

# === PRESET CIKIS & GECERLI OLCEK (0.50 - 1.00) ===
$consoleProfile = 'series-s'
$presetLabel = ''
$outRes = '1920x1080'
$scale = '0.6667'
switch ($Preset) {
    'ultralow' {
        $presetLabel = 'ULTRA LOW (720p x0.50 = 640x360)'
        $outRes = '1280x720'
        $scale = '0.5'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1280'
    }
    'low' {
        $presetLabel = 'LOW (720p x0.67 = 854x480)'
        $outRes = '1280x720'
        $scale = '0.6667'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1536'
    }
    'medium' {
        $presetLabel = 'MEDIUM (1080p x0.50 = 960x540)'
        $outRes = '1920x1080'
        $scale = '0.5'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1536'
    }
    'high' {
        $presetLabel = 'HIGH (1080p x0.75 = 1440x810)'
        $outRes = '1920x1080'
        $scale = '0.75'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1536'
    }
    'ultra' {
        $presetLabel = 'VERY HIGH / ULTRA (Series X 1080p x1.0)'
        $consoleProfile = 'series-x'
        $outRes = '1920x1080'
        $scale = '1.0'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1536'
    }
}

# === KORUNAN AYARLAR: hicbir profil bunlari ezmez ===
$env:VICTORY_AMBIENT          = 'constant:0.18'
$env:VICTORY_RELAY            = '2'
$env:VICTORY_PORT_CBV         = '0'
$env:VICTORY_PORT_DESCRIPTORS = '0'
$env:VICTORY_CLAMP_WAITS      = 'ON'

# === UPSCALER & CIKIS GUVENLIGI ===
if ($useFsr -and $consoleProfile -eq 'series-s') {
    $env:VICTORY_UPSCALER = '7'
    $presetLabel = "$Preset [FSR4 Sabit 1280x720 -> 1080p]"
} else {
    $env:VICTORY_UPSCALER = '6'
    $env:VICTORY_OUTPUT   = $outRes
    $env:VICTORY_RENDER_SCALE = $scale
}

$modeBadge = if ($turboMode) { 'TURBO' } else { 'SAFE' }
$rtBadge   = if ($rtEnabled) { 'RT=ON' } else { 'RT=OFF' }
$fsrBadge  = if ($useFsr -and $consoleProfile -eq 'series-s') { 'FSR4-720p' } else { "OWN($outRes x$scale)" }
$Host.UI.RawUI.WindowTitle = "clawz - GTA VI [$Preset | $modeBadge | $rtBadge | $fsrBadge]"
Write-Host ""
Write-Host "============================================================================" -ForegroundColor Magenta
Write-Host "  Profil   : $presetLabel  |  $modeBadge  |  $rtBadge" -ForegroundColor Cyan
Write-Host "  Cikis    : $fsrBadge | Konsol=$consoleProfile | Doku=$($env:VICTORY_TEXTURE_BUDGET_MB) MB" -ForegroundColor White
Write-Host "  Shader   : Standart DXIL (PORT_CBV=0, CLAMP_WAITS=ON, RT_SKIP=Guvenli)" -ForegroundColor Green
Write-Host "  Not      : Intro/ara sahneyi gecmek icin oyun penceresinde Enter'a basili tut." -ForegroundColor Yellow
Write-Host "============================================================================" -ForegroundColor Magenta
Write-Host ""

Push-Location $repo
$ErrorActionPreference = 'Continue'
try {
    & $victory run $gameExe --console $consoleProfile --execute
} finally {
    Pop-Location
}
