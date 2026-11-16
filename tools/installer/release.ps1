# release.ps1 - Builds, signs and publishes a MyNoX release on GitHub from this PC.
# Needs: gh (signed in), gpg, the release key in .private\gnupg (fingerprint in .private\release-key-fingerprint.txt).
#   powershell -File tools\installer\release.ps1            # build, sign, publish vX.Y.Z (commit pushed first)
#   powershell -File tools\installer\release.ps1 -NoBuild   # reuse target\dist and target\extension
param([switch]$NoBuild)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$dist = Join-Path $root "target\dist"
$private = Join-Path $root ".private"

$version = [regex]::Match((Get-Content (Join-Path $root "Cargo.toml") -Raw), '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$tag = "v$version"

# The build: installer, zips and the extension packages.
if (-not $NoBuild) {
  & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "build.ps1")
  if ($LASTEXITCODE) { throw "build.ps1 failed" }
}
# This version's setup and portable .exe only, then one SHA256SUMS.txt for both.
$files = Get-ChildItem $dist -File | Where-Object { $_.Name -like "MyNoX-$version-*.exe" } | Sort-Object Name
if (-not $files) { throw "no MyNoX-$version-* files in $dist" }
Remove-Item (Join-Path $dist "SHA256SUMS.txt*") -ErrorAction SilentlyContinue
$sums = $files | ForEach-Object { "$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower())  $($_.Name)" }
[IO.File]::WriteAllText((Join-Path $dist "SHA256SUMS.txt"), (($sums -join "`n") + "`n"))

# The signature the app's updater checks (crates/tools/keys/mynox-release.asc).
$gpg = @((Get-Command gpg.exe -ErrorAction SilentlyContinue).Source, "$env:ProgramFiles\Git\usr\bin\gpg.exe") |
  Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $gpg) { throw "gpg not found" }
$fpr = (Get-Content (Join-Path $private "release-key-fingerprint.txt") -Raw).Trim()
$home_ = Join-Path $private "gnupg"
# Git's gpg is an MSYS program: its agent needs the folder as /d/..., not D:\...
if ($gpg -like "*\usr\bin\gpg.exe") { $home_ = "/" + $home_.Substring(0, 1).ToLower() + ($home_.Substring(2) -replace '\\', '/') }
$env:GNUPGHOME = $home_
& $gpg --batch --yes --pinentry-mode loopback --passphrase "" --detach-sign --digest-algo SHA256 --local-user $fpr `
  --output (Join-Path $dist "SHA256SUMS.txt.sig") (Join-Path $dist "SHA256SUMS.txt")
if ($LASTEXITCODE) { throw "signing failed" }
& $gpg --batch --verify (Join-Path $dist "SHA256SUMS.txt.sig") (Join-Path $dist "SHA256SUMS.txt")
if ($LASTEXITCODE) { throw "the signature doesn't verify" }

# Notes: this version's section of CHANGELOG.md.
$log = Get-Content (Join-Path $root "CHANGELOG.md") -Raw
$m = [regex]::Match($log, "(?ms)^## \[$([regex]::Escape($version))\][^\n]*\n(.*?)(?=^## |\z)")
$notes = Join-Path $dist "notes.md"
[IO.File]::WriteAllText($notes, $(if ($m.Success) { $m.Groups[1].Value.Trim() } else { "MyNoX Download Manager $tag" }))

# The release with every file; GitHub tags the pushed commit (HEAD must be on origin) in the same step.
Push-Location $root
try {
  $sha = (git rev-parse HEAD).Trim()
  $assets = @($files.FullName) + @((Join-Path $dist "SHA256SUMS.txt"), (Join-Path $dist "SHA256SUMS.txt.sig"))
  gh release create $tag --target $sha --latest --title "MyNoX Download Manager $tag" --notes-file $notes @assets
  if ($LASTEXITCODE) { throw "gh release create failed" }
  git fetch --quiet origin "refs/tags/${tag}:refs/tags/${tag}"
} finally {
  Pop-Location
}
"released $tag"
