"use strict";

/* ============================================================
 * Tinted Studio — static, client-only scheme editor.
 * Builds Base16 / Base24 / Tinted8 schemes and exports valid
 * scheme YAML. Tinted8 slots/tokens are derived from the eight
 * base colors per the Tinted8 spec; any derived value can be
 * overridden, and the override cleared to fall back to
 * derivation.
 * ========================================================== */

/* ---------- Scheme library (injected by `tinty studio`) ---------- */
// The Rust generator replaces __TINTY_SCHEMES__ with the JSON array of known
// schemes. When the studio source is served raw (no substitution), the token is
// undefined and we fall back to an empty library — the editor still works fully,
// only the "Start from" picker is hidden.
let LIBRARY = [];
try { LIBRARY = __TINTY_SCHEMES__; } catch (_e) { LIBRARY = []; }
if (!Array.isArray(LIBRARY)) LIBRARY = [];

function libraryEntry(id) {
  return LIBRARY.find((s) => s.id === id) || null;
}

/* ---------- Slot definitions ---------- */

const BASE16_SLOTS = [
  ["base00", "Default background"],
  ["base01", "Lighter background"],
  ["base02", "Selection background"],
  ["base03", "Comments, invisibles"],
  ["base04", "Dark foreground"],
  ["base05", "Default foreground"],
  ["base06", "Light foreground"],
  ["base07", "Lightest foreground"],
  ["base08", "Red — variables, errors"],
  ["base09", "Orange — integers, constants"],
  ["base0A", "Yellow — classes, search"],
  ["base0B", "Green — strings"],
  ["base0C", "Cyan — support, escapes"],
  ["base0D", "Blue — functions"],
  ["base0E", "Magenta — keywords"],
  ["base0F", "Brown — deprecated"],
];

const BASE24_EXTRA_SLOTS = [
  ["base10", "Darker black"],
  ["base11", "Brighter white"],
  ["base12", "Bright red"],
  ["base13", "Bright yellow"],
  ["base14", "Bright green"],
  ["base15", "Bright cyan"],
  ["base16", "Bright blue"],
  ["base17", "Bright magenta"],
];

const BASE24_SLOTS = BASE16_SLOTS.concat(BASE24_EXTRA_SLOTS);

const BASE8 = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];
const SUPPLEMENTAL = ["orange", "brown", "gray"];
const ALL11 = BASE8.concat(SUPPLEMENTAL);
const VARIANTS = ["normal", "dim", "bright"];

const TINTED8_COLOR_DESC = {
  black: "ANSI 0 — default background",
  red: "ANSI 1 — errors",
  green: "ANSI 2 — strings, success",
  yellow: "ANSI 3 — constants, warnings",
  blue: "ANSI 4 — functions",
  magenta: "ANSI 5 — keywords",
  cyan: "ANSI 6 — support, regex",
  white: "ANSI 7 — text, light backgrounds",
  orange: "Supplemental — derived from yellow",
  brown: "Supplemental — derived from yellow",
  gray: "Supplemental — derived from black + white",
};

/* ---------- Tinted8 UI token defaults (variant-aware) ---------- */
// key -> { dark: "<color>-<variant>", light: "<color>-<variant>" }
const UI_DEFAULTS = {
  "global.background.normal": { dark: "black-normal", light: "white-normal" },
  "global.background.dark": { dark: "black-dim", light: "white-dim" },
  "global.background.light": { dark: "black-bright", light: "white-bright" },
  "global.foreground.normal": { dark: "white-normal", light: "black-normal" },
  "global.foreground.dark": { dark: "white-dim", light: "black-bright" },
  "global.foreground.light": { dark: "white-bright", light: "black-dim" },
  "chrome.background.normal": { dark: "black-bright", light: "white-dim" },
  "chrome.background.dark": { dark: "black-dim", light: "gray-bright" },
  "chrome.background.light": { dark: "gray-dim", light: "white-normal" },
  "chrome.foreground.normal": { dark: "white-normal", light: "black-normal" },
  "chrome.foreground.dark": { dark: "white-dim", light: "black-dim" },
  "chrome.foreground.light": { dark: "white-bright", light: "black-bright" },
  "accent.normal": { dark: "cyan-normal", light: "cyan-normal" },
  "border.normal": { dark: "gray-dim", light: "gray-dim" },
  "cursor.normal.background": { dark: "white-normal", light: "black-normal" },
  "cursor.normal.foreground": { dark: "black-normal", light: "white-normal" },
  "cursor.muted.background": { dark: "gray-bright", light: "gray-dim" },
  "cursor.muted.foreground": { dark: "gray-dim", light: "gray-bright" },
  "gutter.background": { dark: "black-normal", light: "white-normal" },
  "gutter.foreground": { dark: "white-dim", light: "black-bright" },
  "highlight.text.background": { dark: "gray-dim", light: "white-dim" },
  "highlight.text.foreground": { dark: "white-normal", light: "black-normal" },
  "highlight.text.active-background": { dark: "gray-normal", light: "gray-normal" },
  "highlight.text.active-foreground": { dark: "white-normal", light: "black-normal" },
  "highlight.button.background": { dark: "black-bright", light: "white-dim" },
  "highlight.button.foreground": { dark: "white-normal", light: "black-normal" },
  "highlight.line.background": { dark: "gray-dim", light: "white-dim" },
  "highlight.line.foreground": { dark: "white-dim", light: "black-bright" },
  "highlight.search.background": { dark: "black-bright", light: "white-dim" },
  "highlight.search.foreground": { dark: "yellow-normal", light: "yellow-normal" },
  "indent-guide.background": { dark: "black-bright", light: "white-dim" },
  "indent-guide.active-background": { dark: "gray-dim", light: "gray-bright" },
  "link.normal.background": { dark: "black-normal", light: "white-normal" },
  "link.normal.foreground": { dark: "cyan-normal", light: "cyan-normal" },
  "selection.background": { dark: "black-bright", light: "white-dim" },
  "selection.foreground": { dark: "white-normal", light: "black-normal" },
  "selection.inactive-background": { dark: "black-bright", light: "white-dim" },
  "status.error": { dark: "red-normal", light: "red-normal" },
  "status.warning": { dark: "yellow-normal", light: "yellow-normal" },
  "status.info": { dark: "orange-normal", light: "orange-normal" },
  "status.success": { dark: "green-normal", light: "green-normal" },
  "tooltip.background": { dark: "black-dim", light: "white-bright" },
  "tooltip.foreground": { dark: "white-normal", light: "black-normal" },
  "whitespace.foreground": { dark: "gray-normal", light: "gray-normal" },
  "deprecated": { dark: "brown-normal", light: "brown-normal" },
};

