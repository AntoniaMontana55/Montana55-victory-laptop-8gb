# cool_fox (codec-d, f4b48ad) kaynaktan derlenen release exe'si için giden + gelen engel (yönetici).
# Geri almak: Remove-NetFirewallRule -DisplayName "victory-fork engel 12*"
Start-Transcript -Path "$env:TEMP\duvar_coolfox.log" -Force | Out-Null
$e = 'C:\CLLL\.calisma\coolfox-wt\target\release\victory.exe'
Remove-NetFirewallRule -DisplayName "victory-fork engel 12 giden" -ErrorAction SilentlyContinue
Remove-NetFirewallRule -DisplayName "victory-fork engel 12 gelen" -ErrorAction SilentlyContinue
New-NetFirewallRule -DisplayName "victory-fork engel 12 giden" -Direction Outbound -Program $e -Action Block -Profile Any | Out-Null
New-NetFirewallRule -DisplayName "victory-fork engel 12 gelen" -Direction Inbound -Program $e -Action Block -Profile Any | Out-Null
Get-NetFirewallRule | Where-Object DisplayName -like 'victory-fork engel 12*' | Select-Object -ExpandProperty DisplayName
Stop-Transcript | Out-Null
