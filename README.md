# Kitten Assistant

A tiny desktop buddy for Windows. A kitten sits at a little computer on top of your
taskbar and shows what your background jobs are doing.

- **Busy:** the kitten types away and looks at its screen, the mug steams, and a speech
  bubble shows the task and how long it has been running. Each new status line gets a
  screen picked at random: scrolling code, progress bars, an equalizer, a spinning gear or
  a line graph.
- **Waiting:** a line that starts with `Waiting` or `Wait` (for example `Waiting: test results`).
  The kitten props its chin on a paw, drums the desk with the other, gives the
  hourglass on its screen a bored look and thinks "...".
- **Problem:** a line that starts with `Failed`, `Fail`, `Error` or `Problem`. Ears droop,
  the kitten frowns and the screen shows a blinking red X.
- **Testing:** a line that starts with `Testing`, `Test`, `Checking`, `Verifying`, `QA` and
  the like. The kitten turns game tester: it plays a little platformer with a controller,
  then pauses the game now and then to jot notes on a notepad.
- **Building:** `Building`, `Compiling`, `Linking`... The kitten types in a yellow hard hat
  while a brick wall goes up on the screen.
- **Researching:** `Researching`, `Reading`, `Reviewing`, `Analyzing`, `Searching`... It
  holds a magnifying glass up to a page on the screen.
- **Transferring:** `Downloading`, `Installing`, `Syncing` (files fly down from a cloud) or
  `Uploading`, `Pushing`, `Deploying` (files fly up), with a progress bar.
- **Thinking:** `Thinking`, `Planning`, `Designing`... Chin on a paw, a light bulb flickers
  in a thought cloud and a flowchart sketches itself on the screen.
- **All done:** when a job goes back to idle, the kitten cheers with its paws up for a
  moment, with sparkles and a green tick, before it settles down for a nap. A job that
  ended on a problem line goes straight to sleep.
- **Coffee break:** every 15 to 35 seconds while it is awake, the kitten picks up its
  mug, takes a sip with its eyes closed, puts it down and carries on.
- **Idle:** the kitten naps on its keyboard. Zzz float up and the screen shows a moon
  and stars.

It is one small native exe. It uses no runtime, needs no install and adds no tray icon.
The window has no frame and is see-through around the kitten, so it sits on top of
whatever is behind it.

| Busy | Waiting | Problem |
|------|---------|---------|
| ![The kitten typing, with a bubble saying "Writing release notes, 12 min so far"](docs/busy.png) | ![The kitten with its chin on a paw and an hourglass on the screen, bubble "Waiting: test results, 3 min waiting"](docs/waiting.png) | ![The kitten frowning with drooped ears and a red X on the screen, bubble "Failed: release build"](docs/problem.png) |

| Testing | Building | Researching |
|---------|----------|-------------|
| ![The kitten holding a game controller, a little platformer on the screen and a notepad on the desk, bubble "Testing: regression suite"](docs/testing.png) | ![The kitten typing in a yellow hard hat while a brick wall goes up on the screen, bubble "Building: release 1.2"](docs/building.png) | ![The kitten holding a magnifying glass up to a page on the screen, bubble "Reviewing: pull request 42"](docs/researching.png) |

| Downloading | Thinking |
|-------------|----------|
| ![The kitten typing while files drop from a cloud on the screen, bubble "Downloading: Rust toolchain"](docs/downloading.png) | ![The kitten with its chin on a paw, a light bulb in a thought cloud and a flowchart on the screen, bubble "Planning: next release"](docs/thinking.png) |

| All done | Napping |
|----------|---------|
| ![The kitten cheering with its paws up, sparkles and a green tick, bubble "all done!"](docs/done.png) | ![The kitten asleep on its keyboard, moon on the screen](docs/nap.png) |

## Why

Long jobs such as builds, renders, test runs and scripts give no sign of
life while they work in the background. Your scripts write one line to a status file
and the kitten shows it. You can glance at the screen and know whether something is
still running.

## Install