/* ---------- Tinted8 syntax token defaults ---------- */
// key -> "<color>-<variant>" (dark-oriented; white<->black swapped for light).
const SYNTAX_DEFAULTS = {
  "comment": "gray-dim",
  "comment.line": "gray-dim",
  "comment.block": "gray-dim",
  "comment.documentation": "gray-dim",
  "invalid": "red-bright",
  "invalid.deprecated": "yellow-bright",
  "invalid.illegal": "red-bright",
  "string": "green-normal",
  "string.quoted": "green-normal",
  "string.quoted.single": "green-normal",
  "string.quoted.double": "green-normal",
  "string.regexp": "red-normal",
  "string.template": "green-normal",
  "string.interpolated": "green-normal",
  "string.unquoted": "green-normal",
  "string.other": "green-normal",
  "constant": "orange-normal",
  "constant.numeric": "orange-normal",
  "constant.numeric.integer": "orange-normal",
  "constant.numeric.float": "orange-normal",
  "constant.numeric.hex": "orange-normal",
  "constant.language": "orange-normal",
  "constant.character": "orange-normal",
  "constant.character.escape": "orange-normal",
  "constant.character.entity": "orange-normal",
  "constant.other": "orange-normal",
  "entity": "white-normal",
  "entity.name": "white-normal",
  "entity.name.class": "yellow-normal",
  "entity.name.function": "blue-normal",
  "entity.name.function.constructor": "blue-normal",
  "entity.name.label": "white-normal",
  "entity.name.tag": "white-normal",
  "entity.name.type": "cyan-normal",
  "entity.name.type.class": "cyan-normal",
  "entity.name.type.enum": "cyan-normal",
  "entity.name.type.struct": "cyan-normal",
  "entity.name.namespace": "yellow-dim",
  "entity.name.section": "cyan-normal",
  "entity.other": "white-normal",
  "entity.other.attribute-name": "magenta-normal",
  "entity.other.inherited-class": "white-normal",
  "keyword": "magenta-normal",
  "keyword.control": "magenta-normal",
  "keyword.control.import": "magenta-normal",
  "keyword.control.flow": "magenta-normal",
  "keyword.declaration": "magenta-normal",
  "keyword.operator": "magenta-normal",
  "keyword.other": "magenta-normal",
  "storage": "magenta-normal",
  "storage.type": "magenta-normal",
  "storage.modifier": "magenta-normal",
  "support": "blue-normal",
  "support.function": "blue-normal",
  "support.function.builtin": "blue-bright",
  "support.class": "blue-normal",
  "support.type": "blue-normal",
  "support.constant": "magenta-normal",
  "support.variable": "cyan-normal",
  "support.other": "blue-normal",
  "variable": "white-normal",
  "variable.parameter": "cyan-bright",
  "variable.language": "magenta-normal",
  "variable.other": "white-normal",
  "variable.other.constant": "white-normal",
  "variable.other.property": "white-normal",
  "variable.other.object": "white-normal",
  "punctuation": "white-dim",
  "punctuation.separator": "white-normal",
  "punctuation.definition": "white-normal",
  "punctuation.definition.string": "green-normal",
  "punctuation.definition.comment": "gray-dim",
  "punctuation.section": "orange-normal",
  "punctuation.brackets": "orange-normal",
  "punctuation.brackets.angle": "orange-normal",
  "punctuation.brackets.curly": "orange-normal",
  "punctuation.brackets.round": "orange-normal",
  "punctuation.brackets.square": "orange-normal",
  "markup": "orange-normal",
  "markup.bold": "orange-normal",
  "markup.italic": "orange-normal",
  "markup.quote": "orange-normal",
  "markup.underline": "orange-normal",
  "markup.heading": "magenta-normal",
  "markup.list": "orange-normal",
  "markup.list.numbered": "cyan-normal",
  "markup.list.unnumbered": "cyan-normal",
  "markup.link": "yellow-normal",
  "markup.raw": "orange-normal",
  "markup.inserted": "green-bright",
  "markup.changed": "yellow-bright",
  "markup.deleted": "red-bright",
  "source": "white-normal",
  "text": "white-normal",
  "meta": "white-normal",
  "meta.annotation": "orange-normal",
  "meta.function": "white-normal",
  "meta.class": "white-normal",
  "meta.block": "white-normal",
  "meta.tag": "white-normal",
  "meta.type": "white-normal",
  "meta.import": "white-normal",
  "meta.preprocessor": "white-normal",
  "meta.embedded": "white-normal",
  "meta.object": "orange-normal",
};

const UI_KEYS = Object.keys(UI_DEFAULTS);
const SYNTAX_KEYS = Object.keys(SYNTAX_DEFAULTS);

/* ---------- Preview role mapping (mirrors gallery) ---------- */

const fallbackPalette = {
  base00: "#101418", base03: "#5f6b76", base05: "#d8dee9",
  base08: "#d35f5f", base09: "#d08f4f", base0A: "#c6a84f",
  base0B: "#72a65a", base0C: "#5aa6a6", base0D: "#5f8fd3", base0E: "#9f7ad3",
};

const TINTED8_ROLE_PATHS = {
  bg: ["ui", "global.background.normal"],
  fg: ["ui", "global.foreground.normal"],
  muted: ["ui", "global.foreground.dark"],
  comment: ["syntax", "comment"],
  keyword: ["syntax", "keyword"],
  function: ["syntax", "entity.name.function"],
  string: ["syntax", "string"],
  number: ["syntax", "constant.numeric"],
  type: ["syntax", "entity.name.type"],
  builtin: ["syntax", "support.function.builtin"],
  parameter: ["syntax", "variable.parameter"],
  added: ["syntax", "markup.inserted"],
  deleted: ["syntax", "markup.deleted"],
};

const PREVIEW_ROLE_KEYS = {
  base16: {
    bg: "base00", fg: "base05", muted: "base04", comment: "base03",
    keyword: "base0E", function: "base0D", string: "base0B", number: "base09",
    deleted: "base08", added: "base0B", type: "base0A", builtin: "base0D", parameter: "base0C",
    "ansi-black": "base00", "ansi-red": "base08", "ansi-green": "base0B", "ansi-yellow": "base0A",
    "ansi-blue": "base0D", "ansi-magenta": "base0E", "ansi-cyan": "base0C", "ansi-white": "base05",
    "ansi-bright-black": "base03", "ansi-bright-red": "base08", "ansi-bright-green": "base0B",
    "ansi-bright-yellow": "base0A", "ansi-bright-blue": "base0D", "ansi-bright-magenta": "base0E",
    "ansi-bright-cyan": "base0C", "ansi-bright-white": "base07",
  },
  base24: {
    "ansi-bright-red": "base12", "ansi-bright-yellow": "base13", "ansi-bright-green": "base14",
    "ansi-bright-cyan": "base15", "ansi-bright-blue": "base16", "ansi-bright-magenta": "base17",
  },
  tinted8: {
    dark: { bg: "black-normal", fg: "white-normal", muted: "white-dim" },
    light: { bg: "white-normal", fg: "black-normal", muted: "black-dim" },
    shared: {
      comment: "gray-dim", keyword: "magenta-normal", function: "blue-normal",
      string: "green-normal", number: "orange-normal", deleted: "red-bright",
      added: "green-bright", type: "yellow-normal", builtin: "blue-bright", parameter: "cyan-bright",
      "ansi-black": "black-normal", "ansi-red": "red-normal", "ansi-green": "green-normal",
      "ansi-yellow": "yellow-normal", "ansi-blue": "blue-normal", "ansi-magenta": "magenta-normal",
      "ansi-cyan": "cyan-normal", "ansi-white": "white-normal",
      "ansi-bright-black": "black-bright", "ansi-bright-red": "red-bright",
      "ansi-bright-green": "green-bright", "ansi-bright-yellow": "yellow-bright",
      "ansi-bright-blue": "blue-bright", "ansi-bright-magenta": "magenta-bright",
      "ansi-bright-cyan": "cyan-bright", "ansi-bright-white": "white-bright",
    },
  },
};

const PREVIEW_ROLES = [
  "bg", "fg", "muted",
  "comment", "keyword", "function", "string", "number",
  "deleted", "added", "type", "builtin", "parameter",
  "ansi-black", "ansi-red", "ansi-green", "ansi-yellow",
  "ansi-blue", "ansi-magenta", "ansi-cyan", "ansi-white",
  "ansi-bright-black", "ansi-bright-red", "ansi-bright-green", "ansi-bright-yellow",
  "ansi-bright-blue", "ansi-bright-magenta", "ansi-bright-cyan", "ansi-bright-white",
];

const PALETTE_LANGUAGE = "palette";
const FALLBACK_LANGUAGE = "rust";
const TINTED8_VARIANT_ORDER = { dim: 0, normal: 1, bright: 2 };
const STYLING_SPEC = "0.2.0";

/* ---------- Default starter schemes ---------- */

