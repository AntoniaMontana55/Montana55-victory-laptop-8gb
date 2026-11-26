#requires -Version 5.1
<#
NVIDIA LUID'leri başlangıçta DXGI EnumAdapters1 ve GetDesc1 ile alınır.
VendorId 0x10DE olan adaptörlerin LUID'leri GPU Engine örnek adlarıyla
eşleştirilir. Böylece Optimus'ta Intel motorları hesaba girmez.
Birden çok motor paralel çalışabildiğinden motor türü ve GPU toplamı için
ilgili motorların en yüksek kullanımı yazılır. Bellek sayaçları süreç
PID'sine göre tüm adaptörler üzerinden toplanır.
Disk sütunları Windows Process G/Ç sayaçlarıdır ve dosya dışı G/Ç içerebilir.
#>
[CmdletBinding()]
param(
    [string]$OutDir,
    [ValidateRange(1, 30)]
    [int]$AralikSn = 2,
    [ValidateRange(1, 1440)]
    [int]$BeklemeDk = 2,
    [ValidatePattern('^[\p{L}\p{Nd}_. -]+$')]
    [string]$SurecAdi = 'victory'
)

$ErrorActionPreference = 'Continue'
$culture = [Globalization.CultureInfo]::InvariantCulture
$SurecAdi = $SurecAdi -replace '\.exe$', ''

function Get-NvidiaLuid {
    if (-not ('VictoryMetrics.Dxgi' -as [type])) {
        Add-Type -ErrorAction Stop -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Globalization;
using System.Runtime.InteropServices;

namespace VictoryMetrics {
    public static class Dxgi {
        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
        private struct AdapterDesc {
            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
            public string Description;
            public uint VendorId, DeviceId, SubSysId, Revision;
            public UIntPtr DedicatedVideoMemory, DedicatedSystemMemory, SharedSystemMemory;
            public uint LuidLow;
            public int LuidHigh;
            public uint Flags;
        }

        [DllImport("dxgi.dll", ExactSpelling = true)]
        private static extern int CreateDXGIFactory1(ref Guid id, out IntPtr factory);
        [UnmanagedFunctionPointer(CallingConvention.StdCall)]
        private delegate int EnumAdapters1(IntPtr factory, uint index, out IntPtr adapter);
        [UnmanagedFunctionPointer(CallingConvention.StdCall)]
        private delegate int GetDesc1(IntPtr adapter, out AdapterDesc desc);

        private static IntPtr Method(IntPtr comObject, int index) {
            return Marshal.ReadIntPtr(Marshal.ReadIntPtr(comObject), index * IntPtr.Size);
        }

        public static string[] NvidiaLuids() {
            var luids = new List<string>();
            var factoryId = new Guid("770aae78-f26f-4dba-a829-253c83d1b387");
            IntPtr factory;
            Marshal.ThrowExceptionForHR(CreateDXGIFactory1(ref factoryId, out factory));
            try {
                // IDXGIFactory1::EnumAdapters1 and IDXGIAdapter1::GetDesc1 vtable slots.
                var enumerate = (EnumAdapters1)Marshal.GetDelegateForFunctionPointer(
                    Method(factory, 12), typeof(EnumAdapters1));
                for (uint index = 0; ; index++) {
                    IntPtr adapter;
                    int result = enumerate(factory, index, out adapter);
                    if (result == unchecked((int)0x887A0002)) break; // DXGI_ERROR_NOT_FOUND
                    Marshal.ThrowExceptionForHR(result);
                    try {
                        var describe = (GetDesc1)Marshal.GetDelegateForFunctionPointer(
                            Method(adapter, 10), typeof(GetDesc1));
                        AdapterDesc desc;
                        Marshal.ThrowExceptionForHR(describe(adapter, out desc));
                        if (desc.VendorId == 0x10DE) {
                            luids.Add(String.Format(CultureInfo.InvariantCulture,
                                "luid_0x{0:x8}_0x{1:x8}", desc.LuidHigh, desc.LuidLow));
                        }
                    } finally {
                        Marshal.Release(adapter);
                    }
                }
            } finally {
                Marshal.Release(factory);
            }
            return luids.ToArray();
        }
    }
}
'@
    }
    [VictoryMetrics.Dxgi]::NvidiaLuids()
}

function Get-ThreadTicks($Process) {
    $times = @{}
    foreach ($thread in $Process.Threads) {
        try {
            $times[$thread.Id] = $thread.TotalProcessorTime.Ticks
        } catch {
            # Bir iş parçacığı örnek alınırken kapanabilir.
        }
    }
    return $times
}

