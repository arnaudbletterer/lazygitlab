# CI/CD & Job Logs

Monitor, debug, and manage your GitLab CI/CD pipelines, stages, and jobs with live terminal output and search.

---

## 🚀 Pipelines Overview (`g p`)

Track pipeline progress across branches and merge requests with status pills (`[ ✔ PASSED ]`, `[ ✖ FAILED ]`, `[ ⏳ RUNNING ]`).

![CI/CD Pipelines](../assets/screenshots/pipelines.svg#only-dark)
![CI/CD Pipelines (Light)](../assets/screenshots/pipelines-light.svg#only-light)
*Click screenshot to expand in lightbox.*

### Pipeline Operations:

* **Trigger New Pipeline**: Press <kbd>n</kbd> to launch a pipeline on any branch or ref, with optional CI environment variables.
* **Retry Pipeline / Job**: Press <kbd>r</kbd> on a failed pipeline or job to retry execution.
* **Cancel Running Pipeline**: Press <kbd>c</kbd> to cancel a currently active pipeline run.
* **Pipeline Schedules**: Press <kbd>S</kbd> to open and trigger configured pipeline schedules.
* **CI Variables Management**: Press <kbd>V</kbd> to view and manage project CI/CD variables directly.

---

## 📜 Live Job Log Viewer (`Enter` on Job)

Drill into any job to view colorized terminal logs in real-time.

![CI/CD Live Job Log Viewer](../assets/screenshots/job_log.svg#only-dark)
![CI/CD Live Job Log Viewer (Light)](../assets/screenshots/job_log-light.svg#only-light)
*Click screenshot to expand in lightbox.*

### Job Log Features:

* **Real-time ANSI Colors**: Full terminal escape code parsing (bold, underline, 256-color palettes).
* **Section Collapsing**: Collapse or expand collapsible runner sections (`section_start`, `section_end`) with <kbd>Space</kbd> or <kbd>Tab</kbd>.
* **Search & Highlights**: Press <kbd>/</kbd> to search within the log; jump to matches instantly with <kbd>n</kbd> and <kbd>N</kbd>.
* **Auto-Follow**: Follow live logs in real time as new runner output streams in.
* **Copy Log**: Press <kbd>y</kbd> to copy the raw job log to system clipboard.

---

💡 *For the complete keymap list, see the [Keybindings & Cheatsheet](../reference/keybindings.md).*
