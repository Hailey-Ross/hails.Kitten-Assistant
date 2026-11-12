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

1. Download `kitten-assistant.exe` from the Releases page and put it in a folder of its own,
   for example `C:\Tools\KittenAssistant\`. Nothing else is needed: no installer, no runtime.
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
| empty, only spaces, `idle` (any case)    | naps; the bubble says "napping... no jobs right now"         |
| the file does not exist or can't be read | naps                                                         |

### Moods

The first word of a busy line picks how the kitten acts (any case; the word ends at the first
character that is not a letter, so `Waiting:`, `waiting...` and `WAITING for CI` all count):

| Mood | First word | What you see |
|------|------------|--------------|
| Busy | anything else | types away, the mug steams; the screen shows scrolling code, progress bars, an equalizer, a spinning gear or a line graph, picked at random each time the line changes (never the same twice in a row) |
| Waiting | `Waiting`, `Wait` | chin propped on one paw, the other paw drums the desk, bored half-closed eyes that glance at the hourglass on the screen, "..." above its head, slow tail swish; the power light pulses amber |
| Problem | `Failed`, `Fail`, `Error`, `Problem` | ears droop, a small frown, a blinking red X on a dark red screen, a red power light |
| All done | (no word: a busy line turns idle) | for 2.5 seconds: paws up, little hops, sparkles, a green tick and "all done!", then the nap |

The all-done moment follows any busy or waiting line that turns idle, but not a problem line:
a job that ended badly goes straight to sleep. To use the moods from a script, write
`Waiting: ...` while you wait on something (a download, a review, another machine) and
`Failed: ...` when a step fails, and leave the problem line up until someone has seen it.

Details that matter when you write the file yourself:

- **Only the first line counts.** Anything after the first line break is ignored, so you can
  keep notes below it.
- **Spaces at the ends are trimmed.** `  Building  ` shows as `Building`.
- **A UTF-8 byte order mark is fine.** Files written by Windows PowerShell's `Out-File` or
  `Set-Content -Encoding utf8` start with one; the kitten skips it.
- **The timer restarts only when the text changes.** Writing the same line again keeps the
  current count. Writing a different line (or going idle and back) starts again at
  "just started". The bubble then counts "1 min so far", "2 min so far", up to
  "1 h 5 min so far" and beyond. After a restart of the kitten, the count starts from when
  the status file was last written.
- **Long text is cut to two lines** with "…" at the end. Short lines read best: about 50
  characters fit. Put the most useful words first ("Tests: 3 of 12 suites" rather than
  "Currently running the test suites, 3 of 12").
- **Updates show within 2 seconds.** Changes faster than that may be skipped; only what is in
  the file at each read is shown.

## 3. Commands

```
kitten-assistant                      start the kitten (watches the status file)
kitten-assistant say "some text"      write "some text" to the status file
kitten-assistant idle                 write "idle" to the status file
kitten-assistant quit                 close the running kitten
kitten-assistant snapshot out.png     render one frame to a PNG, no window
kitten-assistant help                 show the command summary (in a message box)
```

- `say` and `idle` only write the status file and exit at once. They work even if no kitten is
  running; the kitten picks the line up when it starts.
- `say` replaces the whole file with your text.
- `quit` closes the running kitten wherever it is. It does nothing if none is running.
- Only one kitten runs at a time. Starting a second one does nothing, so it is safe to put
  `kitten-assistant` at the top of every script.
- The exe is a windowed app, so it prints nothing to the terminal: `help` and errors (for
  example a status folder that does not exist) show a message box, and errors are also written
  to `%TEMP%\kitten-assistant.log`.
- **In PowerShell, add `| Out-Null`** to `say` and `idle` (for example
  `kitten-assistant say "Backup" | Out-Null`). PowerShell does not wait for windowed apps, so
  without it the next line of your script can run before the file is written. Command Prompt
  batch files, bash, Python and other shells wait on their own.

## 4. Options

| Option          | Used by        | Meaning                                                         |
|-----------------|----------------|-----------------------------------------------------------------|
| `--status FILE` | run, say, idle | the status file (see [section 5](#5-choosing-a-status-file))    |
| `--scale N`     | run, snapshot  | size multiplier; default follows the screen's DPI (snapshot: 2) |
| `--monitor N`   | run            | start on display N (the number in Windows' Display settings) and save it as the default |
| `--x N --y N`   | run            | window position in screen pixels; default bottom-right of the default display, above its taskbar |
| `--busy TEXT`   | snapshot       | draw the busy pose with TEXT; default is napping                |
| `--minutes N`   | snapshot       | elapsed minutes shown in the bubble                             |
| `--time SECONDS`| snapshot       | which moment of the animation to draw (default 3.2)             |
| `--bg RRGGBB`   | snapshot       | background colour as hex, for example `6a8caf`; default transparent |
| `--done`        | snapshot       | draw the all-done moment (use with `--time 3.3`)               |
| `--screen N`    | snapshot       | typing screen: 0 code (default), 1 progress bars, 2 equalizer, 3 gear, 4 line graph |

Options go after the command: `kitten-assistant say "Backup" --status C:\jobs\status.txt`.

## 5. Choosing a status file

The kitten and every `say` / `idle` must point at **the same file**. The file is picked in this
order:

1. `--status FILE` on the command line;
2. the `KITTEN_STATUS` environment variable;
3. `status.txt` in the same folder as `kitten-assistant.exe`.

The default is the easiest: run the kitten and `say` from the same exe and they agree. A fixed
path such as `C:\jobs\status.txt` is better once several scripts or tools share the kitten.

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

## 7. Recipes

### A wrapper for any command

Save as `kitten-run.ps1`, then run `kitten-run "Building" cargo build --release`:

```powershell
param([string]$Label, [Parameter(ValueFromRemainingArguments)]$Cmd)
kitten-assistant say $Label | Out-Null
$exe, $rest = $Cmd
try { & $exe @rest } finally { kitten-assistant idle | Out-Null }
```

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

There is one kitten and one line. Two simple ways to share it:

- Let the longest job own the kitten and leave the others out.
- Write a combined line, for example `Build 2/4, tests running`, from one place that knows
  about all jobs.

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

- **Move it:** drag the kitten with the left mouse button. It goes back to the bottom-right
  corner of its display the next time it starts.
- **Back to the corner:** right-click the kitten and pick **Back to corner**.
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
- **Bigger or smaller:** `kitten-assistant --scale 3` (whole or decimal numbers). By default the
  size follows your screen's scaling setting.
- **It stays on top** of other windows but never takes the keyboard focus, and it has no
  taskbar button. Clicks on the clear area around the kitten go through to whatever is
  behind it.
- **Start with Windows:** press Win+R, type `shell:startup` and press Enter. In the folder that
  opens, create a shortcut to `kitten-assistant.exe`. Add any options at the end of the
  shortcut's **Target** field, for example
  `"C:\Tools\KittenAssistant\kitten-assistant.exe" --status C:\jobs\status.txt`.

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
| The text is cut off with "…"                   | Only two lines fit. Shorten the line and put the important words first.        |
| The kitten is on the wrong screen             | Right-click it, open **Monitor** and pick the display. To forget the choice, delete `kitten-assistant.ini` next to the exe. |
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
src/canvas.rs   drawing helpers (shapes, outline, text, word wrap)
```
