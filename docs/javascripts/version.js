document.addEventListener("DOMContentLoaded", function () {
  // Determine relative root from current page URL
  const path = window.location.pathname.replace(/\/$/, "");
  const segments = path.split("/").filter(Boolean);
  // If at root or index, 0 steps up, else N steps
  const depth = segments.length;
  let root = "./";
  if (depth > 0) {
    root = "../".repeat(depth);
  }

  fetch(root + "assets/version.json")
    .then((res) => {
      if (!res.ok) throw new Error("version.json not found");
      return res.json();
    })
    .then((data) => {
      const verEl = document.getElementById("doc-version");
      if (verEl) {
        verEl.textContent = data.display;
      }
      const commitEl = document.getElementById("doc-commit");
      if (commitEl) {
        commitEl.innerHTML = `git: <code>${data.commit}</code>`;
      }
      const dirtyEl = document.getElementById("doc-dirty");
      if (dirtyEl) {
        dirtyEl.style.display = data.is_dirty ? "inline-flex" : "none";
      }
    })
    .catch(() => {
      const verEl = document.getElementById("doc-version");
      if (verEl && verEl.textContent === "Loading...") {
        verEl.textContent = "v0.16.0";
      }
    });
});