const DEFAULT_BASE16 = {
  base00: "#181818", base01: "#282828", base02: "#383838", base03: "#585858",
  base04: "#b8b8b8", base05: "#d8d8d8", base06: "#e8e8e8", base07: "#f8f8f8",
  base08: "#ab4642", base09: "#dc9656", base0A: "#f7ca88", base0B: "#a1b56c",
  base0C: "#86c1b9", base0D: "#7cafc2", base0E: "#ba8baf", base0F: "#a16946",
};

const DEFAULT_BASE24 = Object.assign({}, DEFAULT_BASE16, {
  base10: "#0f0f0f", base11: "#ffffff", base12: "#ab4642", base13: "#f7ca88",
  base14: "#a1b56c", base15: "#86c1b9", base16: "#7cafc2", base17: "#ba8baf",
});

const DEFAULT_TINTED8 = {
  black: "#181818", red: "#ab4642", green: "#a1b56c", yellow: "#f7ca88",
  blue: "#7cafc2", magenta: "#ba8baf", cyan: "#86c1b9", white: "#d8d8d8",
};

/* ---------- Color math (HSL, matches tinted-builder derivation) ---------- */

function clamp01(x) {
  return Math.min(Math.max(x, 0), 1);
}

function normalizeHex(input) {
  if (typeof input !== "string") return null;
  let s = input.trim().replace(/^#/, "").toLowerCase();
  if (/^[0-9a-f]{3}$/.test(s)) {
    s = s.split("").map((c) => c + c).join("");
  }
  if (/^[0-9a-f]{6}$/.test(s)) return "#" + s;
  return null;
}

function hexToRgb(hex) {
  const h = normalizeHex(hex) || "#000000";
  return {
    r: parseInt(h.slice(1, 3), 16),
    g: parseInt(h.slice(3, 5), 16),
    b: parseInt(h.slice(5, 7), 16),
  };
}

function rgbToHex({ r, g, b }) {
  const c = (n) => Math.round(clamp01(n / 255) * 255).toString(16).padStart(2, "0");
  return "#" + c(r) + c(g) + c(b);
}

function rgbToHsl({ r, g, b }) {
  const rn = r / 255, gn = g / 255, bn = b / 255;
  const max = Math.max(rn, gn, bn), min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  let h = 0, s = 0;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === rn) h = (gn - bn) / d + (gn < bn ? 6 : 0);
    else if (max === gn) h = (bn - rn) / d + 2;
    else h = (rn - gn) / d + 4;
    h *= 60;
  }
  return { h, s, l };
}

function hslToRgb({ h, s, l }) {
  const hue = ((h % 360) + 360) % 360;
  if (s === 0) {
    const v = Math.round(clamp01(l) * 255);
    return { r: v, g: v, b: v };
  }
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((hue / 60) % 2) - 1));
  const m = l - c / 2;
  let rp = 0, gp = 0, bp = 0;
  if (hue < 60) { rp = c; gp = x; }
  else if (hue < 120) { rp = x; gp = c; }
  else if (hue < 180) { gp = c; bp = x; }
  else if (hue < 240) { gp = x; bp = c; }
  else if (hue < 300) { rp = x; bp = c; }
  else { rp = c; bp = x; }
  return {
    r: Math.round(clamp01(rp + m) * 255),
    g: Math.round(clamp01(gp + m) * 255),
    b: Math.round(clamp01(bp + m) * 255),
  };
}

function hexToHsl(hex) { return rgbToHsl(hexToRgb(hex)); }
function hslToHex(hsl) { return rgbToHex(hslToRgb(hsl)); }

const DL = 0.12;

function deriveVariant(hsl, variant) {
  let k, deltaL, l;
  if (variant === "dim") {
    k = hsl.l < 0.4 ? 1.04 : hsl.l < 0.7 ? 1.07 : 1.10;
    deltaL = Math.min(DL, hsl.l);
    l = clamp01(hsl.l - deltaL);
  } else {
    k = hsl.l < 0.5 ? 1.08 : hsl.l < 0.8 ? 1.0 : 0.9;
    deltaL = Math.min(DL, 1 - hsl.l);
    l = clamp01(hsl.l + deltaL);
  }
  return { h: hsl.h, s: clamp01(hsl.s * k), l };
}

function deriveOrange(yellowHsl) {
  return { h: ((yellowHsl.h - 10) % 360 + 360) % 360, s: yellowHsl.s, l: yellowHsl.l };
}

function deriveBrown(yellowHsl) {
  return {
    h: ((yellowHsl.h - 15) % 360 + 360) % 360,
    s: clamp01(yellowHsl.s * 0.65),
    l: clamp01(yellowHsl.l - 0.30),
  };
}

function deriveGray(blackHsl, whiteHsl) {
  const d = ((blackHsl.h - whiteHsl.h + 540) % 360) - 180;
  return {
    h: ((whiteHsl.h + 0.5 * d) % 360 + 360) % 360,
    s: 0,
    l: 0.5 * (blackHsl.l + whiteHsl.l),
  };
}

/* ---------- State ---------- */

const STATE_STORAGE_KEY = "tinty-studio-state";
const PAGE_THEME_STORAGE_KEY = "tinty-studio-page-theme";

function freshState() {
  return {
    flavor: "base16",
    language: FALLBACK_LANGUAGE,
    // Each flavor is its own workspace. `touched` (per workspace) means it has
    // been edited since last loaded/reset — drives the tab "edited" dot and the
    // "replace your work?" confirmations.
    base16: {
      meta: { name: "Untitled", author: "", slug: "", description: "", variant: "dark" },
      palette: Object.assign({}, DEFAULT_BASE16),
      loadedFrom: null,
      touched: false,
    },
    base24: {
      meta: { name: "Untitled", author: "", slug: "", description: "", variant: "dark" },
      palette: Object.assign({}, DEFAULT_BASE24),
      loadedFrom: null,
      touched: false,
    },
    tinted8: {
      meta: { name: "Untitled", author: "", slug: "", description: "", family: "", style: "", variant: "dark" },
      palette: Object.assign({}, DEFAULT_TINTED8),
      overrides: { palette: {}, ui: {}, syntax: {} },
      loadedFrom: null,
      touched: false,
    },
  };
}

let state = loadState();

function loadState() {
  try {
    const raw = localStorage.getItem(STATE_STORAGE_KEY);
    if (!raw) return freshState();
    const parsed = JSON.parse(raw);
    return mergeState(freshState(), parsed);
  } catch (_e) {
    return freshState();
  }
}

// Shallow-merge persisted values onto a fresh state so new fields added in a
// later version don't break an old saved blob.
function mergeState(base, saved) {
  if (!saved || typeof saved !== "object") return base;
  if (typeof saved.flavor === "string") base.flavor = saved.flavor;
  if (typeof saved.language === "string") base.language = saved.language;
  for (const flavor of ["base16", "base24", "tinted8"]) {
    const s = saved[flavor];
    if (!s) continue;
    if (s.meta) Object.assign(base[flavor].meta, s.meta);
    if (s.palette) Object.assign(base[flavor].palette, s.palette);
    if (typeof s.loadedFrom === "string" || s.loadedFrom === null) base[flavor].loadedFrom = s.loadedFrom;
    base[flavor].touched = Boolean(s.touched);
    if (flavor === "tinted8" && s.overrides) {
      base.tinted8.overrides.palette = s.overrides.palette || {};
      base.tinted8.overrides.ui = s.overrides.ui || {};
      base.tinted8.overrides.syntax = s.overrides.syntax || {};
    }
  }
  return base;
}

function saveState() {
  try {
    localStorage.setItem(STATE_STORAGE_KEY, JSON.stringify(state));
  } catch (_e) {
    /* storage full / unavailable — ignore */
  }
}

