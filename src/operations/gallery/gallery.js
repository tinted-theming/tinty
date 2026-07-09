const SCHEMES = __TINTY_SCHEMES__;

// Remote-control mode flag. Substituted by the Rust generator: `true` only for
// the default `tinty gallery` (the live server), `false` for the static site
// produced by `--no-rc` / `--dump`. When false the gallery makes no network
// requests and the apply controls stay hidden, so the static build is portable.
const TINTY_SERVE = __TINTY_SERVE__;
// Live-server only: `user@hostname` of the machine running the server, shown
// in the header so it's clear which system an Apply affects. `null` in static
// builds.
const TINTY_HOST = __TINTY_HOST__;
const CURRENT_POLL_INTERVAL = 2000;

const state = {
  search: "",
  system: "all",
  appearance: "all",
  pageTheme: "system",
  // Live-server only: id of the scheme currently applied on this machine, or
  // null. Tracked so the matching card can be highlighted and the modal's
  // Apply button can reflect the applied state. Always null in static builds.
  appliedSchemeId: null,
  // Gallery-card preview language. Can be a code lang or the special
  // PALETTE_LANGUAGE value. Persisted under LANGUAGE_STORAGE_KEY.
  language: "rust",
  // Modal code-preview language. Always a code lang — the modal does not
  // expose a Palette option in its chip toolbar. Persisted under
  // MODAL_LANGUAGE_STORAGE_KEY. Stays in lockstep with `state.language`
  // whenever the gallery is on a code lang; diverges (preserves its prior
  // value) when the gallery flips to palette.
  modalLanguage: "rust",
  variablesView: "palette",
};
let currentSheetId = null;
let tooltipTimeoutId = null;
let isFirstRender = true;
const PAGE_THEME_STORAGE_KEY = "tinty-gallery-page-theme";
const LANGUAGE_STORAGE_KEY = "tinty-gallery-preview-language";
const MODAL_LANGUAGE_STORAGE_KEY = "tinty-gallery-modal-language";

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

function getSnippet(lang) {
  const template = document.getElementById(`snippet-${lang}`);
  return template ? template.innerHTML : "";
}

function hasSnippet(lang) {
  return Boolean(document.getElementById(`snippet-${lang}`));
}

const FALLBACK_LANGUAGE = "rust";

// For Tinted8 schemes, map each non-ANSI preview role to a canonical
// dotted-path key in `scheme.syntax` or `scheme.ui`. Lets authored scheme
// overrides drive the gallery preview instead of the hand-rolled palette
// guess. ANSI roles aren't in the syntax/ui spec — they fall through to
// the palette mapping.
const TINTED8_ROLE_PATHS = {
  bg: ["ui", "global.background.normal"],
  fg: ["ui", "global.foreground.normal"],
  muted: ["ui", "global.foreground.dark"],
  comment: ["syntax", "comment"],
  keyword: ["syntax", "keyword"],
  include: ["syntax", "keyword.control.import"],
  function: ["syntax", "entity.name.function"],
  namespace: ["syntax", "entity.name.namespace"],
  string: ["syntax", "string"],
  escape: ["syntax", "constant.character.escape"],
  number: ["syntax", "constant.numeric"],
  type: ["syntax", "entity.name.type"],
  punctuation: ["syntax", "punctuation.separator"],
  added: ["syntax", "markup.inserted"],
  deleted: ["syntax", "markup.deleted"],
};

