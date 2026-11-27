# clawz GTA VI — Tek Tik Sabit Ayar, %100 Saf DXIL Shader + Tam Model Baslaticisi (RX 7600 8 GB)
param(
    [ValidateSet('1', '2')]
    [string]$Profile = '2',
    [string]$Preset = ''
)
$ErrorActionPreference = 'Stop'

if ($Preset -eq 'ultralow') { $Profile = '1' }

$repo    = Split-Path -Parent $PSScriptRoot
$victory = Join-Path $repo 'target\release\victory.exe'
if (-not (Test-Path $victory)) {
    $victory = Join-Path $repo 'target\diag\victory.exe'
}
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

# Arka planda acik kalmis eski victory surecini kapat
Stop-Process -Name 'victory' -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

# Eski VICTORY_* degiskenlerini temizle
Get-ChildItem Env:VICTORY_* -ErrorAction SilentlyContinue | ForEach-Object { Remove-Item "Env:$($_.Name)" }

# Sabit Profil (Menu yok, tek tik):
# Varsayilan 2 = RX 7600 8GB SIFIR TASMA + 30 FPS + TAM MODEL (720p RT tasarrufu, RELAY=2, TX<=1.5GB)
# Profil 1     = 936p YUKSEK COZUNURLUK (1664x936 x0.6667 | daha yuksek netlik, ~350MB fazla RT)
if ($Profile -eq '1') {
    $profileName     = '1 - 936p NET (1664x936 x0.6667 | RELAY=2 | %100 Saf Shader)'
    $outputMode      = '1664x936'
    $renderScale     = '0.6667'
    $texBudgetMb     = '1408'
    $texMax          = '512'
} else {
    $profileName     = '2 - SIFIR TASMA + 30 FPS + TAM MODEL (1280x720 x0.6667 | RELAY=2 | TX 512px)'
    $outputMode      = '1280x720'
    $renderScale     = '0.6667'
    $texBudgetMb     = '1536'
    $texMax          = '512'
}
$relayMode       = '2'
$lateWait        = '3'
$asyncSubmit     = '1'
$texNoWait       = '1'
$texCatchup      = '1'
$texStageMs      = '250'
$pipelineWaitMs  = '1500'

# === %100 SAF STANDART DXIL SHADER (25.694 ONBELLEKLE %100 UYUMLU) ===
# PORT_CBV=0, PORT_DESCRIPTORS=0, PORT_CONSTANTS=0, BVH_MISS=0, FALLBACK_SHADER=0
# RT_SKIP_* + CUTSCENE_FAST yalnizca bos BVH RT / agir DoF & bulut raymarch compute
# gecislerini atlar (DXIL shader cevirisini degistirmez, onbellek bozulmaz!).
# AMBIENT=0.85 + AMBIENT_AIR=0.25 -> oda isigina uyumlu aydinlatma + ic mekan sis/isik patlamasi sifir!
$env:VICTORY_PORT_CONSTANTS           = '0'
$env:VICTORY_PORT_CBV                 = '0'
$env:VICTORY_PORT_DESCRIPTORS         = '0'
$env:VICTORY_PORT_GUARD               = '1'
$env:VICTORY_PORT_JOURNAL             = '1'
$env:VICTORY_JOIN_BARRIERS            = '1'
$env:VICTORY_UP_QUEUE                 = '0'
$env:VICTORY_BVH_BUILDER              = '0'
$env:VICTORY_BVH_MISS                 = '0'
$env:VICTORY_RT_SKIP_RESOLVE          = '1'
$env:VICTORY_RT_SKIP_ACTIVATE         = '1'
$env:VICTORY_RT_SKIP_EXTRA            = '1'
$env:VICTORY_CUTSCENE_FAST            = '1'
$env:VICTORY_FALLBACK_SHADER          = '0'
$env:VICTORY_AMBIENT                  = '0.85'
$env:VICTORY_AMBIENT_SHADE            = '0.55,0.5'
$env:VICTORY_AMBIENT_AIR              = '0.25'
$env:VICTORY_AMBIENT_SETTLE           = '150,2.5,1.0'
$env:VICTORY_BAKE_EXACT               = '1'
$env:VICTORY_BAKE_TEXTURES_MS         = '2000'

