const SCHEMES = __TINTY_SCHEMES__;

const state = {
  search: "",
  system: "all",
  appearance: "all",
  pageTheme: "system",
  language: "rust",
  variablesView: "palette",
};
let currentSheetId = null;
let tooltipTimeoutId = null;
let isFirstRender = true;
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

function getSnippet(lang) {
  const template = document.getElementById(`snippet-${lang}`);
  return template ? template.innerHTML : "";
}

function hasSnippet(lang) {
  return Boolean(document.getElementById(`snippet-${lang}`));
}

const FALLBACK_LANGUAGE = "rust";

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

function snippetFor(lang) {
  return hasSnippet(lang) ? getSnippet(lang) : getSnippet(FALLBACK_LANGUAGE);
}

function setPreviewLanguage(language) {
  document.getElementById("sheet-code").innerHTML = snippetFor(language);
  document
    .querySelectorAll("[data-preview-language]")
    .forEach((candidate) => candidate.classList.toggle("active", candidate.dataset.previewLanguage === language));
}

function setLanguage(lang) {
  state.language = lang;
  window.localStorage.setItem(LANGUAGE_STORAGE_KEY, lang);
  document.getElementById("language-select").value = lang;
  setPreviewLanguage(lang);
  const html = snippetFor(lang);
  document.querySelectorAll(".card .code-preview code").forEach((el) => {
    el.innerHTML = html;
  });
}

function loadSavedLanguage() {
  const saved = window.localStorage.getItem(LANGUAGE_STORAGE_KEY);
  if (saved && hasSnippet(saved)) {
    state.language = saved;
    document.getElementById("language-select").value = saved;
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

function relativeLuminance(rgb) {
  const channels = rgb.map((c) => {
    const norm = c / 255;
    return norm <= 0.03928 ? norm / 12.92 : Math.pow((norm + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function pillTextColor(rgb) {
  return relativeLuminance(rgb) > 0.45 ? "#0b0d10" : "#f8f9fb";
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
      pill.style.color = pillTextColor(value.rgb);

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
  card.querySelector(".code-preview code").innerHTML = snippetFor(state.language);

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
  setLanguage(event.target.value);
});

document.querySelectorAll("[data-preview-language]").forEach((button) => {
  button.addEventListener("click", () => {
    setLanguage(button.dataset.previewLanguage);
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

loadSavedLanguage();
loadSavedPageTheme();
syncSheetToHash();