const PREVIEW_ROLE_KEYS = {
  base16: {
    bg: "base00",
    fg: "base05",
    muted: "base04",
    comment: "base03",
    keyword: "base0E",
    include: "base0D",
    function: "base0D",
    macro: "base08",
    namespace: "base08",
    string: "base0B",
    escape: "base0C",
    number: "base09",
    punctuation: "base0F",
    type: "base0A",
    deleted: "base08",
    added: "base0B",
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
      include: "blue-normal",
      function: "blue-normal",
      macro: "red-normal",
      namespace: "red-normal",
      string: "green-normal",
      escape: "cyan-normal",
      number: "orange-normal",
      punctuation: "red-dim",
      deleted: "red-bright",
      added: "green-bright",
      type: "yellow-normal",
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
  "comment", "keyword", "include", "function", "macro", "namespace",
  "string", "escape", "number", "punctuation", "type",
  "deleted", "added",
  "ansi-black", "ansi-red", "ansi-green", "ansi-yellow",
  "ansi-blue", "ansi-magenta", "ansi-cyan", "ansi-white",
  "ansi-bright-black", "ansi-bright-red", "ansi-bright-green", "ansi-bright-yellow",
  "ansi-bright-blue", "ansi-bright-magenta", "ansi-bright-cyan", "ansi-bright-white",
];

function palettePreviewKey(scheme, role) {
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

function previewColor(scheme, role) {
  if (String(scheme.system).toLowerCase() === "tinted8") {
    const path = TINTED8_ROLE_PATHS[role];
    if (path) {
      const [source, key] = path;
      const entry = scheme[source]?.[key];
      if (entry?.hex_str) {
        return entry.hex_str;
      }
    }
  }
  const key = palettePreviewKey(scheme, role);
  return scheme.palette[key]?.hex_str || fallbackPalette[key] || fallbackPalette.base05;
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
    card.style.setProperty(`--preview-${role}`, previewColor(scheme, role));
  });
}

const PALETTE_LANGUAGE = "palette";
const TINTED8_VARIANT_ORDER = { dim: 0, normal: 1, bright: 2 };

function snippetFor(lang) {
  return hasSnippet(lang) ? getSnippet(lang) : getSnippet(FALLBACK_LANGUAGE);
}

// Per scheme system, return the grid dimensions that pack the palette evenly:
//   Base16   16 colors  → 8 cols × 2 rows (darks row, accents row)
//   Base24   24 colors  → 8 cols × 3 rows (adds the bright-ANSI row)
//   Tinted8  33 colors  → 11 cols × 3 rows (cols = color, rows = variant)
function paletteGridShape(scheme) {
  const system = String(scheme.system).toLowerCase();
  if (system === "tinted8") return { cols: 11, rows: 3 };
  if (system === "base24") return { cols: 8, rows: 3 };
  return { cols: 8, rows: 2 };
}

// Order palette entries so a row-major grid lays out the natural shape:
//   Base16/Base24: alphabetical (base00, base01, …) places base0X..base1X
//                  rows in order.
//   Tinted8:       sort by (variant, color) so row 1 = all dims, row 2 = all
//                  normals, row 3 = all brights; each column is one color.
function paletteEntriesInGridOrder(scheme) {
  const all = Object.entries(scheme.palette);
  if (String(scheme.system).toLowerCase() === "tinted8") {
    return all.sort(([a], [b]) => {
      const [aColor, aVariant] = a.split("-");
      const [bColor, bVariant] = b.split("-");
      const vd = TINTED8_VARIANT_ORDER[aVariant] - TINTED8_VARIANT_ORDER[bVariant];
      if (vd !== 0) return vd;
      return aColor.localeCompare(bColor);
    });
  }
  return all.sort(([a], [b]) => a.localeCompare(b));
}

