Set-StrictMode -Version Latest

if ($IsWindows -and $null -eq ('LiveGalaxy.BoundedFileNative' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace LiveGalaxy {
    public static class BoundedFileNative {
        [StructLayout(LayoutKind.Sequential)]
        private struct FileTime {
            public uint Low;
            public uint High;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct ByHandleFileInformation {
            public uint Attributes;
            public FileTime CreationTime;
            public FileTime LastAccessTime;
            public FileTime LastWriteTime;
            public uint VolumeSerialNumber;
            public uint FileSizeHigh;
            public uint FileSizeLow;
            public uint NumberOfLinks;
            public uint FileIndexHigh;
            public uint FileIndexLow;
        }

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern uint GetFinalPathNameByHandle(
            SafeFileHandle handle, StringBuilder path, uint length, uint flags);

        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern bool GetFileInformationByHandle(
            SafeFileHandle handle, out ByHandleFileInformation information);

        public static string GetIdentity(SafeFileHandle handle) {
            ByHandleFileInformation information;
            if (!GetFileInformationByHandle(handle, out information)) {
                throw new IOException("GetFileInformationByHandle failed",
                    new Win32Exception(Marshal.GetLastWin32Error()));
            }
            ulong index = ((ulong)information.FileIndexHigh << 32) |
                information.FileIndexLow;
            return information.VolumeSerialNumber.ToString("x8") + ":" +
                index.ToString("x16");
        }

        public static string GetFinalPath(SafeFileHandle handle) {
            var path = new StringBuilder(32768);
            uint written = GetFinalPathNameByHandle(
                handle, path, (uint)path.Capacity, 0);
            if (written == 0 || written >= path.Capacity) {
                throw new IOException("GetFinalPathNameByHandle failed",
                    new Win32Exception(Marshal.GetLastWin32Error()));
            }
            return path.ToString();
        }
    }
}
'@
}

function Assert-BoundedFilePath(
    [string]$Path,
    [string]$FailureCode,
    [string]$ReparseCode
) {
    $full = [IO.Path]::GetFullPath($Path)
    $root = [IO.Path]::GetPathRoot($full)
    $current = $root
    foreach ($segment in @($full.Substring($root.Length) -split '[\\/]+')) {
        if ([string]::IsNullOrEmpty($segment)) { continue }
        $current = Join-Path $current $segment
        $item = Get-Item -LiteralPath $current -Force -ErrorAction SilentlyContinue
        if ($null -eq $item) { throw [IO.FileNotFoundException]::new($FailureCode) }
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or
            $null -ne $item.LinkType) {
            throw [IO.IOException]::new($ReparseCode)
        }
    }
    return $full
}

function ConvertTo-NormalizedHandlePath([string]$Path) {
    if ($Path.StartsWith('\\?\UNC\', [StringComparison]::OrdinalIgnoreCase)) {
        return '\\' + $Path.Substring(8)
    }
    if ($Path.StartsWith('\\?\', [StringComparison]::OrdinalIgnoreCase)) {
        return $Path.Substring(4)
    }
    return $Path
}

function Get-OpenedFileTarget([IO.FileStream]$Stream) {
    if ($IsWindows) {
        return ConvertTo-NormalizedHandlePath `
            ([LiveGalaxy.BoundedFileNative]::GetFinalPath($Stream.SafeFileHandle))
    }
    $descriptorPath = "/proc/self/fd/$($Stream.SafeFileHandle.DangerousGetHandle().ToInt64())"
    if (Test-Path -LiteralPath $descriptorPath) {
        $target = [IO.File]::ResolveLinkTarget($descriptorPath, $true)
        if ($null -ne $target) { return $target.FullName }
    }
    return [IO.Path]::GetFullPath($Stream.Name)
}

function Get-OpenedFileIdentity([IO.FileStream]$Stream, [string]$ResolvedTarget) {
    if ($IsWindows) {
        return [LiveGalaxy.BoundedFileNative]::GetIdentity($Stream.SafeFileHandle)
    }
    return $ResolvedTarget
}

function Read-BoundedFile(
    [string]$Path,
    [long]$MaximumBytes,
    [string]$FailureCode = 'FILE_READ_REJECTED',
    [string]$BoundCode = 'FILE_BYTES_EXCEEDED',
    [string]$IdentityCode = 'PATH_IDENTITY_CHANGED',
    [string]$ReparseCode = $FailureCode,
    [scriptblock]$BeforeReadTestHook,
    [scriptblock]$AfterReadTestHook
) {
    if ($MaximumBytes -lt 0) { throw [ArgumentOutOfRangeException]::new('MaximumBytes') }
    $full = Assert-BoundedFilePath $Path $FailureCode $ReparseCode
    try {
        $stream = [IO.FileStream]::new(
            $full, [IO.FileMode]::Open, [IO.FileAccess]::Read,
            [IO.FileShare]::Read, 65536, [IO.FileOptions]::SequentialScan
        )
    }
    catch { throw [IO.IOException]::new($FailureCode, $_.Exception) }
    try {
        $resolvedTarget = Get-OpenedFileTarget $stream
        if (-not [IO.Path]::GetFullPath($resolvedTarget).Equals(
                $full, [StringComparison]::OrdinalIgnoreCase)) {
            throw [IO.IOException]::new($IdentityCode)
        }
        $identity = Get-OpenedFileIdentity $stream $resolvedTarget
        $length = $stream.Length
        if ($length -gt $MaximumBytes -or $length -gt [int]::MaxValue) {
            throw [IO.InvalidDataException]::new($BoundCode)
        }
        if ($null -ne $BeforeReadTestHook) {
            & $BeforeReadTestHook ([pscustomobject]@{ Path = $full; Phase = 'before-read' })
        }
        [byte[]]$bytes = [byte[]]::new([int]$length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $read = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($read -le 0) { throw [IO.IOException]::new($IdentityCode) }
            $offset += $read
        }
        if ($null -ne $AfterReadTestHook) {
            & $AfterReadTestHook ([pscustomobject]@{ Path = $full; Phase = 'after-read' })
        }
        $afterTarget = Get-OpenedFileTarget $stream
        $afterIdentity = Get-OpenedFileIdentity $stream $afterTarget
        if ($stream.Length -ne $length -or $offset -ne $length -or
            -not $afterTarget.Equals($resolvedTarget, [StringComparison]::OrdinalIgnoreCase) -or
            $afterIdentity -cne $identity) {
            throw [IO.IOException]::new($IdentityCode)
        }
        return [pscustomobject]@{
            Bytes = $bytes
            FullPath = $full
            ResolvedTarget = $resolvedTarget
            Identity = $identity
            Length = $length
        }
    }
    finally {
        $stream.Dispose()
    }
}

Export-ModuleMember -Function Read-BoundedFile
