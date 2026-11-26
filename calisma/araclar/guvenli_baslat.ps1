[CmdletBinding(DefaultParameterSetName = 'Run')]
param(
    [Parameter(Mandatory, ParameterSetName = 'Run')]
    [ValidateNotNullOrEmpty()]
    [string]$Exe,

    [Parameter(ParameterSetName = 'Run')]
    [AllowEmptyString()]
    [string]$ArgumentList = '',

    [Parameter(ParameterSetName = 'Run')]
    [string]$WorkDir,

    [Parameter(ParameterSetName = 'Run')]
    [ValidateRange(0.1, 1024)]
    [double]$MaxCommitGB = 13,

    [Parameter(ParameterSetName = 'Run')]
    [ValidateRange(0, 1024)]
    [double]$MinFreeRamGB = 1.5,

    [Parameter(ParameterSetName = 'Run')]
    [ValidateRange(0, 1024)]
    [double]$MinFreeCommitGB = 6,

    [Parameter(ParameterSetName = 'Run')]
    [ValidateRange(1, 150)]
    [double]$MaxGpuTempC = 87,

    [Parameter(ParameterSetName = 'Run')]
    [ValidateRange(0.01, 1440)]
    [double]$MaxMinutes = 10,

    [Parameter(ParameterSetName = 'Run')]
    [ValidateRange(0, 3600)]
    [double]$SustainSeconds = 5,

    [Parameter(ParameterSetName = 'Run')]
    [string]$LogDir = (Join-Path 'C:\CLLL\.calisma\repo_durum\kosu_loglari' (Get-Date -Format 'yyyyMMdd_HHmmss')),

    [Parameter(ParameterSetName = 'Run')]
    [ValidateSet('Idle', 'BelowNormal', 'Normal', 'AboveNormal', 'High')]
    [string]$Priority = 'BelowNormal',

    [Parameter(ParameterSetName = 'Run')]
    [ValidateNotNullOrEmpty()]
    [string]$StdOutFile,

    [Parameter(ParameterSetName = 'Run')]
    [ValidateNotNullOrEmpty()]
    [string]$StdErrFile,

    [Parameter(Mandatory, ParameterSetName = 'Test')]
    [switch]$SelfTest
)

$ErrorActionPreference = 'Stop'

