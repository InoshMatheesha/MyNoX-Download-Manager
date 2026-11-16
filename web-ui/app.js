// MyNoX website: theme toggle, latest release links, sticky nav border and reveal on scroll.
(function () {
  "use strict";

  var REPO = "InoshMatheesha/MyNoX-Download-Manager";
  var root = document.documentElement;
  var dark = window.matchMedia("(prefers-color-scheme: dark)");

  // The theme in use: the saved choice, else the system's.
  function effective() {
    var t = root.getAttribute("data-theme");
    return t === "light" || t === "dark" ? t : dark.matches ? "dark" : "light";
  }

  // Screenshots follow the theme in use, the toggle's too.
  function syncImages() {
    var mode = effective();
    document.querySelectorAll("img.theme-img").forEach(function (img) {
      var src = img.getAttribute("data-" + mode);
      if (src && img.getAttribute("src") !== src) img.setAttribute("src", src);
      var pic = img.parentElement;
      if (pic && pic.tagName === "PICTURE") {
        pic.querySelectorAll("source[data-theme-src]").forEach(function (s) {
          s.setAttribute("srcset", src);
        });
      }
    });
  }

  document.getElementById("theme-toggle").addEventListener("click", function () {
    var next = effective() === "dark" ? "light" : "dark";
    root.setAttribute("data-theme", next);
    try { localStorage.setItem("mynox-theme", next); } catch (e) {}
    syncImages();
  });
  if (dark.addEventListener) dark.addEventListener("change", syncImages);
  syncImages();

  // A bottom line under the nav once the page scrolls.
  var nav = document.querySelector(".nav");
  function onScroll() { nav.classList.toggle("scrolled", window.scrollY > 8); }
  window.addEventListener("scroll", onScroll, { passive: true });
  onScroll();

  // Sections fade in as they come into view.
  if ("IntersectionObserver" in window) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
      });
    }, { rootMargin: "0px 0px -8% 0px" });
    document.querySelectorAll(".section-head, .card, .split > *, .cta-inner").forEach(function (el) {
      el.classList.add("reveal");
      io.observe(el);
    });
  }

  function size(bytes) {
    return (bytes / 1048576).toFixed(1) + " MB";
  }

  // The latest release from GitHub: its version, date, files and sizes on every download link.
  fetch("https://api.github.com/repos/" + REPO + "/releases/latest", { headers: { Accept: "application/vnd.github+json" } })
    .then(function (r) { return r.ok ? r.json() : Promise.reject(r.status); })
    .then(function (rel) {
      var version = (rel.tag_name || "").replace(/^v/, "");
      var find = function (re) {
        return (rel.assets || []).filter(function (a) { return re.test(a.name); })[0];
      };
      var assets = {
        setup: find(/-setup\.exe$/i),
        portable: find(/-portable\.exe$/i),
        sums: find(/^SHA256SUMS\.txt$/i)
      };
      Object.keys(assets).forEach(function (k) {
        var a = assets[k];
        if (!a) return;
        document.querySelectorAll('[data-asset="' + k + '"]').forEach(function (el) {
          el.setAttribute("href", a.browser_download_url);
        });
        document.querySelectorAll('[data-name="' + k + '"]').forEach(function (el) { el.textContent = a.name; });
        document.querySelectorAll('[data-size="' + k + '"]').forEach(function (el) { el.textContent = size(a.size); });
      });
      if (version) {
        document.querySelectorAll("[data-version]").forEach(function (el) { el.textContent = "version " + version; });
        document.querySelectorAll("[data-version-text]").forEach(function (el) {
          el.textContent = "Version " + version + " · Windows 10 & 11";
        });
      }
      if (rel.published_at) {
        var d = new Date(rel.published_at);
        var when = d.toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" });
        document.querySelectorAll("[data-release-date]").forEach(function (el) {
          el.textContent = "Released " + when + ". Free for Windows 10 and 11, 64-bit.";
        });
      }
    })
    .catch(function () {
      // Offline or rate-limited: the links already point at the latest release page.
    });
})();