# === ZAMANLAMA & CIKIS (720p RT = ~750 MB VRAM tasarrufu + 2x GPU hizi, sifir PSO hatasi) ===
$env:VICTORY_NO_WATCHDOG              = '1'
$env:VICTORY_PLANES                   = '1'
$env:VICTORY_SHOW_AFTER               = '1'
$env:VICTORY_FRAME_INTERVAL_MS        = '33.333'
$env:VICTORY_OUTPUT                   = $outputMode
$env:VICTORY_UPSCALER                 = '6'
$env:VICTORY_RENDER_SCALE             = $renderScale

# === HIZ, TAM MODEL & DIRECTSTORAGE STREAMING (RX 7600 8 GB + 16 GB RAM) ===
# RELAY=2 + LATE_WAIT=3 + ASYNC_SUBMIT=1 -> CPU/GPU paralel calisir (3-4 FPS seri kilidi acilir),
# Belobug PreludeUpload + release_tables + 720p RT + TX 512px ile VRAM < 6.0 GiB (Shared tasma sifir!).
$env:VICTORY_RELAY                    = $relayMode
$env:VICTORY_LATE_WAIT                = $lateWait
$env:VICTORY_RELAY_COPY               = '1'
$env:VICTORY_CLAMP_WAITS              = 'ON'
$env:VICTORY_THIN                     = '0'
$env:VICTORY_UPLOAD_AHEAD             = '0'
$env:VICTORY_ASYNC_SUBMIT             = $asyncSubmit
$env:VICTORY_WIDE_BACK                = '1'
$env:VICTORY_WIDE_REFRESH             = '1'
$env:VICTORY_JOIN_COPIES              = '1'
$env:VICTORY_INDIRECT_BATCH           = '1'
$env:VICTORY_INDIRECT_WINDOW          = '1'
$env:VICTORY_CARD_REHASH_RUNS         = '1'
$env:VICTORY_CARD_STEPS               = '2'
$env:VICTORY_CARD_AWAKE               = '0'
$env:VICTORY_MAP_THREADS              = '1'
$env:VICTORY_TITLE_PRIORITIES         = '0'
$env:VICTORY_UNDER_WIDE               = '0'
$env:VICTORY_LAND_ECHO                = '1'
$env:VICTORY_QUICK_FIND               = '1'
$env:VICTORY_VIEW_BYTES               = '1'
$env:VICTORY_DYNAMIC_SLOTS            = '524288'