if (-not ('GuvenliBaslat.NativeJob' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Win32.SafeHandles;

namespace GuvenliBaslat
{
    public sealed class NativeJob : IDisposable
    {
        private const uint KillOnClose = 0x2000;
        private const uint ProcessMemory = 0x100;
        private const uint PriorityClass = 0x20;
        private const uint CreateSuspended = 4;
        private const uint UseStdHandles = 0x100;
        private const uint GenericRead = 0x80000000;
        private const uint GenericWrite = 0x40000000;
        private const uint FileShareRead = 1;
        private const uint CreateAlways = 2;
        private const uint OpenExisting = 3;
        public const uint StoppedExitCode = 70;

        [StructLayout(LayoutKind.Sequential)]
        private struct BasicLimits
        {
            public long PerProcessTime, PerJobTime;
            public uint Flags;
            public UIntPtr MinWorkingSet, MaxWorkingSet;
            public uint ActiveProcesses;
            public UIntPtr Affinity;
            public uint Priority, Scheduling;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct IoCounts
        {
            public ulong ReadOperations, WriteOperations, OtherOperations;
            public ulong ReadBytes, WriteBytes, OtherBytes;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct ExtendedLimits
        {
            public BasicLimits Basic;
            public IoCounts Io;
            public UIntPtr ProcessMemory, JobMemory, PeakProcessMemory, PeakJobMemory;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct Accounting
        {
            public long TotalUserTime, TotalKernelTime, PeriodUserTime, PeriodKernelTime;
            public uint PageFaults, TotalProcesses, ActiveProcesses, TerminatedProcesses;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct StartupInfo
        {
            public uint Size;
            public IntPtr Reserved, Desktop, Title;
            public uint X, Y, XSize, YSize, XChars, YChars, Fill, Flags;
            public ushort Show, ReservedSize;
            public IntPtr ReservedData, StdIn, StdOut, StdErr;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct SecurityAttributes
        {
            public uint Size;
            public IntPtr SecurityDescriptor;
            [MarshalAs(UnmanagedType.Bool)]
            public bool InheritHandle;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct ProcessInfo
        {
            public IntPtr Process, Thread;
            public uint ProcessId, ThreadId;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct ProcessMemoryCounters
        {
            public uint Size, PageFaults;
            public UIntPtr PeakWorkingSet, WorkingSet, QuotaPeakPagedPool, QuotaPagedPool;
            public UIntPtr QuotaPeakNonPagedPool, QuotaNonPagedPool, Pagefile, PeakPagefile, Private;
        }

        public struct MemoryStatus
        {
            public ulong AvailablePhysical, AvailableCommit;
        }

        // GetPerformanceInfo reports system-wide commit even when the caller belongs to a job.
        [StructLayout(LayoutKind.Sequential)]
        private struct PerformanceInfo
        {
            public uint Size;
            public UIntPtr CommitTotal, CommitLimit, CommitPeak, PhysicalTotal, PhysicalAvailable;
            public UIntPtr SystemCache, KernelTotal, KernelPaged, KernelNonpaged, PageSize;
            public uint Handles, Processes, Threads;
        }

        public sealed class ProcessSample
        {
            public ulong PrivateBytes, WorkingSetBytes, CpuTicks;
        }

        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
        private static extern SafeFileHandle CreateJobObject(IntPtr attributes, string name);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool SetInformationJobObject(SafeFileHandle job, int kind, ref ExtendedLimits info, uint size);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool QueryInformationJobObject(SafeFileHandle job, int kind, out Accounting info, uint size, IntPtr returned);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool AssignProcessToJobObject(SafeFileHandle job, IntPtr process);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool TerminateJobObject(SafeFileHandle job, uint exitCode);

        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
        private static extern bool CreateProcess(string application, StringBuilder command, IntPtr processAttributes,
            IntPtr threadAttributes, bool inheritHandles, uint flags, IntPtr environment, string directory,
            ref StartupInfo startup, out ProcessInfo process);

        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
        private static extern SafeFileHandle CreateFile(string path, uint access, uint share,
            ref SecurityAttributes attributes, uint disposition, uint flags, IntPtr template);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern uint ResumeThread(IntPtr thread);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool TerminateProcess(IntPtr process, uint exitCode);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool CloseHandle(IntPtr handle);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetExitCodeProcess(IntPtr process, out uint exitCode);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetProcessTimes(IntPtr process, out ulong created, out ulong exited,
            out ulong kernel, out ulong user);

        [DllImport("psapi.dll", SetLastError = true)]
        private static extern bool GetProcessMemoryInfo(IntPtr process, ref ProcessMemoryCounters counters, uint size);

        [DllImport("psapi.dll", SetLastError = true)]
        private static extern bool GetPerformanceInfo(ref PerformanceInfo info, uint size);

        private readonly SafeFileHandle job;
        private IntPtr process;

        public NativeJob(ulong memoryBytes, uint priority)
        {
            var limits = new ExtendedLimits();
            limits.Basic.Flags = ProcessMemory | KillOnClose | PriorityClass;
            limits.Basic.Priority = priority;
            limits.ProcessMemory = new UIntPtr(memoryBytes);
            job = CreateJobObject(IntPtr.Zero, null);
            if (job.IsInvalid) throw Error("CreateJobObject");
            if (!SetInformationJobObject(job, 9, ref limits, (uint)Marshal.SizeOf(limits)))
            {
                var failure = Error("SetInformationJobObject");
                job.Dispose();
                throw failure;
            }
        }

        public void Start(string exe, string arguments, string directory, string stdout, string stderr)
        {
            var startup = new StartupInfo();
            startup.Size = (uint)Marshal.SizeOf(startup);
            var command = new StringBuilder("\"" + exe + "\"");
            if (!String.IsNullOrEmpty(arguments)) command.Append(' ').Append(arguments);

            ProcessInfo child;
            bool redirect = !String.IsNullOrEmpty(stdout) || !String.IsNullOrEmpty(stderr);
            using (var input = redirect ? OpenStream(null, GenericRead) : null)
            using (var output = redirect ? OpenStream(stdout, GenericWrite) : null)
            using (var error = redirect ? OpenStream(stderr, GenericWrite) : null)
            {
                if (redirect)
                {
                    startup.Flags = UseStdHandles;
                    startup.StdIn = input.DangerousGetHandle();
                    startup.StdOut = output.DangerousGetHandle();
                    startup.StdErr = error.DangerousGetHandle();
                }
                if (!CreateProcess(exe, command, IntPtr.Zero, IntPtr.Zero, redirect, CreateSuspended,
                    IntPtr.Zero, directory, ref startup, out child))
                    throw Error("CreateProcess");
            }

            process = child.Process;
            try
            {
                if (!AssignProcessToJobObject(job, process)) throw Error("AssignProcessToJobObject");
                if (ResumeThread(child.Thread) == UInt32.MaxValue) throw Error("ResumeThread");
            }
            catch
            {
                TerminateProcess(process, StoppedExitCode);
                throw;
            }
            finally
            {
                CloseHandle(child.Thread);
            }
        }

        private static SafeFileHandle OpenStream(string path, uint access)
        {
            bool useNull = String.IsNullOrEmpty(path);
            var attributes = new SecurityAttributes();
            attributes.Size = (uint)Marshal.SizeOf(attributes);
            attributes.InheritHandle = true;
            var handle = CreateFile(useNull ? "NUL" : path, access, FileShareRead,
                ref attributes, useNull ? OpenExisting : CreateAlways, 0, IntPtr.Zero);
            if (!handle.IsInvalid) return handle;
            var failure = Error("CreateFile (" + (useNull ? "NUL" : path) + ")");
            handle.Dispose();
            throw failure;
        }

        public uint ActiveProcesses()
        {
            Accounting accounting;
            if (!QueryInformationJobObject(job, 1, out accounting, (uint)Marshal.SizeOf(typeof(Accounting)), IntPtr.Zero))
                throw Error("QueryInformationJobObject");
            return accounting.ActiveProcesses;
        }

        public void Stop()
        {
            if (!TerminateJobObject(job, StoppedExitCode)) throw Error("TerminateJobObject");
        }

        public uint? ExitCode(uint waitMilliseconds)
        {
            uint state = WaitForSingleObject(process, waitMilliseconds);
            if (state == 258) return null;
            if (state != 0) throw Error("WaitForSingleObject");
            uint code;
            if (!GetExitCodeProcess(process, out code)) throw Error("GetExitCodeProcess");
            return code;
        }

        public ProcessSample Sample()
        {
            if (ExitCode(0).HasValue) return null;
            var memory = new ProcessMemoryCounters();
            memory.Size = (uint)Marshal.SizeOf(memory);
            ulong created, exited, kernel, user;
            if (!GetProcessMemoryInfo(process, ref memory, memory.Size) ||
                !GetProcessTimes(process, out created, out exited, out kernel, out user))
            {
                var failure = Error("Süreç ölçümü");
                if (ExitCode(0).HasValue) return null;
                throw failure;
            }
            return new ProcessSample {
                PrivateBytes = memory.Private.ToUInt64(),
                WorkingSetBytes = memory.WorkingSet.ToUInt64(),
                CpuTicks = kernel + user
            };
        }

        public static MemoryStatus SystemMemory()
        {
            var info = new PerformanceInfo();
            info.Size = (uint)Marshal.SizeOf(info);
            if (!GetPerformanceInfo(ref info, info.Size)) throw Error("GetPerformanceInfo");
            ulong pageSize = info.PageSize.ToUInt64();
            ulong total = info.CommitTotal.ToUInt64(), limit = info.CommitLimit.ToUInt64();
            return new MemoryStatus {
                AvailablePhysical = info.PhysicalAvailable.ToUInt64() * pageSize,
                AvailableCommit = (limit > total ? limit - total : 0) * pageSize
            };
        }

        private static Win32Exception Error(string operation)
        {
            int code = Marshal.GetLastWin32Error();
            return new Win32Exception(code, operation + ": " + new Win32Exception(code).Message);
        }

        public void Dispose()
        {
            job.Dispose();
            if (process != IntPtr.Zero)
            {
                CloseHandle(process);
                process = IntPtr.Zero;
            }
        }
    }

    public sealed class GpuProbe : IDisposable
    {
        private readonly Process process;
        private readonly Stopwatch clock;
        private readonly Task<string> output;
        private readonly Task<string> error;
        private readonly Timer timeout;
        private readonly object cleanupLock = new object();
        private bool disposed;

        public GpuProbe(string executable)
        {
            process = new Process();
            process.StartInfo.FileName = executable;
            process.StartInfo.Arguments = "--query-gpu=temperature.gpu,memory.used,memory.total,utilization.gpu --format=csv,noheader,nounits";
            process.StartInfo.UseShellExecute = false;
            process.StartInfo.CreateNoWindow = true;
            process.StartInfo.RedirectStandardOutput = true;
            process.StartInfo.RedirectStandardError = true;
            clock = Stopwatch.StartNew();
            try
            {
                process.Start();
                output = process.StandardOutput.ReadToEndAsync();
                error = process.StandardError.ReadToEndAsync();
                int remaining = (int)Math.Max(0, 2000 - clock.ElapsedMilliseconds);
                timeout = new Timer(delegate { KillIfRunning(); }, null, remaining, Timeout.Infinite);
            }
            catch
            {
                Dispose();
                throw;
            }
        }

        public string Result(bool expired)
        {
            if (!expired && process.ExitCode == 0 && output.Status == TaskStatus.RanToCompletion)
                return output.Result;
            return null;
        }

        public bool Finished { get { return process.HasExited && output.IsCompleted && error.IsCompleted; } }
        public bool Expired { get { return clock.ElapsedMilliseconds >= 2000; } }

        private void KillIfRunning()
        {
            lock (cleanupLock)
            {
                if (disposed) return;
                try
                {
                    if (!process.HasExited) process.Kill();
                }
                catch (InvalidOperationException) { }
                catch (Win32Exception) { }
            }
        }

        public void Dispose()
        {
            if (timeout != null) timeout.Dispose();
            lock (cleanupLock)
            {
                KillIfRunning();
                disposed = true;
                process.Dispose();
            }
        }
    }
}
'@
}

$invariant = [System.Globalization.CultureInfo]::InvariantCulture
$utf8Bom = New-Object System.Text.UTF8Encoding($true)
$gpuCommand = Get-Command nvidia-smi.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1

function Format-Measurement($Value) {
    if ($null -ne $Value) { return ([double]$Value).ToString('0.000', $invariant) }
    return ''
}

function Format-SummaryValue($Value, [string]$Unit) {
    if ($null -eq $Value) { return '-' }
    return ('{0} {1}' -f (Format-Measurement $Value), $Unit)
}

function Read-GpuFields([string]$OutputText) {
    foreach ($line in ($OutputText -split '\r?\n')) {
        $fields = $line -split ','
        if ($fields.Count -ne 4) { continue }

        $values = foreach ($field in $fields) {
            $number = 0.0
            if ([double]::TryParse($field.Trim(), [System.Globalization.NumberStyles]::Float, $invariant, [ref]$number) -and
                -not [double]::IsNaN($number) -and -not [double]::IsInfinity($number)) { $number }
            else { $null }
        }

        return [pscustomobject]@{ Temp = $values[0]; Used = $values[1]; Total = $values[2]; Util = $values[3] }
    }
}

function Invoke-GuardedRun {
    param(
        [string]$Executable, [string]$Arguments, [string]$Directory, [string]$Logs,
        [double]$CommitGB, [double]$FreeRamGB, [double]$FreeCommitGB, [double]$GpuTempC,
        [double]$Seconds, [double]$Sustain, [string]$RunPriority, [switch]$MemoryTest, [string]$OutputFile, [string]$ErrorFile
    )

    $null = [System.IO.Directory]::CreateDirectory($Logs)
    $stopFile = Join-Path $Logs 'DURDUR'
    $csvPath = Join-Path $Logs 'olcum.csv'
    $summaryPath = Join-Path $Logs 'ozet.txt'
    $csvFile = [System.IO.File]::Open($csvPath, 'CreateNew', 'Write', 'Read')
    $writer = $null
    $job = $null
    $gpu = $null
    $startedProcess = $false
    $startTime = [DateTimeOffset]::Now
    $reason = 'Ctrl+C veya oturum kapatıldı'
    $exitCode = $null
    $peakPrivate = $null
    $lowRam = $null
    $peakGpuTemp = $null
    $peakGpuVram = $null

    try {
        $writer = New-Object System.IO.StreamWriter($csvFile, $utf8Bom)
        $writer.AutoFlush = $true
        $writer.WriteLine('Zaman,BosRAM_GB,BosCommit_GB,PrivateBytes_GB,WorkingSet_GB,CPU_Yuzde,GPU_Sicaklik_C,VRAM_Kullanilan_MiB,VRAM_Toplam_MiB,GPU_Yuzde')

        $priorityClasses = @{ Idle = 0x40; BelowNormal = 0x4000; Normal = 0x20; AboveNormal = 0x8000; High = 0x80 }
        Write-Host "Tarayıcıyı ve ağır programları kapatın, bilgisayarı şarja takın. Durdurmak için Ctrl+C veya $stopFile dosyasını kullanın."

        $job = New-Object GuvenliBaslat.NativeJob([uint64]($CommitGB * 1GB), [uint32]$priorityClasses[$RunPriority])
        $job.Start($Executable, $Arguments, $Directory, $OutputFile, $ErrorFile)
        $startedProcess = $true
        $clock = [System.Diagnostics.Stopwatch]::StartNew()
        $nextSample = 0.0
        $nextGpu = 0.0
        $ramSince = $null
        $commitSince = $null
        $gpuSince = $null
        $gpuFields = $null
        $previousCpu = 0
        $previousTime = 0.0

        while ($true) {
            $elapsed = $clock.Elapsed.TotalSeconds
            if ($job.ActiveProcesses() -eq 0) {
                $exitCode = $job.ExitCode(1000)
                $reason = 'kendi kapandı'
                if ($MemoryTest -and $exitCode -eq 42) {
                    $reason = 'Job süreç bellek sınırında bellek ayırma başarısız oldu'
                }
                break
            }
            if ([System.IO.File]::Exists($stopFile)) {
                $reason = 'DURDUR dosyası oluşturuldu'
                break
            }
            if ($elapsed -ge $Seconds) {
                $reason = 'Azami çalışma süresi doldu'
                break
            }

            if ($null -ne $gpu -and ($gpu.Finished -or $gpu.Expired)) {
                $gpuFields = Read-GpuFields ($gpu.Result($gpu.Expired))
                $gpu.Dispose()
                $gpu = $null
                if ($null -eq $gpuFields.Temp -or $gpuFields.Temp -le $GpuTempC) {
                    $gpuSince = $null
                }
                elseif ($null -eq $gpuSince) {
                    $gpuSince = $elapsed
                }
                if ($null -ne $gpuFields.Temp -and ($null -eq $peakGpuTemp -or $gpuFields.Temp -gt $peakGpuTemp)) {
                    $peakGpuTemp = $gpuFields.Temp
                }
                if ($null -ne $gpuFields.Used -and ($null -eq $peakGpuVram -or $gpuFields.Used -gt $peakGpuVram)) {
                    $peakGpuVram = $gpuFields.Used
                }
                if ($null -ne $gpuSince -and ($elapsed - $gpuSince) -ge $Sustain) {
                    $reason = 'GPU sıcaklığı eşik üzerinde kaldı'
                    break
                }
            }

            if ($elapsed -ge $nextSample) {
                $memory = [GuvenliBaslat.NativeJob]::SystemMemory()
                $sample = $job.Sample()
                $ram = $memory.AvailablePhysical / 1GB
                $commit = $memory.AvailableCommit / 1GB
                $privateGB = $null
                $workingSet = $null
                $cpu = $null
                if ($null -ne $sample) {
                    $privateGB = $sample.PrivateBytes / 1GB
                    $workingSet = $sample.WorkingSetBytes / 1GB
                    if ($previousTime -gt 0 -and $elapsed -gt $previousTime) {
                        $cpuSeconds = ([double]$sample.CpuTicks - $previousCpu) / 10000000
                        $cpu = 100 * $cpuSeconds / (($elapsed - $previousTime) * [Environment]::ProcessorCount)
                        $cpu = [Math]::Min(100, $cpu)
                    }
                    $previousCpu = $sample.CpuTicks
                    $previousTime = $elapsed
                    if ($null -eq $peakPrivate -or $privateGB -gt $peakPrivate) { $peakPrivate = $privateGB }
                }
                if ($null -eq $lowRam -or $ram -lt $lowRam) { $lowRam = $ram }

                $measurements = @($ram, $commit, $privateGB, $workingSet, $cpu,
                    $gpuFields.Temp, $gpuFields.Used, $gpuFields.Total, $gpuFields.Util)
                $row = foreach ($value in $measurements) { Format-Measurement $value }
                $writer.WriteLine(([DateTimeOffset]::Now.ToString('o', $invariant) + ',' + ($row -join ',')))
                $gpuFields = $null
                $nextSample = [Math]::Floor($elapsed) + 1

                if ($ram -ge $FreeRamGB) { $ramSince = $null }
                elseif ($null -eq $ramSince) { $ramSince = $elapsed }
                if ($commit -ge $FreeCommitGB) { $commitSince = $null }
                elseif ($null -eq $commitSince) { $commitSince = $elapsed }

                if ($null -ne $ramSince -and ($elapsed - $ramSince) -ge $Sustain) {
                    $reason = 'Boş fiziksel RAM eşik altında kaldı'
                    break
                }
                if ($null -ne $commitSince -and ($elapsed - $commitSince) -ge $Sustain) {
                    $reason = 'Boş sistem commit alanı eşik altında kaldı'
                    break
                }
            }

            if ($null -ne $gpuCommand -and $null -eq $gpu -and $elapsed -ge $nextGpu) {
                try { $gpu = New-Object GuvenliBaslat.GpuProbe($gpuCommand.Source) }
                catch { $gpuSince = $null }
                $nextGpu = [Math]::Floor($elapsed) + 1
            }
            Start-Sleep -Milliseconds 50
        }

        if ($null -eq $exitCode) {
            $job.Stop()
            $exitCode = $job.ExitCode(3000)
        }
    }
    catch {
        $reason = 'Başlatıcı hatası - ' + $_.Exception.GetBaseException().Message
        throw
    }
    finally {
        # Close the job first on cancellation or on any exception.
        if ($null -ne $job) { $job.Dispose() }
        if ($null -ne $gpu) { $gpu.Dispose() }
        if ($null -ne $writer) { $writer.Dispose() }
        else { $csvFile.Dispose() }

        $endTime = [DateTimeOffset]::Now
        $exitText = '-'
        if ($null -ne $exitCode) { $exitText = $exitCode }
        elseif ($startedProcess) { $exitText = 'alınamadı' }

        [System.IO.File]::WriteAllLines($summaryPath, [string[]]@(
            ('Başlangıç: {0:o}' -f $startTime)
            ('Bitiş: {0:o}' -f $endTime)
            "Çıkış kodu: $exitText"
            "Durdurma sebebi: $reason"
            ('En yüksek private bytes: ' + (Format-SummaryValue $peakPrivate 'GB'))
            ('En düşük boş RAM: ' + (Format-SummaryValue $lowRam 'GB'))
            ('En yüksek GPU sıcaklığı: ' + (Format-SummaryValue $peakGpuTemp '°C'))
            ('En yüksek VRAM kullanımı: ' + (Format-SummaryValue $peakGpuVram 'MiB'))
        ), $utf8Bom)
        Write-Host "$reason | Çıkış kodu $exitText | $summaryPath"
    }

    [pscustomobject]@{ Reason = $reason; ExitCode = $exitCode }
}

if ($SelfTest) {
    $testRoot = Join-Path 'C:\CLLL\.calisma\repo_durum\guvenli_test' (Get-Date -Format 'yyyyMMdd_HHmmss_fff')
    $testExe = Join-Path $PSHOME 'powershell.exe'
    $testSettings = @{
        Executable = $testExe
        Directory = $PSHOME
        CommitGB = 1
        FreeRamGB = $MinFreeRamGB
        FreeCommitGB = $MinFreeCommitGB
        GpuTempC = $MaxGpuTempC
        Sustain = $SustainSeconds
        RunPriority = $Priority
    }
    $allocationTest = @'
$ErrorActionPreference = 'Stop'
$held = New-Object 'System.Collections.Generic.List[byte[]]'
for ($i = 0; $i -lt 16; $i++) {
    try {
        $held.Add([byte[]]::new(100MB))
    }
    catch {
        $cause = $_.Exception.GetBaseException()
        if ($cause -is [OutOfMemoryException] -or
            ($cause -is [System.ComponentModel.Win32Exception] -and $cause.NativeErrorCode -in 8, 14, 1455)) {
            exit 42
        }
        exit 43
    }
    Start-Sleep -Milliseconds 300
}
exit 44
'@
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($allocationTest))
    $memoryTestResult = Invoke-GuardedRun @testSettings -Seconds 25 -MemoryTest -Logs (Join-Path $testRoot 'bellek') -Arguments "-NoLogo -NoProfile -NonInteractive -EncodedCommand $encoded"
    # The timeout case isolates the timer from memory pressure left by the allocation test.
    $timeoutSettings = $testSettings.Clone()
    $timeoutSettings.FreeRamGB = 0
    $timeoutResult = Invoke-GuardedRun @timeoutSettings -Seconds 8 -Logs (Join-Path $testRoot 'sure') -Arguments '-NoLogo -NoProfile -NonInteractive -Command "Start-Sleep -Seconds 600"'
    $streamLogs = Join-Path $testRoot 'akislari'
    $stdout = Join-Path $streamLogs 'out.log'
    $stderr = Join-Path $streamLogs 'run.log'
    $streamTest = "[Console]::Out.WriteLine('stdout-selftest'); [Console]::Error.WriteLine('stderr-selftest')"
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($streamTest))
    $streamResult = Invoke-GuardedRun @testSettings -Seconds 15 -Logs $streamLogs -OutputFile $stdout -ErrorFile $stderr -Arguments "-NoLogo -NoProfile -NonInteractive -EncodedCommand $encoded"
    if ($memoryTestResult.ExitCode -ne 42 -or $timeoutResult.ExitCode -ne 70 -or
        $streamResult.ExitCode -ne 0 -or
        [IO.File]::ReadAllText($stdout).Trim() -cne 'stdout-selftest' -or
        [IO.File]::ReadAllText($stderr).Trim() -cne 'stderr-selftest' -or
        $timeoutResult.Reason -ne 'Azami çalışma süresi doldu') {
        throw "SelfTest başarısız. Günlükler $testRoot"
    }
    Write-Host 'SelfTest 3/3 başarılı.'
    return
}

if ($Exe -notmatch '^[A-Za-z]:[\\/]') {
    throw '-Exe için yerel, mutlak bir dosya yolu girin.'
}
$exePath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Exe)
if ($WorkDir) {
    $WorkDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($WorkDir)
}
$resolvedLogs = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($LogDir)
if ($StdOutFile) { $StdOutFile = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($StdOutFile) }
if ($StdErrFile) { $StdErrFile = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($StdErrFile) }
$reserved = @($exePath, (Join-Path $resolvedLogs 'olcum.csv'), (Join-Path $resolvedLogs 'ozet.txt'), (Join-Path $resolvedLogs 'DURDUR'))
if (($StdOutFile -and $StdOutFile -in $reserved) -or
    ($StdErrFile -and $StdErrFile -in $reserved) -or
    ($StdOutFile -and $StdOutFile -eq $StdErrFile)) {
    throw 'StdOutFile ve StdErrFile ayrı dosyaları göstermeli ve başlatıcının dosyalarıyla çakışmamalı.'
}
foreach ($localPath in @($exePath, $WorkDir, $resolvedLogs, $StdOutFile, $StdErrFile)) {
    if (-not $localPath) { continue }
    if ($localPath.StartsWith('\\') -or [System.IO.DriveInfo]::new([IO.Path]::GetPathRoot($localPath)).DriveType -eq 'Network') {
        throw 'Yürütülebilir dosya, çalışma klasörü ve günlükler yerel sürücüde olmalı.'
    }
}
$exeFile = Get-Item -LiteralPath $exePath
if ($exeFile -isnot [System.IO.FileInfo] -or $exeFile.Extension -ine '.exe') {
    throw '-Exe mevcut bir .exe dosyasını göstermeli.'
}
if (-not $WorkDir) { $WorkDir = $exeFile.DirectoryName }
$directoryItem = Get-Item -LiteralPath $WorkDir
if ($directoryItem -isnot [System.IO.DirectoryInfo]) {
    throw '-WorkDir mevcut bir klasörü göstermeli.'
}
$null = Invoke-GuardedRun -Executable $exeFile.FullName -Arguments $ArgumentList -Directory $directoryItem.FullName -Logs $resolvedLogs -CommitGB $MaxCommitGB -FreeRamGB $MinFreeRamGB -FreeCommitGB $MinFreeCommitGB -GpuTempC $MaxGpuTempC -Seconds ($MaxMinutes * 60) -Sustain $SustainSeconds -RunPriority $Priority -OutputFile $StdOutFile -ErrorFile $StdErrFile
