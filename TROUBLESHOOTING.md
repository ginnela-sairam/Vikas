# VIKAS Troubleshooting — real faults, real fixes

Every failure below happened to a **real friend** running VIKAS on WSL.
Each one is now fixed in the tool itself where possible — the fix commands
are printed here for older binaries and edge cases.

**First move, always:** `vikas doctor` — it reports binary freshness, PATH,
the WSL browser bridge and git/cargo state in seconds.
**Second move:** `vikas update` — pull + rebuild + reinstall in one command.

---

## 1. "The GUI didn't open after I pulled new code"

**What really happened:** you ran `git pull`, but the installed binary at
`~/.local/bin/vikas` was still the **old compiled binary**. Pulling source
files never updates a compiled program — it must be rebuilt. The old binary
predates the WSL browser bridge, so it only printed a path.

**What VIKAS does now:** the binary stamps itself with the commit it was
built from. Any command warns immediately:

```
[VIKAS] BINARY IS STALE: built from commit 0bee045, but the checkout at
        /home/you/Vikas is now at c971f2a.
[VIKAS] fix: `vikas update`   (or: cd /home/you/Vikas && bash install_linux.sh)
```

**Fix (any binary):**

```bash
cd ~/Vikas && bash install_linux.sh    # rebuild + reinstall
# or, on any build from this commit onward:
vikas update
```

---

## 2. "`destination path 'Vikas' already exists`"

**Cause:** `git clone` is only for the **first** download. Running it again
in (or near) an existing copy fails by design.

**Fix:** update the copy you already have:

```bash
cd ~/Vikas
git pull
```

---

## 3. "`Your local changes would be overwritten by merge`"

**Cause:** earlier runs created files git considers changes (e.g. an old
test-matrix run wrote into `qa_results/`; the matrix now writes to the
ignored `.vikas_matrix/` instead, so this should not recur).

**Fix:**

```bash
git stash        # park your local changes safely
git pull
git stash pop    # bring them back (optional)
# or just discard them if they're only generated artifacts:
git checkout -- .
```

---

## 4. `vikas gui` printed a path but no browser window (WSL)

**Cause:** WSL is Linux *inside* Windows — it has no desktop of its own.
Opening a browser requires bridging out to Windows (`wslpath -w` +
`explorer.exe`). Recent builds do this automatically; older binaries don't.

**Fix (works with any build):**

```bash
explorer.exe vikas_gui.html
# or from anywhere:
explorer.exe "$(wslpath -w $(pwd)/vikas_gui.html)"
```

If no window pops up with a **current** build, run `vikas doctor` and check
the `platform` line — it lists exactly which bridges (wslview → powershell.exe
→ cmd.exe → explorer.exe) it can see.

---

## 5. "This site can't be reached — `localhost:...`"

**Cause:** `vikas gui` is **not a web server**. It writes one fully
self-contained HTML *file* (all CSS, JS and waveform data embedded) and opens
it directly. There is no port, no localhost, no internet required.

**Fix:** open the **file** — double-click `vikas_gui.html` in Windows File
Explorer, or `explorer.exe vikas_gui.html` from WSL. The page's sidebar
shows the build stamp and the note *"open directly; this is NOT a localhost
page"*.

---

## 6. "`command not found: vikas`"

**Cause:** `~/.local/bin` is not on PATH in the *current* shell (the
installer wires it into `.bashrc`, but shells do not re-read rc files
mid-session).

**Fix:**

```bash
export PATH="$HOME/.local/bin:$PATH"    # right now
# or simply open a new terminal
```

`vikas doctor` shows the exact state of PATH for you.

---

## 7. "`Permission denied: ./install_linux.sh`"

**Cause:** a push from Windows stripped the Linux executable bit
(`git add -A` on NTFS cannot store `+x`). A CI job now catches this within
seconds of any future push, so fresh clones are protected.

**Fix (works whatever the bits say):**

```bash
bash install_linux.sh
```

---

## 8. Everyday workflow (the whole thing, three commands)

```bash
cd ~/Vikas
vikas update                                   # git pull + rebuild + reinstall
vikas run examples/demo_riscv_alu_hazard.sv    # simulate
vikas gui                                      # browser pops open (WSL-aware)
```

If anything ever feels wrong: `vikas doctor` diagnoses, `vikas update` heals,
and `vikas version` shows exactly which commit your binary was built from.