function Format-Number($Value) {
    if ($null -ne $Value) { ([double]$Value).ToString('0.00', $culture) }
}

function Format-Stats($Values, [switch]$Median) {
    if ($Values.Count -eq 0) { return 'veri yok' }
    $stats = $Values | Measure-Object -Average -Maximum
    $label = "ort. $(Format-Number $stats.Average)"
    if ($Median) {
        $sorted = @($Values | Sort-Object)
        $lower = [int][Math]::Floor(($sorted.Count - 1) / 2)
        $upper = [int][Math]::Floor($sorted.Count / 2)
        $label += " / medyan $(Format-Number (($sorted[$lower] + $sorted[$upper]) / 2))"
    }
    "$label / maks. $(Format-Number $stats.Maximum)"
}

$wait = [Diagnostics.Stopwatch]::StartNew()
do {
    $target = Get-Process -Name $SurecAdi -ErrorAction SilentlyContinue |
        Where-Object { $_.Id -ne $PID } |
        Sort-Object -Property StartTime -Descending |
        Select-Object -First 1
    if ($target) { break }
    Start-Sleep -Milliseconds 250
} while ($wait.Elapsed.TotalMinutes -lt $BeklemeDk)
if (-not $target) {
    Write-Host "$SurecAdi.exe başlamadı"
    exit 1
}

if (-not $OutDir) {
    $runs = Join-Path $env:LOCALAPPDATA 'Victory\kosular'
    $latest = Get-ChildItem -LiteralPath $runs -Directory -ErrorAction SilentlyContinue |
        Sort-Object -Property LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $latest) {
        Write-Error "$runs altında koşu klasörü bulunamadı. -OutDir belirtin."
        exit 1
    }
    $OutDir = $latest.FullName
}

$OutDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutDir)
if ($OutDir -notmatch '^[A-Za-z]:[\\/]' -or
    [IO.DriveInfo]::new([IO.Path]::GetPathRoot($OutDir)).DriveType -eq 'Network') {
    Write-Error '-OutDir yerel bir klasör olmalı.'
    exit 1
}
$null = [IO.Directory]::CreateDirectory($OutDir)
$csvPath = Join-Path $OutDir 'kaynak.csv'
$summaryPath = Join-Path $OutDir 'kaynak_ozet.txt'

$nvidiaLuids = @()
try {
    $nvidiaLuids = @(Get-NvidiaLuid)
} catch {
    Write-Verbose "DXGI adaptör bilgisi alınamadı. $_"
}
Write-Verbose "NVIDIA LUID: $($nvidiaLuids -join ', ')"

# Perflib'in sabit başlık kimlikleri yerelleştirilmiş sayaç yollarını çözer.
$perflib = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Perflib'
$localizedNames = @{}
$table = (Get-ItemProperty -LiteralPath "$perflib\CurrentLanguage" -Name Counter -ErrorAction SilentlyContinue).Counter
for ($i = 0; $i + 1 -lt $table.Count; $i += 2) {
    $localizedNames[$table[$i]] = $table[$i + 1]
}
$processSet = $localizedNames['230']
$processCounters = [ordered]@{ '784' = 'id'; '28' = 'faults'; '1420' = 'read'; '1422' = 'write' }
$counterPaths = [Collections.Generic.List[string]]::new()
$processCounterByName = @{}
foreach ($key in $processCounters.Keys) {
    $name = $localizedNames[$key]
    if ($processSet -and $name) {
        $counterPaths.Add("\$processSet($SurecAdi*)\$name")
        $processCounterByName[$name] = $processCounters[$key]
    }
}

$targetPid = $target.Id
$gpuEngineSet = $localizedNames['5802']
if (-not $gpuEngineSet) { $gpuEngineSet = 'GPU Engine' }
$gpuEngineUsage = $localizedNames['5806']
if (-not $gpuEngineUsage) { $gpuEngineUsage = 'Utilization Percentage' }
$gpuMemorySet = $localizedNames['5808']
if (-not $gpuMemorySet) { $gpuMemorySet = 'GPU Process Memory' }
$gpuDedicated = $localizedNames['5816']
if (-not $gpuDedicated) { $gpuDedicated = 'Dedicated Usage' }
$gpuShared = $localizedNames['5818']
if (-not $gpuShared) { $gpuShared = 'Shared Usage' }
foreach ($luid in $nvidiaLuids) {
    $counterPaths.Add("\$gpuEngineSet(pid_${targetPid}_${luid}*)\$gpuEngineUsage")
}
$counterPaths.Add("\$gpuMemorySet(pid_${targetPid}_*)\$gpuDedicated")
$counterPaths.Add("\$gpuMemorySet(pid_${targetPid}_*)\$gpuShared")

