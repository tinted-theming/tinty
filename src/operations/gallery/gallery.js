const SCHEMES = __TINTY_SCHEMES__;

const state = {
  search: "",
  system: "all",
  appearance: "all",
  pageTheme: "system",
  language: "rust",
};
let currentSheetId = null;
let tooltipTimeoutId = null;
const PAGE_THEME_STORAGE_KEY = "tinty-gallery-page-theme";
const LANGUAGE_STORAGE_KEY = "tinty-gallery-preview-language";

const fallbackPalette = {
  base00: "#101418",
  base03: "#5f6b76",
  base05: "#d8dee9",
  base08: "#d35f5f",
  base09: "#d08f4f",
  base0A: "#c6a84f",
  base0B: "#72a65a",
  base0C: "#5aa6a6",
  base0D: "#5f8fd3",
  base0E: "#9f7ad3",
};

SCHEMES.sort((a, b) => a.id.localeCompare(b.id));

const previewSnippets = {
  rust: `<span class="keyword">use</span> tinty::{<span class="type">Scheme</span>, <span class="type">Theme</span>};

<span class="comment">// load and apply a color scheme</span>
<span class="keyword">fn</span> <span class="function">apply</span>(<span class="parameter">name</span>: &amp;<span class="type">str</span>) -&gt; <span class="type">Option</span>&lt;<span class="type">Theme</span>&gt; {
    <span class="keyword">let</span> scheme = <span class="type">Scheme</span>::<span class="function">load</span>(<span class="parameter">name</span>)?;
    <span class="keyword">let</span> theme = scheme.<span class="function">with_base</span>(<span class="number">16</span>).<span class="function">build</span>();
    theme.<span class="function">apply</span>();
    <span class="builtin">println!</span>(<span class="string">"applied: {}"</span>, theme.<span class="function">name</span>());
    <span class="type">Some</span>(theme)
}`,
  kotlin: `<span class="keyword">import</span> tinty.<span class="type">Scheme</span>

<span class="comment">// load and apply a color scheme</span>
<span class="keyword">fun</span> <span class="function">apply</span>(<span class="parameter">name</span>: <span class="type">String</span>) = <span class="builtin">runCatching</span> {
    <span class="keyword">val</span> theme = <span class="type">Scheme</span>.<span class="function">load</span>(<span class="parameter">name</span>)
        .<span class="function">withBase</span>(<span class="number">16</span>)
        .<span class="function">build</span>()
    theme.<span class="function">apply</span>()
    <span class="builtin">println</span>(<span class="string">"applied: \${theme.name}"</span>)
}`,
  lisp:`<span class="comment">;; load and apply a color scheme</span>
(<span class="keyword">defpackage</span> <span class="string">:tinty</span> (:use :cl))

(<span class="keyword">defun</span> <span class="function">apply-scheme</span> (<span class="parameter">name</span>)
  (<span class="keyword">let*</span> ((scheme (<span class="function">scheme:load</span> <span class="parameter">name</span>))
         (theme (<span class="function">scheme:build</span> scheme :base <span class="number">16</span>)))
    (<span class="function">theme:apply</span> theme)
    (<span class="builtin">format</span> t <span class="string">"applied: ~a~%"</span>
      (<span class="function">theme:name</span> theme))))`,
  elixir:`<span class="keyword">defmodule</span> <span class="type">Tinty</span> <span class="keyword">do</span>
  <span class="comment"># load and apply a color scheme</span>
  <span class="keyword">def</span> <span class="function">apply</span>(<span class="parameter">name</span>) <span class="keyword">do</span>
    {<span class="string">:ok</span>, theme} =
      <span class="parameter">name</span>
      |&gt; <span class="type">Scheme</span>.<span class="function">load</span>()
      |&gt; <span class="type">Theme</span>.<span class="function">build</span>(base: <span class="number">16</span>)
    <span class="type">IO</span>.<span class="builtin">puts</span>(<span class="string">"applied: #{theme.name}"</span>)
    theme
  <span class="keyword">end</span>
<span class="keyword">end</span>`,
  diff: `<span class="comment">diff --git a/apply.rs b/apply.rs</span>
<span class="diff-del">--- a/apply.rs</span><span class="diff-add">+++ b/apply.rs</span><span class="function">@@ -3,7 +3,9 @@ use tinty;</span>

<span class="diff-del">-fn apply(name: &amp;str) {
-    let colors = 8;</span><span class="diff-add">+fn apply(name: &amp;str) -&gt; Theme {
+    let colors = 16;
+    println!("applying: {name}");</span>     scheme.apply(colors);
 }`,
  haskell: `<span class="keyword">import</span> <span class="type">Tinty</span> (<span class="type">Scheme</span>, <span class="type">Theme</span>)

<span class="comment">-- load and apply a color scheme</span>
<span class="function">apply</span> :: <span class="type">String</span> -&gt; <span class="type">IO</span> ()
<span class="function">apply</span> <span class="parameter">name</span> = <span class="keyword">do</span>
  scheme &lt;- <span class="function">loadScheme</span> <span class="parameter">name</span>
  <span class="keyword">let</span> theme = <span class="function">buildWith</span> scheme <span class="number">16</span>
  <span class="function">applyTheme</span> theme
  <span class="builtin">putStrLn</span> (<span class="string">"applied: "</span> ++ <span class="function">themeName</span> theme)`,
  terminal: `<span class="ansi-bright-green">user@host</span> <span class="ansi-bright-blue">~/dev/scheme</span>  <span class="ansi-bright-magenta">main</span><span class="ansi-bright-white">$</span> ls <span class="ansi-cyan">-F</span>
Cargo.toml  README.md  <span class="ansi-bright-cyan">current@</span>  <span class="ansi-blue">drafts/</span>  <span class="ansi-blue">scripts/</span>  <span class="ansi-blue">src/</span>

<span class="ansi-bright-green">user@host</span> <span class="ansi-bright-blue">~/dev/scheme</span>  <span class="ansi-bright-magenta">main</span><span class="ansi-bright-white">$</span> tree <span class="ansi-cyan">-L</span> <span class="ansi-bright-yellow">2</span> <span class="ansi-blue">src/</span>
<span class="ansi-blue">src/</span>
<span class="ansi-bright-black">├──</span> lib.rs
<span class="ansi-bright-black">├──</span> <span class="ansi-blue">scheme/</span>
<span class="ansi-bright-black">│   ├──</span> base16.rs
<span class="ansi-bright-black">│   └──</span> tinted8.rs
<span class="ansi-bright-black">└──</span> <span class="ansi-blue">templates/</span>
<span class="ansi-bright-black">    ├──</span> <span class="ansi-yellow">alacritty.tpl</span>
<span class="ansi-bright-black">    └──</span> <span class="ansi-yellow">kitty.tpl</span>

<span class="ansi-bright-green">user@host</span> <span class="ansi-bright-blue">~/dev/scheme</span>  <span class="ansi-bright-magenta">main</span><span class="ansi-bright-white">$</span> grep <span class="ansi-cyan">-rn</span> <span class="ansi-green">"TODO"</span> <span class="ansi-blue">src/</span>
<span class="ansi-magenta">src/scheme/tinted8.rs</span><span class="ansi-cyan">:</span><span class="ansi-green">42</span><span class="ansi-cyan">:</span> // <span class="ansi-bright-red">TODO</span>: validate non-standard variants`,
};

