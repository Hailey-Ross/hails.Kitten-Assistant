# Kitten Assistant

A tiny desktop buddy for Windows. A kitten sits at a little computer on top of your
taskbar and shows what your background jobs are doing.

- **Busy:** the kitten types away and looks at its screen, the mug steams, and a speech
  bubble shows the task and how long it has been running. Each new status line gets a
  screen picked at random: scrolling code, progress bars, an equalizer, a spinning gear or
  a line graph.
- **Waiting:** a line that starts with `Waiting` (for example `Waiting: test results`).
  The kitten props its chin on a paw, drums the desk with the other, gives the
  hourglass on its screen a bored look and thinks "...".
- **Problem:** a line that starts with `Failed` or `Error`. Ears droop, the kitten
  frowns and the screen shows a blinking red X.
- **All done:** when a job goes back to idle, the kitten cheers with its paws up for a
  moment, with sparkles and a green tick, before it settles down for a nap.
- **Coffee break:** every minute or two while it works or waits, the kitten picks up its
  mug, takes a sip with its eyes closed, puts it down and carries on.
- **Idle:** the kitten naps on its keyboard. Zzz float up and the screen shows a moon
  and stars.

It is one small native exe. It uses no runtime, needs no install and adds no tray icon.
The window has no frame and is see-through around the kitten, so it sits on top of
whatever is behind it.

| Busy | Waiting | Problem |
|------|---------|---------|
| ![The kitten typing, with a bubble saying "Building release 1.2, 12 min so far"](docs/busy.png) | ![The kitten with its chin on a paw and an hourglass on the screen, bubble "Waiting: test results, 3 min waiting"](docs/waiting.png) | ![The kitten frowning with drooped ears and a red X on the screen, bubble "Failed: release build"](docs/problem.png) |

| All done | Napping |
|----------|---------|
| ![The kitten cheering with its paws up, sparkles and a green tick, bubble "all done!"](docs/done.png) | ![The kitten asleep on its keyboard, moon on the screen](docs/nap.png) |

## Why

Long jobs such as builds, renders, test runs and scripts give no sign of
life while they work in the background. Your scripts write one line to a status file
and the kitten shows it. You can glance at the screen and know whether something is
still running.

## Install

Download `kitten-assistant.exe` from the Releases page, or build it yourself:

```
cargo build --release
```

The exe lands in `target\release\kitten-assistant.exe`. Building needs Rust 1.80 or newer
on Windows 10 or 11.

## Use

Start the kitten:

```
kitten-assistant
```

Tell it what you are doing:

```
kitten-assistant say "Building release 1.2"
kitten-assistant idle
kitten-assistant quit
```

The bubble's timer starts again whenever the text changes. Saying the same text again keeps
the count going.

The [full guide](docs/USAGE.md) covers every command and option, the status file rules,
scripts in several languages, recipes, starting with Windows and troubleshooting.

Drag the kitten anywhere with the left mouse button. Right-click it for:

- **Back to corner:** back to the bottom-right of its display.
- **Monitor:** pick which display the kitten lives on. It moves there at once and starts
  there from then on (the choice is saved in `kitten-assistant.ini` next to the exe).
- **Always on top:** on (checked) keeps the kitten above other windows; off lets other
  windows cover it. Saved in `kitten-assistant.ini` too.
- **Quit.**

Only one kitten runs at a time, and starting a second one does nothing.

### The status file

The kitten reads the first line of a text file every 2 seconds:

| First line                 | Kitten                           |
|----------------------------|----------------------------------|
| any text                   | types, bubble shows the text     |
| `Waiting...`               | waits, bubble shows the text     |
| `Failed...` or `Error...`  | looks worried, bubble shows the text |
| empty, missing, or `idle`  | naps (after a short cheer if a job just ended) |

The default file is `status.txt` next to the exe. Pick another with `--status FILE`
or the `KITTEN_STATUS` environment variable. Anything that can write a file can drive
the kitten:

```powershell
# PowerShell
Set-Content -Path C:\jobs\status.txt -Value "Nightly backup"
```

```bash
# bash / Git Bash / WSL
echo "Running tests" > /c/jobs/status.txt
```

```python
# Python
open(r"C:\jobs\status.txt", "w").write("Training model, epoch 3")
```

A typical job wrapper:

```bash
echo "Rendering video" > "$KITTEN_STATUS"
./render.sh
echo idle > "$KITTEN_STATUS"
```

### With an AI assistant

The kitten can be used in conjunction with an AI assistant to track what it is working on.
The [`skills/kitten-assistant`](skills/kitten-assistant/SKILL.md) folder is a ready-made skill
that has the assistant set a short status line at each step and `idle` when it is done. See
[the guide](docs/USAGE.md#8-using-it-with-an-ai-assistant) for setup.

### Options

| Option         | Meaning                                                     |
|----------------|-------------------------------------------------------------|
| `--status FILE`| status file to watch (default `status.txt` beside the exe)  |
| `--scale N`    | size multiplier (default follows the screen DPI)            |
| `--monitor N`  | start on display N and make it the default                  |
| `--x N --y N`  | start position in screen pixels (default bottom-right)      |

### Start with Windows

Press Win+R and type `shell:startup`. Add a shortcut to `kitten-assistant.exe` there,
and put any options in the shortcut's Target field.

### Snapshots

`snapshot` renders one frame to a PNG with no window. This is handy for checking art
changes and for screenshots:

```
kitten-assistant snapshot busy.png --busy "Building release 1.2" --minutes 12 --bg 6a8caf
kitten-assistant snapshot waiting.png --busy "Waiting: test results" --minutes 3 --time 4.2 --bg 6a8caf
kitten-assistant snapshot problem.png --busy "Failed: release build" --bg 6a8caf
kitten-assistant snapshot done.png --done --time 3.3 --bg 6a8caf
kitten-assistant snapshot nap.png --bg 6a8caf
```

`--time SECONDS` picks the animation moment. `--scale N` sets the size (default 2).
`--screen N` picks the typing screen (0 code, 1 progress bars, 2 equalizer, 3 gear, 4 graph).
Leave out `--bg` for a transparent PNG.

## How it works

- The art is drawn in code every frame with [tiny-skia](https://crates.io/crates/tiny-skia),
  using flat shapes with a soft outline. There are no image files. Text uses
  [fontdue](https://crates.io/crates/fontdue) with Segoe UI from the Windows font folder.
- The frame goes to Windows as a per-pixel-alpha layered window (`UpdateLayeredWindow`).
  That gives the kitten soft edges on any wallpaper, and clicks pass through the clear
  parts.
- The window is topmost, never takes focus and has no taskbar button. It is built to stay
  light: 30 fps while busy and 15 fps while napping at the lowest CPU priority in Windows'
  efficiency mode (about 1 ms of drawing a frame), only the part of the picture that changed
  is handed to Windows, and while a borderless or exclusive full-screen app or game has the focus the kitten
  holds still and only redraws when its status changes.
- Ordinary screenshot tools may not capture layered windows. Use `snapshot` for
  pictures.

Errors are written to `%TEMP%\kitten-assistant.log`.

## Layout

```
src/main.rs     command line, window, timers, layered-window output
src/scene.rs    the kitten, desk, computer, keyboard and their animation
src/canvas.rs   drawing helpers (shapes, outline, text, word wrap)
docs/USAGE.md   the full guide
skills/         a ready-made skill for using the kitten with an AI assistant
```

## License

MIT. See [LICENSE](LICENSE).