// Flag the current scheme as user-edited and persist. Once the user edits a
// deep-linked scheme, drop the hash from the URL so a reload restores their
// edited work instead of prompting to re-apply the linked scheme.
function markTouched() {
  const ws = state[state.flavor];
  if (!ws.touched) {
    ws.touched = true;
    renderWorkspaceTabs();
  }
  saveState();
  if (location.hash) setHash("");
}

/* ---------- Undo / redo history ---------- */
// Bounded, in-session (not persisted). Each entry is a serialized snapshot of
// the full editable state. Consecutive edits to the same control coalesce into
// one step so typing/dragging doesn't flood the history.
const MAX_HISTORY = 100;
const undoStack = [];
const redoStack = [];
let coalesceKey = null;

function snapshotState() {
  return JSON.stringify(state);
}

function restoreSnapshot(json) {
  state = mergeState(freshState(), JSON.parse(json));
  coalesceKey = null;
  saveState();
  renderAll();
  syncHistoryButtons();
  // Keep the shareable URL consistent with what's now on screen.
  setHash(state[state.flavor].loadedFrom || "");
}

// Record the pre-mutation state. `coalesceId` collapses a run of edits to the
// same control into a single undo step; pass a unique id (or null) for discrete
// actions that should always create their own step.
function pushHistory(coalesceId) {
  if (coalesceId != null && coalesceId === coalesceKey) return;
  undoStack.push(snapshotState());
  if (undoStack.length > MAX_HISTORY) undoStack.shift();
  redoStack.length = 0;
  coalesceKey = coalesceId == null ? null : coalesceId;
  syncHistoryButtons();
}

function undo() {
  if (!undoStack.length) return;
  redoStack.push(snapshotState());
  restoreSnapshot(undoStack.pop());
}

function redo() {
  if (!redoStack.length) return;
  undoStack.push(snapshotState());
  restoreSnapshot(redoStack.pop());
}

function syncHistoryButtons() {
  const u = document.getElementById("undo-btn");
  const r = document.getElementById("redo-btn");
  if (u) u.disabled = undoStack.length === 0;
  if (r) r.disabled = redoStack.length === 0;
}

/* ---------- Tinted8 effective colors ---------- */

function t8Normals() {
  const p = state.tinted8.palette;
  const ov = state.tinted8.overrides.palette;
  const normals = {};
  BASE8.forEach((c) => { normals[c] = normalizeHex(p[c]) || "#000000"; });
  const yellowHsl = hexToHsl(normals.yellow);
  normals.orange = ov.orange || hslToHex(deriveOrange(yellowHsl));
  normals.brown = ov.brown || hslToHex(deriveBrown(yellowHsl));
  normals.gray = ov.gray || hslToHex(deriveGray(hexToHsl(normals.black), hexToHsl(normals.white)));
  return normals;
}

// Full 33-entry palette keyed "<color>-<variant>".
function effectivePaletteFull() {
  const normals = t8Normals();
  const ov = state.tinted8.overrides.palette;
  const out = {};
  ALL11.forEach((c) => {
    const n = normals[c];
    out[`${c}-normal`] = n;
    const nhsl = hexToHsl(n);
    out[`${c}-dim`] = ov[`${c}-dim`] || hslToHex(deriveVariant(nhsl, "dim"));
    out[`${c}-bright`] = ov[`${c}-bright`] || hslToHex(deriveVariant(nhsl, "bright"));
  });
  return out;
}

function isLightVariant(variant) {
  return String(variant || "").trim().toLowerCase() === "light";
}

function swapForLight(colorVariant, variant) {
  if (!isLightVariant(variant)) return colorVariant;
  const dash = colorVariant.indexOf("-");
  const color = colorVariant.slice(0, dash);
  const v = colorVariant.slice(dash + 1);
  if (color === "white") return `black-${v}`;
  if (color === "black") return `white-${v}`;
  return colorVariant;
}

function effectiveUi(paletteFull, variant) {
  const ov = state.tinted8.overrides.ui;
  const out = {};
  UI_KEYS.forEach((key) => {
    out[key] = ov[key] || paletteFull[UI_DEFAULTS[key][isLightVariant(variant) ? "light" : "dark"]];
  });
  return out;
}

function effectiveSyntax(paletteFull, variant) {
  const ov = state.tinted8.overrides.syntax;
  const out = {};
  SYNTAX_KEYS.forEach((key) => {
    out[key] = ov[key] || paletteFull[swapForLight(SYNTAX_DEFAULTS[key], variant)];
  });
  return out;
}

/* ---------- Preview scheme (gallery-shaped) ---------- */

function wrapHex(map) {
  const out = {};
  for (const k in map) out[k] = { hex_str: map[k] };
  return out;
}

function previewScheme() {
  const flavor = state.flavor;
  if (flavor === "tinted8") {
    const variant = state.tinted8.meta.variant;
    const pf = effectivePaletteFull();
    return {
      system: "tinted8",
      variant,
      palette: wrapHex(pf),
      ui: wrapHex(effectiveUi(pf, variant)),
      syntax: wrapHex(effectiveSyntax(pf, variant)),
    };
  }
  const data = state[flavor];
  const palette = {};
  for (const k in data.palette) palette[k] = { hex_str: normalizeHex(data.palette[k]) || "#000000" };
  return { system: flavor, variant: data.meta.variant, palette };
}

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
      const entry = scheme[path[0]]?.[path[1]];
      if (entry?.hex_str) return entry.hex_str;
    }
  }
  const key = palettePreviewKey(scheme, role);
  return scheme.palette[key]?.hex_str || fallbackPalette[key] || fallbackPalette.base05;
}

function setPreviewColors(target, scheme) {
  PREVIEW_ROLES.forEach((role) => {
    target.style.setProperty(`--preview-${role}`, previewColor(scheme, role));
  });
}

/* ---------- Snippets / palette grid ---------- */

function getSnippet(lang) {
  const template = document.getElementById(`snippet-${lang}`);
  return template ? template.innerHTML : "";
}

function hasSnippet(lang) {
  return Boolean(document.getElementById(`snippet-${lang}`));
}

function snippetFor(lang) {
  return hasSnippet(lang) ? getSnippet(lang) : getSnippet(FALLBACK_LANGUAGE);
}

function paletteGridShape(scheme) {
  const system = String(scheme.system).toLowerCase();
  if (system === "tinted8") return { cols: 11, rows: 3 };
  if (system === "base24") return { cols: 8, rows: 3 };
  return { cols: 8, rows: 2 };
}

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

/* ---------- DOM refs ---------- */

const els = {
  propertiesGrid: document.getElementById("properties-grid"),
  paletteSlots: document.getElementById("palette-slots"),
  advancedSections: document.getElementById("advanced-sections"),
  uiSlots: document.getElementById("ui-slots"),
  syntaxSlots: document.getElementById("syntax-slots"),
  uiCount: document.getElementById("ui-count"),
  syntaxCount: document.getElementById("syntax-count"),
  slotsHint: document.getElementById("slots-hint"),
  codePreview: document.querySelector(".code-preview"),
  languageSelect: document.getElementById("language-select"),
  yamlOutput: document.getElementById("yaml-output"),
  toast: document.getElementById("toast"),
  slotRowTemplate: document.getElementById("slot-row-template"),
};

// Row registries: scope -> Map(rowKey -> refs)
const rows = { palette: new Map(), ui: new Map(), syntax: new Map() };

/* ---------- Value access ---------- */

// A "row descriptor" identifies one editable slot.
//   scope:     "palette" | "ui" | "syntax"
//   fullKey:   key into effectivePaletteFull / effectiveUi / effectiveSyntax
//   storeKey:  where the value lives (palette base normal) OR override key
//   required:  true => always explicit (no derive/clear)
function effectiveValue(desc, eff) {
  if (state.flavor !== "tinted8") {
    return normalizeHex(state[state.flavor].palette[desc.fullKey]) || "#000000";
  }
  return eff[desc.scope][desc.fullKey];
}

