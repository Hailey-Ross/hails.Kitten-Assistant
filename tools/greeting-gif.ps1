# Makes a looping animation of the kitten waving hello with your own text in its speech bubble (for a GitHub profile,
# a website, a chat). One loop is 4 seconds; the file repeats it with no seam. Needs ffmpeg on the PATH.
# Usage:
#   powershell -File tools\greeting-gif.ps1 -Text "Hi, welcome to my GitHub!" [-Sub "make yourself at home"]
#       [-Out greeting.webp] [-Background none] [-Coat "Dilute calico"] [-Seed 1] [-Scale 1.5] [-Exe <kitten exe>]
# The file type follows -Out:
#   .webp  animated WebP, lossless, soft see-through edges (the default; GitHub, browsers and chats show it)
#   .png   animated PNG, the same soft edges, about twice the size
#   .gif   GIF: see-through only as on/off, so with -Background none its edges are jagged; give the page's color
#          instead (GitHub dark 0d1117, GitHub light ffffff)
# -Background none (the default) leaves the background see-through; a hex color fills it.
# -Coat and -Seed default to your own kitten's (kitten-assistant.ini).
param(
    [Parameter(Mandatory = $true)][string]$Text,
    [string]$Sub = '',
    [string]$Out = 'greeting.webp',
    [string]$Background = 'none',
    [string]$Coat = '',
    [long]$Seed = 0,
    [double]$Scale = 1.5,
    [string]$Exe = ''
)
$ErrorActionPreference = 'Stop'
if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) { throw 'ffmpeg is not on the PATH (winget install Gyan.FFmpeg)' }
$kind = [IO.Path]::GetExtension($Out).ToLowerInvariant()
if ($kind -notin '.webp', '.png', '.gif') { throw "-Out must end in .webp, .png or .gif" }
$clear = $Background -in 'none', 'transparent', ''
if (-not $Exe) {
    $built = Join-Path $(if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path (Split-Path $PSScriptRoot -Parent) 'target' }) 'release\kitten-assistant.exe'
    $Exe = if (Test-Path $built) { $built } else { (Get-Command kitten-assistant -ErrorAction Stop).Source }
}
# your own kitten's coat and patches, unless given
foreach ($f in @((Join-Path (Split-Path $Exe -Parent) 'kitten-assistant.ini'), (Join-Path $env:LOCALAPPDATA 'KittenAssistant\kitten-assistant.ini'))) {
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
    $args = @('snapshot', "`"$work\f.png`"", '--greet', "`"$greet`"", '--coat', "`"$Coat`"", '--seed', "$Seed",
        '--scale', "$Scale", '--time', '8', '--frames', "$frames", '--fps', "$fps")
    if (-not $clear) { $args += @('--bg', $Background) }
    $p = Start-Process -FilePath $Exe -Wait -PassThru -NoNewWindow -ArgumentList $args
    $made = @(Get-ChildItem $work -Filter 'f-*.png').Count
    if ($made -ne $frames) { throw "the kitten rendered $made of $frames frames (exit $($p.ExitCode); see %TEMP%\kitten-assistant.log)" }
    $outPath = [IO.Path]::GetFullPath($Out)
    $in = @('-v', 'error', '-y', '-framerate', "$fps", '-i', "$work\f-%04d.png")
    switch ($kind) {
        '.webp' { ffmpeg @in -c:v libwebp_anim -lossless 1 -loop 0 $outPath }
        '.png' { ffmpeg @in -plays 0 -f apng $outPath }
        '.gif' {
            $vf = if ($clear) { 'split[a][b];[a]palettegen=reserve_transparent=1[p];[b][p]paletteuse=alpha_threshold=128' }
                  else { 'split[a][b];[a]palettegen=max_colors=256:stats_mode=full[p];[b][p]paletteuse=dither=sierra2_4a' }
            ffmpeg @in -vf $vf -loop 0 $outPath
        }
    }
    if ($LASTEXITCODE -ne 0) { throw 'ffmpeg failed' }
    $kb = [math]::Round((Get-Item $outPath).Length / 1KB)
    $bg = if ($clear) { 'see-through' } else { "#$Background" }
    Write-Output "made $outPath ($kb KB, $frames frames, $Coat, $bg)"
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
