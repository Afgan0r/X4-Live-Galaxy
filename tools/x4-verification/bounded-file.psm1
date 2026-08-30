Set-StrictMode -Version Latest

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

function Read-BoundedFile(
    [string]$Path,
    [long]$MaximumBytes,
    [string]$FailureCode = 'FILE_READ_REJECTED',
    [string]$BoundCode = 'FILE_BYTES_EXCEEDED',
    [string]$IdentityCode = 'PATH_IDENTITY_CHANGED',
    [string]$ReparseCode = $FailureCode
) {
    if ($MaximumBytes -lt 0) { throw [ArgumentOutOfRangeException]::new('MaximumBytes') }
    $full = Assert-BoundedFilePath $Path $FailureCode $ReparseCode
    $before = Get-Item -LiteralPath $full -Force
    if ($before.PSIsContainer -or $before.Length -gt $MaximumBytes) {
        $code = if ($before.Length -gt $MaximumBytes) { $BoundCode } else { $FailureCode }
        throw [IO.InvalidDataException]::new($code)
    }
    [byte[]]$bytes = [IO.File]::ReadAllBytes($before.FullName)
    $afterFull = Assert-BoundedFilePath $Path $FailureCode $ReparseCode
    $after = Get-Item -LiteralPath $afterFull -Force
    if (-not $after.FullName.Equals($before.FullName, [StringComparison]::OrdinalIgnoreCase) -or
        $after.Length -ne $before.Length -or
        $after.LastWriteTimeUtc -ne $before.LastWriteTimeUtc -or
        (($after.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne
            ($before.Attributes -band [IO.FileAttributes]::ReparsePoint)) -or
        $after.LinkType -ne $before.LinkType -or $bytes.Length -ne $before.Length) {
        throw [IO.IOException]::new($IdentityCode)
    }
    if ($bytes.Length -gt $MaximumBytes) { throw [IO.InvalidDataException]::new($BoundCode) }
    return [pscustomobject]@{
        Bytes = $bytes
        FullPath = $before.FullName
        Length = $before.Length
        LastWriteTimeUtc = $before.LastWriteTimeUtc
    }
}

Export-ModuleMember -Function Read-BoundedFile
