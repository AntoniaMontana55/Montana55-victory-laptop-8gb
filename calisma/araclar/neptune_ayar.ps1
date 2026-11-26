# Neptune_VI (b900c646) tools/play-zm.ps1 ayar seti, 8 GB VRAM / 16 GB RAM'e uyarlanmış (26.11, Neon raporu).
# Bellek tavanları (*) play-zm'de 16 GB kart / 64 GB RAM içindi; -Ram32 profilinin de altına çekildi.
# Tuş: Neptune'ün AUTO_A'sı kapalı, bizim Enter deseni (kosu.ps1 VICTORY_KEYS) kullanılır — iki taraf aynı girdiyi alsın.
# Kullanım: kosu.ps1 -Tag n1-neptune -TabanYok -VictoryExe <neptune exe> -Ek (& neptune_ayar.ps1)
@{
    # Bellek / havuz (* uyarlandı: play-zm 4096/1536/2048/2048/512)
    VICTORY_CARD_WINDOWS_MB       = '2048'
    VICTORY_CARD_BUFFERS_MB       = '512'
    VICTORY_LIST_POOL_MB          = '512'
    VICTORY_LIST_POOL_MOST        = '1024'
    VICTORY_TEXTURE_STAGE_MB      = '256'
    VICTORY_WIDE_THREADS          = '12'
    VICTORY_LATE_WAIT_LISTS       = '1024'
    VICTORY_WINDOW_KB             = '1024'
    VICTORY_CARD_BUFFERS_MIN_KB   = '64'
    # Indirect / çizim
    VICTORY_CLAMP_AFTER_WRITER    = '1'
    VICTORY_CLAMP_BATCH           = '1'
    VICTORY_INDIRECT_RAW          = '3'
    VICTORY_THIN                  = '1'
    VICTORY_WATCH_4K              = '1'
    # Geri kopya / senkron
    VICTORY_RELAY                 = '2'
    VICTORY_RELAY_OWN_QUEUE       = '1'
    VICTORY_WIDE_MERGE            = '1'
    VICTORY_WIDE_REFRESH          = '1'
    VICTORY_UNDER_WRITES_WORDS    = '1'
    VICTORY_UNDER_WRITES_RUNS     = '100000'
    VICTORY_UNDER_WRITES_MASKS    = '1'
    VICTORY_UNDER_WRITES_MASKS_LEAST = '1'
    VICTORY_ASK_ONCE              = '1'
    VICTORY_ASK_QUIET             = '8'
    VICTORY_COPY_CHANGED          = '1'
    VICTORY_QUIET_BACK            = '1'
    VICTORY_CARD_REHASH_RUNS      = '1'
    VICTORY_JOIN_COPIES           = '1'
    VICTORY_WINDOW_TAILS          = '1'
    VICTORY_D3D12_BACKGROUND      = '0'
    # Doku / shader / diğer
    VICTORY_BAKE_TEXTURES_MS      = '10000'
    VICTORY_EARLY_TEXTURES_MS     = '40'
    VICTORY_WAVE_OWN_LDS_GROUPS   = '64,128'
    VICTORY_TEXTURE_NO_WAIT       = '1'
    VICTORY_TEXTURE_HASH_EVERY    = '32'
    VICTORY_LEVELS_ON_DEMAND      = '1'
    VICTORY_CARD_WINDOW_UPLOAD    = '1'
    VICTORY_DROP_DXIL             = '1'
    VICTORY_DESC_DATA             = '1'
    VICTORY_VRAM_IDLE_MS          = '15000,2500,1000'
    VICTORY_PIPELINE_WAIT_MS      = '2500'
    VICTORY_AUTO_A                = '0'
    VICTORY_DISCORD               = '0'
    VICTORY_OCEAN_READBACK        = '1'
}
