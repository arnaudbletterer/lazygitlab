# CI/CD & Job Logs

Monitor, debug, and manage GitLab CI/CD pipelines, stages, and live streaming job logs directly inside your terminal.

![CI/CD Pipelines](../assets/screenshots/pipelines.svg#only-dark)
![CI/CD Pipelines (Light)](../assets/screenshots/pipelines-light.svg#only-light)
*Click screenshot to expand in full-resolution lightbox.*

---

## 🚀 Pipelines Overview (<kbd>g</kbd> <kbd>p</kbd>)

Track pipeline status across branches, tags, and merge requests with real-time status indicators (`[ ✔ PASSED ]`, `[ ✖ FAILED ]`, `[ ⏳ RUNNING ]`, `[ ⏸ MANUAL ]`).

### Interactive Pipeline Control Journey

=== "Step 1: Monitor Status & Stages"

    Jump to the Pipelines view with <kbd>g</kbd> <kbd>p</kbd>:

    1. Use <kbd>j</kbd> and <kbd>k</kbd> to move through recent pipeline runs.
    2. Press <kbd>/</kbd> to filter pipelines by branch name, commit SHA, or author.
    3. Review the stage breakdown pills (*build*, *test*, *security*, *deploy*) in the detail pane.
    4. Press <kbd>Tab</kbd> to focus the jobs list within the selected pipeline.

=== "Step 2: Trigger & Control Runs"

    Manage execution of pipelines and jobs on the fly:

    1. Press <kbd>n</kbd> to open the **New Pipeline Dialog**. Select the target branch/tag and add custom environment variables if needed.
    2. Press <kbd>R</kbd> to retry all failed jobs in the active pipeline.
    3. Press <kbd>C</kbd> to abort and cancel a running pipeline.
    4. Highlight an individual job and press <kbd>r</kbd> to retry only that specific job.

=== "Step 3: Schedules & CI Variables"

    Inspect automated schedules and configuration secrets:

    1. Press <kbd>S</kbd> to open the **Pipeline Schedules** manager. Inspect cron expressions, next run times, and press <kbd>Enter</kbd> to trigger an immediate run.
    2. Press <kbd>v</kbd> to open the **CI Variables** inspector to view project environment keys, protected flags, and masked status.

---

### Pipeline Management Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>n</kbd> | **Launch Pipeline** | Trigger new pipeline modal with ref and variable options |
| <kbd>R</kbd> | **Retry Pipeline** | Re-run all failed jobs in selected pipeline |
| <kbd>C</kbd> | **Cancel Pipeline** | Abort active running pipeline |
| <kbd>S</kbd> | **Schedules** | Open scheduled pipelines view (press <kbd>Enter</kbd> to execute) |
| <kbd>v</kbd> | **CI Variables** | Inspect and manage project CI/CD environment variables |
| <kbd>/</kbd> | **Filter Pipelines** | Search by branch ref, author, or commit message |
| <kbd>Enter</kbd> | **Drill into Jobs** | Expand stages and inspect individual jobs |
| <kbd>o</kbd> | **Open in Browser** | Open pipeline URL in your default browser |

---

## 📜 Live Job Log Viewer (<kbd>Enter</kbd> on Job)

Drill into any job to view colorized terminal logs with live streaming and ANSI formatting.

![CI/CD Live Job Log Viewer](../assets/screenshots/job_log.svg#only-dark)
![CI/CD Live Job Log Viewer (Light)](../assets/screenshots/job_log-light.svg#only-light)
*Live streaming job log viewer with ANSI color support and runner section folding.*

### Interactive Log Inspection Workflow

=== "Step 1: Real-Time Streaming & Navigation"

    Press <kbd>Enter</kbd> on any job in the pipeline detail pane:

    1. The log viewer opens immediately, streaming live stdout/stderr from the GitLab runner.
    2. Full ANSI color codes (bold, underline, 256 colors) render faithfully.
    3. Auto-follow follows the tail of the log in real time.
    4. Scroll manually with <kbd>j</kbd> / <kbd>k</kbd>, <kbd>PgUp</kbd> / <kbd>PgDn</kbd>, or jump to start/end with <kbd>Home</kbd> / <kbd>G</kbd>.

=== "Step 2: Section Collapsing"

    Isolate build failures instantly by collapsing noise:

    1. GitLab runner groups log outputs into collapsible sections (e.g. `prepare_executor`, `get_sources`, `restore_cache`, `step_script`).
    2. Highlight any section header and press <kbd>Tab</kbd> or <kbd>l</kbd> / <kbd>Space</kbd> to collapse or expand it.
    3. Collapsing boilerplate setup steps lets you jump straight to the relevant failure stacktrace.

=== "Step 3: Search & Snippet Extraction"

    Find errors and export logs without leaving your terminal:

    1. Press <kbd>/</kbd> to start an in-log search for keywords (e.g. `FAIL`, `error:`, `panic`).
    2. Press <kbd>Enter</kbd> or <kbd>n</kbd> to jump to the next occurrence (<kbd>N</kbd> for previous).
    3. Hold <kbd>Shift</kbd> + <kbd>j</kbd>/<kbd>k</kbd> to highlight a range of lines.
    4. Press <kbd>y</kbd> to copy the selected log snippet (or the full raw log) to the system clipboard.

---

### Job Log Viewer Shortcuts

| Shortcut | Action | Description |
| :--- | :--- | :--- |
| <kbd>j</kbd> / <kbd>k</kbd> | **Scroll** | Scroll up / down line by line |
| <kbd>PgUp</kbd> / <kbd>PgDn</kbd> | **Page Scroll** | Scroll up / down half a page |
| <kbd>Home</kbd> / <kbd>G</kbd> | **Start / End** | Jump to the very beginning or end of log |
| <kbd>Tab</kbd> / <kbd>l</kbd> | **Toggle Section** | Collapse or expand runner step section |
| <kbd>/</kbd> | **Search Log** | Search for text or regex pattern in log |
| <kbd>n</kbd> / <kbd>N</kbd> | **Next / Prev Match** | Cycle through search matches |
| <kbd>Shift+j/k</kbd> | **Select Range** | Select multiple log lines |
| <kbd>y</kbd> | **Copy to Clipboard** | Copy selected lines or raw log output |
| <kbd>Esc</kbd> / <kbd>q</kbd> | **Close Log** | Return to pipeline job list |

!!! tip "Fast Failure Debugging"
    When a build fails, open the job log with <kbd>Enter</kbd>, press <kbd>G</kbd> to jump to the end of the output, then press <kbd>/</kbd> and type `error` to jump straight to the root cause.

---

## 📚 Related Guides

* **[Code Review & Diffs](code-review.md)**: Inspect head pipeline results directly on Merge Requests (<kbd>P</kbd>).
* **[Personal Dashboard](dashboard.md)**: Monitor cross-project pipelines on your home screen.
* **[Keybindings & Cheatsheet](../reference/keybindings.md)**: Full shortcut reference.