function color(scheme, key) {
  return scheme.palette[key]?.hex_str || fallbackPalette[key] || fallbackPalette.base05;
}

const PREVIEW_ROLE_KEYS = {
  base16: {
    bg: "base00",
    fg: "base05",
    muted: "base04",
    comment: "base03",
    keyword: "base0E",
    function: "base0D",
    string: "base0B",
    number: "base09",
    deleted: "base08",
    added: "base0B",
    type: "base0A",
    builtin: "base0D",
    parameter: "base0C",
    "ansi-black": "base00",
    "ansi-red": "base08",
    "ansi-green": "base0B",
    "ansi-yellow": "base0A",
    "ansi-blue": "base0D",
    "ansi-magenta": "base0E",
    "ansi-cyan": "base0C",
    "ansi-white": "base05",
    "ansi-bright-black": "base03",
    "ansi-bright-red": "base08",
    "ansi-bright-green": "base0B",
    "ansi-bright-yellow": "base0A",
    "ansi-bright-blue": "base0D",
    "ansi-bright-magenta": "base0E",
    "ansi-bright-cyan": "base0C",
    "ansi-bright-white": "base07",
  },
  base24: {
    "ansi-bright-red": "base12",
    "ansi-bright-yellow": "base13",
    "ansi-bright-green": "base14",
    "ansi-bright-cyan": "base15",
    "ansi-bright-blue": "base16",
    "ansi-bright-magenta": "base17",
  },
  tinted8: {
    dark: {
      bg: "black-normal",
      fg: "white-normal",
      muted: "white-dim",
    },
    light: {
      bg: "white-normal",
      fg: "black-normal",
      muted: "black-dim",
    },
    shared: {
      comment: "gray-dim",
      keyword: "magenta-normal",
      function: "blue-normal",
      string: "green-normal",
      number: "orange-normal",
      deleted: "red-bright",
      added: "green-bright",
      type: "yellow-normal",
      builtin: "blue-bright",
      parameter: "cyan-bright",
      "ansi-black": "black-normal",
      "ansi-red": "red-normal",
      "ansi-green": "green-normal",
      "ansi-yellow": "yellow-normal",
      "ansi-blue": "blue-normal",
      "ansi-magenta": "magenta-normal",
      "ansi-cyan": "cyan-normal",
      "ansi-white": "white-normal",
      "ansi-bright-black": "black-bright",
      "ansi-bright-red": "red-bright",
      "ansi-bright-green": "green-bright",
      "ansi-bright-yellow": "yellow-bright",
      "ansi-bright-blue": "blue-bright",
      "ansi-bright-magenta": "magenta-bright",
      "ansi-bright-cyan": "cyan-bright",
      "ansi-bright-white": "white-bright",
    },
  },
};

