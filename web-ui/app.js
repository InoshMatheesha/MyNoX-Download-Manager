// MyNoX website: theme, the island nav, latest release and download count, 3D motion.
(function () {
  "use strict";

  var REPO = "InoshMatheesha/MyNoX-Download-Manager";
  var API = "https://api.github.com/repos/" + REPO;
  var root = document.documentElement;
  var dark = window.matchMedia("(prefers-color-scheme: dark)");
  var calm = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  var fine = window.matchMedia("(hover: hover) and (pointer: fine)").matches;

  // ---- Theme ----------------------------------------------------------------------------------

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
        pic.querySelectorAll("source[data-theme-src]").forEach(function (s) { s.setAttribute("srcset", src); });
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

  // ---- The island ---------------------------------------------------------------------------

  var island = document.querySelector(".island");
  var menu = document.getElementById("island-menu");
  function setMenu(open) {
    island.classList.toggle("menu-open", open);
    menu.setAttribute("aria-expanded", open ? "true" : "false");
  }
  menu.addEventListener("click", function () { setMenu(!island.classList.contains("menu-open")); });
  document.querySelectorAll(".island-links a").forEach(function (a) {
    a.addEventListener("click", function () { setMenu(false); });
  });
  document.addEventListener("keydown", function (e) { if (e.key === "Escape") setMenu(false); });
  document.addEventListener("click", function (e) { if (!island.contains(e.target)) setMenu(false); });

  // ---- Scroll: the island folds, the hero picture stands up ------------------------------------

  var shot = document.getElementById("hero-shot");
  var ticking = false;
  function onScroll() {
    ticking = false;
    if (shot && !calm) {
      // 0 while the picture's top is low in the window, 1 once it has risen to its upper third.
      var r = shot.getBoundingClientRect();
      var vh = window.innerHeight;
      var p = Math.min(1, Math.max(0, (vh * 0.95 - r.top) / (vh * 0.6)));
      shot.style.setProperty("--tilt", (20 * (1 - p)).toFixed(2) + "deg");
      shot.style.setProperty("--lift", (0.93 + 0.07 * p).toFixed(4));
    }
  }
  window.addEventListener("scroll", function () {
    if (!ticking) { ticking = true; requestAnimationFrame(onScroll); }
  }, { passive: true });
  window.addEventListener("resize", onScroll);
  onScroll();

  // ---- Cards tilt toward the pointer -----------------------------------------------------------

  if (fine && !calm) {
    document.querySelectorAll(".tilt").forEach(function (el) {
      el.addEventListener("pointermove", function (e) {
        var r = el.getBoundingClientRect();
        var x = (e.clientX - r.left) / r.width;
        var y = (e.clientY - r.top) / r.height;
        el.style.transition = "transform 0.12s ease-out";
        el.style.transform = "perspective(900px) rotateX(" + ((0.5 - y) * 8).toFixed(2) + "deg) rotateY(" + ((x - 0.5) * 10).toFixed(2) + "deg) translateY(-3px)";
        el.style.setProperty("--mx", (x * 100).toFixed(1) + "%");
        el.style.setProperty("--my", (y * 100).toFixed(1) + "%");
      });
      el.addEventListener("pointerleave", function () {
        el.style.transition = "transform 0.6s cubic-bezier(0.2, 0.8, 0.2, 1)";
        el.style.transform = "";
      });
    });
  }

  // ---- Reveal on scroll, one after another ------------------------------------------------------

  if ("IntersectionObserver" in window && !calm) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
      });
    }, { rootMargin: "0px 0px -8% 0px" });
    document.querySelectorAll(".section-head, .grid, .steps, .split > *, .cta-inner, .faq").forEach(function (group) {
      var items = group.matches(".grid, .steps") ? group.children : [group];
      Array.prototype.forEach.call(items, function (el, i) {
        el.classList.add("reveal");
        el.style.setProperty("--i", i);
        io.observe(el);
      });
    });
  }

  // ---- Numbers count up when they come into view -------------------------------------------------

  function format(n) { return Math.round(n).toLocaleString(); }
  function countUp(el) {
    var to = Number(el.getAttribute("data-count")) || 0;
    if (calm || to === 0) { el.textContent = format(to); return; }
    var start = performance.now();
    var dur = 1400;
    (function step(t) {
      var k = Math.min(1, (t - start) / dur);
      el.textContent = format(to * (1 - Math.pow(1 - k, 4)));
      if (k < 1) requestAnimationFrame(step);
    })(start);
  }
  var counted = false;
  function countAll() {
    document.querySelectorAll("[data-count]").forEach(countUp);
  }
  var stats = document.querySelector(".stats");
  if ("IntersectionObserver" in window && stats) {
    var so = new IntersectionObserver(function (entries) {
      if (entries[0].isIntersecting) { counted = true; countAll(); so.disconnect(); }
    }, { threshold: 0.4 });
    so.observe(stats);
  } else {
    counted = true;
    countAll();
  }

  // ---- The latest release, and every release's downloads together --------------------------------

  function size(bytes) { return (bytes / 1048576).toFixed(1) + " MB"; }
  function short(n) {
    if (n >= 1e6) return (n / 1e6).toFixed(1).replace(/\.0$/, "") + "M";
    if (n >= 1e3) return (n / 1e3).toFixed(1).replace(/\.0$/, "") + "K";
    return String(n);
  }
  var gh = { headers: { Accept: "application/vnd.github+json" } };
  // The island's live status: the version and the downloads, whichever answer comes first.
  var live = { version: "", total: null };
  function renderLive() {
    var parts = [];
    if (live.version) parts.push("v" + live.version);
    if (live.total !== null) parts.push(short(live.total) + " downloads");
    if (!parts.length) return;
    document.querySelectorAll("[data-island-status]").forEach(function (el) { el.textContent = parts.join(" · "); });
  }

  fetch(API + "/releases/latest", gh)
    .then(function (r) { return r.ok ? r.json() : Promise.reject(r.status); })
    .then(function (rel) {
      var version = (rel.tag_name || "").replace(/^v/, "");
      var find = function (re) { return (rel.assets || []).filter(function (a) { return re.test(a.name); })[0]; };
      var assets = { setup: find(/-setup\.exe$/i), portable: find(/-portable\.exe$/i), sums: find(/^SHA256SUMS\.txt$/i) };
      Object.keys(assets).forEach(function (k) {
        var a = assets[k];
        if (!a) return;
        document.querySelectorAll('[data-asset="' + k + '"]').forEach(function (el) { el.setAttribute("href", a.browser_download_url); });
        document.querySelectorAll('[data-name="' + k + '"]').forEach(function (el) { el.textContent = a.name; });
        document.querySelectorAll('[data-size="' + k + '"]').forEach(function (el) { el.textContent = size(a.size); });
      });
      if (version) {
        document.querySelectorAll("[data-version]").forEach(function (el) { el.textContent = "version " + version; });
        document.querySelectorAll("[data-version-text]").forEach(function (el) { el.textContent = "Version " + version + " · Windows 10 & 11"; });
        live.version = version;
        renderLive();
      }
      if (rel.published_at) {
        var when = new Date(rel.published_at).toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" });
        document.querySelectorAll("[data-release-date]").forEach(function (el) {
          el.textContent = "Released " + when + ". Free for Windows 10 and 11, 64-bit.";
        });
      }
    })
    .catch(function () {});

})();
