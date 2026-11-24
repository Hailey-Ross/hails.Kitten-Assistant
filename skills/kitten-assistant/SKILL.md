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

**Every `say` and `idle` you run carries `--session "<your name>"`** (step 4). Without it, all
agents write the same shared line and overwrite each other: the user sees one status while two
agents work, and one agent's `idle` hides the other's work.

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

   A recent kitten starts itself on the first `say` when none is running (`idle` does not), so
   this step is only needed for an older one. If it is not running, start it detached so it
   does not block you. It is a single instance, so a second start does nothing. Use the full
   path if it is not on the `PATH`, and add `--status "<path>"` if you use a custom file:
   - PowerShell: `Start-Process "<path>\kitten-assistant.exe"`
   - bash: `"<path>/kitten-assistant.exe" &` (it is a windowed app and does not hold the
     terminal). Avoid `cmd //c start "" ...` from Git Bash: the empty title argument gets
     mangled and Windows shows a "cannot find" error dialog.

   If the user has said they do not want it started for them, skip this and only write the
   status file.
4. **Name your session (required).** Several agents or programs share one kitten, each on its
   own line. Pick one name now and pass it as `--session "<name>"` on **every** `say` and
   `idle` for the rest of the conversation, sub-agents and background watchers included:
   - **Unique to you:** the project plus what you are doing (`--session "skate cas"`,
     `--session "kitten updater"`). Two agents in the same project need different names; if
     `kitten-assistant status` already shows your name in brackets from another agent, pick
     another.
   - **Stable:** the same name all conversation long, so each `say` replaces your own line
     and your final `idle` removes it. A new name per step leaves stale lines behind.
   - **Everywhere:** a script or watcher you start sets `KITTEN_SESSION` or passes the flag
     too; give a watcher its own name (`"skate cas build"`).

   The kitten shows the line that matters most (a `Failed` line first, then work, then
   `Waiting`, the newest of those), adds "+N more" for the others, and naps only when every
   session is idle. `idle` with a session removes just that line. `kitten-assistant status`
   prints every busy line, each with its `[name]`; a line without a name came from a call
   that left the flag out. Lines untouched for 3 hours are dropped as left over. A kitten from
   before sessions answers `--session` with an "unknown option" message box: update it first
   (step 6), and leave the option out only if it cannot be updated.
5. If the exe cannot be found and the user does not answer, carry on with the task without
   the kitten. Never let the kitten block the real work.
6. The moods below need a recent kitten. `kitten-assistant update` checks for a newer release
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
| You ask the user a question and wait for the answer (an approval, a choice, missing information) | `Question: <what you need>` (the kitten raises a paw; the line shows ahead of work lines). On a kitten without the Question mood, `idle` |

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

Always end your turn with `idle`, with the `Failed: ...` line when the turn ends on a
failure, or with a `Question: ...` line when the turn ends waiting on the user's answer, even when
a background build or test of yours is still running. A turn that only reports finished work ends
with `idle`. A kitten that keeps
typing, building or waiting after you stopped tells the user something false. Your next task's
first line replaces a failed line.

The kitten also takes little breaks on its own (a sip of coffee, a look around, now and then a
stretch, a yawn or a coffee refill); they need nothing from you, and a new line that comes in
during one shows once it ends.

Commands (always with your `--session`; add `--status "<path>"` if the setup check found one):

```
kitten-assistant say "Fixing login timeout: running tests" --session "webapp login"
kitten-assistant idle --session "webapp login"
kitten-assistant status
```

`status` prints the line the kitten shows now, then every other busy line with its session
name (`idle` while it naps). In PowerShell pipe it on (`kitten-assistant status | Write-Output`);
a plain `$x = kitten-assistant status` gets nothing.

**In PowerShell, pipe each call to `Out-Null`:**

```powershell
kitten-assistant say "Fixing login timeout: running tests" --session "webapp login" | Out-Null
```

PowerShell does not wait for a windowed app, so without it the call returns before the file is
written; a check or a second write straight after it can race the first. Bash, Python and
other shells wait normally.

Prefer the exe's `say` / `idle` over writing the file yourself; it writes plain text with no
encoding surprises. If you write the file directly, write only the line (UTF-8, a byte order
mark is fine) and replace the file, do not append.

## How to write the line