function palettePreviewHtml(scheme) {
  return paletteEntriesInGridOrder(scheme)
    .map(([name, value]) => {
      const safeName = name.replace(/[<>&"]/g, "");
      return `<span class="palette-cell" style="background:${value.hex_str}" title="${safeName}: ${value.hex_str}"></span>`;
    })
    .join("");
}

function renderPreviewInto(codePre, scheme, lang) {
  const codeEl = codePre.querySelector("code");
  const isPalette = lang === PALETTE_LANGUAGE && scheme;
  codePre.classList.toggle("is-palette", isPalette);
  if (isPalette) {
    codeEl.innerHTML = palettePreviewHtml(scheme);
    const { cols, rows } = paletteGridShape(scheme);
    codeEl.style.gridTemplateColumns = `repeat(${cols}, 1fr)`;
    codeEl.style.gridTemplateRows = `repeat(${rows}, 1fr)`;
  } else {
    codeEl.style.gridTemplateColumns = "";
    codeEl.style.gridTemplateRows = "";
    codeEl.innerHTML = snippetFor(lang);
  }
}

function schemeForCard(card) {
  return SCHEMES.find((s) => s.id === card.dataset.schemeId);
}

// Render the modal's code preview using whatever state.modalLanguage is.
// Also refresh the chip toolbar's active state. No state writes.
function renderModalPreview() {
  const sheetPre = document.getElementById("sheet-code").closest(".code-preview");
  const scheme = SCHEMES.find((s) => s.id === currentSheetId);
  renderPreviewInto(sheetPre, scheme, state.modalLanguage);
  document
    .querySelectorAll("[data-preview-language]")
    .forEach((candidate) =>
      candidate.classList.toggle("active", candidate.dataset.previewLanguage === state.modalLanguage),
    );
}

// Internal: set the gallery-card language and re-render. Does NOT sync to
// the modal. Public callers should go through onGalleryLanguageChange.
function applyGalleryLanguage(lang) {
  state.language = lang;
  window.localStorage.setItem(LANGUAGE_STORAGE_KEY, lang);
  document.getElementById("language-select").value = lang;
  document.querySelectorAll(".card").forEach((card) => {
    const scheme = schemeForCard(card);
    if (!scheme) return;
    renderPreviewInto(card.querySelector(".code-preview"), scheme, lang);
  });
}

// Internal: set the modal-preview language and re-render. Does NOT sync to
// the gallery. Public callers should go through onModalLanguageChange.
function applyModalLanguage(lang) {
  state.modalLanguage = lang;
  window.localStorage.setItem(MODAL_LANGUAGE_STORAGE_KEY, lang);
  renderModalPreview();
}

// Event handler for the gallery's language <select>.
// Always drives gallery; drives modal too when the new lang is a code lang
// (so the two stay synced). When the user picks palette, the modal's
// last-selected code lang is preserved.
function onGalleryLanguageChange(lang) {
  applyGalleryLanguage(lang);
  if (lang !== PALETTE_LANGUAGE) {
    applyModalLanguage(lang);
  }
}

// Event handler for the modal's chip toolbar.
// Always drives the modal. Also drives the gallery — UNLESS the gallery is
// in palette mode, in which case the modal goes independent and the
// gallery stays on palette.
function onModalLanguageChange(lang) {
  applyModalLanguage(lang);
  if (state.language !== PALETTE_LANGUAGE) {
    applyGalleryLanguage(lang);
  }
}

function isValidGalleryLanguage(lang) {
  return lang === PALETTE_LANGUAGE || hasSnippet(lang);
}

function loadSavedLanguage() {
  const savedGallery = window.localStorage.getItem(LANGUAGE_STORAGE_KEY);
  if (savedGallery && isValidGalleryLanguage(savedGallery)) {
    state.language = savedGallery;
    document.getElementById("language-select").value = savedGallery;
  }
  const savedModal = window.localStorage.getItem(MODAL_LANGUAGE_STORAGE_KEY);
  if (savedModal && hasSnippet(savedModal)) {
    state.modalLanguage = savedModal;
  } else if (state.language !== PALETTE_LANGUAGE) {
    // First-time load (or invalid saved): seed modal from gallery's code
    // language so they start in sync.
    state.modalLanguage = state.language;
  }
}

function metadataRow(label, value) {
  const row = document.createElement("div");
  const labelEl = document.createElement("span");
  const valueEl = document.createElement("span");
  row.className = "metadata-row";
  labelEl.className = "metadata-label";
  valueEl.className = "metadata-value";
  labelEl.textContent = label;
  valueEl.textContent = value || "n/a";
  row.append(labelEl, valueEl);
  return row;
}

function metadataGroup(className, ...rows) {
  const group = document.createElement("div");
  group.className = className;
  group.append(...rows);
  return group;
}

function setVariablesView(view) {
  state.variablesView = view;
  document.querySelectorAll("[data-variables-view]").forEach((button) => {
    button.classList.toggle("active", button.dataset.variablesView === view);
  });
  document.getElementById("sheet-palette").hidden = view !== "palette";
  document.getElementById("sheet-ui").hidden = view !== "ui";
  document.getElementById("sheet-syntax").hidden = view !== "syntax";
}

function renderColorMap(container, map) {
  container.textContent = "";
  container.dataset.size = String(Object.keys(map).length);

  Object.entries(map)
    .sort(([a], [b]) => a.localeCompare(b))
    .forEach(([name, value]) => {
      const swatch = document.createElement("div");
      const block = document.createElement("div");
      const hex = document.createElement("span");
      const label = document.createElement("div");

      swatch.className = "swatch";
      block.className = "swatch-color";
      hex.className = "swatch-hex";
      label.className = "swatch-label";

      block.style.background = value.hex_str;
      hex.textContent = value.hex_str;
      hex.style.color = pillTextColor(value.rgb, value.hex_str);
      label.textContent = name;

      block.append(hex);
      swatch.append(block, label);
      container.append(swatch);
    });
}

function relativeLuminance(rgb) {
  const channels = rgb.map((c) => {
    const norm = c / 255;
    return norm <= 0.03928 ? norm / 12.92 : Math.pow((norm + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

// Hue-aware text color: starts from a near-black or near-white base depending
// on the swatch luminance, then mixes ~25% of the swatch color in. Keeps the
// text high-contrast (dark on light, light on dark) while picking up a tonal
// hint of the swatch's hue. Uses color-mix in oklab for perceptual mixing.
function pillTextColor(rgb, hexStr) {
  const isLight = relativeLuminance(rgb) > 0.45;
  const base = isLight ? "#06080a" : "#fafbfd";
  return `color-mix(in oklab, ${hexStr} 25%, ${base} 75%)`;
}

function renderVariableList(container, map) {
  container.textContent = "";

  Object.entries(map)
    .sort(([a], [b]) => a.localeCompare(b))
    .forEach(([name, value]) => {
      const row = document.createElement("div");
      const key = document.createElement("span");
      const pill = document.createElement("span");

      row.className = "variable-row";
      key.className = "variable-key";
      pill.className = "variable-pill";

      key.textContent = name;
      pill.textContent = value.hex_str;
      pill.style.background = value.hex_str;
      pill.style.color = pillTextColor(value.rgb, value.hex_str);

      row.append(key, pill);
      container.append(row);
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
  renderModalPreview();
  document.getElementById("sheet-title").textContent = scheme.name;
  document.querySelector("#sheet-system span").textContent = scheme.system;
  document.querySelector("#sheet-appearance span").textContent = appearance(scheme);
  document.getElementById("sheet-command").textContent = command;
  document.getElementById("copy-command").dataset.command = command;
  document.getElementById("copy-command").dataset.tooltip = "Copy command";

  const metadata = document.getElementById("sheet-metadata");
  metadata.textContent = "";
  metadata.append(
    metadataGroup(
      "metadata-top",
      metadataRow("ID", scheme.id),
      metadataRow("Author", scheme.author),
    ),
    metadataGroup(
      "metadata-cols",
      metadataGroup(
        "metadata-col",
        metadataRow("System", scheme.system),
        metadataRow("Variant", scheme.variant),
        metadataRow("Appearance", appearance(scheme)),
      ),
      metadataGroup(
        "metadata-col",
        metadataRow("Bg L*", scheme.lightness?.background?.toFixed(2)),
        metadataRow("Fg L*", scheme.lightness?.foreground?.toFixed(2)),
      ),
    ),
  );
  renderColorMap(document.getElementById("sheet-palette"), scheme.palette);

  const hasVariables = Boolean(scheme.ui && scheme.syntax);
  const label = document.getElementById("variables-label");
  const toggle = document.getElementById("variables-toggle");
  label.hidden = hasVariables;
  toggle.hidden = !hasVariables;

  if (hasVariables) {
    renderVariableList(document.getElementById("sheet-ui"), scheme.ui);
    renderVariableList(document.getElementById("sheet-syntax"), scheme.syntax);
    setVariablesView(state.variablesView);
  } else {
    setVariablesView("palette");
  }

  if (updateHash) {
    setSheetHash(scheme.id);
  }

  updateApplyButton();

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
  renderPreviewInto(card.querySelector(".code-preview"), scheme, state.language);

  card.querySelector(".preview-button").addEventListener("click", () => {
    openSheet(scheme, true, card);
  });

  if (scheme.id === currentSheetId) {
    card.classList.add("is-sheet-source");
  }

  if (TINTY_SERVE && scheme.id === state.appliedSchemeId) {
    card.classList.add("is-applied");
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

// Live-server only: pin the currently applied scheme to the front of the
// already-filtered list. Filters/search still decide membership, so a scheme
// that doesn't match isn't force-shown; it just leads when it is present.
function pinAppliedFirst(visible) {
  if (!TINTY_SERVE || !state.appliedSchemeId) return visible;

  const applied = [];
  const rest = [];
  visible.forEach((scheme) => {
    (scheme.id === state.appliedSchemeId ? applied : rest).push(scheme);
  });
  return applied.concat(rest);
}

function render() {
  const gallery = document.getElementById("gallery");
  const empty = document.getElementById("empty");
  const count = document.getElementById("result-count");
  const fragment = document.createDocumentFragment();
  const visible = pinAppliedFirst(SCHEMES.filter(matchesFilters));

  gallery.classList.toggle("is-first-render", isFirstRender);
  gallery.textContent = "";
  visible.forEach((scheme) => fragment.append(createCard(scheme)));
  gallery.append(fragment);

  if (isFirstRender) {
    let rowIndex = -1;
    let lastTop = null;
    Array.from(gallery.children).forEach((card) => {
      const top = card.getBoundingClientRect().top;
      if (lastTop === null || Math.abs(top - lastTop) > 4) {
        rowIndex++;
        lastTop = top;
      }
      card.style.setProperty("--enter-delay", `${Math.min(rowIndex * 70, 700)}ms`);
    });
  }

  empty.hidden = visible.length !== 0;
  count.textContent = `${visible.length} of ${SCHEMES.length} schemes`;

  isFirstRender = false;
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
  onGalleryLanguageChange(event.target.value);
});

document.querySelectorAll("[data-preview-language]").forEach((button) => {
  button.addEventListener("click", () => {
    onModalLanguageChange(button.dataset.previewLanguage);
  });
});

document.querySelectorAll("[data-variables-view]").forEach((button) => {
  button.addEventListener("click", () => {
    setVariablesView(button.dataset.variablesView);
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

// ---------------------------------------------------------------------------
// Live-server mode (TINTY_SERVE only)
//
// Everything below is inert in the static build: it is gated on TINTY_SERVE
// and only wired up by setupLiveServer(), which no-ops when the flag is false.
// ---------------------------------------------------------------------------

let toastTimeoutId = null;
// Whether the gallery server is currently reachable. Starts true (the page was
// just served by it); flipped by the poll / apply requests.
let serverConnected = true;

function showToast(message) {
  const toast = document.getElementById("toast");
  if (!toast) return;

  toast.textContent = message;
  toast.hidden = false;
  // Force layout flush so the slide-up transition plays from the hidden state.
  void toast.offsetWidth;
  toast.classList.add("open");

  if (toastTimeoutId) {
    window.clearTimeout(toastTimeoutId);
  }
  toastTimeoutId = window.setTimeout(() => {
    toast.classList.remove("open");
  }, 2400);
}

// Toggle the offline fallback UI and the header badge when the server's
// reachability changes. When the server stops, the page can no longer apply
// schemes, so we surface a persistent panel prompting a restart; the poll keeps
// running and clears it automatically once the server is back.
function setConnected(connected) {
  if (!TINTY_SERVE) return;
  if (serverConnected === connected) return;
  serverConnected = connected;

  const banner = document.getElementById("offline-banner");
  if (banner) banner.hidden = connected;

  const indicator = document.getElementById("live-indicator");
  if (indicator) {
    indicator.classList.toggle("is-offline", !connected);
    const label = indicator.querySelector(".live-label");
    if (label) label.textContent = connected ? "Live" : "Offline";
    if (TINTY_HOST) {
      indicator.title = connected
        ? `Live — schemes you apply here change ${TINTY_HOST}`
        : `Disconnected from the gallery server on ${TINTY_HOST}`;
    }
  }
}

// Reflect the applied scheme onto the card grid and modal. Short-circuits when
// the value is unchanged; createCard re-applies the marker on re-render.
function setAppliedScheme(id) {
  const normalized = id || null;
  if (state.appliedSchemeId === normalized) {
    updateApplyButton();
    return;
  }

  state.appliedSchemeId = normalized;
  // Re-render so the applied scheme is re-pinned to the front and the
  // highlight markers refresh. Animate the reorder when the modal is closed;
  // re-render plainly while it's open so the view transition doesn't snapshot
  // the sheet.
  if (currentSheetId) {
    render();
  } else {
    transitionLayout(render);
  }

  updateApplyButton();
}

function updateApplyButton() {
  const button = document.getElementById("apply-scheme");
  if (!button || !TINTY_SERVE) return;

  const isApplied = Boolean(currentSheetId) && currentSheetId === state.appliedSchemeId;
  button.classList.toggle("is-applied", isApplied);
  const label = button.querySelector(".apply-label");
  if (label) {
    label.textContent = isApplied ? "Applied" : "Apply";
  }
}

async function fetchCurrentScheme() {
  if (!TINTY_SERVE) return;

  try {
    const response = await fetch("api/current", { cache: "no-store" });
    // Any HTTP response means the server is reachable.
    setConnected(true);
    if (!response.ok) return;
    const data = await response.json();
    setAppliedScheme(data.scheme || null);
  } catch (_error) {
    // Server unreachable (e.g. stopped): show the offline fallback. The last
    // known applied state is left in place.
    setConnected(false);
  }
}

async function applyCurrentSheet() {
  const button = document.getElementById("apply-scheme");
  if (!button || !currentSheetId) return;

  const schemeId = currentSheetId;
  button.disabled = true;
  button.classList.add("is-applying");

  try {
    const response = await fetch("api/apply", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ scheme: schemeId }),
    });
    const data = await response.json().catch(() => ({}));

    setConnected(true);
    if (response.ok && data.ok) {
      setAppliedScheme(schemeId);
      showToast(`Applied ${schemeId}`);
    } else {
      showToast(data.error ? `Apply failed: ${data.error}` : "Apply failed");
    }
  } catch (_error) {
    setConnected(false);
    showToast("Apply failed: server unreachable");
  } finally {
    button.disabled = false;
    button.classList.remove("is-applying");
    updateApplyButton();
  }
}

// Minimum time the Retry button stays in its "waiting" state, so a click
// always reads as a deliberate attempt even when the check resolves instantly.
const RETRY_MIN_WAIT = 5000;

function delay(ms) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

// Retry button handler: enter a waiting state, run a connection check, and hold
// the waiting state for at least RETRY_MIN_WAIT before restoring the button.
// If the check reconnects, setConnected() hides the whole banner anyway.
async function retryConnection() {
  const retry = document.getElementById("offline-retry");
  if (!retry || retry.classList.contains("is-waiting")) return;

  const label = retry.querySelector(".offline-retry-label");
  retry.classList.add("is-waiting");
  retry.disabled = true;
  if (label) label.textContent = "Retrying…";

  await Promise.all([fetchCurrentScheme(), delay(RETRY_MIN_WAIT)]);

  retry.classList.remove("is-waiting");
  retry.disabled = false;
  if (label) label.textContent = "Retry";
}

function setupLiveServer() {
  if (!TINTY_SERVE) return;

  document.body.classList.add("tinty-serve");

  const indicator = document.getElementById("live-indicator");
  if (indicator) {
    const host = indicator.querySelector(".live-host");
    if (host && TINTY_HOST) {
      host.textContent = TINTY_HOST;
    }
    if (TINTY_HOST) {
      indicator.title = `Live — schemes you apply here change ${TINTY_HOST}`;
    }
    indicator.hidden = false;
  }

  const button = document.getElementById("apply-scheme");
  if (button) {
    button.hidden = false;
    button.addEventListener("click", applyCurrentSheet);
  }

  const retry = document.getElementById("offline-retry");
  if (retry) {
    retry.addEventListener("click", retryConnection);
  }

  fetchCurrentScheme();
  window.setInterval(fetchCurrentScheme, CURRENT_POLL_INTERVAL);
}

loadSavedLanguage();
loadSavedPageTheme();
syncSheetToHash();
setupLiveServer();
