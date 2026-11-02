[CmdletBinding()]
param(
    [switch]$SkipCargoBuild,
    [string]$NsisCompiler = ''
)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$out = Join-Path $root 'dist/unsigned'
if (Test-Path -LiteralPath $out) { throw "Refusing to overwrite an existing build: $out" }
if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) { throw 'Run in the x64 Visual Studio developer environment.' }
if (-not (Get-Command rc.exe -ErrorAction SilentlyContinue)) { throw 'Windows SDK resource compiler is required.' }
if (-not $NsisCompiler) {
    $nsis = Get-Command makensis.exe -ErrorAction SilentlyContinue
    if ($nsis) { $NsisCompiler = $nsis.Source }
    else { $NsisCompiler = Join-Path ${env:ProgramFiles(x86)} 'NSIS/makensis.exe' }
}
if (-not (Test-Path -LiteralPath $NsisCompiler)) { throw 'Install NSIS 3.12 or pass -NsisCompiler.' }

Push-Location $root
try {
    if (-not $SkipCargoBuild) {
        & cargo --config 'profile.release.package.oneclient_app.codegen-units=16' rustc -p oneclient_app --bin oneclient_app --release --locked --target x86_64-pc-windows-msvc -- -C lto=thin
        if ($LASTEXITCODE -ne 0) { throw 'Cargo release build failed.' }
    }
    $app = Join-Path $root 'target/x86_64-pc-windows-msvc/release/oneclient_app.exe'
    if (-not (Test-Path -LiteralPath $app)) { throw 'Built launcher executable is missing.' }
    $bundle = Join-Path $out 'OneClient-Offline-x64'
    New-Item -ItemType Directory -Path $bundle | Out-Null
    Copy-Item -LiteralPath $app -Destination (Join-Path $bundle 'oneclient_app.exe')

    Push-Location $PSScriptRoot
    try {
        $res = Join-Path $out 'portable-launcher.res'
        & rc.exe /nologo "/fo$res" portable-launcher.rc
        if ($LASTEXITCODE -ne 0) { throw 'Portable launcher resource compilation failed.' }
        & cl.exe /nologo /O2 /MT /DUNICODE /D_UNICODE portable-launcher.cpp $res "/Fo$(Join-Path $out 'portable-launcher.obj')" "/Fe$(Join-Path $bundle 'Start OneClient.exe')" /link user32.lib /SUBSYSTEM:WINDOWS /MANIFEST:EMBED /MANIFESTUAC:level=asInvoker
        if ($LASTEXITCODE -ne 0) { throw 'Portable launcher compilation failed.' }
    } finally { Pop-Location }

    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $runtimeFiles = @(& $vswhere -latest -products '*' -find 'VC\Redist\MSVC\**\x64\Microsoft.VC*.CRT\*.dll')
    if ($LASTEXITCODE -ne 0 -or -not $runtimeFiles.Count) { throw 'No app-local x64 VC++ runtime was found.' }
    $runtimeDir = $runtimeFiles | ForEach-Object { [IO.Path]::GetDirectoryName($_) } | Sort-Object -Unique | Select-Object -Last 1
    Get-ChildItem -LiteralPath $runtimeDir -Filter '*.dll' -File | ForEach-Object {
        $signature = Get-AuthenticodeSignature -LiteralPath $_.FullName
        if ($signature.Status -ne 'Valid') { throw "Runtime signature is not valid: $($_.Name) ($($signature.Status))" }
        Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $bundle $_.Name)
    }
    foreach ($name in @('LICENSE','ATTRIBUTION.md')) { Copy-Item -LiteralPath (Join-Path $root $name) -Destination $bundle }
    Copy-Item -LiteralPath (Join-Path $root 'OFFLINE_MODE.md') -Destination (Join-Path $bundle 'OFFLINE-MODE.md')
    'OneClient Offline 2.9.0, community revision 4. UNSIGNED review build.' | Set-Content -LiteralPath (Join-Path $bundle 'oneclient-offline-build.txt') -Encoding utf8
    'Open Start OneClient.exe. This unsigned review build is not a signed release. Keep Windows protection enabled.' | Set-Content -LiteralPath (Join-Path $bundle 'START-HERE.txt') -Encoding utf8
    "Unsigned source build. GitHub commit: $env:GITHUB_SHA`nNo test profiles, account credentials, skins or game data are included." | Set-Content -LiteralPath (Join-Path $bundle 'BUILD-INFO.txt') -Encoding utf8
    if (Test-Path -LiteralPath (Join-Path $bundle 'data')) { throw 'A release payload must not contain a profile.' }

    $setup = Join-Path $out 'OneClient-Offline-2.9.0.4-x64-setup-UNSIGNED.exe'
    & $NsisCompiler /V3 "/DPAYLOAD=$bundle" "/DOUTPUT=$setup" "/DICON=$(Join-Path $root 'packages/oneclient_app/icons/icon.ico')" (Join-Path $PSScriptRoot 'offline-installer.nsi')
    if ($LASTEXITCODE -ne 0) { throw 'NSIS packaging failed.' }
    $zip = Join-Path $out 'OneClient-Offline-Windows-x64-UNSIGNED.zip'
    Compress-Archive -LiteralPath $bundle -DestinationPath $zip -CompressionLevel Optimal
    @($setup, $zip, (Join-Path $bundle 'oneclient_app.exe'), (Join-Path $bundle 'Start OneClient.exe')) | ForEach-Object {
        $hash = Get-FileHash -LiteralPath $_ -Algorithm SHA256
        "$($hash.Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($_))"
    } | Set-Content -LiteralPath (Join-Path $out 'SHA256SUMS.txt') -Encoding utf8
    Write-Host 'Built UNSIGNED review artifacts only. Production signing requires project approval and trusted-signature verification.'
} finally { Pop-Location }