function isOverridden(desc) {
  if (desc.required) return false;
  if (state.flavor !== "tinted8") return false;
  if (desc.scope === "palette") {
    return Object.prototype.hasOwnProperty.call(state.tinted8.overrides.palette, desc.storeKey);
  }
  return Object.prototype.hasOwnProperty.call(state.tinted8.overrides[desc.scope], desc.storeKey);
}

function setValue(desc, hex) {
  if (state.flavor !== "tinted8") {
    state[state.flavor].palette[desc.fullKey] = hex;
    return;
  }
  if (desc.required) {
    state.tinted8.palette[desc.storeKey] = hex;
  } else if (desc.scope === "palette") {
    state.tinted8.overrides.palette[desc.storeKey] = hex;
  } else {
    state.tinted8.overrides[desc.scope][desc.storeKey] = hex;
  }
}

function clearOverride(desc) {
  if (desc.scope === "palette") delete state.tinted8.overrides.palette[desc.storeKey];
  else delete state.tinted8.overrides[desc.scope][desc.storeKey];
}

/* ---------- Validation ---------- */
// Slot inputs currently holding an empty/malformed value. Keyed per row; while
// non-empty, export is blocked and the field shows an error.
const invalidFields = new Map();
const REQUIRED_PROPS = ["name", "author"];

function fieldKey(desc) {
  return `${desc.scope}:${desc.storeKey}:${desc.fullKey}`;
}

function validateScheme() {
  const meta = state[state.flavor].meta;
  const missing = REQUIRED_PROPS.filter((k) => !String(meta[k] || "").trim());
  return { ok: missing.length === 0 && invalidFields.size === 0, missing, invalidCount: invalidFields.size };
}

function updateExportValidity() {
  const v = validateScheme();
  const copyBtn = document.getElementById("copy-yaml");
  const dlBtn = document.getElementById("download-yaml");
  if (copyBtn) copyBtn.disabled = !v.ok;
  if (dlBtn) dlBtn.disabled = !v.ok;

  const status = document.getElementById("export-status");
  if (status) {
    if (v.ok) {
      status.hidden = true;
      status.textContent = "";
    } else {
      const parts = [];
      if (v.missing.length) parts.push(`${v.missing.join(" & ")} required`);
      if (v.invalidCount) {
        parts.push(`${v.invalidCount} color${v.invalidCount > 1 ? "s" : ""} ${v.invalidCount > 1 ? "need" : "needs"} a value`);
      }
      status.textContent = parts.join(" · ");
      status.hidden = false;
    }
  }

  // Mark empty required property fields — but only once the user has started
  // editing, so a freshly loaded scheme isn't pre-littered with red.
  document.querySelectorAll("#properties-grid input[data-meta-key]").forEach((inp) => {
    const k = inp.dataset.metaKey;
    const required = REQUIRED_PROPS.includes(k);
    const empty = !String(state[state.flavor].meta[k] || "").trim();
    inp.classList.toggle("is-invalid", required && empty && state[state.flavor].touched);
  });
}

/* ---------- Slot row construction ---------- */

function paletteRowDescriptors() {
  const flavor = state.flavor;
  if (flavor === "base16" || flavor === "base24") {
    const slots = flavor === "base16" ? BASE16_SLOTS : BASE24_SLOTS;
    return slots.map(([key, desc]) => ({
      scope: "palette", fullKey: key, storeKey: key, label: key, desc, required: true, group: null,
    }));
  }
  // Tinted8: 11 colors x 3 variants
  const descs = [];
  ALL11.forEach((color) => {
    const isBase = BASE8.includes(color);
    VARIANTS.forEach((variant) => {
      const fullKey = `${color}-${variant}`;
      const required = isBase && variant === "normal";
      let storeKey;
      if (variant === "normal") storeKey = color; // base normal -> palette[color]; supp normal -> override["orange"]
      else storeKey = `${color}-${variant}`;
      descs.push({
        scope: "palette",
        fullKey,
        storeKey,
        label: variant === "normal" ? color : `${color}-${variant}`,
        desc: variant === "normal" ? TINTED8_COLOR_DESC[color] : `${variant} variant`,
        required,
        group: color,
      });
    });
  });
  return descs;
}

function buildRow(desc) {
  const node = els.slotRowTemplate.content.firstElementChild.cloneNode(true);
  const swatch = node.querySelector(".slot-swatch");
  const picker = node.querySelector(".slot-picker");
  const keyEl = node.querySelector(".slot-key");
  const descEl = node.querySelector(".slot-desc");
  const hexInput = node.querySelector(".slot-hex");
  const clearBtn = node.querySelector(".slot-clear");

  keyEl.textContent = desc.label;
  if (!desc.required && state.flavor === "tinted8") {
    const tag = document.createElement("span");
    tag.className = "derived-tag";
    tag.textContent = "derived";
    keyEl.appendChild(tag);
  }
  descEl.textContent = desc.desc || "";

  const key = fieldKey(desc);
  const commit = (raw, fromPicker) => {
    const trimmed = String(raw).trim();
    const hex = normalizeHex(trimmed);
    if (hex) {
      invalidFields.delete(key);
      hexInput.classList.remove("is-invalid");
      pushHistory(`slot:${desc.scope}:${desc.storeKey}`);
      setValue(desc, hex);
      markTouched();
      refreshValues();
      renderPreview();
      renderYaml();
      updateExportValidity();
      if (fromPicker) hexInput.value = hex;
    } else if (trimmed === "" && !desc.required) {
      // Emptying an optional/derived slot reverts it to derivation.
      invalidFields.delete(key);
      hexInput.classList.remove("is-invalid");
      if (isOverridden(desc)) {
        pushHistory(`clear:${desc.scope}:${desc.storeKey}`);
        clearOverride(desc);
        markTouched();
      }
      refreshValues();
      renderPreview();
      renderYaml();
      updateExportValidity();
    } else {
      // Empty required slot, or malformed value: flag and block export.
      invalidFields.set(key, true);
      hexInput.classList.add("is-invalid");
      markTouched();
      updateExportValidity();
    }
  };

  picker.addEventListener("input", () => commit(picker.value, true));
  hexInput.addEventListener("input", () => commit(hexInput.value, false));
  hexInput.addEventListener("blur", () => refreshValues());
  clearBtn.addEventListener("click", () => {
    invalidFields.delete(key);
    pushHistory(`clear:${desc.scope}:${desc.storeKey}`);
    clearOverride(desc);
    markTouched();
    refreshValues();
    renderPreview();
    renderYaml();
    updateExportValidity();
  });

  const refs = { desc, node, swatch, picker, keyEl, descEl, hexInput, clearBtn, tag: keyEl.querySelector(".derived-tag") };
  return refs;
}

function buildSlotList(container, descriptors, registry) {
  container.textContent = "";
  registry.clear();
  const frag = document.createDocumentFragment();
  descriptors.forEach((desc) => {
    const refs = buildRow(desc);
    registry.set(desc.storeKey + "|" + desc.fullKey, refs);
    frag.appendChild(refs.node);
  });
  container.appendChild(frag);
}

