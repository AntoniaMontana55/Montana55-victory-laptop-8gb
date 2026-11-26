# Güncel en iyi ayarlarla (B13b / c0-ltosuz tabanı) tek komutla ölçüm koşusu.
# Taban koşul her seferinde aynı kalsın diye ayarlar burada sabit; deneme yalnız -Ek ile eklenir (TEK değişken).
# Kullanım:
#   kosu.ps1 -Tag p1-profil -Ek @{ VICTORY_PROFILE = '150,240,10,C:\...\profil.txt' }
#   kosu.ps1 -Tag c2-deneme -Ek @{ VICTORY_LOOKUP_CACHE = '1' } -Dakika 4.5
param(
    [Parameter(Mandatory)][ValidatePattern('^[\p{L}\p{Nd}_-]{1,64}$')][string]$Tag,
    [hashtable]$Ek = @{},
    [double]$Dakika = 4.5,
    [string]$VictoryExe = 'C:\CLLL\.calisma\vw-avatar\target\diag\victory.exe',
    [switch]$DryRun
)
$ErrorActionPreference = 'Stop'

# 26.11 taban ayar seti (c0-ltosuz 172.7 ms, B13b 169.6 ms; kaynak: kosular\20261126-050543_c0-ltosuz\ayarlar.txt).
$taban = [ordered]@{
    VICTORY_AMBIENT            = '0.8'
    VICTORY_BOOK               = '0'
    VICTORY_CARD_ACCOUNTING    = $null
    VICTORY_CARD_AWAKE         = '0'
    VICTORY_CARD_BUFFERS_MB    = $null
    VICTORY_CARD_REHASH_RUNS   = '1'
    VICTORY_CARD_STEPS         = '2'
    VICTORY_D3D12_DRED         = 'light'
    VICTORY_INDIRECT_BATCH     = '2'
    VICTORY_INDIRECT_WINDOW    = '1'
    VICTORY_INPLACE_PROBE      = $null
    VICTORY_JOIN_COPIES        = '1'
    VICTORY_LAND_ECHO          = '0'
    VICTORY_MAP_THREADS        = '1'
    VICTORY_MEMORY_MAP         = '1'
    VICTORY_PIPELINE_BOOK      = '1'
    VICTORY_RELAY              = '2'
    VICTORY_RELAY_COPY         = '1'
    VICTORY_SEND_KEPT_MB       = '128'
    VICTORY_SHOW_AFTER         = '1'
    VICTORY_SKIP_PIPELINES     = $null
    VICTORY_TEXTURE_BUDGET_MB  = '1024'
    VICTORY_TEXTURE_NO_WAIT    = '1'
    VICTORY_TEXTURE_ROOM_PERCENT = $null
    VICTORY_TITLE_PRIORITIES   = '0'
    VICTORY_UNDER_WIDE         = '0'
    VICTORY_WIDE_BACK          = '1'
    VICTORY_WINDOW_KB          = '256'
    VICTORY_WINDOW_TAILS       = '1'
    VICTORY_WINDOWS_BAR_MB     = '2048'
    VICTORY_WINDOWS_BAR_PERCENT = '88'
}
$set = @{}
foreach ($k in $taban.Keys) { $set[$k] = $taban[$k] }
foreach ($k in $Ek.Keys) { $set[$k] = $Ek[$k] }

# Enter deseni (Yiğit "unutma", 26.11): 20-150 sn, 2.5 sn basılı / 0.5 sn bırak (intro/hikâye basılı-tut ile atlanır).
$tuslar = @(for ($t = 20; $t -lt 149; $t += 3) { "$t-$($t + 2.5):Enter" }) + '149-150:Enter'

$arg = @{
    Keys           = $tuslar -join ','
    Game           = '<OYUN_KLASORU>\game_scarlett_master.exe'
    Build          = 'diag'
    VictoryExe     = $VictoryExe
    Console        = 'series-s'
    Tag            = $Tag
    MaxMinutes     = $Dakika
    Set            = $set
    # Yiğit onaylı koruma (26.11): boş RAM 0.2 GB / 30 sn, commit tavanı 28 GB.
    MinFreeRamGB   = 0.2
    SustainSeconds = 30
    MaxCommitGB    = 28
}
if ($DryRun) { $arg.DryRun = $true }
"Koşu $Tag | ek: " + (($Ek.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ', ')
# Erken teşhis (düz renk / donma / dünya gelmedi → DURDUR) koşuyla birlikte, ayrı süreçte.
if (-not $DryRun) {
    Start-Process powershell -WindowStyle Hidden -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File',
        (Join-Path $PSScriptRoot 'erken_teshis.ps1') | Out-Null
}
& (Join-Path $PSScriptRoot 'olcum_profili.ps1') @arg
