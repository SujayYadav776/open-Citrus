# Registers Citrus in the Explorer context menu (HKCU only: no admin, reversible).
# The app already reads its selected files from argv, so each verb just passes the path,
# mirroring the old Nautilus script handoff.
# Usage:
#   powershell -File register-explorer.ps1 [-Remove]
#   CITRUS_BIN overrides the exe path (defaults to the debug build in this working copy).
param([switch]$Remove)

$exe = if ($env:CITRUS_BIN) { $env:CITRUS_BIN } else { "E:\QwenWork\Mandarina\src-tauri\target\debug\mandarina.exe" }
$verb = "Citrus.Convert"

$locations = @(
    # one selected file (and, per Windows behavior, each file of a multi-selection
    # opens its own invocation - documented difference vs Nautilus)
    @{ Path = "HKCU:\Software\Classes\*"; Key = $verb; Arg = "`"%1`"" },
    # a folder, clicked on its icon
    @{ Path = "HKCU:\Software\Classes\Directory"; Key = $verb; Arg = "`"%1`"" },
    # the background inside a folder: converts the folder itself
    @{ Path = "HKCU:\Software\Classes\Directory\background"; Key = $verb; Arg = "`"%V`"" }
)

foreach ($loc in $locations) {
    $root = Join-Path $loc.Path $loc.Key
    if ($Remove) {
        if (Test-Path $root) { Remove-Item -Recurse $root; Write-Output "removed $root" }
        continue
    }
    New-Item -Force -Path $root | Out-Null
    New-ItemProperty -Force -Path $root -Name MUIVerb -Value "Convert with Citrus" | Out-Null
    New-ItemProperty -Force -Path $root -Name Icon -Value "`"$exe`",0" | Out-Null
    New-Item -Force -Path (Join-Path $root "command") | Out-Null
    # %1 expands to the clicked item; %V to the background folder
    New-ItemProperty -Force -Path (Join-Path $root "command") -Name "(Default)" -Value "`"$exe`" $($loc.Arg)" | Out-Null
    Write-Output "registered $root"
}
if (-not $Remove) {
    Write-Output "bin: $exe"
    Write-Output "Tip: restart Explorer (or sign out/in) if the menu does not refresh."
}