function renderSlots() {
  buildSlotList(els.paletteSlots, paletteRowDescriptors(), rows.palette);
  // Only Tinted8 has overridable slots, so only it reserves the clear column.
  const t8 = state.flavor === "tinted8";
  els.paletteSlots.classList.toggle("reserve-clear", t8);
  els.uiSlots.classList.toggle("reserve-clear", t8);
  els.syntaxSlots.classList.toggle("reserve-clear", t8);
  if (state.flavor === "tinted8") {
    els.advancedSections.hidden = false;
    buildSlotList(
      els.uiSlots,
      UI_KEYS.map((key) => ({ scope: "ui", fullKey: key, storeKey: key, label: key, desc: "", required: false, group: null })),
      rows.ui
    );
    buildSlotList(
      els.syntaxSlots,
      SYNTAX_KEYS.map((key) => ({ scope: "syntax", fullKey: key, storeKey: key, label: key, desc: "", required: false, group: null })),
      rows.syntax
    );
    els.uiCount.textContent = `${UI_KEYS.length} tokens`;
    els.syntaxCount.textContent = `${SYNTAX_KEYS.length} tokens`;
    els.slotsHint.textContent = "8 base colors required · derived slots can be overridden";
  } else {
    els.advancedSections.hidden = true;
    rows.ui.clear();
    rows.syntax.clear();
    const n = state.flavor === "base16" ? 16 : 24;
    els.slotsHint.textContent = `${n} colors`;
  }
}

/* ---------- Refresh values in place (preserves focus) ---------- */

function computeEffective() {
  if (state.flavor !== "tinted8") return null;
  const variant = state.tinted8.meta.variant;
  const pf = effectivePaletteFull();
  return { palette: pf, ui: effectiveUi(pf, variant), syntax: effectiveSyntax(pf, variant) };
}

function refreshRow(refs, eff) {
  const overridden = isOverridden(refs.desc);
  // A row whose input is currently invalid keeps its error state and the user's
  // text — don't overwrite it with the last valid value.
  if (invalidFields.has(fieldKey(refs.desc))) {
    refs.hexInput.classList.add("is-invalid");
    refs.node.classList.toggle("is-overridden", overridden);
    return;
  }
  refs.hexInput.classList.remove("is-invalid");
  const value = effectiveValue(refs.desc, eff);
  if (document.activeElement !== refs.hexInput) refs.hexInput.value = value;
  refs.picker.value = value;
  refs.swatch.style.background = value;
  refs.node.classList.toggle("is-overridden", overridden);
  if (refs.tag) refs.tag.textContent = overridden ? "custom" : "derived";
}

function refreshValues() {
  const eff = computeEffective();
  rows.palette.forEach((refs) => refreshRow(refs, eff));
  if (state.flavor === "tinted8") {
    rows.ui.forEach((refs) => refreshRow(refs, eff));
    rows.syntax.forEach((refs) => refreshRow(refs, eff));
    const uiN = Object.keys(state.tinted8.overrides.ui).length;
    const synN = Object.keys(state.tinted8.overrides.syntax).length;
    els.uiCount.textContent = uiN ? `${uiN} overridden / ${UI_KEYS.length}` : `${UI_KEYS.length} tokens`;
    els.syntaxCount.textContent = synN ? `${synN} overridden / ${SYNTAX_KEYS.length}` : `${SYNTAX_KEYS.length} tokens`;
    els.uiCount.classList.toggle("has-overrides", uiN > 0);
    els.syntaxCount.classList.toggle("has-overrides", synN > 0);
  }
}

/* ---------- Preview ---------- */

function renderPreview() {
  const scheme = previewScheme();
  setPreviewColors(els.codePreview, scheme);
  renderPreviewInto(els.codePreview, scheme, state.language);
}

/* ---------- Properties form ---------- */

function field(label, key, value, opts) {
  const wrap = document.createElement("div");
  wrap.className = "field" + (opts && opts.wide ? " field-wide" : "");
  const lbl = document.createElement("label");
  lbl.textContent = label;
  const input = document.createElement("input");
  input.type = "text";
  input.value = value || "";
  input.placeholder = opts && opts.placeholder ? opts.placeholder : "";
  input.spellcheck = false;
  input.dataset.metaKey = key;
  input.addEventListener("input", () => {
    pushHistory(`meta:${state.flavor}:${key}`);
    state[state.flavor].meta[key] = input.value;
    markTouched();
    renderYaml();
    syncLibrarySelect();
    updateExportValidity();
  });
  wrap.appendChild(lbl);
  wrap.appendChild(input);
  return wrap;
}

// The scheme spec requires `variant` to be exactly "dark" or "light"
// (styling.md: "Either dark or light"; the builder types it as an enum), so
// this is a strict two-way choice rather than free text.
function variantField() {
  const wrap = document.createElement("div");
  wrap.className = "field";
  const lbl = document.createElement("label");
  lbl.textContent = "Variant";
  const group = document.createElement("div");
  group.className = "field-variant";
  ["dark", "light"].forEach((v) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "chip" + (state[state.flavor].meta.variant === v ? " active" : "");
    btn.textContent = v.charAt(0).toUpperCase() + v.slice(1);
    btn.addEventListener("click", () => {
      if (state[state.flavor].meta.variant === v) return;
      pushHistory(`variant:${state.flavor}`);
      state[state.flavor].meta.variant = v;
      group.querySelectorAll(".chip").forEach((c) => c.classList.remove("active"));
      btn.classList.add("active");
      markTouched();
      refreshValues();
      renderPreview();
      renderYaml();
      updateExportValidity();
    });
    group.appendChild(btn);
  });
  wrap.appendChild(lbl);
  wrap.appendChild(group);
  return wrap;
}

function renderProperties() {
  const meta = state[state.flavor].meta;
  const grid = els.propertiesGrid;
  grid.textContent = "";
  const nameField = field("Name", "name", meta.name, { placeholder: "My Scheme", wide: true });
  nameField.classList.add("field-nameplate");
  // The slug is derived from the name and shown read-only beneath it.
  const slugCaption = document.createElement("p");
  slugCaption.className = "slug-caption";
  const updateSlugCaption = () => {
    const s = slugify(state[state.flavor].meta.name);
    slugCaption.textContent = `slug · ${s || "—"}`;
  };
  updateSlugCaption();
  nameField.querySelector("input").addEventListener("input", updateSlugCaption);
  nameField.appendChild(slugCaption);
  grid.appendChild(nameField);
  grid.appendChild(field("Author", "author", meta.author, { placeholder: "Your Name" }));
  grid.appendChild(variantField());
  if (state.flavor === "tinted8") {
    grid.appendChild(field("Family", "family", meta.family, { placeholder: "optional" }));
    grid.appendChild(field("Style", "style", meta.style, { placeholder: "optional" }));
  }
  grid.appendChild(field("Description", "description", meta.description, { wide: true, placeholder: "optional" }));
}

/* ---------- Slug ---------- */

