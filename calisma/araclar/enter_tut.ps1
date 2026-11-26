# VICTORY_KEYS için Enter deseni: başlangıç ekranını geçer, intro/hikâye ara
# sahnelerini "basılı tut" ile atlar. Her döngü: Tut sn basılı, Birak sn bırak
# (bırakma, menülerin yeni basışı görmesi için).
# Kullanım: $keys = & enter_tut.ps1 [-Bas 20] [-Bit 150] [-Tut 2.5] [-Birak 0.5]
param(
    [double]$Bas = 20,
    [double]$Bit = 150,
    [double]$Tut = 2.5,
    [double]$Birak = 0.5
)
$inv = [Globalization.CultureInfo]::InvariantCulture
$k = @()
$t = $Bas
while ($t -lt $Bit) {
    $son = [math]::Min($t + $Tut, $Bit)
    $k += ('{0}-{1}:Enter' -f $t.ToString($inv), $son.ToString($inv))
    $t = [math]::Round($son + $Birak, 2)
}
$k -join ','