Download `kitten-assistant.exe` from the
[latest release](https://github.com/Hailey-Ross/hails.Kitten-Assistant/releases/latest) and
put it in a folder of its own that you can write to (for example `C:\Tools\KittenAssistant`),
because it keeps its settings and updates next to the exe. The `experimental` pre-release is
the newest test build; pick it only if you want to try unfinished changes.

### Updates

A kitten from a numbered release updates itself: it checks GitHub 30 seconds after it
starts and then every 6 hours, downloads a newer stable release, checks it and restarts
with the same settings. `kitten-assistant update` checks right away. Add the line
`auto_update=0` to `kitten-assistant.ini` next to the exe to turn this off. The
`experimental` build and builds you make yourself never update themselves.

### Build it yourself

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

Put the whole status in quotes, and do not start it with `--`: `kitten-assistant "--check-assets hang"`
is read as an unknown option, not a status: an error message box opens and the call waits
until someone clicks OK. Put a word in front, for example
`kitten-assistant "Check: --check-assets hang"`.

The bubble's timer starts again whenever the text changes. Saying the same text again keeps
the count going.

The [full guide](docs/USAGE.md) covers every command and option, the status file rules,
scripts in several languages, recipes, starting with Windows and troubleshooting.

Drag the kitten anywhere with the left mouse button. It remembers the spot and opens there
next time. Right-click it for:

- **Back to corner:** back to the bottom-right of its display, and forget the dragged-to spot.
- **Monitor:** pick which display the kitten lives on. It moves there at once and starts
  there from then on (the choice is saved in `kitten-assistant.ini` next to the exe).
- **Color:** pick one of 32 coats, from tabbies and tuxedos to calico and galaxy. Patched
  coats get new patches each time you pick them. Saved in `kitten-assistant.ini`.
- **Size:** make the kitten bigger or smaller (50% to 300%), or back to the default that
  follows the display scaling. It resizes at once and keeps that size at every start (saved
  in `kitten-assistant.ini`); `--scale N` on the command line wins over it.
- **Always on top:** on (checked) keeps the kitten above other windows; off lets other
  windows cover it. Saved in `kitten-assistant.ini` too.
- **Quit.**

The menu is the kitten's own: a little card with cat ears that pops open. Monitor, Color and
Size slide in as pages with a back button, and Color shows the kitten in every coat, so you
can see a coat before you pick it. It works with the keyboard too: the arrow keys move,
Enter or Space picks, Left or Backspace goes back, and Escape goes back or closes it.
Clicking anywhere else closes it.

![The Color page of the menu: a grid of kitten portraits, one per coat, with "Russian blue" under the hovered one](docs/menu.png)

Only one kitten runs at a time, and starting a second one does nothing.

### The status file

The kitten reads the first line of a text file every 2 seconds:

| First line                 | Kitten                           |
|----------------------------|----------------------------------|
| any text                   | types, bubble shows the text     |
| `Waiting...` or `Wait...`  | waits, bubble shows the text     |
| `Failed...`, `Fail...`, `Error...` or `Problem...` | looks worried, bubble shows the text |
| `Testing...`, `Building...`, `Researching...`, `Downloading...`, `Thinking...` and the other mood words | acts it out (see the moods above), bubble shows the text |
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
| `--scale N`    | size multiplier (default: right-click > Size, else the screen DPI) |
| `--monitor N`  | start on display N and make it the default                  |
| `--x N --y N`  | start position in screen pixels (default bottom-right)      |

### Start with Windows

Press Win+R and type `shell:startup`. Add a shortcut to `kitten-assistant.exe` there,
and put any options in the shortcut's Target field.

### Snapshots

`snapshot` renders one frame to a PNG with no window. This is handy for checking art
changes and for screenshots:

```
kitten-assistant snapshot busy.png --busy "Writing release notes" --minutes 12 --bg 6a8caf
kitten-assistant snapshot testing.png --busy "Testing: regression suite" --minutes 4 --bg 6a8caf
kitten-assistant snapshot building.png --busy "Building: release 1.2" --minutes 12 --bg 6a8caf
kitten-assistant snapshot researching.png --busy "Reviewing: pull request 42" --bg 6a8caf
kitten-assistant snapshot downloading.png --busy "Downloading: Rust toolchain" --time 3.4 --bg 6a8caf
kitten-assistant snapshot thinking.png --busy "Planning: next release" --time 4.4 --bg 6a8caf
kitten-assistant snapshot waiting.png --busy "Waiting: test results" --minutes 3 --time 4.2 --bg 6a8caf
kitten-assistant snapshot problem.png --busy "Failed: release build" --bg 6a8caf
kitten-assistant snapshot done.png --done --time 3.3 --bg 6a8caf
kitten-assistant snapshot nap.png --bg 6a8caf
```

`--time SECONDS` picks the animation moment. `--scale N` sets the size (default 2).
`--screen N` picks the typing screen (0 code, 1 progress bars, 2 equalizer, 3 gear, 4 graph).
`--sip` draws a coffee sip, `--coat NAME` picks a color (for example `--coat "Gray tabby"`) and
`--seed N` moves the patches on patched coats. Leave out `--bg` for a transparent PNG.

## How it works

- The art is drawn in code every frame with [tiny-skia](https://crates.io/crates/tiny-skia),
  using flat shapes with a soft outline. There are no image files. Text uses
  [fontdue](https://crates.io/crates/fontdue) with Segoe UI from the Windows font folder
  (Arial if Segoe UI is missing).
- The frame goes to Windows as a per-pixel-alpha layered window (`UpdateLayeredWindow`).
  That gives the kitten soft edges on any wallpaper, and clicks pass through the clear
  parts.
- The window stays on top (unless you turn that off), never takes focus and has no taskbar button. It is built to stay
  light: 30 fps while busy and 15 fps while napping at the lowest CPU priority in Windows'
  efficiency mode (about 1 ms of drawing a frame), only the part of the picture that changed
  is handed to Windows. It keeps animating whatever has the focus, games included.
- Ordinary screenshot tools may not capture layered windows. Use `snapshot` for
  pictures.

Errors are written to `%TEMP%\kitten-assistant.log`, except a mistyped option, which only
shows its message box.

## Layout

```
src/main.rs     command line, window, timers, layered-window output
src/scene.rs    the kitten, desk, computer, keyboard and their animation
src/canvas.rs   drawing helpers (shapes, outline, text, word wrap)
src/update.rs   the self-updater (newest stable release, SHA-256 check, swap and restart)
.github/        the build and release workflow
docs/USAGE.md   the full guide
skills/         a ready-made skill for using the kitten with an AI assistant
```

## License

MIT. See [LICENSE](LICENSE).