const PREVIEW_ROLES = [
  "bg", "fg", "muted",
  "comment", "keyword", "function", "string", "number",
  "deleted", "added",
  "type", "builtin", "parameter",
  "ansi-black", "ansi-red", "ansi-green", "ansi-yellow",
  "ansi-blue", "ansi-magenta", "ansi-cyan", "ansi-white",
  "ansi-bright-black", "ansi-bright-red", "ansi-bright-green", "ansi-bright-yellow",
  "ansi-bright-blue", "ansi-bright-magenta", "ansi-bright-cyan", "ansi-bright-white",
];

function previewKey(scheme, role) {
  const system = String(scheme.system).toLowerCase();
  if (system === "tinted8") {
    const variant = String(scheme.variant || "").toLowerCase() === "light" ? "light" : "dark";
    const t8 = PREVIEW_ROLE_KEYS.tinted8;
    return t8[variant][role] ?? t8.shared[role];
  }
  if (system === "base24") {
    return PREVIEW_ROLE_KEYS.base24[role] ?? PREVIEW_ROLE_KEYS.base16[role];
  }
  return PREVIEW_ROLE_KEYS.base16[role];
}

function appearance(scheme) {
  const background = scheme.lightness?.background;
  if (typeof background !== "number") {
    return String(scheme.variant || "unknown").toLowerCase();
  }
  return background >= 50 ? "light" : "dark";
}

function searchableText(scheme) {
  return [
    scheme.id,
    scheme.name,
    scheme.slug,
  ].join(" ").toLowerCase();
}

function matchesFilters(scheme) {
  if (state.system !== "all" && String(scheme.system).toLowerCase() !== state.system) {
    return false;
  }

  if (state.appearance !== "all" && appearance(scheme) !== state.appearance) {
    return false;
  }

  return searchableText(scheme).includes(state.search);
}

