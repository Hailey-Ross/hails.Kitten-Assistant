# Makes a looping GIF of the kitten waving hello with your own text in its speech bubble (for a GitHub profile, a
# website, a chat). One loop is 4 seconds; the GIF repeats it with no seam. Needs ffmpeg on the PATH.
# Usage:
#   powershell -File tools\greeting-gif.ps1 -Text "Hi, welcome to my GitHub!" [-Sub "make yourself at home"]
#       [-Coat "Dilute calico"] [-Seed 1] [-Background 0d1117] [-Scale 1.5] [-Out greeting.gif] [-Exe <kitten exe>]
# -Coat and -Seed default to your own kitten's (kitten-assistant.ini next to the exe). -Background is a hex color:
# GIFs have no soft transparency, so match the page (GitHub dark 0d1117, GitHub light ffffff).
param(
    [Parameter(Mandatory = $true)][string]$Text,
    [string]$Sub = '',
    [string]$Coat = '',
    [long]$Seed = 0,
    [string]$Background = '0d1117',
    [double]$Scale = 1.5,
    [string]$Out = 'greeting.gif',
    [string]$Exe = ''
)
$ErrorActionPreference = 'Stop'
if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) { throw 'ffmpeg is not on the PATH (winget install Gyan.FFmpeg)' }
if (-not $Exe) {
    $built = Join-Path $(if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path (Split-Path $PSScriptRoot -Parent) 'target' }) 'release\kitten-assistant.exe'
    $Exe = if (Test-Path $built) { $built } else { (Get-Command kitten-assistant -ErrorAction Stop).Source }
}
# your own kitten's coat and patches, unless given
$ini = Join-Path (Split-Path $Exe -Parent) 'kitten-assistant.ini'
$installed = Join-Path $env:LOCALAPPDATA 'KittenAssistant\kitten-assistant.ini'
foreach ($f in @($ini, $installed)) {
    if (Test-Path $f) {
        $cfg = @{}
        foreach ($l in Get-Content $f) { if ($l -match '^\s*(\w+)\s*=\s*(.*)$') { $cfg[$Matches[1]] = $Matches[2].Trim() } }
        if (-not $Coat -and $cfg['coat']) { $Coat = $cfg['coat'] }
        if ($Seed -le 0 -and $cfg['coat_seed']) { $Seed = [long]$cfg['coat_seed'] }
        break
    }
}
if (-not $Coat) { $Coat = 'Orange tabby' }
if ($Seed -le 0) { $Seed = 1 }

$fps = 25
$frames = 4 * $fps
$work = Join-Path ([IO.Path]::GetTempPath()) ("kitten-greeting-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force $work | Out-Null
try {
    $greet = if ($Sub) { "$Text|$Sub" } else { $Text }
    # two loops of warm-up first, so the frames start where the loop is settled
    $p = Start-Process -FilePath $Exe -Wait -PassThru -NoNewWindow -ArgumentList @(
        'snapshot', "`"$work\f.png`"", '--greet', "`"$greet`"", '--coat', "`"$Coat`"", '--seed', "$Seed",
        '--scale', "$Scale", '--bg', $Background, '--time', '8', '--frames', "$frames", '--fps', "$fps")
    $made = @(Get-ChildItem $work -Filter 'f-*.png').Count
    if ($made -ne $frames) { throw "the kitten rendered $made of $frames frames (exit $($p.ExitCode); see %TEMP%\kitten-assistant.log)" }
    $outPath = [IO.Path]::GetFullPath($Out)
    ffmpeg -v error -y -framerate $fps -i "$work\f-%04d.png" -vf 'split[a][b];[a]palettegen=max_colors=256:stats_mode=full[p];[b][p]paletteuse=dither=sierra2_4a' -loop 0 $outPath
    if ($LASTEXITCODE -ne 0) { throw 'ffmpeg failed' }
    $kb = [math]::Round((Get-Item $outPath).Length / 1KB)
    Write-Output "made $outPath ($kb KB, $frames frames, $Coat)"
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
