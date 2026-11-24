# Kitten Assistant: the full guide

This guide covers everything the kitten can do and every way to drive it. The
[README](../README.md) has the short version.

- [1. First run in five minutes](#1-first-run-in-five-minutes)
- [2. How the kitten decides what to show](#2-how-the-kitten-decides-what-to-show)
- [3. Commands](#3-commands)
- [4. Options](#4-options)
- [5. Choosing a status file](#5-choosing-a-status-file)
- [6. Driving it from scripts](#6-driving-it-from-scripts)
- [7. Recipes](#7-recipes)
- [8. Using it with an AI assistant](#8-using-it-with-an-ai-assistant)
- [9. Moving, resizing and starting with Windows](#9-moving-resizing-and-starting-with-windows)
- [10. Snapshots](#10-snapshots)
- [11. Troubleshooting](#11-troubleshooting)
- [12. Building from source](#12-building-from-source)

## 1. First run in five minutes

1. Download `kitten-assistant.exe` from the
   [latest release](https://github.com/Hailey-Ross/hails.Kitten-Assistant/releases/latest) and
   put it in a folder of its own that you can write to, for example `C:\Tools\KittenAssistant\`
   (not Program Files: the kitten saves its settings and updates next to the exe). Nothing else
   is needed: no installer, no runtime.
2. Double-click it. A kitten appears at the bottom-right of the screen, sitting on the taskbar,
   asleep on its keyboard.
3. Open a terminal (PowerShell, Command Prompt or Git Bash) in that folder and run:

   ```
   .\kitten-assistant.exe say "Trying out the kitten"
   ```

   Within two seconds the kitten wakes up, types, and a bubble shows "Trying out the kitten"
   and "just started".
4. Send it back to sleep:

   ```
   .\kitten-assistant.exe idle
   ```

5. Close it with `.\kitten-assistant.exe quit`, or right-click the kitten and pick **Quit**.

That is the whole idea: something writes a line, the kitten shows it.

## 2. How the kitten decides what to show

The running kitten reads its **status file** every 2 seconds and looks only at the first line.

| First line of the file                   | What the kitten does                                         |
|------------------------------------------|--------------------------------------------------------------|
| any text                                 | types; the bubble shows the text and the time since it began |
| text starting with `Waiting` or `Wait`   | waits (see moods below); the bubble counts "N min waiting"   |
| text starting with `Failed`, `Fail`, `Error` or `Problem` | looks worried (see moods below)             |
| text starting with another mood word (`Testing`, `Building`, `Researching`, `Downloading`, `Thinking`, ...) | acts out that mood (see moods below) |
| empty, only spaces, `idle` (any case)    | naps; the bubble says "napping... no jobs right now"         |
| the file does not exist or can't be read | naps                                                         |

### Moods

The first word of a busy line picks how the kitten acts (any case; the word ends at the first
character that is not a letter, so `Waiting:`, `waiting...` and `WAITING for CI` all count):

| Mood | First word | What you see |
|------|------------|--------------|
| Busy | anything else | types away, the mug steams; the screen shows scrolling code, progress bars, an equalizer, a spinning gear or a line graph, picked at random each time the line changes (never the same twice in a row) |
| Waiting | `Waiting`, `Wait` | chin propped on one paw, the other paw drums the desk, bored half-closed eyes that glance at the hourglass on the screen (the sand runs down, then the glass turns over), "..." above its head, slow tail swish; the power light pulses amber |
| Problem | `Failed`, `Fail`, `Error`, `Problem` | ears droop, a small frown, a blinking red X on a dark red screen, a red power light |
| Question | `Question`, `Asking`, `Ask`, `Confirm`, `Input` | needs your answer: the right paw goes up beside its head and stays there, bobbing a little, a big question mark pulses on the screen with three waiting dots, and the power light blinks amber. A question shows ahead of work, like a problem |
| Testing | `Testing`, `Test`, `Tests`, `Playtesting`, `Playtest`, `Checking`, `Check`, `Verifying`, `Verify`, `QA`, `Autotest`, `Regression`, `Smoke` | the game tester: holds a controller (a button lights up with each press) and plays a little platformer, eyes on the screen; for the last 2.2 seconds of every 6.5-second round the game shows a pause sign and the kitten jots three lines on the spiral notepad lying flat on the desk with a pencil, looking down; a blue power light |
| Building | `Building`, `Build`, `Rebuilding`, `Rebuild`, `Compiling`, `Compile`, `Linking`, `Packaging`, `Bundling` | types in a yellow hard hat while a brick wall goes up course by course on the screen |
| Researching | `Researching`, `Research`, `Reading`, `Read`, `Reviewing`, `Review`, `Analyzing`, `Analyze`, `Searching`, `Search`, `Investigating`, `Investigate`, `Studying`, `Exploring`, `Digging` | puts on round reading glasses, paws folded on the keyboard, and leans in to the screen, where a page scrolls slowly up and some lines get a yellow highlight as they pass the reading spot; a blue power light |
| Transferring | down: `Downloading`, `Download`, `Fetching`, `Fetch`, `Pulling`, `Pull`, `Installing`, `Install`, `Syncing`, `Sync`, `Copying`, `Copy`; up: `Uploading`, `Upload`, `Pushing`, `Push`, `Deploying`, `Deploy`, `Publishing`, `Publish`, `Backing`, `Backup` | types while files fly between a cloud and a tray on the screen (down or up, with an arrow) and a progress bar fills |
| Thinking | `Thinking`, `Think`, `Planning`, `Plan`, `Designing`, `Design`, `Brainstorming`, `Pondering` | draws on a whiteboard on an easel beside the desk with a blue marker: a different picture every round (a flowchart, a bar chart, a line graph, a mind map, a roadmap, a pie chart, a rocket, a Venn diagram or a light bulb), drawn over about 6.5 seconds and admired for 2 more; the screen shows a checklist that ticks off as the drawing grows |
| All done | (no word: a busy line turns idle) | for 2.5 seconds: paws up, little hops, sparkles, a green tick and "all done!", then the nap |

Every mood and the nap has eight screens. The table names the first one; each new status line
(or new nap) picks one of the eight at random, never the one that mood showed last time:

| Mood | The other seven screens |
|------|------------------------|
| Waiting | a wall clock, dots hopping over a sweeping bar, a snail heading for a flag, a spinner, a bar stuck at 99%, a queue shuffling forward, the sun and moon crossing the sky |
| Problem | a warning sign, a sad face over a broken bar, a bug scuttling under a "!", a pulled plug sparking, a snapped chain, a crack running over the glass, a thermometer running hot |
| Testing | a space shooter, a racing game, a paddle game, a catch game, a gem puzzle, a golf game, a fishing game (all freeze under the pause sign while notes are taken) |
| Building | a crane stacking crates, two gears turning, blocks piling up in a pit, a little house going up, a bridge going across, a robot arm welding, scaffolding climbing |
| Researching | a magnifying glass over the lines, an open book turning its pages, a search with results dropping in, a turning globe, a microscope view, a bar chart read bar by bar, a map with a route |
| Transferring | packets hopping between two computers, an arrow in a filling ring, parcels riding a belt into a folder, a dish sending (or catching) rings, a paper plane, data bubbling through a pipe, a moving truck |
| Thinking | a mind map, sticky notes, a light bulb that brightens, a timeline, a jigsaw, a blueprint, index cards lining up (each grows with the drawing) |
| Napping | a fish tank, a "zzz" badge drifting round, rain on the window, a lava lamp, a candle, northern lights, a snow globe |

The all-done moment follows any busy or waiting line that turns idle, but not a problem line:
a job that ended badly goes straight to sleep. To use the moods from a script, write
`Waiting: ...` while you wait on something (a download, a review, another machine) and
`Failed: ...` when a step fails, and leave the problem line up until someone has seen it.

Now and then (at random, every 15 to 35 seconds) the kitten takes a short break from its mood.
One part of the kitten decides what it does: the mood's own loop, or one break at a time, and
the break has both paws until it ends. Breaks start only where the mood can pause (a testing
kitten between rounds, a planning kitten once a drawing is done), the mood's loop stops while
the break runs and picks up where it stopped, and the paws move over smoothly whenever the
kitten switches. A new status line that comes in during a break shows once the break is over
(the bubble text changes at once); a napping kitten takes no breaks, and going idle ends one.

- **A sip** (about 2.4 seconds): the paw on the mug's side (the right of the picture) lifts the
  mug straight up and over to its mouth, the kitten sips with its eyes closed, puts the mug
  back and returns to its mood. `snapshot --sip` draws the middle of a sip.
- **A stretch** (about 2.6 seconds): both arms up high, eyes squeezed shut, head back a little.
- **A yawn** (about 2.2 seconds): a wide-open mouth with its eyes closed.

Stretches and yawns are the rare ones: about one break in eleven, and never two of them within 4
minutes (a stretch or yawn that comes up sooner is a sip instead).
- **A look around** (about 3.2 seconds): it glances off to the left, then the right, then back.

About one coffee break in four, the mug turns out to be empty (about 7 seconds): the kitten
lifts it, looks in, droops its ears with a little red annoyed mark, holds the mug out in front
of it, reaches off the left side of the picture for a red kettle, pours until the mug is full,
puts the kettle back and settles in, eyes closed and content, then takes a proper sip of the fresh coffee (one break: the mood waits for both). The mug steams again once it is
full.

Details that matter when you write the file yourself:

- **Only the first line is the text.** The lines after it are read only when they look like
  `until=...`, `progress=...`, `mood=...`, `open=...`, `attention=1`, `steps=...`, `step=...`
  or `countdown=1` (see [The file format](#the-file-format)); anything else there is ignored, so
  you can keep notes below the text.
- **Spaces at the ends are trimmed.** `  Building  ` shows as `Building`.
- **Write plain text or UTF-8.** A UTF-8 byte order mark is fine (Windows PowerShell's
  `Set-Content -Encoding utf8` writes one; the kitten skips it). In Windows PowerShell 5.1,
  `>` and `Out-File` without `-Encoding` write UTF-16, which the kitten cannot read, so it
  naps. Use `Set-Content`, add `-Encoding utf8`, or use `kitten-assistant say`.
- **The timer restarts only when the text changes.** Writing the same line again keeps the
  current count. Writing a different line (or going idle and back) starts again at
  "just started". The bubble then counts "1 min so far", "2 min so far", up to
  "1 h 5 min so far" and beyond. After a restart of the kitten, the count starts from when
  the status file was last written.
- **Keep text to two lines.** Longer text may overfill a little, then is cut with "…". Short lines read best: about 50
  characters fit. Put the most useful words first ("Tests: 3 of 12 suites" rather than
  "Currently running the test suites, 3 of 12").
- **Updates show within 2 seconds.** Changes faster than that may be skipped; only what is in
  the file at each read is shown.

## 3. Commands

```
kitten-assistant                      start the kitten (watches the status file)
kitten-assistant say "some text"      write "some text" to the status file
kitten-assistant idle                 write "idle" to the status file
kitten-assistant status               print the line the kitten shows, then every other busy line
kitten-assistant status --json        the same as JSON, with each line's session, mood, age and progress
kitten-assistant sessions             one busy line per row: session, mood, seconds, text (tabs between)
kitten-assistant sessions clear       end every program's line at once (the kitten naps)
kitten-assistant do ACTION [TEXT]     make the running kitten act now (sip, refill, stretch, yawn,
                                      look, wave, cheer); see "Hooking other programs in"
kitten-assistant wrap -- PROGRAM ...  run PROGRAM with a line up, then idle or a Failed line;
                                      exits with PROGRAM's exit code
kitten-assistant wrap --pid N         the same for a process that is already running
kitten-assistant watch --log FILE     keep a line current from a job's log file
kitten-assistant timer TIME [TEXT]    a countdown in the bubble and a cheer at the end
kitten-assistant timer stop           end the timer early
kitten-assistant history              what ran today and for how long (history on|off)
kitten-assistant quit                 close the running kitten
kitten-assistant snapshot out.png     render one frame to a PNG, no window
kitten-assistant update               check for a newer release now and install it
kitten-assistant version              show the version (in a message box)
kitten-assistant help                 show the command summary (in a message box; also -h, --help)
```

- **No kitten running?** `say` starts one, watching the same status file. `idle` does not.
- **Forgot `say`?** Text that is not a command is taken as the status line:
  `kitten-assistant "Building"` works like `kitten-assistant say "Building"` and never opens a
  message box, so a script waiting on the call does not get stuck. A status that is exactly one
  command word (`quit`, `update`, `version`, `help`, `snapshot`, `status`) runs that command
  instead, so use `say` for those.

- **`status` prints the line the kitten shows**, read the same way the kitten reads it (the
  first line, trimmed), or `idle` when the file is empty, missing or says `idle`. It works with
  no kitten running and takes `--status FILE` like `say`. It is the one command that prints to
  the terminal. Bash, Command Prompt and Python get the line as usual. In PowerShell, pipe it on:
  `kitten-assistant status | Write-Output` shows it, and
  `$line = (kitten-assistant status | Out-String).Trim()` keeps it in a variable. A plain
  `$line = kitten-assistant status` comes back empty, because Windows PowerShell gives a
  windowed app nothing to print to unless its output goes into a pipe.

- **Never start a status with `--`.** Anything that starts with `--` is read as an option, even
  inside quotes and even after `say`. `kitten-assistant "--check-assets hang"` or
  `kitten-assistant say Building --release` (unquoted) is an unknown option: the status is not
  written, and a message box with the error opens and keeps the call waiting until someone
  clicks OK, so a script that waits on it hangs. A bare `--` only ends the options for `wrap`
  (everything after it is the program to run). Put a word in front (`kitten-assistant "Check: --check-assets hang"`), quote the whole line,
  or write the status file directly.

- **Updates install themselves.** Every kitten with a build number (a numbered release, the
  `experimental` build, a build made with `tools\build-local.ps1`) checks GitHub
  30 seconds after it starts and then every 6 hours. When a newer stable release is out (a
  higher version, or the same version with a higher build number than its own), it downloads it,
  checks it against the SHA-256 the release publishes, swaps it in and restarts with the same
  options (your status, color and position stay). A kitten whose build number is higher than the
  latest release's keeps itself, so it never goes back to older code. Pre-releases (the rolling
  `experimental` build) are never installed this way. `update` checks
  right away: with a kitten running, that kitten checks and restarts itself only if there is
  something newer, with no message; with none running, it updates the exe and says so in a
  message box. To turn automatic updates off, add the line `auto_update=0` to
  `kitten-assistant.ini` next to the exe. A plain `cargo build` has no build number and never
  updates itself (`update` tells you so). The exe's folder must be writable for
  updates and settings. Update problems are written to `%TEMP%\kitten-assistant.log`.

- `say` and `idle` only write the status file and exit at once. They work even if no kitten is
  running; the kitten picks the line up when it starts.
- `say` replaces the whole file with your text.
- `quit` closes the running kitten wherever it is. It does nothing if none is running.
- Only one kitten runs at a time. Starting a second one does nothing, so it is safe to put
  `kitten-assistant` at the top of every script.
- `status`, `sessions`, `do` and `wrap` are made for other programs: they print to the terminal, and their errors
  are printed too (starting with `kitten-assistant:`) with exit code 1, never a message box.
- The exe is a windowed app, so apart from those it prints nothing to the terminal: `help`, `version` and errors
  (for example a status folder that does not exist) show a message box, and the call does not
  return until it is closed. Errors are also written to `%TEMP%\kitten-assistant.log`, except
  a mistyped or unknown option (see above), which only shows the message box.
- **In PowerShell, add `| Out-Null`** to `say` and `idle` (for example
  `kitten-assistant say "Backup" | Out-Null`). PowerShell does not wait for windowed apps, so
  without it the next line of your script can run before the file is written. Command Prompt
  batch files, bash, Python and other shells wait on their own.

## 4. Options

| Option          | Used by        | Meaning                                                         |
|-----------------|----------------|-----------------------------------------------------------------|
| `--status FILE` | run, say, idle | the status file (see [section 5](#5-choosing-a-status-file))    |
| `--session NAME`| say, idle, wrap | this program's own status line (see [section 5](#5-choosing-a-status-file)); also `KITTEN_SESSION`; `wrap` defaults to `wrap PROGRAM` |
| `--for TIME`    | say            | the line clears itself after TIME: `90` (seconds), `30s`, `5m`, `1h`, `1h30m` |
| `--progress P`  | say, snapshot  | a progress bar in the bubble, and P after the timer: `3/10`, `45%` or `0.45` |
| `--line TEXT`   | wrap           | the line while the program runs (default `Running: PROGRAM ARGS`); its first word picks the mood |
| `--json`        | status, history | print JSON instead of text (see [Hooking other programs in](#hooking-other-programs-in)) |
| `--mood NAME`   | say, wrap, watch, timer | the mood whatever the first word says: `typing`, `waiting`, `problem`, `question`, `testing`, `building`, `researching`, `downloading`, `uploading`, `thinking` (or any mood word) |
| `--open TARGET` | say, wrap, watch | a click on the bubble opens it: a web page (`https://...`), a folder, or a log / text file in its app; any other file is only shown in Explorer, never run |
| `--attention`   | say            | the bubble hops until it is clicked |
| `--steps A,B,C` | say            | a checklist on the screen and "step N of M" in the bubble |
| `--step N`      | say            | the current step, 1 for the first; without `--steps` the line keeps the steps it has |
| `--cheer TEXT`  | idle           | end the line with a cheer showing TEXT |
| `--log FILE`    | watch          | the log to follow |
| `--match TEXT`  | watch          | only log lines containing TEXT count |
| `--tail`        | watch          | show the newest (matching) log line after the line's text |
| `--pid N`       | wrap, watch    | wait for process N to end: idle on exit code 0, else a Failed line |
| `--background`  | watch          | keep watching as a background process and return at once |
| `--days N`      | history        | the last N days instead of today |
| `--listen ADDR` | run            | serve the local endpoint on ADDR, for example `127.0.0.1:47613` (also `listen=` in the ini) |
| `--scale N`     | run, snapshot  | size multiplier; default: right-click > Size, else the screen's DPI (snapshot: 2) |
| `--monitor N`   | run            | start on display N (the number in Windows' Display settings) and save it as the default |
| `--x N --y N`   | run            | window position in screen pixels; default bottom-right of the default display, above its taskbar |
| `--busy TEXT`   | snapshot       | draw the busy pose with TEXT; default is napping                |
| `--minutes N`   | snapshot       | elapsed minutes shown in the bubble                             |
| `--time SECONDS`| snapshot       | which moment of the animation to draw (default 3.2)             |
| `--bg RRGGBB`   | snapshot       | background color as hex, for example `6a8caf`; default transparent |
| `--done`        | snapshot       | draw the all-done moment (use with `--time 3.3`)               |
| `--screen N`    | snapshot       | the mood's screen 0-7 (default 0, the first); typing: 0 code, 1 progress bars, 2 equalizer, 3 gear, 4 line graph |
| `--sip`         | snapshot       | draw the middle of a coffee sip (snapshots never sip otherwise) |
| `--coat NAME`   | snapshot       | the color, as named under right-click > **Color** (any case), for example `"Gray tabby"`; default Orange tabby |
| `--seed N`      | snapshot       | where the patches sit on patched coats (default 1)              |

Options can go before or after the command: `kitten-assistant say "Backup" --status C:\jobs\status.txt`.
Options that take a value use the next word as the value.

## 5. Choosing a status file

The kitten and every `say` / `idle` must point at **the same file**. The file is picked in this
order:

1. `--status FILE` on the command line;
2. the `KITTEN_STATUS` environment variable;
3. `status.txt` in the same folder as `kitten-assistant.exe`.

The default is the easiest: run the kitten and `say` from the same exe and they agree. A fixed
path such as `C:\jobs\status.txt` is better once several scripts or tools share the kitten.

### Several programs at once (sessions)

When more than one script, tool or agent reports to the same kitten, give each one a name with
`--session NAME` (or the `KITTEN_SESSION` environment variable), for example
`kitten-assistant --session backup "Backing up photos"`. Each name gets its own line in a
`sessions` folder next to the status file, so one program's `idle` never hides another's work:

- The kitten shows the line that matters most: a problem (`Failed`, `Error`, `Problem`) first,
  then work, then `Waiting`; among those, the newest. The bubble adds "+N more" when other
  programs are busy too. Click "+N more" and the bubble shows each other program's line in
  turn, a few seconds each with its name, then goes back; click it again to stop early. It
  also goes round them on its own every 95 to 120 seconds.
- It naps only when every program is idle. `idle` with a session removes just that line.
- `kitten-assistant status` prints the shown line first, then every other busy line with its
  name in brackets.
- A line nobody has touched for 3 hours is treated as left over from a program that ended
  without `idle`, and ignored.
- Calls without `--session` keep using the status file itself, which counts as one more line.
- A kitten from before sessions does not know `--session` and shows an "unknown option" message
  box: update it first.

Set `KITTEN_STATUS` once for your user so every terminal and script uses the same file:

```powershell
[Environment]::SetEnvironmentVariable("KITTEN_STATUS", "C:\jobs\status.txt", "User")
```

Open a new terminal afterwards (and restart the kitten) so the variable is picked up.

The folder must exist; the kitten does not create it. A file on a network share works too,
so a job on another computer can drive the kitten on yours (see
[recipes](#a-job-on-another-computer)).

## 6. Driving it from scripts

Anything that can write a text file can drive the kitten. Use the exe's `say` / `idle`
commands, or write the file directly.

**PowerShell**

```powershell
kitten-assistant say "Nightly backup" | Out-Null
# or, without the exe:
Set-Content -Path $env:KITTEN_STATUS -Value "Nightly backup"
```

**Command Prompt / batch**

```bat
kitten-assistant say "Nightly backup"
rem or:
echo Nightly backup> "%KITTEN_STATUS%"
```

(No space before `>` in batch, or the space ends up in the line; it is trimmed anyway.)

**bash / Git Bash / WSL**

```bash
echo "Running tests" > "$KITTEN_STATUS"
```

In WSL, point it at the Windows path, for example `/mnt/c/jobs/status.txt`.

**Python**

```python
import os
path = os.environ.get("KITTEN_STATUS", r"C:\jobs\status.txt")
with open(path, "w", encoding="utf-8") as f:
    f.write("Training model, epoch 3")
```

**Node.js**

```js
require("fs").writeFileSync(process.env.KITTEN_STATUS, "Bundling assets");
```

### Always go back to idle

If a script fails halfway, the kitten keeps typing the last line forever. Reset it on every
exit path:

```powershell
try {
    kitten-assistant say "Rendering video" | Out-Null
    ./render.ps1
} finally {
    kitten-assistant idle | Out-Null
}
```

```bash
trap 'echo idle > "$KITTEN_STATUS"' EXIT
echo "Rendering video" > "$KITTEN_STATUS"
./render.sh
```

### Show progress

Update the line at each step. Because the timer restarts when the text changes, the bubble
then shows how long the current step has taken:

```powershell
$steps = "Fetching", "Building", "Testing", "Packaging"
foreach ($i in 0..($steps.Count - 1)) {
    kitten-assistant say "$($steps[$i]) ($($i + 1) of $($steps.Count))" | Out-Null
    # ... do the step ...
}
kitten-assistant idle | Out-Null
```

If you want the timer to show the whole job instead, keep the same text for the whole job.

For a bar, keep the text and send `--progress` each time; the timer keeps counting because the
text did not change:

```powershell
foreach ($i in 1..$files.Count) {
    kitten-assistant say "Copying: photos" --session photos --progress "$i/$($files.Count)" | Out-Null
    # ... copy one ...
}
kitten-assistant idle --session photos | Out-Null
```

### Lines that end on their own

`--for TIME` gives a line an end time: `kitten-assistant say "Waiting: deploy to settle" --for 5m`.
After that the kitten drops it as if `idle` had been sent. Handy for notices from programs that
might never come back to say `idle`.

### Hooking other programs in

**Wrap a command.** `wrap` puts a line up, runs the program in your terminal (its output and
input stay there), then sends `idle` when it exits with 0 (the kitten cheers) or leaves a
`Failed` line otherwise, and exits with the program's own exit code:

```powershell
kitten-assistant wrap --line "Building: release" -- cargo build --release | Out-Default
```

```bash
kitten-assistant wrap --line "Testing: unit tests" -- npm test && echo passed
```

The failed line names what failed: `Failed: release (exit 101)`, or `(canceled)` when you
stopped it with Ctrl+C. In PowerShell add `| Out-Default` so it waits for the program and keeps
its output on the screen (`| Out-Null` would hide it); `$LASTEXITCODE` holds the exit code.

**Make the kitten act.** `do` reaches the running kitten straight away:

| Action        | What the kitten does |
|---------------|----------------------|
| `sip`         | a sip of coffee |
| `refill`      | finds the mug empty, puts the kettle on, pours, sips |
| `stretch`, `yawn` | a stretch, a yawn |
| `look`        | looks around |
| `wave [TEXT]` | the hello wave, with TEXT in the bubble (default: the usual hello) |
| `cheer [TEXT]`| a cheer with a green tick, TEXT on the screen and in the bubble (default `yay!`) |

`kitten-assistant do cheer "Deployed to prod!"`. `wave` and `cheer` work any time; the others only
while the kitten is working (a napping kitten sleeps through them). With no kitten running,
`do` prints an error and exits with 1.

**Read what is going on.** `status --json` gives every busy line, the shown one first:

```json
{"idle":false,"lines":[
  {"session":"backup","text":"Copying: photos","mood":"downloading","age_secs":42,
   "progress":0.300,"progress_label":"3/10","expires_in_secs":null,"shown":true}]}
```

`session` is `null` for the plain status file; `mood` is one of `typing`, `waiting`, `problem`,
`testing`, `building`, `researching`, `downloading`, `uploading`, `thinking`. `sessions` prints
the same lines as tab-separated rows (session, mood, seconds, text), easy to read in a shell;
`sessions clear` ends them all.

**The file format.** Programs that write the status file directly can add the same things on the
lines after the text, one `key=value` per line. Older kittens read only the first line, so the
extra lines never confuse them:

```
Copying: photos
until=1795482140
progress=3/10
```

`until` is the end time in Unix seconds; `progress` takes the same forms as `--progress`. The
other keys match the options: `mood=testing`, `open=https://...`, `attention=1`,
`steps=fetch,build,test`, `step=2`, and `countdown=1` (a timer's line: the bubble counts down to
`until`).

### Questions and things that need you

Start a line with `Question` (or `Asking`, `Ask`, `Confirm`, `Input`) when a job is waiting for
someone's answer: the kitten raises a paw and shows a question mark, and the line shows ahead of
work lines, like a problem. Add `--attention` to make the bubble hop until someone clicks it, and
`--open` to make that click open something useful:

```
kitten-assistant say "Question: deploy to production?" --attention --open https://ci.example.com/run/812
```

A click on the bubble stops the hop (for that line; a new line hops again) and opens the link,
folder or file. Web pages open in the browser, folders in Explorer, and `.log`, `.txt`, `.md`,
`.json`, `.csv`, `.xml`, `.yaml`, `.ini`, `.html`, `.png` and `.pdf` files in their own app. Any
other file is shown selected in Explorer and never run. `--open` works on any line, not only
questions.

### Steps and time left

`--steps` puts a checklist on the screen (done steps ticked, the current one with an arrow) and
"step N of M" in the bubble. Move along with `--step` alone; the line keeps its steps:

```
kitten-assistant say "Deploying: site" --session deploy --steps "fetch,build,test,upload" --step 1
kitten-assistant say "Deploying: site" --session deploy --step 2
```

With `--progress`, the kitten also works out the time left from how fast the progress moves, and
after about 10 seconds the bubble adds "about 4 min left".

`--mood` picks the mood when the first word should read naturally:
`kitten-assistant say "Unit tests: 40 of 120" --mood testing`.

### Watching a log

`watch` keeps a line current from a job that writes a log, with no script of your own:

```
kitten-assistant watch --log build.log --match Compiling --tail --line "Building: game" --background
```

Every 2 seconds it reads what the log added (UTF-8, or UTF-16 as Windows PowerShell's `>`
writes), takes the progress from the newest matching line's first `3/10` or `45%`, and with
`--tail` shows that log line after the text. Each write lasts a minute, so if the watcher dies
its line goes away on its own. It stops on Ctrl+C, when someone sends `idle` for its session
(`kitten-assistant idle --session "watch build"`; the session is `watch` plus the log's name
unless you give `--session`), or with `--pid N` when that process ends (idle on exit code 0, a
Failed line otherwise). `--background` starts it as its own hidden process and returns at once.

`wrap --pid N` waits for a process that is already running in the same way, without a log:
`kitten-assistant wrap --pid 4120 --line "Rendering: final cut"`.

### Ending with a cheer

`kitten-assistant idle --session deploy --cheer "Deployed!"` ends the line and cheers with your
words (a green tick on the screen, the words in the bubble). Without a running kitten it just
ends the line.

### Timers

`kitten-assistant timer 25m Focus` puts up a line that counts down ("24:59 left", with a bar that
runs down), and when it runs out the kitten cheers "Focus: time's up!". `timer stop` ends it early
without a cheer. The timer uses the session `timer` (another with `--session`) and the waiting
mood (another with `--mood`).

### History

Off by default. Turn it on with right-click > **Keep a history** or `kitten-assistant history on`.
The kitten then writes every line that ends (its start, end, session, mood and text) to
`history.tsv` next to the status file and keeps 30 days. `kitten-assistant history` prints
today's lines with their times and lengths, `--days 7` the last week, `--json` the rows as JSON,
and right-click > **What ran today** opens today's in Notepad. Lines can hold anything a program
wrote, so the file stays on your computer and is never sent anywhere.

### The local endpoint

For programs that cannot start the exe or share a file (a browser extension, a container, WSL,
another computer), the kitten can answer HTTP. It is off by default: add `listen=127.0.0.1:47613`
to `kitten-assistant.ini` (or start the kitten with `--listen 127.0.0.1:47613`) and restart it.
On its first start it makes a token in `listen-token.txt` next to the exe; every call must send
it as `Authorization: Bearer <token>`.

| Call | Fields | Does |
|------|--------|------|
| `GET /status` | | the `status --json` document |
| `POST /say` | `text`, and any of `session`, `progress`, `for`, `mood`, `open`, `attention`, `steps`, `step` | `say` |
| `POST /idle` | `session`, `cheer` | `idle` |
| `POST /do` | `action`, `text` | `do` |

Fields go in the query string or a form body:

```bash
curl -X POST http://127.0.0.1:47613/say -H "Authorization: Bearer $TOKEN" \
     -d session=ext -d "text=Uploading: photos" -d progress=3/10
```

Answers are JSON (`{"ok":true}`, or `{"ok":false,"error":"..."}` with 400, 401, 404 or 405). There
are no CORS headers, so a web page cannot call it from a browser. `127.0.0.1` only answers this
computer; to let another computer in, listen on its network address and keep the token secret
(anyone with it can change the kitten's lines).

### Sounds

Off by default. Right-click > **Sounds** (or `sound=1` in the ini) plays a Windows sound when a
problem or a question becomes the line the kitten shows.

## 7. Recipes

### A wrapper for any command

Use `wrap` (see [Hooking other programs in](#hooking-other-programs-in)):
`kitten-assistant wrap --line "Building" -- cargo build --release`.

### Scheduled tasks

In Task Scheduler, make the action a PowerShell script that writes the line at the start and
`idle` in a `finally` block (see above). The kitten must be running in your session to be
seen; the scheduled job only writes the file.

### A job on another computer

Share a folder (for example `\\DESKTOP\jobs`), start the kitten on your computer with
`--status \\DESKTOP\jobs\status.txt` or a local path to the shared folder, and have the job on
the other computer write to the same file. The kitten shows it within 2 seconds of the file
changing.

### Several jobs at once

Give each job its own `--session NAME` (see
[Several programs at once](#several-programs-at-once-sessions)); the kitten shows the one that
matters most and counts the rest.

### Builds in an editor

Most editors can run a task before and after a build. Add `kitten-assistant say "Building"`
before and `kitten-assistant idle` after, or call the wrapper above as the build task.

## 8. Using it with an AI assistant

The kitten can be used in conjunction with an AI assistant to track what it is working on: the
assistant writes a short line about its current task to the status file, updates it as it
moves to the next step, and sets `idle` when it is finished. You can then glance at the
kitten to see whether the assistant is still busy and on what.

A ready-made skill for this is in [`skills/kitten-assistant/`](../skills/kitten-assistant/SKILL.md).
Copy that folder into your assistant's skills folder (for your user, or inside one project
for that project only), and make sure the assistant can run
`kitten-assistant` (on the `PATH`, or tell it the full path) and knows the status file
(`KITTEN_STATUS`).

## 9. Moving, resizing and starting with Windows

- **Move it:** drag the kitten with the left mouse button. It remembers the spot (saved in
  `kitten-assistant.ini` as `position=x,y`) and opens there next time, until you drag it
  again. If that spot is no longer on any display (a monitor was unplugged), it starts in the
  corner instead.
- **The menu:** right-click the kitten. It opens as a small card at the pointer. **Monitor**,
  **Color** and **Size** open as pages that slide in, with a round back button at the top.
  The keyboard works too: arrow keys move, Enter or Space picks, Left or Backspace goes back,
  Escape goes back or closes. A click anywhere else closes it; a click on the kitten itself
  does not (right-click again to start over).
- **Back to the corner:** right-click the kitten and pick **Back to corner**. This also forgets
  the dragged-to spot, so it starts in the corner again. Picking a display under **Monitor**
  (or `--monitor N`) does the same.
- **Change its color:** right-click the kitten and open **Color**. There are 32 coats: tabbies
  (orange, ginger, cream, gray, silver, brown, golden), solids (black, white, Russian blue,
  chocolate, cinnamon, lilac, fawn), a tuxedo, two pointed coats, patched ones (calico,
  tortoiseshell, cow cat, dilute calico) and fun ones (cotton candy, lavender, mint chip, sky, lemon,
  peach, strawberry, grape, galaxy, ghost, rainbow; most of these have their own markings: hearts, flower dots, chocolate chips, clouds, seeds and a leafy top, stars, rainbow stripes). Each one shows as a portrait of the kitten in that coat; the one under the pointer lifts and shows its name, and the current one has an orange ring. The
  choice is saved in `kitten-assistant.ini` (`coat=Gray tabby`, for example). Patched coats (calico, dilute calico, tortoiseshell, cow cat) roll new patches every time you pick a color: where they sit, their size and shape, and which ears are colored. The roll is saved too (`coat_seed=`), so the kitten looks the same after a restart. Pick the same color again for a new roll.
- **Pick a display:** with more than one monitor, right-click the kitten and open **Monitor**.
  It lists every display with the number Windows uses in Settings > System > Display, the
  model name when Windows knows it and the resolution; the current one has a check mark.
  Pick one and the kitten moves to its bottom-right corner (above the taskbar if that
  display has one). The choice is saved in `kitten-assistant.ini` next to the exe, so the
  kitten starts there from then on. `--monitor N` on the command line does the same, which
  is handy in a startup shortcut. If the saved display is unplugged, the kitten uses the
  main display until it comes back.
- **Start somewhere else:** `kitten-assistant --x 100 --y 900` (screen pixels; displays
  above or left of the main one have negative positions).
- **Bigger or smaller:** right-click the kitten and open **Size**. Pick 50% to 300% and the
  kitten resizes at once, keeping its bottom-right corner where it was. The choice is saved in
  `kitten-assistant.ini` (`scale=1.5`, for example) and used at every start. **Default** forgets
  it, so the size follows your screen's scaling setting again. `kitten-assistant --scale 3` (whole
  or decimal numbers) sets the size for that run only and wins over the saved one.
- **It stays on top** of other windows by default but never takes the keyboard focus, and it
  has no taskbar button. Clicks on the clear area around the kitten go through to whatever is
  behind it.
- **It keeps going while you play:** the kitten keeps animating and showing new statuses while
  a full-screen (or borderless full-screen) app or game has the focus, at the same light frame
  rate, and never takes the focus.
- **Check for an update now:** right-click the kitten and click **Check for Update**. A newer
  release is downloaded, checked and started in place of the old one (the kitten restarts);
  otherwise the kitten cheers for 3 seconds with "up to date" and a green tick drawing itself on the computer screen,
  whatever it was doing. A build without a build number says it does not update.
- **Always on top on or off:** right-click the kitten and click **Always on top**. A check
  mark means on. Off lets other windows cover the kitten (click the desktop or minimize them
  to see it again). The choice is saved in `kitten-assistant.ini` (`always_on_top=0` or `1`)
  and used at every start.
- **Start with Windows:** press Win+R, type `shell:startup` and press Enter. In the folder that
  opens, create a shortcut to `kitten-assistant.exe`. Add any options at the end of the
  shortcut's **Target** field, for example
  `"C:\Tools\KittenAssistant\kitten-assistant.exe" --status C:\jobs\status.txt`.

### The settings file

Everything you pick in the right-click menu is saved in `kitten-assistant.ini` next to the exe,
one `key=value` per line. You can edit it while the kitten is closed, or delete it to go back to
all the defaults.

| Key             | Set by                         | Meaning                                                  |
|-----------------|--------------------------------|----------------------------------------------------------|
| `position`      | dragging the kitten            | `x,y` of the top-left corner; removed by **Back to corner** and **Monitor** |
| `monitor`       | **Monitor**, `--monitor N`     | the home display, for example `\\.\DISPLAY2`            |
| `scale`         | **Size**                       | size multiplier, for example `1.5`; removed by **Default** |
| `coat`          | **Color**                      | the color's name, for example `Gray tabby`               |
| `coat_seed`     | **Color**                      | where the patches sit on patched coats                   |
| `always_on_top` | **Always on top**              | `1` (default) or `0`                                     |
| `sound`         | **Sounds**                     | `1` or `0` (default)                                     |
| `history`       | **Keep a history**             | `1` or `0` (default)                                     |
| `listen`        | (none)                         | an address such as `127.0.0.1:47613` for the local endpoint; empty or missing = off |
| `auto_update`   | you, by hand                   | `0` turns off automatic updates (default on)             |

## 10. Snapshots

`snapshot` draws one frame to a PNG file without opening a window. Use it for screenshots
(ordinary screenshot tools often miss see-through windows like this one) and to check what a
status line will look like:

```
kitten-assistant snapshot busy.png --busy "Building release 1.2" --minutes 12 --bg 6a8caf
kitten-assistant snapshot waiting.png --busy "Waiting: test results" --minutes 3 --time 4.2 --bg 6a8caf
kitten-assistant snapshot problem.png --busy "Failed: release build" --bg 6a8caf
kitten-assistant snapshot done.png --done --time 3.3 --bg 6a8caf
kitten-assistant snapshot nap.png --bg 6a8caf
kitten-assistant snapshot big.png --busy "Long text test" --scale 4
```

Leave out `--bg` for a transparent background.

## 11. Troubleshooting

| What you see                                   | What to check                                                                 |
|------------------------------------------------|-------------------------------------------------------------------------------|
| The kitten never wakes up                      | The kitten and your script use different files. Use the same `--status` path or set `KITTEN_STATUS` for both, and restart the kitten after changing the variable. |
| It wakes up, then naps again a moment later    | Something writes `idle` or an empty file. A batch `echo.>` or an empty `Set-Content` makes an empty file. |
| It keeps typing after the job ended            | The job stopped before it wrote `idle`. Use `finally` / `trap` ([section 6](#always-go-back-to-idle)). |
| The text is cut off with "…"                   | Two lines are supported; longer text may overfill, then is cut. Shorten the line and put the important words first.        |
| It naps although PowerShell wrote the file     | Windows PowerShell's `>` and `Out-File` write UTF-16, which the kitten cannot read. Use `Set-Content` or `kitten-assistant say` ([section 2](#2-how-the-kitten-decides-what-to-show)). |
| A script hangs on a `kitten-assistant` call    | A message box is waiting for OK (it may be behind other windows). Usually the status started with `--` and was read as an option; see [section 3](#3-commands). `help` and `version` also wait for OK. |
| The kitten is on the wrong screen             | Right-click it, open **Monitor** and pick the display. Deleting `kitten-assistant.ini` next to the exe forgets this and every other saved choice. |
| Nothing happens when I start it                | A kitten is already running (maybe dragged off-screen): run `kitten-assistant quit`, then start it again. |
| An error message box appears                   | The message is also in `%TEMP%\kitten-assistant.log`.                           |
| My screenshot has no kitten                    | Use `snapshot` instead (section 10).                                           |
| Text looks wrong or is missing                 | The kitten uses Segoe UI from the Windows fonts folder; check it is installed. |

## 12. Building from source

You need Rust 1.80 or newer on Windows 10 or 11 (install from <https://rustup.rs>).

```
git clone https://github.com/Hailey-Ross/hails.Kitten-Assistant
cd hails.Kitten-Assistant
cargo build --release
```

The exe is `target\release\kitten-assistant.exe`. The source is small:

```
src/main.rs     command line, window, timers, see-through window output
src/scene.rs    the kitten, desk, computer, keyboard and their animation
src/scene/      the computer's screens for each mood
src/brain.rs    what the kitten does: the mood's loop or one short break at a time
src/menu.rs     the right-click menu (pages, coat portraits, keyboard)
src/canvas.rs   drawing helpers (shapes, outline, text, word wrap)
src/update.rs   the self-updater
```

A plain `cargo build` is build 0, a dev build: it shows "(dev build)" under `version` and in the
right-click menu, and never updates itself.
`powershell -File tools\build-local.ps1` builds it with a build number (the newest Actions run
plus one, read with the GitHub CLI, or `-Build <n>`): it keeps itself until a release built after
it comes out, then updates to it. `cargo test --release` runs the tests.

### How releases are made

The workflow in `.github/workflows/release.yml` tests and builds the exe on every push.

- A push to `main` replaces the `experimental` pre-release with the new build. Installed
  kittens never update to it; it updates itself to the next stable release built after it.
- Publishing a GitHub release (not marked as a pre-release) whose tag is `v` plus the version in `Cargo.toml` (for example
  `v0.1.1`) builds the exe for it and attaches `kitten-assistant.exe`, its `.sha256` and a
  `release.json` with the version and build number. That is what installed kittens update to.
  A tag that does not match the version fails the build.