function setPreviewColors(card, scheme) {
  PREVIEW_ROLES.forEach((role) => {
    card.style.setProperty(`--preview-${role}`, color(scheme, previewKey(scheme, role)));
  });
}

function setPreviewLanguage(language) {
  document.getElementById("sheet-code").innerHTML = previewSnippets[language] || previewSnippets.rust;
  document
    .querySelectorAll("[data-preview-language]")
    .forEach((candidate) => candidate.classList.toggle("active", candidate.dataset.previewLanguage === language));
}

function setLanguage(lang) {
  state.language = lang;
  window.localStorage.setItem(LANGUAGE_STORAGE_KEY, lang);
  document.getElementById("language-select").value = lang;
  setPreviewLanguage(lang);
  document.querySelectorAll(".card .code-preview code").forEach((el) => {
    el.innerHTML = previewSnippets[lang] || previewSnippets.rust;
  });
}

function loadSavedLanguage() {
  const saved = window.localStorage.getItem(LANGUAGE_STORAGE_KEY);
  if (saved && previewSnippets[saved]) {
    state.language = saved;
    document.getElementById("language-select").value = saved;
  }
}

function metadataItem(label, value) {
  const fragment = document.createDocumentFragment();
  const dt = document.createElement("dt");
  const dd = document.createElement("dd");
  dt.textContent = label;
  dd.textContent = value || "n/a";
  fragment.append(dt, dd);
  return fragment;
}

function renderPalette(container, scheme) {
  container.textContent = "";

  Object.entries(scheme.palette)
    .sort(([a], [b]) => a.localeCompare(b))
    .forEach(([name, value]) => {
      const swatch = document.createElement("div");
      const block = document.createElement("div");
      const label = document.createElement("div");
      const hex = document.createElement("span");

      swatch.className = "swatch";
      block.className = "swatch-color";
      label.className = "swatch-label";
      block.style.background = value.hex_str;
      label.textContent = name;
      hex.textContent = value.hex_str;

      label.append(hex);
      swatch.append(block, label);
      container.append(swatch);
    });
}

function transitionLayout(callback) {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    callback();
    return;
  }

  if (document.startViewTransition) {
    document.startViewTransition(callback);
    return;
  }

  callback();
}

