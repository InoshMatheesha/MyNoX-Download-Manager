# Builds the Windows release into target/dist/:
#   MyNoX-<version>-setup.exe     the x64 installer (tools/installer/mynox.iss)
#   MyNoX-<version>-portable.exe  the portable copy: unpacks with a `data` folder, no registry
#   SHA256SUMS.txt
# Needs Inno Setup 6 (ISCC.exe on PATH, in $env:ISCC, or in its default folders).
# Code signing: set $env:MYNOX_SIGN_COMMAND to a signtool command line with $f for the file, e.g.
#   signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 /a $f
# Inno Setup then signs the programs, the installer, its uninstaller and the portable .exe.
param([switch]$NoBuild, [string]$SignCommand = $env:MYNOX_SIGN_COMMAND)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$dist = Join-Path $root "target\dist"
$bin = Join-Path $root "target\release"

$cargo = Get-Content (Join-Path $root "Cargo.toml") -Raw
$version = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
if (-not $version) { throw "no version in Cargo.toml" }

if (-not $NoBuild) {
  # The build machine's paths out of the programs (#25): panic locations and debug info say
  # "cargo\..." for cargo's folder and "mynox\..." for this checkout (shorter, and no user name).
  # Encoded flags, as those paths may hold spaces.
  $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE ".cargo" }
  $env:CARGO_ENCODED_RUSTFLAGS = "--remap-path-prefix=$cargoHome=cargo" + [char]0x1f + "--remap-path-prefix=$root=mynox"
  # GNU toolchains embed debug info in the exe (MSVC keeps it in the .pdb): strip it there.
  if ((& rustc -vV | Out-String) -match "host: .*-gnu") { $env:CARGO_PROFILE_RELEASE_STRIP = "debuginfo" }
  try {
    & cargo build --release -p mynox-app -p mynox-nmhost -p mynox-cli -p mynox-mcp
    if ($LASTEXITCODE) { throw "cargo build failed" }
  } finally {
    Remove-Item Env:CARGO_ENCODED_RUSTFLAGS
    Remove-Item Env:CARGO_PROFILE_RELEASE_STRIP -ErrorAction SilentlyContinue
  }
}
& (Join-Path $root "tools\extension\pack.ps1") | Out-Null

# The installer's languages (tools/installer/languages.py writes them; committed, so no key needed).
if (-not (Test-Path (Join-Path $PSScriptRoot "languages.iss"))) { throw "run python tools/installer/languages.py first" }

$iscc = @($env:ISCC, (Get-Command ISCC.exe -ErrorAction SilentlyContinue).Source,
  "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
  "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") |
  Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) { throw "Inno Setup 6 (ISCC.exe) not found" }

New-Item -ItemType Directory -Force $dist | Out-Null
$isccArgs = @("/Qp", "/DAppVersion=$version")
if ($SignCommand) { $isccArgs += @("/DSign", "/Smynoxsign=$SignCommand") }
& $iscc @isccArgs (Join-Path $PSScriptRoot "mynox.iss")
if ($LASTEXITCODE) { throw "ISCC failed" }

# The portable .exe: the same programs unpacked with a `data` folder (tools/installer/mynox-portable.iss).
& $iscc @isccArgs (Join-Path $PSScriptRoot "mynox-portable.iss")
if ($LASTEXITCODE) { throw "ISCC (portable) failed" }

$files = Get-ChildItem $dist -File | Where-Object { $_.Name -like "MyNoX-$version-*.exe" }
$files | ForEach-Object { "{0}  {1}" -f (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower(), $_.Name } |
  Set-Content -Encoding ascii (Join-Path $dist "SHA256SUMS.txt")
$files | ForEach-Object { "{0}  {1:N1} MB" -f $_.Name, ($_.Length / 1MB) }
