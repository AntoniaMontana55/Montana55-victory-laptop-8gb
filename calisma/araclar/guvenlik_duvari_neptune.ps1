# Neptune_VI kaynak derlemesi ve bizim release exe'si için giden + gelen engel (yönetici).
# Geri almak: Remove-NetFirewallRule -DisplayName "victory-fork engel 9*", "victory-fork engel 10*", "victory-fork engel 11*"
Start-Transcript -Path "$env:TEMP\duvar_neptune.log" -Force | Out-Null
$exes = [ordered]@{
  '9'  = 'C:\CLLL\.calisma\neptune-src\target\diag\victory.exe'
  '10' = 'C:\CLLL\.calisma\neptune-src\target\release\victory.exe'
  '11' = 'C:\CLLL\.calisma\vw-avatar\target\release\victory.exe'
}
foreach ($i in $exes.Keys) {
  $e = $exes[$i]
  Remove-NetFirewallRule -DisplayName "victory-fork engel $i giden" -ErrorAction SilentlyContinue
  Remove-NetFirewallRule -DisplayName "victory-fork engel $i gelen" -ErrorAction SilentlyContinue
  New-NetFirewallRule -DisplayName "victory-fork engel $i giden" -Direction Outbound -Program $e -Action Block -Profile Any | Out-Null
  New-NetFirewallRule -DisplayName "victory-fork engel $i gelen" -Direction Inbound -Program $e -Action Block -Profile Any | Out-Null
}
Get-NetFirewallRule | Where-Object DisplayName -like 'victory-fork engel*' | Select-Object -ExpandProperty DisplayName
Stop-Transcript | Out-Null