function schemeForHash() {
  const targetId = window.location.hash.replace(/^#/, "");
  if (!targetId) {
    return null;
  }

  return SCHEMES.find((candidate) => candidate.id === targetId) || null;
}

function setSheetHash(id) {
  const url = new URL(window.location.href);
  url.hash = id;
  window.history.replaceState(null, "", url);
}

function clearSheetHash() {
  const url = new URL(window.location.href);
  url.hash = "";
  window.history.replaceState(null, "", url);
}

function showButtonTooltip(button, message) {
  button.dataset.tooltip = message;
  button.classList.add("show-tooltip");

  if (tooltipTimeoutId) {
    window.clearTimeout(tooltipTimeoutId);
  }

  tooltipTimeoutId = window.setTimeout(() => {
    button.classList.remove("show-tooltip");
  }, 1100);
}

const SHARED_TRANSITION_NAME = "scheme-shared";
let originCard = null;

function effectivePageTheme() {
  if (state.pageTheme === "dark" || state.pageTheme === "light") {
    return state.pageTheme;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applySheetState(scheme, updateHash) {
  const sheet = document.getElementById("detail-sheet");
  const backdrop = document.getElementById("sheet-backdrop");
  const command = `tinty apply ${scheme.id}`;

  currentSheetId = scheme.id;
  document
    .querySelectorAll(".card.is-sheet-source")
    .forEach((c) => c.classList.remove("is-sheet-source"));
  const matchingCard = document.querySelector(`.card[data-scheme-id="${CSS.escape(scheme.id)}"]`);
  if (matchingCard) matchingCard.classList.add("is-sheet-source");

  const schemeAppearance = appearance(scheme);
  const themeAppearance = effectivePageTheme();
  sheet.dataset.contrastMismatch =
    (schemeAppearance === "light" || schemeAppearance === "dark") &&
    schemeAppearance !== themeAppearance
      ? "true"
      : "false";
  setPreviewColors(sheet, scheme);
  setPreviewLanguage(state.language);
  document.getElementById("sheet-title").textContent = scheme.name;
  document.querySelector("#sheet-system span").textContent = scheme.system;
  document.querySelector("#sheet-appearance span").textContent = appearance(scheme);
  document.getElementById("sheet-command").textContent = command;
  document.getElementById("copy-command").dataset.command = command;
  document.getElementById("copy-command").dataset.tooltip = "Copy command";

  const metadata = document.getElementById("sheet-metadata");
  metadata.textContent = "";
  metadata.append(
    metadataItem("ID", scheme.id),
    metadataItem("Author", scheme.author),
    metadataItem("System", scheme.system),
    metadataItem("Variant", scheme.variant),
    metadataItem("Appearance", appearance(scheme)),
    metadataItem("Background L*", scheme.lightness?.background?.toFixed(2)),
    metadataItem("Foreground L*", scheme.lightness?.foreground?.toFixed(2)),
  );
  renderPalette(document.getElementById("sheet-palette"), scheme);

  if (updateHash) {
    setSheetHash(scheme.id);
  }

  backdrop.hidden = false;
  document.body.classList.add("sheet-open");
  // Force layout flush so the opacity transition plays from the pre-`.open` state
  // when no view transition is running.
  void backdrop.offsetWidth;
  backdrop.classList.add("open");
  sheet.classList.add("open");
  sheet.setAttribute("aria-hidden", "false");
}

function clearSheetState(updateHash) {
  const sheet = document.getElementById("detail-sheet");
  const backdrop = document.getElementById("sheet-backdrop");

  currentSheetId = null;
  document
    .querySelectorAll(".card.is-sheet-source")
    .forEach((c) => c.classList.remove("is-sheet-source"));
  if (updateHash) {
    clearSheetHash();
  }
  document.body.classList.remove("sheet-open");
  sheet.classList.remove("open");
  backdrop.classList.remove("open");
  sheet.setAttribute("aria-hidden", "true");
}

function openSheet(scheme, updateHash = true, sourceCard = null) {
  const sheet = document.getElementById("detail-sheet");

  if (sourceCard && document.startViewTransition) {
    sourceCard.style.viewTransitionName = SHARED_TRANSITION_NAME;
    const transition = document.startViewTransition(() => {
      sourceCard.style.viewTransitionName = "";
      sheet.style.viewTransitionName = SHARED_TRANSITION_NAME;
      applySheetState(scheme, updateHash);
    });
    originCard = sourceCard;
    transition.finished.finally(() => {
      sheet.style.viewTransitionName = "";
    });
    return;
  }

  originCard = sourceCard;
  applySheetState(scheme, updateHash);
}

function closeSheet(updateHash = true) {
  const sheet = document.getElementById("detail-sheet");
  const backdrop = document.getElementById("sheet-backdrop");
  const card = originCard;

  if (card && document.body.contains(card) && document.startViewTransition) {
    sheet.style.viewTransitionName = SHARED_TRANSITION_NAME;
    const transition = document.startViewTransition(() => {
      sheet.style.viewTransitionName = "";
      card.style.viewTransitionName = SHARED_TRANSITION_NAME;
      clearSheetState(updateHash);
    });
    transition.finished.finally(() => {
      card.style.viewTransitionName = "";
      backdrop.hidden = true;
    });
    originCard = null;
    return;
  }

  clearSheetState(updateHash);
  originCard = null;
  window.setTimeout(() => {
    if (!sheet.classList.contains("open")) {
      backdrop.hidden = true;
    }
  }, 220);
}

function createCard(scheme) {
  const template = document.getElementById("card-template");
  const card = template.content.firstElementChild.cloneNode(true);

  setPreviewColors(card, scheme);
  card.dataset.schemeId = scheme.id;
  card.querySelector("h2").textContent = scheme.name;
  card.querySelector(".card-title p").textContent = scheme.id;
  card.querySelector(".scheme-system span").textContent = scheme.system;
  card.querySelector(".scheme-appearance span").textContent = appearance(scheme);
  card.querySelector(".code-preview code").innerHTML = previewSnippets[state.language] || previewSnippets.rust;

  card.querySelector(".preview-button").addEventListener("click", () => {
    openSheet(scheme, true, card);
  });

  if (scheme.id === currentSheetId) {
    card.classList.add("is-sheet-source");
  }

  return card;
}

function syncSheetToHash() {
  const scheme = schemeForHash();
  if (!scheme) {
    closeSheet(false);
    return;
  }

  if (currentSheetId !== scheme.id) {
    openSheet(scheme, false);
  }
}

function render() {
  const gallery = document.getElementById("gallery");
  const empty = document.getElementById("empty");
  const count = document.getElementById("result-count");
  const fragment = document.createDocumentFragment();
  const visible = SCHEMES.filter(matchesFilters);

  gallery.textContent = "";
  visible.forEach((scheme) => fragment.append(createCard(scheme)));
  gallery.append(fragment);

  empty.hidden = visible.length !== 0;
  count.textContent = `${visible.length} of ${SCHEMES.length} schemes`;
}

function setFilter(group, value) {
  state[group] = value;
  document
    .querySelectorAll(`[data-filter="${group}"]`)
    .forEach((candidate) => candidate.classList.toggle("active", candidate.dataset.value === value));
}

function setPageTheme(theme) {
  state.pageTheme = theme;
  window.localStorage.setItem(PAGE_THEME_STORAGE_KEY, theme);
  if (theme === "system") {
    document.documentElement.removeAttribute("data-theme");
  } else {
    document.documentElement.dataset.theme = theme;
  }

  document
    .querySelectorAll("[data-page-theme]")
    .forEach((candidate) => candidate.classList.toggle("active", candidate.dataset.pageTheme === theme));

  render();
}

function loadSavedPageTheme() {
  const savedTheme = window.localStorage.getItem(PAGE_THEME_STORAGE_KEY);
  const validThemes = new Set(["system", "dark", "light"]);

  if (savedTheme && validThemes.has(savedTheme)) {
    setPageTheme(savedTheme);
    return;
  }

  setPageTheme("system");
}

document.getElementById("search").addEventListener("input", (event) => {
  state.search = event.target.value.trim().toLowerCase();
  render();
});

document.querySelectorAll("[data-filter]").forEach((button) => {
  button.addEventListener("click", () => {
    transitionLayout(() => {
      setFilter(button.dataset.filter, button.dataset.value);
      render();
    });
  });
});

document.querySelectorAll("[data-page-theme]").forEach((button) => {
  button.addEventListener("click", () => {
    transitionLayout(() => setPageTheme(button.dataset.pageTheme));
  });
});

document.getElementById("language-select").addEventListener("change", (event) => {
  setLanguage(event.target.value);
});

document.querySelectorAll("[data-preview-language]").forEach((button) => {
  button.addEventListener("click", () => {
    setLanguage(button.dataset.previewLanguage);
  });
});

document.getElementById("sheet-close").addEventListener("click", closeSheet);
document.getElementById("sheet-backdrop").addEventListener("click", closeSheet);
document.getElementById("copy-command").addEventListener("click", async (event) => {
  const button = event.currentTarget;

  try {
    await navigator.clipboard.writeText(button.dataset.command);
    showButtonTooltip(button, "Copied");
  } catch (_error) {
    showButtonTooltip(button, "Copy failed");
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeSheet();
  }
});

window.addEventListener("hashchange", syncSheetToHash);

loadSavedLanguage();
loadSavedPageTheme();
syncSheetToHash();
render();