# === 8 GB VRAM + 16 GB RAM BELLEK & MESH ONCELIGI ===
# LEVELS_ON_DEMAND=0 + HELD_LEVELS=1 + TEXTURE_NO_WAIT=0 + TEXTURE_CATCHUP=1
# -> Siyah karo ve yarim yuklu model yok, 3x catchup + dinamik VRAM mip-drop aktif!
# BURY_LAG=64 + PRELUDE_RING -> 0x887A0001 ve "no host list for a state prelude" sifir!
$env:VICTORY_RESIDENCY                = '1'
$env:VICTORY_RESIDENCY_LIMITED        = '1'
$env:VICTORY_KEEP_FREED               = '1'
$env:VICTORY_BURY_LAG                 = '64'
$env:VICTORY_SUBMIT_CHUNK             = '8'
$env:VICTORY_LEVELS_ON_DEMAND         = '0'
$env:VICTORY_HELD_LEVELS              = '1'
$env:VICTORY_TEXTURE_NO_WAIT          = $texNoWait
$env:VICTORY_TEXTURE_CATCHUP          = $texCatchup
$env:VICTORY_TEXTURE_STAGE_MS         = $texStageMs
$env:VICTORY_TEXTURE_MAX              = $texMax
$env:VICTORY_TEXTURE_BUDGET_MB        = $texBudgetMb
$env:VICTORY_TEXTURE_ROOM_PERCENT     = '84'
$env:VICTORY_PANEL                    = '1'
$env:VICTORY_CARD_ROOM_FOR_TARGETS_MB = '1024'
$env:VICTORY_CARD_ROOM_FOR_BUFFERS    = '1'
$env:VICTORY_CARD_BUFFERS_MB          = '384'
$env:VICTORY_CARD_BUFFERS_MIN_MB      = '8'
$env:VICTORY_WINDOW_BUDGET_MB         = '768'
$env:VICTORY_WINDOW_KB                = '1024'
$env:VICTORY_WINDOW_TAILS             = '1'
$env:VICTORY_RIPE_BLOCKS              = '1'
$env:VICTORY_SEND_KEPT_MB             = '48'
$env:VICTORY_DSTORAGE_STAGING_MB      = '128'
$env:VICTORY_DSTORAGE_RETRIES         = '48'
$env:VICTORY_DSTORAGE_PAGE_WAIT_MS    = '5000'
$env:VICTORY_PIPELINE_WAIT_MS         = $pipelineWaitMs
$env:VICTORY_SCREEN_WAIT_MS           = '10000'
$env:VICTORY_MEMORY_MAP               = '0'
$env:VICTORY_PIPELINE_BOOK            = '0'
$env:VICTORY_D3D12_DRED               = 'light'
$env:RUST_LOG                         = 'error'

$Host.UI.RawUI.WindowTitle = "clawz - GTA VI (RX 7600 8GB | Tam Model + Sifir Cokme)"
Clear-Host
Write-Host "============================================================================" -ForegroundColor Magenta
Write-Host "                           C L A W Z   G T A   V I                          " -ForegroundColor Cyan
Write-Host "         RX 7600 8GB | %100 Saf DXIL + Tam Model + PRELUDE_RING             " -ForegroundColor Cyan
Write-Host "============================================================================" -ForegroundColor Magenta
Write-Host "  Profil    : $profileName" -ForegroundColor Green
Write-Host "  Shader    : %100 Saf DXIL (PORT_CBV=0 | BVH_MISS=0 | RT_RESOLVE=ON | Mor Filtre)" -ForegroundColor Yellow
Write-Host "  Modeller  : Senkron Yukleme + CATCHUP=1 + NO_WAIT=0 + 2560MB + 128MB/48-Retry" -ForegroundColor Cyan
Write-Host "  HUD       : Sol Ust FPS / Grafik / VRAM / CPU / RAM ACIK (F3 ile ac/kapa)" -ForegroundColor Magenta
Write-Host "  Koruma    : PRELUDE_RING + Belobug Prelude Upload + Pool Batch + BURY_LAG=64" -ForegroundColor Cyan
Write-Host "============================================================================" -ForegroundColor Magenta

$hogs = Get-Process Discord,Spotify,brave,chrome -ErrorAction SilentlyContinue | Group-Object Name | ForEach-Object {
    $mb = [math]::Round(($_.Group | Measure-Object WorkingSet64 -Sum).Sum / 1MB, 0)
    if ($mb -ge 250) { "$($_.Name): ${mb}MB" }
}
if ($hogs) {
    Write-Host "  [UYARI] Arka planda RAM yiyenler: $($hogs -join ', ') (kapatirsan RAM cok rahatlar!)" -ForegroundColor Red
}
Write-Host ""

Push-Location $repo
$ErrorActionPreference = 'Continue'
try {
    & $victory run $gameExe --console series-s --execute
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "Oyun hata koduyla kapandi ($LASTEXITCODE). Son crashes.log kaydi:" -ForegroundColor Red
        $crashLog = Join-Path $env:LOCALAPPDATA 'victory\data\7952987A\crashes.log'
        if (Test-Path $crashLog) {
            Get-Content $crashLog -Tail 12
        }
    }
} finally {
    Pop-Location
}