- **Specific, so the user knows exactly where you are:** which part of the plan, which item of
  how many, its name, the stage within it, and live counts:
  `Building: part 1, game 5 of 6 (Ultra Sun), compiling 277/574 (48%)` beats
  `Waiting: part 1, building`. Drop filler words, not facts. The bubble shows two lines of about
  35 characters and wraps at word breaks; longer text may overfill, then is cut with "…", so
  keep the counts within the first 65 characters or so.
- **Long background jobs get a watcher.** For a job over a few minutes that you are not stepping
  through yourself, start a small hidden script that reads the job's real progress (its log, its
  output folder, its process) every ~15 seconds and writes that line with its own `--session`,
  so its work line outranks a `Waiting` line from the same job. Stop it when the job ends. In
  PowerShell, quote the script path inside the arguments
  (`Start-Process powershell -ArgumentList '-File', "`"$path`""`): a path with a space, passed
  unquoted, never runs.
- **Most useful words first:** `Tests: 3 of 12 suites` beats `Currently running the test suites`.
- **Plain words the user recognizes:** the task they asked for, the step you are on. No
  internal ids or tool names unless the user uses them too.
- **Progress when you know it:** `Refactor: file 4 of 9`, `Build 2 of 3 targets`. A recent kitten
  also draws a bar: keep the text and add `--progress 4/9` (or `45%`); the timer keeps counting
  because the text did not change.
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
kitten-assistant say "Upgrading dependencies" --session "api upgrade"
kitten-assistant say "Upgrade: fixing 3 compile errors" --session "api upgrade"
kitten-assistant say "Upgrade: running test suite" --session "api upgrade"
kitten-assistant idle --session "api upgrade"
```

### Hook commands (a recent kitten)

These print their errors and exit with 1 (never a message box). An older kitten does not know
them: it opens an "unknown option" message box for the new options (which blocks the call until
someone clicks), and takes `do`, `wrap` or `sessions` as a status line. Do not probe for them by
trying one; use them only when the user says their kitten has them (or ask once).

- `say ... --progress 3/10`: a progress bar in the bubble.
- `say ... --for 10m`: the line clears itself after that time. Good for a watcher's line, so a
  watcher that dies cannot leave it up.
- `wrap --session "<name>" --line "Building: release" -- cargo build --release`: the line is up
  while the program runs, then `idle` on exit code 0 or `Failed: release (exit N)`; it exits
  with the program's code. In PowerShell add `| Out-Default` so it waits and keeps the output.
- `status --json`: every busy line with its session, mood, age and progress (the shown one first).
- `do cheer "Tests pass"`: a cheer with that text. Use it sparingly, for a real milestone;
  `idle --session "<name>" --cheer "Tests pass"` ends your line with it in one call.
- `say ... --steps "plan,edit,build,test" --step 2`: a checklist on the screen and "step 2 of 4";
  later `--step 3` alone keeps the steps.
- `say "Unit tests: 40 of 120" --mood testing`: the mood without changing the first word.
- `watch --session "<name> build" --log build.log --tail --background`: a built-in watcher for a
  long job's log (progress from `3/10` or `45%` in it), in place of a hand-written watcher
  script. It stops when you send `idle` for its session, or with `--pid N` when that process
  ends. `wrap --pid N` waits on a running process the same way.
- `Question:` lines raise the kitten's paw and show ahead of work; `--attention` makes the bubble
  hop until clicked and `--open URL|PATH` opens something on that click (for example the PR or
  the plan you want them to look at).
- Never run `sessions clear`: it ends every other agent's line too.

## Several assistants or jobs

There is one kitten with one line per session. Your `--session` line is yours alone: write and
idle it freely. Never write or idle another session's line, and never write without
`--session` (that is the shared line, which belongs to plain scripts and older tools). If you
find your own work on the shared line because an earlier call left the flag out, set it to
`idle` once you have moved to your session line, but only if it still holds your text.

A line can also be left over from a job that has ended, for example an earlier session that
stopped without writing `idle`. If you are not sure it is still live, ask the user before
clearing it.

## Cleanup

- Leave the kitten running when you finish; the user decides when to close it
  (`kitten-assistant quit`).
- Do not leave temporary status files behind if you created a custom one only for a test. A
  `say` without `--status` writes `status.txt` next to the exe, so a stray one there usually
  means a call missed the option.