$columns = [ordered]@{
    zaman = $null
    surec_cpu_yuzde_toplam = $null
    surec_cpu_cekirdek = $null
    thread_sayisi = $null
}
for ($rank = 1; $rank -le 5; $rank++) {
    $columns["top${rank}_tid"] = $null
    $columns["top${rank}_cekirdek_yuzde"] = $null
}
foreach ($name in 'gpu_3d_yuzde', 'gpu_compute_yuzde', 'gpu_copy_yuzde', 'gpu_toplam_yuzde',
    'gpu_ozel_bellek_mb', 'gpu_paylasimli_bellek_mb', 'sayfa_hatasi_sn',
    'disk_okuma_mb_sn', 'disk_yazma_mb_sn') {
    $columns[$name] = $null
}

$totals = @{}
foreach ($name in 'top', '3d', 'compute', 'copy', 'faults') {
    $totals[$name] = [Collections.Generic.List[double]]::new()
}
$sampleCount = 0
$bottleneckCount = 0
$utf8 = [Text.UTF8Encoding]::new($true)
$csv = [IO.StreamWriter]::new($csvPath, $false, $utf8)
try {
    $csv.AutoFlush = $true
    $header = [pscustomobject]$columns | ConvertTo-Csv -NoTypeInformation | Select-Object -First 1
    $csv.WriteLine($header)
    $clock = [Diagnostics.Stopwatch]::StartNew()
    $previousTime = 0.0
    $previousCpu = $null
    $previousThreads = @{}
    $delayMs = $AralikSn * 1000

    while (-not $target.HasExited) {
        $target.Refresh()
        try {
            $cpuTicks = $target.TotalProcessorTime.Ticks
            $threadCount = $target.Threads.Count
            $threadTicks = Get-ThreadTicks $target
        } catch {
            if ($target.HasExited) { break }
            $cpuTicks = $null
            $threadCount = $null
            $threadTicks = @{}
        }
        $sampleTime = $clock.Elapsed.TotalMilliseconds
        $wallTicks = ($sampleTime - $previousTime) * [TimeSpan]::TicksPerMillisecond

        if ($previousTime -gt 0 -and $wallTicks -gt 0) {
            $row = [ordered]@{}
            foreach ($name in $columns.Keys) { $row[$name] = $null }
            $row.zaman = [DateTimeOffset]::Now.ToString('o', $culture)
            $row.thread_sayisi = $threadCount
            if ($null -ne $previousCpu -and $null -ne $cpuTicks) {
                $cpuPercent = 100.0 * ($cpuTicks - $previousCpu) / $wallTicks
                $row.surec_cpu_cekirdek = Format-Number $cpuPercent
                $row.surec_cpu_yuzde_toplam = Format-Number ($cpuPercent / [Environment]::ProcessorCount)
            }
            $rank = 0
            $topThreads = foreach ($tid in $threadTicks.Keys) {
                if ($previousThreads.ContainsKey($tid)) {
                    $delta = $threadTicks[$tid] - $previousThreads[$tid]
                    if ($delta -ge 0) { [pscustomobject]@{ Id = $tid; Delta = $delta } }
                }
            }
            foreach ($thread in ($topThreads | Sort-Object -Property Delta -Descending | Select-Object -First 5)) {
                $rank++
                $percent = 100.0 * $thread.Delta / $wallTicks
                $row["top${rank}_tid"] = $thread.Id
                $row["top${rank}_cekirdek_yuzde"] = Format-Number $percent
                if ($rank -eq 1) {
                    $totals.top.Add($percent)
                    if ([Math]::Round($percent, 2, [MidpointRounding]::AwayFromZero) -ge 90) { $bottleneckCount++ }
                }
            }

            $processValues = @{}
            $gpuUsage = @{}
            $gpuMemory = @{}
            $lookupPid = '^pid_' + $targetPid + '_'
            $batch = Get-Counter -Counter $counterPaths.ToArray() -ErrorAction SilentlyContinue
            foreach ($counter in $batch.CounterSamples) {
                if ($counter.Status -notin 0, 1 -or [double]::IsNaN($counter.CookedValue) -or
                    [double]::IsInfinity($counter.CookedValue) -or $counter.CookedValue -lt 0) { continue }
                $counterSeparator = $counter.Path.LastIndexOf('\')
                $leaf = $counter.Path.Substring($counterSeparator + 1)
                $instance = $counter.InstanceName
                $value = $counter.CookedValue
                if ($processCounterByName.ContainsKey($leaf) -and $instance -notmatch $lookupPid) {
                    # InstanceName, aynı adlı süreçlerin #n ekini içermez.
                    $processInstancePath = $counter.Path.Substring(0, $counterSeparator)
                    if (-not $processValues.ContainsKey($processInstancePath)) { $processValues[$processInstancePath] = @{} }
                    $processValues[$processInstancePath][$processCounterByName[$leaf]] = $value
                } elseif ($instance -match $lookupPid) {
                    if ($leaf -eq $gpuEngineUsage -and
                        $instance -match '(luid_0x[0-9a-f]+_0x[0-9a-f]+)_.*_engtype_(.*)$' -and
                        $Matches[1] -in $nvidiaLuids) {
                        $kind = $Matches[2]
                        if ($kind -match '^compute') { $kind = 'compute' }
                        $gpuUsage[$kind] = [Math]::Max($gpuUsage[$kind], $value)
                    } elseif ($leaf -eq $gpuDedicated -or $leaf -eq $gpuShared) {
                        $gpuMemory[$leaf] += $value
                    }
                }
            }

            foreach ($kind in '3d', 'compute', 'copy') {
                if ($gpuUsage.ContainsKey($kind)) {
                    $row["gpu_${kind}_yuzde"] = Format-Number $gpuUsage[$kind]
                    $totals[$kind].Add($gpuUsage[$kind])
                }
            }
            if ($gpuUsage.Count) {
                $row.gpu_toplam_yuzde = Format-Number (($gpuUsage.Values | Measure-Object -Maximum).Maximum)
            }
            if ($gpuMemory.ContainsKey($gpuDedicated)) {
                $row.gpu_ozel_bellek_mb = Format-Number ($gpuMemory[$gpuDedicated] / 1MB)
            }
            if ($gpuMemory.ContainsKey($gpuShared)) {
                $row.gpu_paylasimli_bellek_mb = Format-Number ($gpuMemory[$gpuShared] / 1MB)
            }
            foreach ($values in $processValues.Values) {
                if ($values.id -ne $targetPid) { continue }
                if ($values.ContainsKey('faults')) {
                    $row.sayfa_hatasi_sn = Format-Number $values.faults
                    $totals.faults.Add($values.faults)
                }
                if ($values.ContainsKey('read')) {
                    $row.disk_okuma_mb_sn = Format-Number ($values.read / 1MB)
                }
                if ($values.ContainsKey('write')) {
                    $row.disk_yazma_mb_sn = Format-Number ($values.write / 1MB)
                }
                break
            }

            $csv.WriteLine(([pscustomobject]$row | ConvertTo-Csv -NoTypeInformation | Select-Object -Last 1))
            $sampleCount++
        }
        $previousTime = $sampleTime
        $previousCpu = $cpuTicks
        $previousThreads = $threadTicks
        $remaining = [Math]::Max(0, $delayMs - ($clock.Elapsed.TotalMilliseconds - $sampleTime))
        if ($target.WaitForExit([int]$remaining)) { break }
    }
} finally {
    $csv.Dispose()
    $target.Dispose()
}

$bottleneck = 'veri yok'
if ($totals.top.Count) {
    $bottleneck = "%$(Format-Number (100.0 * $bottleneckCount / $sampleCount)) ($bottleneckCount/$sampleCount örnek)"
}
$pageFaults = 'veri yok'
if ($totals.faults.Count) {
    $pageFaults = Format-Number (($totals.faults | Measure-Object -Average).Average)
}
[IO.File]::WriteAllLines($summaryPath, [string[]]@(
    "Örnek sayısı: $sampleCount"
    "En yoğun iş parçacığı (% çekirdek): $(Format-Stats $totals.top -Median)"
    "Tek çekirdek darboğazı (en yoğun iş parçacığı ≥ %90): $bottleneck"
    "GPU (%): 3D $(Format-Stats $totals['3d']) | Compute $(Format-Stats $totals.compute) | Copy $(Format-Stats $totals.copy)"
    "Ortalama sayfa hatası/sn: $pageFaults"
), $utf8)