// Derive a slug from a name: fold accents to ASCII, lowercase, collapse every
// run of non-alphanumerics to a single "-", and trim "-" from both ends.
function slugify(s) {
  return String(s || "")
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

// The slug is always derived from the name — never hand-edited.
function effectiveSlug(meta) {
  return slugify(meta.name) || "scheme";
}

/* ---------- YAML export ---------- */

function yamlStr(s) {
  return '"' + String(s == null ? "" : s).replace(/\\/g, "\\\\").replace(/"/g, '\\"') + '"';
}

function buildBaseYaml(flavor) {
  const data = state[flavor];
  const meta = data.meta;
  const slots = flavor === "base16" ? BASE16_SLOTS : BASE24_SLOTS;
  const lines = [];
  lines.push(`system: ${yamlStr(flavor)}`);
  lines.push(`name: ${yamlStr(meta.name || "Untitled")}`);
  if (slugify(meta.name)) lines.push(`slug: ${yamlStr(slugify(meta.name))}`);
  lines.push(`author: ${yamlStr(meta.author)}`);
  lines.push(`variant: ${yamlStr(meta.variant)}`);
  if (meta.description) lines.push(`description: ${yamlStr(meta.description)}`);
  lines.push("palette:");
  slots.forEach(([key]) => {
    lines.push(`  ${key}: ${yamlStr(normalizeHex(data.palette[key]) || "#000000")}`);
  });
  return lines.join("\n") + "\n";
}

function buildTinted8Yaml() {
  const meta = state.tinted8.meta;
  const ov = state.tinted8.overrides;
  const lines = [];
  lines.push("scheme:");
  lines.push(`  system: "tinted8"`);
  lines.push("  supports:");
  lines.push(`    styling-spec: ${yamlStr(STYLING_SPEC)}`);
  lines.push(`  author: ${yamlStr(meta.author)}`);
  if (meta.name) lines.push(`  name: ${yamlStr(meta.name)}`);
  if (slugify(meta.name)) lines.push(`  slug: ${yamlStr(slugify(meta.name))}`);
  if (meta.family) lines.push(`  family: ${yamlStr(meta.family)}`);
  if (meta.style) lines.push(`  style: ${yamlStr(meta.style)}`);
  if (meta.description) lines.push(`  description: ${yamlStr(meta.description)}`);
  lines.push(`variant: ${yamlStr(meta.variant)}`);

  lines.push("palette:");
  BASE8.forEach((c) => {
    lines.push(`  ${c}: ${yamlStr(normalizeHex(state.tinted8.palette[c]) || "#000000")}`);
  });
  // Overridden derived palette slots, in canonical order.
  ALL11.forEach((c) => {
    if (SUPPLEMENTAL.includes(c) && ov.palette[c]) {
      lines.push(`  ${c}: ${yamlStr(ov.palette[c])}`);
    }
    ["dim", "bright"].forEach((v) => {
      const key = `${c}-${v}`;
      if (ov.palette[key]) lines.push(`  ${key}: ${yamlStr(ov.palette[key])}`);
    });
  });

  const synKeys = SYNTAX_KEYS.filter((k) => ov.syntax[k]);
  if (synKeys.length) {
    lines.push("syntax:");
    synKeys.forEach((k) => lines.push(`  ${k}: ${yamlStr(ov.syntax[k])}`));
  }
  const uiKeys = UI_KEYS.filter((k) => ov.ui[k]);
  if (uiKeys.length) {
    lines.push("ui:");
    uiKeys.forEach((k) => lines.push(`  ${k}: ${yamlStr(ov.ui[k])}`));
  }
  return lines.join("\n") + "\n";
}

function buildYaml() {
  return state.flavor === "tinted8" ? buildTinted8Yaml() : buildBaseYaml(state.flavor);
}

function renderYaml() {
  els.yamlOutput.textContent = buildYaml();
}

/* ---------- Toast ---------- */

let toastTimer = null;
function showToast(message) {
  els.toast.textContent = message;
  els.toast.classList.add("open");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => els.toast.classList.remove("open"), 2000);
}

/* ---------- Download / copy ---------- */

function downloadYaml() {
  if (!validateScheme().ok) return;
  const yaml = buildYaml();
  const fileName = `${state.flavor}-${effectiveSlug(state[state.flavor].meta)}.yaml`;
  const blob = new Blob([yaml], { type: "text/yaml" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = fileName;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  showToast(`Downloaded ${fileName}`);
}

async function copyYaml() {
  if (!validateScheme().ok) return;
  const yaml = buildYaml();
  try {
    await navigator.clipboard.writeText(yaml);
    showToast("YAML copied to clipboard");
  } catch (_e) {
    showToast("Copy failed — select the text manually");
  }
}

/* ---------- Flavor + theme ---------- */

// Reflect which workspace is active and which ones have unsaved edits.
function renderWorkspaceTabs() {
  document.querySelectorAll(".ws-tab").forEach((tab) => {
    const flavor = tab.dataset.flavor;
    tab.classList.toggle("active", flavor === state.flavor);
    tab.setAttribute("aria-selected", flavor === state.flavor ? "true" : "false");
    tab.classList.toggle("is-edited", Boolean(state[flavor] && state[flavor].touched));
  });
}

function renderAll() {
  // A full re-render replaces every input with valid state, so any transient
  // invalid-input flags no longer apply.
  invalidFields.clear();
  renderWorkspaceTabs();
  renderProperties();
  renderSlots();
  refreshValues();
  renderPreview();
  renderYaml();
  populateLibrarySelect();
  syncLibrarySelect();
  updateExportValidity();
}

function setFlavor(flavor) {
  if (!["base16", "base24", "tinted8"].includes(flavor)) return;
  state.flavor = flavor;
  saveState();
  renderAll();
}

/* ---------- Scheme library: load / reconstruct ---------- */

// Reconstruct Tinted8 overrides from a fully-expanded library entry: any
// expanded value that differs from what our derivation would produce was an
// explicit author choice and becomes an override. Requires the 8 base palette
// colors to already be set on state.tinted8.palette.
function reconstructTinted8(entry) {
  const t8 = state.tinted8;
  t8.overrides = { palette: {}, ui: {}, syntax: {} };
  const variant = t8.meta.variant;

  // Pass 1: supplemental normals (orange/brown/gray) — these feed their own
  // dim/bright derivation, so resolve them before the variants.
  SUPPLEMENTAL.forEach((c) => {
    const ev = entry.palette?.[`${c}-normal`]?.hex_str;
    if (!ev) return;
    const derived = effectivePaletteFull()[`${c}-normal`];
    if (ev.toLowerCase() !== derived.toLowerCase()) t8.overrides.palette[c] = ev.toLowerCase();
  });

  // Pass 2: dim/bright for every color, compared against the (now supplemental-
  // aware) derivation. NOTE: `orange-dim` is intentionally skipped — upstream
  // tinted-builder 0.16.0 emits orange-dim == orange-bright (a known bug), so
  // library data for that one slot is unreliable and would create noise.
  let pf = effectivePaletteFull();
  ALL11.forEach((c) => {
    ["dim", "bright"].forEach((v) => {
      if (c === "orange" && v === "dim") return;
      const ev = entry.palette?.[`${c}-${v}`]?.hex_str;
      if (!ev) return;
      if (ev.toLowerCase() !== pf[`${c}-${v}`].toLowerCase()) t8.overrides.palette[`${c}-${v}`] = ev.toLowerCase();
    });
  });

  // Pass 3: ui + syntax, compared against the full (override-aware) palette.
  pf = effectivePaletteFull();
  const ui = effectiveUi(pf, variant);
  const syn = effectiveSyntax(pf, variant);
  if (entry.ui) {
    UI_KEYS.forEach((k) => {
      const ev = entry.ui[k]?.hex_str;
      if (ev && ev.toLowerCase() !== ui[k].toLowerCase()) t8.overrides.ui[k] = ev.toLowerCase();
    });
  }
  if (entry.syntax) {
    SYNTAX_KEYS.forEach((k) => {
      const ev = entry.syntax[k]?.hex_str;
      if (ev && ev.toLowerCase() !== syn[k].toLowerCase()) t8.overrides.syntax[k] = ev.toLowerCase();
    });
  }
}

function applyEntryToState(entry) {
  const flavor = String(entry.system).toLowerCase();
  if (flavor === "base16" || flavor === "base24") {
    const data = state[flavor];
    const slots = flavor === "base16" ? BASE16_SLOTS : BASE24_SLOTS;
    slots.forEach(([key]) => {
      const v = entry.palette?.[key]?.hex_str;
      if (v) data.palette[key] = v.toLowerCase();
    });
    data.meta = {
      name: entry.name || "Untitled",
      author: entry.author || "",
      slug: entry.slug || "",
      description: "",
      variant: String(entry.variant || "dark").toLowerCase() === "light" ? "light" : "dark",
    };
    data.loadedFrom = entry.id;
  } else if (flavor === "tinted8") {
    const t8 = state.tinted8;
    t8.meta = {
      name: entry.name || "Untitled",
      author: entry.author || "",
      slug: entry.slug || "",
      description: entry.description || "",
      family: entry.family || "",
      style: entry.style || "",
      variant: String(entry.variant || "dark").toLowerCase() === "light" ? "light" : "dark",
    };
    BASE8.forEach((c) => {
      const v = entry.palette?.[`${c}-normal`]?.hex_str;
      if (v) t8.palette[c] = v.toLowerCase();
    });
    reconstructTinted8(entry);
    t8.loadedFrom = entry.id;
  } else {
    return false;
  }
  state.flavor = flavor;
  return true;
}

// Load a known scheme as the starting point. Returns true if loaded.
function loadScheme(entry) {
  if (!entry) return false;
  pushHistory(`load:${entry.id}`);
  if (!applyEntryToState(entry)) {
    undoStack.pop();
    syncHistoryButtons();
    return false;
  }
  state[state.flavor].touched = false;
  saveState();
  renderAll();
  return true;
}

/* ---------- Deep linking ---------- */

function hashId() {
  return decodeURIComponent(location.hash.replace(/^#/, "")).trim();
}

function setHash(id) {
  const url = new URL(location.href);
  url.hash = id ? encodeURIComponent(id) : "";
  history.replaceState(null, "", url);
}

// Apply the scheme named in the URL hash, confirming first if it would discard
// unsaved edits. Called on load and on hashchange.
function applyDeepLink() {
  const id = hashId();
  if (!id) return;
  const entry = libraryEntry(id);
  if (!entry) return;
  const flavor = String(entry.system).toLowerCase();
  // Already showing this exact scheme, untouched — nothing to do.
  if (state[flavor]?.loadedFrom === id && state.flavor === flavor && !state[flavor].touched) return;

  // The target workspace has no edits to lose — load straight away.
  if (!state[flavor]?.touched) {
    loadScheme(entry);
    return;
  }

  // Replacing edited work needs confirmation. Defer it past two frames so the
  // current UI (correct theme + content) paints behind the dialog first,
  // instead of prompting over a bare, half-rendered shell.
  requestAnimationFrame(() => requestAnimationFrame(() => {
    if (hashId() !== id) return;
    if (!confirm(`Load “${entry.name}”? This replaces your current scheme and can't be undone.`)) {
      setHash(state[state.flavor].loadedFrom || "");
      return;
    }
    loadScheme(entry);
  }));
}

/* ---------- Reset ---------- */

// Reset the current scheme to its baseline: the scheme it was loaded from, or
// the stock default if it was started blank. Confirmed; not undoable.
function resetScheme() {
  const flavor = state.flavor;
  const loadedId = state[flavor].loadedFrom;
  const entry = loadedId ? libraryEntry(loadedId) : null;
  const target = entry ? `“${entry.name}”` : "the default starting colors";
  if (!confirm(`Reset this scheme to ${target}? Your changes will be discarded and this can't be undone.`)) return;

  if (entry) {
    loadScheme(entry);
  } else {
    pushHistory(`reset:${flavor}`);
    state[flavor] = freshState()[flavor];
    saveState();
    renderAll();
  }
  setHash(state[flavor].loadedFrom || "");
}

// Clear everything back to a blank scheme: all properties (name back to
// "Untitled"), the palette, and any Tinted8 overrides reset to stock — and drop
// any loaded-scheme identity. Confirmed.
function clearAll() {
  const flavor = state.flavor;
  const extra = flavor === "tinted8" ? ", palette and overrides" : " and palette";
  if (!confirm(`Clear everything back to a blank scheme? All properties${extra} reset to stock colors.`)) return;
  pushHistory(`clear-all:${flavor}`);
  state[flavor] = freshState()[flavor];
  saveState();
  renderAll();
  setHash("");
}

/* ---------- Library picker UI ---------- */

const SYSTEM_LABELS = { base16: "Base16", base24: "Base24", tinted8: "Tinted8" };

// The picker only offers schemes that match the active workspace — you load a
// Base16 scheme into the Base16 workspace, etc. Rebuilt on every workspace
// switch.
function populateLibrarySelect() {
  const picker = document.getElementById("library-picker");
  const select = document.getElementById("library-select");
  if (!select) return;
  const flavor = state.flavor;
  const list = LIBRARY
    .filter((s) => String(s.system).toLowerCase() === flavor)
    .sort((a, b) => String(a.name).localeCompare(String(b.name)));
  if (!list.length) {
    picker.hidden = true;
    return;
  }
  picker.hidden = false;
  select.textContent = "";
  const placeholder = document.createElement("option");
  placeholder.value = "";
  placeholder.textContent = `Select a ${SYSTEM_LABELS[flavor]} scheme…`;
  select.appendChild(placeholder);
  list.forEach((s) => {
    const opt = document.createElement("option");
    opt.value = s.id;
    opt.textContent = s.name;
    select.appendChild(opt);
  });
}

function syncLibrarySelect() {
  const select = document.getElementById("library-select");
  if (!select) return;
  const loaded = state[state.flavor].loadedFrom;
  // Once edited, the picker no longer reflects a pristine known scheme.
  select.value = !state[state.flavor].touched && loaded && libraryEntry(loaded) ? loaded : "";
}

function setPageTheme(theme) {
  const root = document.documentElement;
  if (theme === "light" || theme === "dark") root.setAttribute("data-theme", theme);
  else root.removeAttribute("data-theme");
  document.querySelectorAll(".theme-switcher .icon-button").forEach((b) => {
    b.classList.toggle("active", b.dataset.pageTheme === theme);
  });
  try { localStorage.setItem(PAGE_THEME_STORAGE_KEY, theme); } catch (_e) { /* ignore */ }
}

function loadPageTheme() {
  let theme = "system";
  try { theme = localStorage.getItem(PAGE_THEME_STORAGE_KEY) || "system"; } catch (_e) { /* ignore */ }
  setPageTheme(theme);
}

/* ---------- Wire up ---------- */

function init() {
  document.querySelectorAll(".ws-tab").forEach((tab) => {
    tab.addEventListener("click", () => setFlavor(tab.dataset.flavor));
  });
  document.querySelectorAll(".theme-switcher .icon-button").forEach((btn) => {
    btn.addEventListener("click", () => setPageTheme(btn.dataset.pageTheme));
  });

  els.languageSelect.value = state.language;
  els.languageSelect.addEventListener("change", () => {
    state.language = els.languageSelect.value;
    saveState();
    renderPreview();
  });

  document.getElementById("download-yaml").addEventListener("click", downloadYaml);
  document.getElementById("copy-yaml").addEventListener("click", copyYaml);
  document.getElementById("reset-btn").addEventListener("click", resetScheme);
  document.getElementById("clear-all-btn").addEventListener("click", clearAll);
  document.getElementById("undo-btn").addEventListener("click", undo);
  document.getElementById("redo-btn").addEventListener("click", redo);

  // Keyboard undo/redo, but only when not editing text (so native text undo in
  // the hex/name fields keeps working).
  document.addEventListener("keydown", (e) => {
    if (!(e.metaKey || e.ctrlKey) || e.key.toLowerCase() !== "z") return;
    const el = document.activeElement;
    const tag = el && el.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
    e.preventDefault();
    if (e.shiftKey) redo();
    else undo();
  });

  const librarySelect = document.getElementById("library-select");
  librarySelect.addEventListener("change", () => {
    const id = librarySelect.value;
    if (!id) { syncLibrarySelect(); return; }
    const entry = libraryEntry(id);
    if (!entry) return;
    const targetFlavor = String(entry.system).toLowerCase();
    if (state[targetFlavor]?.touched &&
        !confirm(`Start from “${entry.name}”? This replaces the ${SYSTEM_LABELS[targetFlavor]} workspace and can't be undone.`)) {
      syncLibrarySelect();
      return;
    }
    if (loadScheme(entry)) setHash(entry.id);
  });

  window.addEventListener("hashchange", applyDeepLink);

  loadPageTheme();
  renderAll();
  syncHistoryButtons();
  applyDeepLink();
}

init();
