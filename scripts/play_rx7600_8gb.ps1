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

# Arka planda acik kalmis eski victory ve RAM yiyen uygulamalari otomatik kapat
Stop-Process -Name 'victory','Spotify','Discord','brave','chrome' -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

# Eski VICTORY_* degiskenlerini temizle
Get-ChildItem Env:VICTORY_* -ErrorAction SilentlyContinue | ForEach-Object { Remove-Item "Env:$($_.Name)" }

# Sabit Profil (Menu yok, tek tik):
# Varsayilan 2 = 1920x1080 FULL HD + 30 FPS + TAM MODEL (RELAY=2, 16MiB Pool, BURY_LAG=8)
# Profil 1     = 1280x720 HAFIF (Maks FPS / En dusuk VRAM)
if ($Profile -eq '1') {
    $profileName     = '1 - 720p MAKS FPS (1280x720 x0.70 | RELAY=2 | %100 Saf Shader)'
    $outputMode      = '1280x720'
    $renderScale     = '0.70'
    $texBudgetMb     = 'dynamic'
    $texMax          = '0'
} else {
    $profileName     = '2 - 1080p FULL HD + TAM DOKU (1920x1080 x0.70 | RELAY=2 | 16MiB Pool | Dinamik TX)'
    $outputMode      = '1920x1080'
    $renderScale     = '0.70'
    $texBudgetMb     = 'dynamic'
    $texMax          = '0'
}
$relayMode       = '2'
$lateWait        = '3'
$asyncSubmit     = '1'
$texNoWait       = '0'
$texCatchup      = '1'
$texStageMs      = '200'
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
# Belobug PreludeUpload + release_tables + 720p RT + Dinamik TX bütçesi ile VRAM < 6.4 GiB
# (Shared tasma sifir, OOM 0x8007000E hatti 6.1 GiB'de korunur!).
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

# === 8 GB VRAM + 16 GB RAM BELLEK, ANTI-OOM & MESH ONCELIGI ===
# BURY_LAG=8 + KEEP_FREED=0 + WINDOW_BUDGET=512MB/512KB + SNAPSHOT_POOL=64
# -> COMMIT 19.2GB / SHARED 6.4GB / O 2.5GB sismesi ve 0x8007000E cokmesi kokten kapanir!
# CARD_BUFFERS_MB=512 (MIN=4MB) -> CPU snapshot/compare yuku (SUB 318ms) GPU VRAM'e tasinir!
# TEXTURE_NO_WAIT=0 + 16MiB havuz + BUDGET=dynamic -> siyah yuz / bos karo ve TX kilitlenmesi biter:
# doku havuzu 2.8-3.4 GiB'ye kadar gercek DXGI durumuna gore genisler, yalnizca 6.8 GiB+
# gercek baskida katmanli LRU tahliyesi (aktif kare pin'iyle) devreye girer.
$env:VICTORY_RESIDENCY                = '1'
$env:VICTORY_RESIDENCY_LIMITED        = '1'
$env:VICTORY_KEEP_FREED               = '0'
$env:VICTORY_BURY_LAG                 = '8'
$env:VICTORY_SUBMIT_CHUNK             = '6'
$env:VICTORY_LEVELS_ON_DEMAND         = '0'
$env:VICTORY_HELD_LEVELS              = '1'
$env:VICTORY_TEXTURE_NO_WAIT          = $texNoWait
$env:VICTORY_TEXTURE_CATCHUP          = $texCatchup
$env:VICTORY_TEXTURE_STAGE_MS         = $texStageMs
$env:VICTORY_TEXTURE_MAX              = $texMax
$env:VICTORY_TEXTURE_BUDGET_MB        = $texBudgetMb
$env:VICTORY_TEXTURE_ROOM_PERCENT     = '93'
$env:VICTORY_PANEL                    = '1'
$env:VICTORY_CARD_ROOM_FOR_TARGETS_MB = '1024'
$env:VICTORY_CARD_ROOM_FOR_BUFFERS    = '1'
$env:VICTORY_CARD_BUFFERS_MB          = '512'
$env:VICTORY_CARD_BUFFERS_MIN_MB      = '4'
$env:VICTORY_WINDOW_BUDGET_MB         = '512'
$env:VICTORY_WINDOW_KB                = '512'
$env:VICTORY_WINDOW_TAILS             = '1'
$env:VICTORY_SNAPSHOT_POOL            = '64'
$env:VICTORY_RIPE_BLOCKS              = '1'
$env:VICTORY_SEND_KEPT_MB             = '32'
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
Write-Host "  Shader    : %100 Saf DXIL (AMBIENT=0.85 | AIR=0.25 | RT_SKIP + CUTSCENE_FAST)" -ForegroundColor Yellow
Write-Host "  Modeller  : Cilt Bake Korumasi + CATCHUP=1 + TX Tam Cozunurluk (Dinamik DXGI)" -ForegroundColor Cyan
Write-Host "  HUD       : Sol Ust FPS / Grafik / VRAM / CPU / RAM ACIK (F3 ile ac/kapa)" -ForegroundColor Magenta
Write-Host "  RAM/VRAM  : Spotify/Discord/Brave/Chrome otomatik kapatildi + PRELUDE_RING" -ForegroundColor Cyan
Write-Host "============================================================================" -ForegroundColor Magenta
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
