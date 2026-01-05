# Manual Testing Guide - Tier 1 Apps

This document provides manual test cases for the TUI Suite workspace and Tier 1 applications.

---

## Table of Contents

1. [Workspace Tests](#workspace-tests)
2. [Habit Tracker Tests](#habit-tracker-tests)
3. [Flashcard Trainer Tests](#flashcard-trainer-tests)
4. [Time Tracker Tests](#time-tracker-tests)

---

## Workspace Tests

### TEST-WS-001: Workspace Build
**Command:**
```bash
cargo build --workspace
```

**Rationale:** Verify that the entire workspace compiles without errors. This catches dependency issues, type errors, and syntax problems across all crates and apps.

**Steps:**
1. Open terminal in the TUI project root directory
2. Run `cargo build --workspace`
3. Wait for compilation to complete

**Expected Output:**
- Build completes without errors
- All packages compile successfully
- Warning count should be noted (not necessarily zero, but no errors)
- Final line shows: `Finished dev [unoptimized + debuginfo] target(s) in X.XXs`

---

### TEST-WS-002: Workspace Check
**Command:**
```bash
cargo check --workspace
```

**Rationale:** Faster than a full build, this verifies type correctness without producing binaries. Useful for quick validation.

**Steps:**
1. Open terminal in the TUI project root directory
2. Run `cargo check --workspace`
3. Wait for check to complete

**Expected Output:**
- Check completes without errors
- Final line shows: `Finished dev [unoptimized + debuginfo] target(s) in X.XXs`

---

### TEST-WS-003: Individual App Build
**Command:**
```bash
cargo build -p habit-tracker
cargo build -p flashcard-trainer
cargo build -p time-tracker
```

**Rationale:** Verify that each Tier 1 app can be built independently.

**Steps:**
1. Run each command separately
2. Verify each completes successfully

**Expected Output:**
- Each app builds without errors
- Binary is created in `target/debug/<app-name>`

---

### TEST-WS-004: Code Formatting Check
**Command:**
```bash
cargo fmt --all -- --check
```

**Rationale:** Ensure code follows consistent Rust formatting standards.

**Steps:**
1. Run the command
2. Check output

**Expected Output:**
- No output if all files are formatted correctly
- If files need formatting, they will be listed

---

### TEST-WS-005: Lint Check
**Command:**
```bash
cargo clippy --workspace
```

**Rationale:** Catch common programming mistakes and ensure idiomatic Rust code.

**Steps:**
1. Run the command
2. Review any warnings

**Expected Output:**
- Clippy runs on all packages
- Note any warnings (ideally zero errors)

---

## Habit Tracker Tests

### TEST-HT-001: Application Launch
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify the application starts and displays the main UI correctly.

**Steps:**
1. Run the command
2. Observe the terminal

**Expected Output:**
- Terminal enters alternate screen mode (full screen TUI)
- Main UI displays with:
  - Header showing "Daily Habits" and current date
  - Empty habit list (or existing habits if database exists)
  - Status bar at bottom with keybindings hint
- No error messages or crashes

---

### TEST-HT-002: Create New Habit
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify users can create new habits.

**Steps:**
1. Launch habit-tracker
2. Press `a` to add a new habit
3. Type "Drink 8 glasses of water"
4. Press `Enter` to confirm

**Expected Output:**
- Input field appears for habit name
- Text is displayed as you type
- After Enter, habit appears in the list
- Success message "Habit created" briefly displays

---

### TEST-HT-003: Toggle Habit Completion
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify users can mark habits as complete/incomplete.

**Steps:**
1. Launch habit-tracker with at least one habit
2. Use `j`/`k` to select a habit
3. Press `Space` or `Enter` to toggle completion
4. Press `Space` again to toggle back

**Expected Output:**
- First toggle: Habit shows as completed (checkmark or filled indicator)
- Message shows "[habit name] completed"
- Second toggle: Habit shows as incomplete
- Message shows "[habit name] uncompleted"

---

### TEST-HT-004: Navigate Between Days
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify date navigation works correctly.

**Steps:**
1. Launch habit-tracker
2. Note the current date in the header
3. Press `h` or `Left Arrow` to go to previous day
4. Press `l` or `Right Arrow` to go to next day
5. Press `t` to return to today

**Expected Output:**
- Date in header changes with each navigation
- `h`/`Left`: Date goes back one day
- `l`/`Right`: Date goes forward one day
- `t`: Returns to current date
- Habit entries update for each date

---

### TEST-HT-005: View Switching
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify all views are accessible and display correctly.

**Steps:**
1. Launch habit-tracker
2. Press `1` for Daily view
3. Press `c` for Calendar view
4. Press `r` for Streaks view
5. Press `s` for Stats view
6. Press `1` to return to Daily view

**Expected Output:**
- `1`: Daily habit list view
- `c`: Calendar heatmap showing completion history
- `r`: Streaks view showing current/longest streaks
- `s`: Statistics view showing completion rates
- Each view has appropriate title in header

---

### TEST-HT-006: Delete Habit with Confirmation
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify deletion requires confirmation to prevent accidental data loss.

**Steps:**
1. Launch habit-tracker with at least one habit
2. Select a habit with `j`/`k`
3. Press `d` to delete
4. Press `n` to cancel
5. Press `d` again
6. Press `y` to confirm

**Expected Output:**
- Step 3: Confirmation dialog appears: "Delete '[habit]'? This cannot be undone. (y/n)"
- Step 4: Dialog closes, habit remains
- Step 6: Habit is removed from list, message "Habit deleted" appears

---

### TEST-HT-007: Help Display
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify help overlay is accessible and informative.

**Steps:**
1. Launch habit-tracker
2. Press `?` to show help
3. Read the help content
4. Press any key to close

**Expected Output:**
- Help popup appears centered on screen
- Shows all available keybindings
- Any keypress closes the popup

---

### TEST-HT-008: Quit Application
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify clean application exit.

**Steps:**
1. Launch habit-tracker
2. Press `q` to quit

**Expected Output:**
- Application exits cleanly
- Terminal returns to normal mode
- Cursor is visible
- No error messages

---

### TEST-HT-009: Database Persistence
**Command:**
```bash
cargo run -p habit-tracker
```

**Rationale:** Verify data persists between sessions.

**Steps:**
1. Launch habit-tracker
2. Create a habit "Test Persistence"
3. Mark it as complete for today
4. Press `q` to quit
5. Launch habit-tracker again

**Expected Output:**
- On relaunch, "Test Persistence" habit exists
- Today's completion status is preserved
- Database file exists at `~/.local/share/habit-tracker/habits.db`

---

## Flashcard Trainer Tests

### TEST-FC-001: Application Launch
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify the application starts correctly.

**Steps:**
1. Run the command
2. Observe the terminal

**Expected Output:**
- TUI displays deck list view
- Shows empty deck list or existing decks
- Status bar shows available actions

---

### TEST-FC-002: Create New Deck
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify users can create flashcard decks.

**Steps:**
1. Launch flashcard-trainer
2. Press `a` to add new deck
3. Type "Spanish Vocabulary"
4. Press `Enter`

**Expected Output:**
- Input field appears for deck name
- After Enter, deck appears in list
- Message "Deck created" displays

---

### TEST-FC-003: Start Study Session
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify study session initiation (requires cards in deck).

**Steps:**
1. Launch flashcard-trainer
2. Select a deck with cards
3. Press `Enter` or `Space` to start studying

**Expected Output:**
- If deck has no cards: Message "No cards to study!"
- If deck has cards: Switches to study view showing card front
- Study UI shows card content and "Press space to flip"

---

### TEST-FC-004: Flashcard Review Flow
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify the spaced repetition review flow.

**Steps:**
1. Launch with a deck containing cards
2. Start study session
3. Press `Space` to reveal answer
4. Rate with `1` (Again), `2` (Hard), `3` (Good), or `4` (Easy)
5. Continue until session ends

**Expected Output:**
- Step 3: Card flips to show back side, rating options appear
- Step 4: Next card appears (or session completion if last card)
- Ratings affect when card will appear again (SRS algorithm)

---

### TEST-FC-005: View Statistics
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify statistics view works.

**Steps:**
1. Launch flashcard-trainer
2. Press `s` to switch to stats view
3. Press `q` or `Esc` to return

**Expected Output:**
- Stats view displays deck statistics
- Shows cards due, new cards, review counts
- Can return to deck list

---

### TEST-FC-006: Card Browser
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify card browsing functionality.

**Steps:**
1. Launch flashcard-trainer
2. Press `b` to open card browser
3. Navigate with `j`/`k`
4. Press `Esc` to return

**Expected Output:**
- Card browser view displays
- Can navigate through cards
- Returns to deck list on Esc

---

### TEST-FC-007: Exit Study Session Early
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify users can exit study mid-session.

**Steps:**
1. Start a study session
2. Press `q` or `Esc` during session

**Expected Output:**
- Session ends immediately
- Returns to deck list
- Reviewed cards keep their updated schedules

---

### TEST-FC-008: Help Display
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify help is accessible.

**Steps:**
1. Launch flashcard-trainer
2. Press `?`
3. Press any key to close

**Expected Output:**
- Help popup displays keybindings
- Closes on any keypress

---

### TEST-FC-009: Quit Application
**Command:**
```bash
cargo run -p flashcard-trainer
```

**Rationale:** Verify clean exit.

**Steps:**
1. Launch flashcard-trainer
2. Press `q` (when not in study mode)

**Expected Output:**
- Application exits cleanly
- Terminal restored to normal
- Cannot quit during editing mode

---

## Time Tracker Tests

### TEST-TT-001: Application Launch
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify the application starts correctly.

**Steps:**
1. Run the command
2. Observe the terminal

**Expected Output:**
- TUI displays timer view
- Shows 00:00:00 timer (if not running)
- Status bar shows keybindings

---

### TEST-TT-002: Start/Stop Timer
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify basic timer functionality.

**Steps:**
1. Launch time-tracker
2. Press `s` to start timer
3. Wait a few seconds
4. Press `s` to stop timer

**Expected Output:**
- Step 2: Message "Timer started", timer begins counting
- Timer display updates every second (00:00:01, 00:00:02, etc.)
- Step 4: Timer stops, message shows duration "Stopped: XX:XX:XX"
- Entry is saved to database

---

### TEST-TT-003: Add Description to Running Timer
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify users can describe what they're working on.

**Steps:**
1. Launch time-tracker
2. Press `s` to start timer
3. Press `Enter` to edit description
4. Type "Working on TUI project"
5. Press `Enter` to save
6. Press `s` to stop timer

**Expected Output:**
- Step 3: Input field appears with current description
- Step 5: Description updates on the timer display
- Step 6: Entry saves with the description

---

### TEST-TT-004: View Time Entries
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify entry list view.

**Steps:**
1. Launch time-tracker (with existing entries)
2. Press `2` to switch to Entries view
3. Navigate with `j`/`k`
4. Press `1` to return to Timer view

**Expected Output:**
- Entries view shows list of time entries for current date
- Each entry shows description, duration, timestamps
- Can navigate through entries

---

### TEST-TT-005: Navigate Between Days
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify date navigation.

**Steps:**
1. Launch time-tracker
2. Press `h` or `Left` for previous day
3. Press `l` or `Right` for next day
4. Press `t` to return to today

**Expected Output:**
- Date changes in the display
- Entries update to show selected date's entries
- `t` returns to current date

---

### TEST-TT-006: Pomodoro Mode
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify Pomodoro timer integration.

**Steps:**
1. Launch time-tracker
2. Press `p` to enable Pomodoro mode
3. Observe the timer (25-minute countdown by default)
4. Press `p` again to disable

**Expected Output:**
- Step 2: Message "Pomodoro mode enabled", timer shows countdown
- Pomodoro session counts down from configured time
- Step 4: Message "Pomodoro mode disabled", timer pauses

---

### TEST-TT-007: Projects View
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify project management.

**Steps:**
1. Launch time-tracker
2. Press `P` (capital) for Projects view
3. Press `a` to add new project
4. Type "TUI Development"
5. Press `Enter`
6. Press `1` to return to Timer

**Expected Output:**
- Projects view displays list of projects
- New project input field appears on `a`
- Project is created and shown in list
- Message "Project created" displays

---

### TEST-TT-008: Reports View
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify reporting functionality.

**Steps:**
1. Launch time-tracker (with existing entries)
2. Press `r` for Reports view
3. Observe the report data

**Expected Output:**
- Reports view displays time summaries
- Shows total hours by project/day
- Provides productivity insights

---

### TEST-TT-009: Delete Time Entry
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify entry deletion.

**Steps:**
1. Launch time-tracker
2. Press `2` for Entries view
3. Select an entry with `j`/`k`
4. Press `d` to delete

**Expected Output:**
- Entry is removed from list
- Message "Entry deleted" displays

---

### TEST-TT-010: Help Display
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify help accessibility.

**Steps:**
1. Launch time-tracker
2. Press `?`
3. Press any key to close

**Expected Output:**
- Help popup shows all keybindings
- Closes on any keypress

---

### TEST-TT-011: Quit Application
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify clean exit.

**Steps:**
1. Launch time-tracker (with no running timer)
2. Press `q` to quit

**Expected Output:**
- Application exits cleanly
- Terminal restored to normal

---

### TEST-TT-012: Running Timer Persistence
**Command:**
```bash
cargo run -p time-tracker
```

**Rationale:** Verify running timers persist across restarts.

**Steps:**
1. Launch time-tracker
2. Press `s` to start timer
3. Press `q` to quit (while timer runs)
4. Relaunch time-tracker

**Expected Output:**
- On relaunch, timer should either:
  - Continue from where it left off, OR
  - Show the entry as stopped at quit time
- No time data is lost

---

## Notes

### Test Environment
- All tests assume a fresh terminal with at least 80x24 characters
- SQLite databases are created in `~/.local/share/<app-name>/`
- Configuration files are in `~/.config/<app-name>/`

### Cleanup Between Tests
To start with a fresh state:
```bash
rm -rf ~/.local/share/habit-tracker
rm -rf ~/.local/share/flashcard-trainer
rm -rf ~/.local/share/time-tracker
rm -rf ~/.config/habit-tracker
rm -rf ~/.config/flashcard-trainer
rm -rf ~/.config/time-tracker
```

### Keyboard Reference
| Key | Common Action |
|-----|---------------|
| `q` | Quit application |
| `?` | Show help |
| `j`/`k` | Navigate down/up |
| `Enter` | Confirm/select |
| `Esc` | Cancel/back |
| `a` | Add new item |
| `d` | Delete item |

---

## Test Execution Checklist

| Test ID | Test Name | Pass/Fail | Notes |
|---------|-----------|-----------|-------|
| TEST-WS-001 | Workspace Build | | |
| TEST-WS-002 | Workspace Check | | |
| TEST-WS-003 | Individual App Build | | |
| TEST-WS-004 | Code Formatting Check | | |
| TEST-WS-005 | Lint Check | | |
| TEST-HT-001 | HT: Application Launch | | |
| TEST-HT-002 | HT: Create New Habit | | |
| TEST-HT-003 | HT: Toggle Habit Completion | | |
| TEST-HT-004 | HT: Navigate Between Days | | |
| TEST-HT-005 | HT: View Switching | | |
| TEST-HT-006 | HT: Delete Habit with Confirmation | | |
| TEST-HT-007 | HT: Help Display | | |
| TEST-HT-008 | HT: Quit Application | | |
| TEST-HT-009 | HT: Database Persistence | | |
| TEST-FC-001 | FC: Application Launch | | |
| TEST-FC-002 | FC: Create New Deck | | |
| TEST-FC-003 | FC: Start Study Session | | |
| TEST-FC-004 | FC: Flashcard Review Flow | | |
| TEST-FC-005 | FC: View Statistics | | |
| TEST-FC-006 | FC: Card Browser | | |
| TEST-FC-007 | FC: Exit Study Session Early | | |
| TEST-FC-008 | FC: Help Display | | |
| TEST-FC-009 | FC: Quit Application | | |
| TEST-TT-001 | TT: Application Launch | | |
| TEST-TT-002 | TT: Start/Stop Timer | | |
| TEST-TT-003 | TT: Add Description to Running Timer | | |
| TEST-TT-004 | TT: View Time Entries | | |
| TEST-TT-005 | TT: Navigate Between Days | | |
| TEST-TT-006 | TT: Pomodoro Mode | | |
| TEST-TT-007 | TT: Projects View | | |
| TEST-TT-008 | TT: Reports View | | |
| TEST-TT-009 | TT: Delete Time Entry | | |
| TEST-TT-010 | TT: Help Display | | |
| TEST-TT-011 | TT: Quit Application | | |
| TEST-TT-012 | TT: Running Timer Persistence | | |
