# vw-pr10 (ZSlayer PR #10, e7c05277) victory.exe icin GIDEN + GELEN engel. Yonetici olarak calistir.
# Mevcut "victory-fork engel 1-6" kurallarina dokunmaz.
# Geri almak: Remove-NetFirewallRule -DisplayName "victory-fork engel 7*"
$e = 'C:\CLLL\.calisma\vw-pr10\target\diag\victory.exe'
Remove-NetFirewallRule -DisplayName "victory-fork engel 7 giden" -ErrorAction SilentlyContinue
Remove-NetFirewallRule -DisplayName "victory-fork engel 7 gelen" -ErrorAction SilentlyContinue
New-NetFirewallRule -DisplayName "victory-fork engel 7 giden" -Direction Outbound -Program $e -Action Block -Profile Any | Out-Null
New-NetFirewallRule -DisplayName "victory-fork engel 7 gelen" -Direction Inbound -Program $e -Action Block -Profile Any | Out-Null
Get-NetFirewallRule -DisplayName "victory-fork engel*" | Select-Object DisplayName, Direction, Action, Enabled | Format-Table -AutoSize
Read-Host "Tamam, Enter ile kapat"
