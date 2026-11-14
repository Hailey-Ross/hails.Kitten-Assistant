---
name: kitten-assistant
description: Show what you are working on with Kitten Assistant, the desktop kitten that types while a job runs and naps when nothing is running. Set a short status line when a task starts, update it at each new step, and set it to idle when the work is done, fails or waits for the user. Use at the start of any multi-step or long-running task (builds, tests, research, refactors, background jobs) on a Windows machine that has kitten-assistant installed, or when the user asks to show progress on the kitten.
---

# Kitten Assistant status

Kitten Assistant (https://github.com/Hailey-Ross/hails.Kitten-Assistant) is a small Windows
desktop buddy. It reads the first line of a status file every 2 seconds: text makes the kitten
type and shows the text in a bubble with the time since it began; `idle`, empty or missing
makes it nap. Keeping that line current lets the user glance at the screen and see whether
you are still working and on what.

## Setup check (once per session)

1. **Find the exe.** Check without running it: `Get-Command kitten-assistant` (PowerShell) or
   `command -v kitten-assistant` (bash). Do not run `kitten-assistant help` to check: it is a
   windowed app, so `help` opens a message box on the user's screen and prints nothing to your
   terminal. If it is not on the `PATH`, look for `kitten-assistant.exe` in the folder the
   user named, or ask the user once where it is, then use the full path for the rest of the
   session.
2. **Find the status file the running kitten watches.** Your `say` / `idle` must write the
   same file, or the kitten never sees them. Check in this order:
   1. a path the user gave you;
   2. the `--status` option of the running kitten, read from its command line:
      `(Get-CimInstance Win32_Process -Filter "Name='kitten-assistant.exe'").CommandLine`;
   3. the `KITTEN_STATUS` environment variable;
   4. `status.txt` next to the exe (the default).

   If you found it through 1 or 2, pass `--status "<path>"` on every `say` / `idle` call.
3. **Check the kitten is running:**
   - PowerShell: `Get-Process kitten-assistant -ErrorAction SilentlyContinue`
   - bash: `tasklist //FI "IMAGENAME eq kitten-assistant.exe" | grep -i kitten`

   If it is not running, start it detached so it does not block you. It is a single instance,
   so a second start does nothing. Use the full path if it is not on the `PATH`, and add
   `--status "<path>"` if you use a custom file:
   - PowerShell: `Start-Process "<path>\kitten-assistant.exe"`
   - bash: `"<path>/kitten-assistant.exe" &` (it is a windowed app and does not hold the
     terminal). Avoid `cmd //c start "" ...` from Git Bash: the empty title argument gets
     mangled and Windows shows a "cannot find" error dialog.

   If the user has said they do not want it started for them, skip this and only write the
   status file.
4. If the exe cannot be found and the user does not answer, carry on with the task without
   the kitten. Never let the kitten block the real work.
5. The moods below need a recent kitten. `kitten-assistant update` checks for a newer release
   and installs it (the running kitten restarts on the new one); the user can also right-click
   the kitten > Check for Update.

## When to write the status

| Moment                                         | Write                                        |
|------------------------------------------------|----------------------------------------------|
| You start a task with more than a step or two  | a line naming the task, starting with the mood word that fits |
| You move to a new step or phase                | a new line for that step, with its own mood word |
| You run a build, a test, a download or a push  | the matching mood (`Building: ...`, `Testing: ...`, `Downloading: ...`, `Pushing: ...`) right before it, and a new line after it with what came out |
| You wait on something outside your hands (CI, another machine, a review) | a line starting with `Waiting`, for example `Waiting: CI on main` |
| You finish and hand back to the user           | `idle` (the kitten cheers for a moment, then naps) |
| You stop because something failed and the user has to look | a line starting with `Failed`, for example `Failed: release build` |
| You ask the user a question and wait for the answer | `idle` (the kitten should not look busy while you are waiting on them) |

### Moods: start every line with the word that fits

The first word of the line picks what the kitten acts out (any case; the word ends at the first
character that is not a letter, so `Building:`, `building...` and `BUILDING the app` all count).
Lead with it on every line, so the user sees what kind of work this is at a glance:

| What you are doing | Start the line with | The kitten |
|--------------------|---------------------|------------|
| Reading code, logs, docs; searching; analyzing | `Reading`, `Reviewing`, `Researching`, `Analyzing`, `Searching`, `Investigating`, `Studying`, `Exploring`, `Digging` | reading glasses, paws folded, a page scrolls by with highlights |
| Designing or planning a change | `Planning`, `Designing`, `Thinking`, `Brainstorming`, `Pondering` | draws on a whiteboard beside the desk |
| Building or compiling | `Building`, `Compiling`, `Rebuilding`, `Linking`, `Packaging`, `Bundling` | types in a hard hat while a brick wall goes up |
| Running tests, checks, a game or app run | `Testing`, `Checking`, `Verifying`, `Playtesting`, `QA`, `Regression`, `Smoke` | plays on a gamepad, jots notes on a notepad between rounds |
| Downloading, fetching, pulling, installing, syncing, copying | `Downloading`, `Fetching`, `Pulling`, `Installing`, `Syncing`, `Copying` | files fly down from a cloud |
| Pushing, uploading, publishing, deploying, backing up | `Pushing`, `Uploading`, `Publishing`, `Deploying`, `Backing` | files fly up to a cloud |
| Waiting on something you do not drive | `Waiting`, `Wait` | chin on a paw, drumming, an hourglass on the screen |
| A failure the user needs to see | `Failed`, `Fail`, `Error`, `Problem` | drooping ears, a red X |
| Writing code or docs, anything else | any other word | types away |

Rules for the moods:
- `Waiting` only for things outside your hands: CI, another machine, a review, a background run
  you are not driving. Your own build is `Building`, your own test run is `Testing`.
- One line per tool call that changes the kind of work: a build, a test run and the reading of
  its results are three lines, not one line left up through all of them.
- `Failed` only for a failure the user needs to see; a step that failed and that you then fixed
  yourself is just your next step.
- Never start a line with `--`: the exe takes it as an option and shows a message box on the
  user's screen that waits for a click.

Always end your turn with `idle`, or with the `Failed: ...` line when the turn ends on a
failure, even when a background build or test of yours is still running. A kitten that keeps
typing, building or waiting after you stopped tells the user something false. Your next task's
first line replaces a failed line.

The kitten also takes little breaks on its own (a sip of coffee, a look around, now and then a
stretch, a yawn or a coffee refill); they need nothing from you, and a new line that comes in
during one shows once it ends.

Commands (add `--status "<path>"` if the setup check found one):

```
kitten-assistant say "Fixing login timeout: running tests"
kitten-assistant idle
```

**In PowerShell, pipe each call to `Out-Null`:**

```powershell
kitten-assistant say "Fixing login timeout: running tests" | Out-Null
```

PowerShell does not wait for a windowed app, so without it the call returns before the file is
written; a check or a second write straight after it can race the first. Bash, Python and
other shells wait normally.

Prefer the exe's `say` / `idle` over writing the file yourself; it writes plain text with no
encoding surprises. If you write the file directly, write only the line (UTF-8, a byte order
mark is fine) and replace the file, do not append.

## How to write the line

- **Short:** about 50 characters. The bubble shows two lines of about 35 characters and wraps
  at word breaks, then cuts the rest with "…"; past about 65 characters the end is usually lost.
- **Most useful words first:** `Tests: 3 of 12 suites` beats `Currently running the test suites`.
- **Plain words the user recognizes:** the task they asked for, the step you are on. No
  internal ids or tool names unless the user uses them too.
- **Progress when you know it:** `Refactor: file 4 of 9`, `Build 2 of 3 targets`.
- **Change the text for each step:** the bubble timer restarts when the text changes, so each
  step shows its own time. Rewriting the same text keeps the timer running; do that while one
  long step is still going. The same text left over from an earlier job also keeps counting,
  so start a new job with a line that differs from the last one.
- **Nothing private:** the line sits on the user's screen where others may see it. No
  passwords, tokens, personal data or full private paths.
- **Don't spam:** one write per real step change is enough. Updates faster than every 2 seconds
  are not seen anyway.

Examples:

```
kitten-assistant say "Upgrading dependencies"
kitten-assistant say "Upgrade: fixing 3 compile errors"
kitten-assistant say "Upgrade: running test suite"
kitten-assistant idle
```

## Several assistants or jobs

There is one kitten and one line. If more than one assistant or job could write to it, only
the one the user started most recently (or the one the user named) owns it. Do not overwrite a
line you did not write unless the user asked you to take over. Remember the last line you
wrote. When you are done, read the file first and set `idle` only if it still holds that line;
if it holds another job's line, leave it.

A line can also be left over from a job that has ended, for example an earlier session that
stopped without writing `idle`. If you are not sure it is still live, ask the user before
clearing it.

## Cleanup

- Leave the kitten running when you finish; the user decides when to close it
  (`kitten-assistant quit`).
- Do not leave temporary status files behind if you created a custom one only for a test. A
  `say` without `--status` writes `status.txt` next to the exe, so a stray one there usually
  means a call missed the option.
