# Builds kitten-assistant.exe locally with a build number: the newest GitHub Actions run of this repo plus one.
# The exe then updates itself only to a release built after it (a higher run number), never to an older release.
# Needs the GitHub CLI (gh) signed in; without it, pass -Build <n>.
# Usage: powershell -File tools\build-local.ps1 [-Build <n>]
param([int]$Build = 0)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if ($Build -le 0) {
    $last = gh run list --repo Hailey-Ross/hails.Kitten-Assistant --limit 1 --json number --jq '.[0].number'
    if ($LASTEXITCODE -ne 0 -or -not $last) { throw "could not read the newest Actions run (pass -Build <n>)" }
    $Build = [int]$last + 1
}
$env:KITTEN_BUILD = "$Build"
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'target' }
Write-Output "built $target\release\kitten-assistant.exe as build $Build"
