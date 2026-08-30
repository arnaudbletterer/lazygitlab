(function () {
  const CACHE_KEY = "lazygitlab_doc_version";

  function getAssetRoot() {
    const extraCss = document.querySelector('link[href*="extra.css"]');
    if (extraCss) {
      const href = extraCss.getAttribute("href");
      return href.substring(0, href.indexOf("stylesheets/extra.css"));
    }
    const path = window.location.pathname.replace(/\/$/, "");
    const depth = path.split("/").filter(Boolean).length;
    return depth > 0 ? "../".repeat(depth) : "./";
  }

  function renderBadge(data) {
    if (!data || !data.display) return;
    const headerTopics = document.querySelectorAll(".md-header__title .md-header__topic");
    if (headerTopics.length > 0) {
      const primaryTopic = headerTopics[0];
      let container = primaryTopic.querySelector(".header-version-container");
      if (!container) {
        container = document.createElement("span");
        container.className = "header-version-container";
        primaryTopic.appendChild(container);
      }
      const isDirty = data.is_dirty || data.display.includes("-dirty");
      const badgeClass = isDirty ? "header-version-badge badge-dirty" : "header-version-badge badge-tag";
      container.innerHTML = `<span class="${badgeClass}"><code>${data.display}</code></span>`;
    }

    const drawerTitle = document.querySelector(".md-nav__title");
    if (drawerTitle) {
      let drawerContainer = drawerTitle.querySelector(".header-version-container");
      if (!drawerContainer) {
        drawerContainer = document.createElement("span");
        drawerContainer.className = "header-version-container";
        drawerContainer.style.display = "flex";
        drawerContainer.style.marginTop = "6px";
        drawerTitle.appendChild(drawerContainer);
      }
      const isDirty = data.is_dirty || data.display.includes("-dirty");
      const badgeClass = isDirty ? "header-version-badge badge-dirty" : "header-version-badge badge-tag";
      drawerContainer.innerHTML = `<span class="${badgeClass}"><code>${data.display}</code></span>`;
    }
  }

  function initVersion() {
    // 1. Instant synchronous render from cache (0ms, 0 blink)
    try {
      const cached = sessionStorage.getItem(CACHE_KEY);
      if (cached) {
        renderBadge(JSON.parse(cached));
      }
    } catch (_) {}

    // 2. Refresh from version.json and update cache
    const root = getAssetRoot();
    fetch(root + "assets/version.json")
      .then((res) => {
        if (!res.ok) throw new Error("version.json not found");
        return res.json();
      })
      .then((data) => {
        try {
          sessionStorage.setItem(CACHE_KEY, JSON.stringify(data));
        } catch (_) {}
        renderBadge(data);
      })
      .catch((err) => {
        console.debug("lazygitlab: version info unavailable", err);
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initVersion);
  } else {
    initVersion();
  }

  if (typeof document$ !== "undefined") {
    document$.subscribe(() => {
      try {
        const cached = sessionStorage.getItem(CACHE_KEY);
        if (cached) renderBadge(JSON.parse(cached));
      } catch (_) {}
    });
  }
})();
