# clawz GTA VI — Grafik, Ray Tracing & Upscaler Secicili Baslatici (RX 7600 8 GB)
param(
    [ValidateSet('', 'ultralow', 'low', 'medium', 'high', 'ultra')]
    [string]$Preset = '',
    [switch]$NoRT,
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

$rtEnabled = -not $NoRT
# Shipping FSR4 yalniz sabit 1280x720 kabul eder (baska cozunurlukte 0xc0000094 verir).
# Ozel olceklerde (0.35 / 0.4 / 0.5 / 0.75) oyunun kendi temporal upscaler'i ZORUNLUDUR.
$useFsr = [bool]$FsrFixed720p
$Host.UI.RawUI.WindowTitle = 'clawz - GTA VI (Grafik + RT + Upscaler Menusu)'

if (-not $Preset) {
    while (-not $Preset) {
        Clear-Host
        $rtTxt  = if ($rtEnabled) { "ACIK  (Isikli ic mekanlar, -%25-35 FPS)" } else { "KAPALI (+%25-35 daha yuksek FPS)" }
        $fsrTxt = if ($useFsr)    { "FSR4 (Yalniz sabit 1280x720 - olcek degistirilemez)" } else { "OYUNUN TEMPORAL UPSCALER'I (Tum olceklerde cokmesiz & hizli)" }
        $rtCol  = if ($rtEnabled) { "Green" } else { "Yellow" }
        $fsrCol = if ($useFsr)    { "Yellow" } else { "Cyan" }

        Write-Host "============================================================================" -ForegroundColor Magenta
        Write-Host "                           C L A W Z   G T A   V I                          " -ForegroundColor Cyan
        Write-Host "            RX 7600 8GB Super-Build | 7900XTX Port #1+#2 + cool_fox         " -ForegroundColor Cyan
        Write-Host "============================================================================" -ForegroundColor Magenta
        Write-Host ""
        Write-Host ("  [R] Isin Izleme (RT) : {0}" -f $rtTxt) -ForegroundColor $rtCol
        Write-Host ("  [F] Upscaler         : {0}" -f $fsrTxt) -ForegroundColor $fsrCol
        Write-Host ""
        Write-Host "  Grafik / Performans Kalitesi Sec:" -ForegroundColor White
        Write-Host "   [1] ULTRA LOW  (Maks FPS)     - 640x360  (0.33x) -> 1080p | 768 MB Doku" -ForegroundColor Green
        Write-Host "   [2] LOW        (ONERILEN 8GB) - 768x432  (0.40x) -> 1080p | 1024 MB Doku" -ForegroundColor Cyan
        Write-Host "   [3] MEDIUM     (Dengeli)      - 960x540  (0.50x) -> 1080p | 1280 MB Doku" -ForegroundColor Yellow
        Write-Host "   [4] HIGH       (Kalite)       - 1440x810 (0.75x) -> 1080p | 1792 MB Doku" -ForegroundColor DarkYellow
        Write-Host "   [5] VERY HIGH / ULTRA         - Series X (1.00x)          | 2560 MB Doku" -ForegroundColor Red
        Write-Host ""
        Write-Host "   [M] Canli Kullanim + Darbogaz Analiz Ekranlarini Ac" -ForegroundColor DarkGray
        Write-Host "============================================================================" -ForegroundColor Magenta
        Write-Host ""

        $choice = (Read-Host "Secimin (1-5, Enter=2, R=RT Ac/Kapat, F=Upscaler Degistir, M=Monitorler)").Trim().ToLowerInvariant()
        switch ($choice) {
            ''  { $Preset = 'low' }
            '1' { $Preset = 'ultralow' }
            '2' { $Preset = 'low' }
            '3' { $Preset = 'medium' }
            '4' { $Preset = 'high' }
            '5' { $Preset = 'ultra' }
            'r' { $rtEnabled = -not $rtEnabled }
            'f' { $useFsr = -not $useFsr }
            'm' {
                Start-Process "explorer.exe" -ArgumentList '"C:\Users\clawz\Desktop\clawz_Monitor.bat"'
                Start-Process "explorer.exe" -ArgumentList '"C:\Users\clawz\Desktop\clawz_Darbogaz_Analiz.bat"'
            }
            default {
                Write-Host "  Gecersiz secim! 1-5, R, F veya M gir." -ForegroundColor Red
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

# === ORTAK TEMEL: 7900XTX AMD Hizlandirmalari + Watchdog Korumasi + Sifir Disk Log ===
$env:VICTORY_NO_WATCHDOG         = '1'
$env:VICTORY_PORT_CONSTANTS      = '1'
$env:VICTORY_PORT_DESCRIPTORS    = '1'
$env:VICTORY_UP_QUEUE            = '1'
$env:VICTORY_JOIN_BARRIERS       = '1'
$env:VICTORY_LEVELS_ON_DEMAND    = '1'
$env:VICTORY_MAX_FPS             = '60'
$env:VICTORY_TEARING             = '0'
$env:VICTORY_THIN                = '1'
$env:VICTORY_THIN_THREADS        = '6'
$env:VICTORY_UPLOAD_AHEAD        = '1'
$env:VICTORY_CLAMP_WAITS         = '0'
$env:VICTORY_CARD_BUFFERS_MIN_MB = '1'
$env:VICTORY_CARD_BUFFERS_MB     = '512'
$env:VICTORY_PIPELINE_WAIT_MS    = '500'
$env:VICTORY_SCREEN_WAIT_MS      = '10000'
$env:VICTORY_SHOW_AFTER          = '1'
$env:VICTORY_SKIP_PIPELINES      = 'none'
$env:VICTORY_RELAY               = '1'
$env:VICTORY_RELAY_COPY          = '1'
$env:VICTORY_WIDE_BACK           = '1'
$env:VICTORY_JOIN_COPIES         = '1'
$env:VICTORY_INDIRECT_BATCH      = '2'
$env:VICTORY_INDIRECT_WINDOW     = '1'
$env:VICTORY_CARD_REHASH_RUNS    = '1'
$env:VICTORY_CARD_STEPS          = '2'
$env:VICTORY_CARD_AWAKE          = '1'
$env:VICTORY_UNDER_WIDE          = '0'
$env:VICTORY_LAND_ECHO           = '0'
$env:VICTORY_MAP_THREADS         = '1'
$env:VICTORY_TITLE_PRIORITIES    = '0'
$env:VICTORY_TEXTURE_NO_WAIT     = '1'
$env:VICTORY_SEND_KEPT_MB        = '128'
$env:VICTORY_WINDOW_KB           = '256'
$env:VICTORY_WINDOW_TAILS        = '1'
$env:VICTORY_D3D12_DRED          = 'light'
$env:VICTORY_PIPELINE_BOOK       = '0'
$env:RUST_LOG                    = 'warn'

# === ISIN IZLEME (RAY TRACING) ===
if ($rtEnabled) {
    $env:VICTORY_BVH_BUILDER       = '1'
    $env:VICTORY_BVH_MISS          = '0'
    $env:VICTORY_RT_SKIP_RESOLVE   = '0'
    $env:VICTORY_RT_SKIP_ACTIVATE  = '0'
    $env:VICTORY_AMBIENT           = '0.6'
} else {
    $env:VICTORY_BVH_BUILDER       = '0'
    $env:VICTORY_BVH_MISS          = '1'
    $env:VICTORY_RT_SKIP_RESOLVE   = '1'
    $env:VICTORY_RT_SKIP_ACTIVATE  = '1'
    $env:VICTORY_AMBIENT           = '0.85'
}

# === PRESET OLCEK & DOKU BUTCELERI ===
$consoleProfile = 'series-s'
$presetLabel = ''
$scale = '0.4'
switch ($Preset) {
    'ultralow' {
        $presetLabel = 'ULTRA LOW (640x360 -> 1080p)'
        $scale = '0.3333'
        $env:VICTORY_TEXTURE_BUDGET_MB = '768'
    }
    'low' {
        $presetLabel = 'LOW (768x432 -> 1080p)'
        $scale = '0.4'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1024'
    }
    'medium' {
        $presetLabel = 'MEDIUM (960x540 -> 1080p)'
        $scale = '0.5'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1280'
    }
    'high' {
        $presetLabel = 'HIGH (1440x810 -> 1080p)'
        $scale = '0.75'
        $env:VICTORY_TEXTURE_BUDGET_MB = '1792'
    }
    'ultra' {
        $presetLabel = 'VERY HIGH / ULTRA (Series X 1.0x)'
        $consoleProfile = 'series-x'
        $scale = '1.0'
        $env:VICTORY_TEXTURE_BUDGET_MB = '2560'
    }
}

# === UPSCALER GUVENLIGI ===
# Shipping FSR4 yalniz sabit 1280x720 kabul ettigi icin FSR aciksa RENDER_SCALE degistirilmez!
if ($useFsr -and $consoleProfile -eq 'series-s') {
    $env:VICTORY_OWN_UPSCALER = '0'
    $presetLabel = "$Preset [FSR4 Sabit 1280x720 -> 1080p]"
} else {
    $env:VICTORY_OWN_UPSCALER = '1'
    $env:VICTORY_RENDER_SCALE = $scale
}

$rtBadge  = if ($rtEnabled) { 'RT=ON' } else { 'RT=OFF' }
$fsrBadge = if ($useFsr -and $consoleProfile -eq 'series-s') { 'FSR4-720p' } else { "OWN-UPSCALER($scale)" }
$Host.UI.RawUI.WindowTitle = "clawz - GTA VI [$Preset | $rtBadge | $fsrBadge]"
Write-Host ""
Write-Host "============================================================================" -ForegroundColor Magenta
Write-Host "  Profil   : $presetLabel  |  $rtBadge  |  $fsrBadge" -ForegroundColor Cyan
Write-Host "  Konsol   : $consoleProfile | Doku=$($env:VICTORY_TEXTURE_BUDGET_MB) MB" -ForegroundColor White
Write-Host "  Motor    : PORT_CONSTANTS + PORT_DESCRIPTORS + UP_QUEUE + JOIN_BARRIERS" -ForegroundColor Green
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
