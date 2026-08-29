# X4 Extension Presets

Use this runbook to preserve the current X4 extension selection, temporarily
enable a minimal Live Galaxy test preset, and restore the exact original
selection afterwards. Do not use it to inspect or modify save files.

## Boundary

- Source: X4 profile `content.xml` below `%USERPROFILE%\Documents\Egosoft\X4`.
- Version/schema: verified against the local X4 9.00 profile schema on 2026-08-29.
- Test preset: `live_galaxy` and SirNukes Mod Support APIs (`ws_2042901274`).
- X4 must be fully closed for capture, apply, or restore.
- The test preset temporarily sets the root `content` synchronization flag to
  `false` so Workshop synchronization cannot re-enable subscribed mods.

The script retains the baseline and a SHA-256 manifest under
`%LOCALAPPDATA%\LiveGalaxy\x4-extension-presets`. It refuses to overwrite a
baseline unless `-Force` is supplied.

## Procedure

From this directory, capture the current preset once:

```powershell
.\x4-extension-preset.ps1 -Action Capture
```

Apply the minimal test preset:

```powershell
.\x4-extension-preset.ps1 -Action ApplyTest
```

Check the active preset and retained baseline without changing anything:

```powershell
.\x4-extension-preset.ps1 -Action Status
```

After testing, restore the exact original `content.xml`:

```powershell
.\x4-extension-preset.ps1 -Action Restore
```

## Validation and Recovery

`Capture` verifies the retained file's SHA-256 digest after writing its
manifest. `ApplyTest` refuses an unverified baseline and verifies that Workshop
synchronization is off and no extensions outside the requested IDs remain
enabled. It also registers every detected non-DLC extension as disabled, so
local extensions whose IDs were absent from the profile cannot default to on.
`Restore` verifies both the baseline and the restored active configuration
against the same digest.

If more than one numeric X4 profile directory exists, pass the intended file
explicitly:

```powershell
.\x4-extension-preset.ps1 -Action Status -ConfigPath 'C:\path\to\content.xml'
```

If X4 is running, close it and rerun the command. If the manifest digest check
fails, do not restore; recapture only after confirming the intended baseline.
