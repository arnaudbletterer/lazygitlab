function renderDocVersion() {
  // Determine relative root from extra.css link or path depth
  let root = "./";
  const extraCss = document.querySelector('link[href*="extra.css"]');
  if (extraCss) {
    const href = extraCss.getAttribute("href");
    root = href.substring(0, href.indexOf("stylesheets/extra.css"));
  } else {
    const path = window.location.pathname.replace(/\/$/, "");
    const segments = path.split("/").filter(Boolean);
    const depth = segments.length;
    if (depth > 0) {
      root = "../".repeat(depth);
    }
  }

  fetch(root + "assets/version.json")
    .then((res) => {
      if (!res.ok) throw new Error("version.json not found");
      return res.json();
    })
    .then((data) => {
      // 1. Inject into header title next to 'lazygitlab'
      const headerTopics = document.querySelectorAll(".md-header__title .md-header__topic");
      if (headerTopics.length > 0) {
        const primaryTopic = headerTopics[0];
        if (!primaryTopic.querySelector(".header-version-container")) {
          const container = document.createElement("span");
          container.className = "header-version-container";
          const isDirty = data.is_dirty || data.display.includes("-dirty");
          const badgeClass = isDirty ? "header-version-badge badge-dirty" : "header-version-badge badge-tag";
          container.innerHTML = `<span class="${badgeClass}"><code>${data.display}</code></span>`;
          primaryTopic.appendChild(container);
        }
      }

      // 2. Also inject into mobile drawer header if present
      const drawerTitle = document.querySelector(".md-nav__title");
      if (drawerTitle && !drawerTitle.querySelector(".header-version-container")) {
        const drawerContainer = document.createElement("span");
        drawerContainer.className = "header-version-container";
        drawerContainer.style.display = "flex";
        drawerContainer.style.marginTop = "6px";
        const isDirty = data.is_dirty || data.display.includes("-dirty");
        const badgeClass = isDirty ? "header-version-badge badge-dirty" : "header-version-badge badge-tag";
        drawerContainer.innerHTML = `<span class="${badgeClass}"><code>${data.display}</code></span>`;
        drawerTitle.appendChild(drawerContainer);
      }
    })
    .catch((err) => {
      console.debug("lazygitlab: version info unavailable", err);
    });
}

document.addEventListener("DOMContentLoaded", renderDocVersion);

if (typeof document$ !== "undefined") {
  document$.subscribe(renderDocVersion);
}
