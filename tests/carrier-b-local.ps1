[CmdletBinding()]
param(
    [switch]$SelfTest,
    [ValidateSet('actual-chain', 'pending-io-unload')]
    [string]$Scenario = 'actual-chain',
    [string]$LuaJitPath,
    [string]$LuaLibraryPath,
    [string]$LimitsFile
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $SelfTest) { throw 'SELF_TEST_REQUIRED' }
throw 'ACTUAL_DLL_DURABLE_READBACK_NOT_IMPLEMENTED'
