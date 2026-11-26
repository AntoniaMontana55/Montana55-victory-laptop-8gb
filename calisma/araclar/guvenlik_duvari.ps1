# victory.exe (victory-fork + vw-runtime, release + diag) için GİDEN bağlantı engeli. Yönetici olarak çalıştır.
# Geri almak: Remove-NetFirewallRule -DisplayName "victory-fork engel*"
$exes = @(
  'C:\CLLL\.calisma\victory-fork\target\release\victory.exe',
  'C:\CLLL\.calisma\victory-fork\target\diag\victory.exe',
  'C:\CLLL\.calisma\vw-runtime\target\release\victory.exe',
  'C:\CLLL\.calisma\vw-runtime\target\diag\victory.exe',
  'C:\CLLL\.calisma\vf-465\target\release\victory.exe',
  'C:\CLLL\.calisma\vf-465\target\diag\victory.exe'
)
$i = 0
foreach ($e in $exes) {
  $i++
  Remove-NetFirewallRule -DisplayName "victory-fork engel $i giden" -ErrorAction SilentlyContinue
  Remove-NetFirewallRule -DisplayName "victory-fork engel $i gelen" -ErrorAction SilentlyContinue
  New-NetFirewallRule -DisplayName "victory-fork engel $i giden" -Direction Outbound -Program $e -Action Block -Profile Any | Out-Null
  New-NetFirewallRule -DisplayName "victory-fork engel $i gelen" -Direction Inbound -Program $e -Action Block -Profile Any | Out-Null
}
Get-NetFirewallRule -DisplayName "victory-fork engel*" | Select-Object DisplayName, Direction, Action, Enabled | Format-Table -AutoSize
Read-Host "Tamam, Enter ile kapat"
