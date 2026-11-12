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

1. Find the exe. Try `kitten-assistant help` (on the PATH). If that fails, look for
   `kitten-assistant.exe` in the folder the user named, or ask the user once where it is,
   then use the full path for the rest of the session.
2. Find the status file. If `KITTEN_STATUS` is set, use it. Otherwise the default is
   `status.txt` next to the exe, which `kitten-assistant say` / `idle` use automatically. If
   the user gave a path, pass it as `--status <path>` on every call.
3. Check the kitten is running:
   - PowerShell: `Get-Process kitten-assistant -ErrorAction SilentlyContinue`
   - bash: `tasklist //FI "IMAGENAME eq kitten-assistant.exe" | grep -i kitten`

   If it is not running, start it detached so it does not block you (it is a single
   instance; a second start does nothing):
   - PowerShell: `Start-Process kitten-assistant`
   - bash: `cmd //c start "" kitten-assistant`

   If the user has said they do not want it started for them, skip this and only write the
   status file.
4. If the exe cannot be found and the user does not answer, carry on with the task without
   the kitten. Never let the kitten block the real work.

## When to write the status

| Moment                                         | Write                                        |
|------------------------------------------------|----------------------------------------------|
| You start a task with more than a step or two  | a line naming the task                       |
| You move to a new step or phase                | a new line for that step                     |
| You wait on something long (build, test, download) | what you wait for, for example `Waiting: release build` |
| You finish, stop on an error, or hand back to the user | `idle`                               |
| You ask the user a question and wait for the answer | `idle` (the kitten should not look busy while you are waiting on them) |

Always end with `idle`. A kitten that keeps typing after you stopped tells the user something
false. If a step fails, still set `idle` before you report the failure.

Commands (add `--status <path>` if you found one in the setup check):

```
kitten-assistant say "Fixing login timeout: running tests"
kitten-assistant idle
```

Prefer the exe's `say` / `idle` over writing the file yourself; it writes plain text with no
encoding surprises. If you write the file directly, write only the line (UTF-8, a byte order
mark is fine) and replace the file, do not append.

## How to write the line

- **Short:** about 50 characters. The bubble shows two lines and cuts the rest with "…".
- **Most useful words first:** `Tests: 3 of 12 suites` beats `Currently running the test suites`.
- **Plain words the user recognises:** the task they asked for, the step you are on. No
  internal ids or tool names unless the user uses them too.
- **Progress when you know it:** `Refactor: file 4 of 9`, `Build 2 of 3 targets`.
- **Change the text for each step:** the bubble timer restarts when the text changes, so each
  step shows its own time. Rewriting the same text keeps the timer running; do that while one
  long step is still going.
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
line you did not write unless the user asked you to take over. When you are done, set `idle`
only if the line is still yours (read the file first; if it holds another job's line, leave
it).

## Cleanup

- Leave the kitten running when you finish; the user decides when to close it
  (`kitten-assistant quit`).
- Do not leave temporary status files behind if you created a custom one only for a test.
