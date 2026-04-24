use crate::{
    constants::ARTIFACTS_DIR,
    operations::list::{scheme_entries_json, schemes_dir_path},
    utils::{ensure_directory_exists, write_to_file},
};
use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const GALLERY_DIR_NAME: &str = "gallery";

pub fn gallery(
    data_path: &Path,
    is_custom: bool,
    dump_dir: Option<&str>,
    should_open: bool,
) -> Result<PathBuf> {
    let schemes_path = schemes_dir_path(data_path, is_custom)?;
    let schemes_json = scheme_entries_json(&schemes_path)?;
    let output_dir = dump_dir.map_or_else(
        || data_path.join(ARTIFACTS_DIR).join(GALLERY_DIR_NAME),
        PathBuf::from,
    );

    write_gallery_files(&output_dir, &schemes_json)?;

    let index_path = output_dir.join("index.html");
    if should_open {
        open_in_browser(&index_path)?;
    }

    println!("Gallery written to {}", index_path.display());

    Ok(index_path)
}

fn write_gallery_files(output_dir: &Path, schemes_json: &str) -> Result<()> {
    let assets_dir = output_dir.join("assets");

    ensure_directory_exists(output_dir)?;
    ensure_directory_exists(&assets_dir)?;

    write_to_file(output_dir.join("index.html"), INDEX_HTML)?;
    write_to_file(assets_dir.join("gallery.css"), GALLERY_CSS)?;
    let gallery_js = format!("const SCHEMES = {schemes_json};\n\n{GALLERY_JS}");
    write_to_file(assets_dir.join("gallery.js"), &gallery_js)?;

    Ok(())
}

fn open_in_browser(index_path: &Path) -> Result<()> {
    let index_path = index_path
        .canonicalize()
        .with_context(|| format!("Unable to resolve {}", index_path.display()))?;

    let mut command = browser_command(&index_path);
    let status = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("Unable to open gallery at {}", index_path.display()))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Unable to open gallery at {}",
            index_path.display()
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn browser_command(path: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn browser_command(path: &Path) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", ""]).arg(path);
    command
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn browser_command(path: &Path) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    command
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Tinty Gallery</title>
  <link rel="stylesheet" href="assets/gallery.css">
</head>
<body>
  <header class="topbar">
    <div>
      <h1>Tinty Gallery</h1>
    </div>
    <div class="topbar-actions">
      <div class="theme-switcher" aria-label="Page theme">
        <button type="button" class="chip active" data-page-theme="system">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="13" rx="2"></rect><path d="M8 21h8"></path><path d="M12 17v4"></path></svg>
          System
        </button>
        <button type="button" class="chip" data-page-theme="dark">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3a6 6 0 0 0 9 7.4A9 9 0 1 1 12 3Z"></path></svg>
          Dark
        </button>
        <button type="button" class="chip" data-page-theme="light">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"></circle><path d="M12 2v2"></path><path d="M12 20v2"></path><path d="m4.93 4.93 1.41 1.41"></path><path d="m17.66 17.66 1.41 1.41"></path><path d="M2 12h2"></path><path d="M20 12h2"></path><path d="m6.34 17.66-1.41 1.41"></path><path d="m19.07 4.93-1.41 1.41"></path></svg>
          Light
        </button>
      </div>
      <output id="result-count" aria-live="polite"></output>
    </div>
  </header>

  <main>
    <section class="controls" aria-label="Gallery controls">
      <label class="search">
        <span><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="8"></circle><path d="m21 21-4.35-4.35"></path></svg> Search</span>
        <input id="search" type="search" autocomplete="off" placeholder="Scheme, author, system">
      </label>
      <div class="filter-group" aria-label="Scheme system">
        <button type="button" class="chip active" data-filter="system" data-value="all">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m12 3 8 4.5-8 4.5-8-4.5Z"></path><path d="m4 12 8 4.5 8-4.5"></path><path d="m4 16.5 8 4.5 8-4.5"></path></svg>
          All systems
        </button>
        <button type="button" class="chip" data-filter="system" data-value="base16">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="3" width="7" height="7" rx="1"></rect><rect x="14" y="3" width="7" height="7" rx="1"></rect><rect x="3" y="14" width="7" height="7" rx="1"></rect><rect x="14" y="14" width="7" height="7" rx="1"></rect></svg>
          Base16
        </button>
        <button type="button" class="chip" data-filter="system" data-value="base24">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="6" height="6" rx="1"></rect><rect x="14" y="4" width="6" height="6" rx="1"></rect><rect x="4" y="14" width="6" height="6" rx="1"></rect><path d="M14 17h6"></path><path d="M17 14v6"></path></svg>
          Base24
        </button>
        <button type="button" class="chip" data-filter="system" data-value="tinted8">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 22a7 7 0 0 0 7-7c0-5-7-13-7-13S5 10 5 15a7 7 0 0 0 7 7Z"></path></svg>
          Tinted8
        </button>
      </div>
      <div class="filter-group" aria-label="Appearance">
        <button type="button" class="chip active" data-filter="appearance" data-value="all">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9"></circle><path d="M12 3v18"></path></svg>
          All appearances
        </button>
        <button type="button" class="chip" data-filter="appearance" data-value="dark">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3a6 6 0 0 0 9 7.4A9 9 0 1 1 12 3Z"></path></svg>
          Dark
        </button>
        <button type="button" class="chip" data-filter="appearance" data-value="light">
          <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"></circle><path d="M12 2v2"></path><path d="M12 20v2"></path><path d="M2 12h2"></path><path d="M20 12h2"></path></svg>
          Light
        </button>
      </div>
    </section>

    <section id="gallery" class="gallery" aria-label="Scheme previews"></section>
    <p id="empty" class="empty" hidden>No matching schemes.</p>
  </main>

  <template id="card-template">
    <article class="card">
      <button type="button" class="preview-button">
        <div class="card-header">
          <span class="scheme-system meta-pill">
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m12 3 8 4.5-8 4.5-8-4.5Z"></path><path d="m4 12 8 4.5 8-4.5"></path></svg>
            <span></span>
          </span>
          <span class="scheme-appearance meta-pill">
            <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9"></circle><path d="M12 3v18"></path></svg>
            <span></span>
          </span>
        </div>
        <pre class="code-preview"><code><span class="comment">// preview.rs</span>
<span class="keyword">fn</span> <span class="function">render_scheme</span>() {
    <span class="keyword">let</span> name = <span class="string">"tinty"</span>;
    <span class="keyword">let</span> colors = <span class="number">16</span>;
    <span class="function">apply</span>(name, colors);
}</code></pre>
        <div class="card-title">
          <h2></h2>
          <p></p>
        </div>
      </button>
      <div class="details" aria-hidden="true">
        <dl class="metadata"></dl>
        <div class="palette"></div>
      </div>
    </article>
  </template>

  <script src="assets/gallery.js"></script>
</body>
</html>
"#;

const GALLERY_CSS: &str = r#":root {
  color-scheme: light dark;
  --page: #f5f6f8;
  --ink: #1f2933;
  --muted: #64717f;
  --border: #d9dee5;
  --panel: #ffffff;
  --accent: #1b6fd8;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: var(--page);
  color: var(--ink);
  font: 15px/1.5 Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-feature-settings: "cv02", "cv03", "cv04", "cv11";
}

.topbar,
main {
  width: min(1440px, calc(100% - 32px));
  margin: 0 auto;
}

.topbar {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 24px;
  padding: 30px 0 20px;
}

.topbar-actions {
  display: flex;
  align-items: end;
  flex-direction: column;
  gap: 10px;
}

.theme-switcher {
  display: flex;
  gap: 8px;
}

.icon {
  width: 16px;
  height: 16px;
  flex: 0 0 16px;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  fill: none;
}

.chip,
.search span,
.meta-pill {
  display: inline-flex;
  align-items: center;
  gap: 7px;
}

.card-title p,
.metadata,
#result-count {
  color: var(--muted);
}

h1,
h2,
p {
  margin: 0;
}

h1 {
  font-size: clamp(32px, 4vw, 56px);
  font-weight: 760;
  line-height: .95;
  letter-spacing: 0;
}

.controls {
  display: grid;
  grid-template-columns: minmax(240px, 1fr) auto auto;
  gap: 12px;
  align-items: end;
  padding: 16px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.search span {
  margin-bottom: 6px;
  color: var(--muted);
  font-size: 12px;
  font-weight: 700;
}

.search input {
  width: 100%;
  min-height: 40px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 10px;
  background: transparent;
  color: inherit;
  font: inherit;
}

.filter-group {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.chip {
  min-height: 40px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 11px;
  background: transparent;
  color: inherit;
  font: inherit;
  cursor: pointer;
  transition: background-color 120ms ease, border-color 120ms ease, color 120ms ease, transform 120ms ease;
}

.chip:hover {
  transform: translateY(-1px);
}

.chip.active {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--accent);
}

.gallery {
  column-count: 4;
  column-gap: 16px;
  padding: 20px 0 40px;
}

.card {
  display: inline-block;
  width: 100%;
  margin: 0 0 16px;
  break-inside: avoid;
  page-break-inside: avoid;
  overflow: hidden;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  transition: border-color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
  view-transition-name: match-element;
}

.card:hover,
.card.expanded {
  border-color: color-mix(in srgb, var(--accent) 38%, var(--border));
  box-shadow: 0 12px 30px rgb(0 0 0 / 10%);
}

.preview-button {
  display: block;
  width: 100%;
  border: 0;
  padding: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.card-header {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 12px;
  color: var(--preview-muted);
  background: var(--preview-bg);
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
}

.code-preview {
  min-height: 188px;
  margin: 0;
  padding: 16px;
  overflow: auto;
  background: var(--preview-bg);
  color: var(--preview-fg);
  font: 13px/1.55 ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
}

.comment {
  color: var(--preview-comment);
}

.keyword {
  color: var(--preview-keyword);
}

.function {
  color: var(--preview-function);
}

.string {
  color: var(--preview-string);
}

.number {
  color: var(--preview-number);
}

.card-title {
  padding: 14px 16px 16px;
}

.card-title h2 {
  overflow-wrap: anywhere;
  font-size: 16px;
  font-weight: 720;
  line-height: 1.2;
  letter-spacing: 0;
}

.details {
  max-height: 0;
  overflow: hidden;
  border-top: 0 solid transparent;
  padding: 0 16px;
  opacity: 0;
  transition: max-height 190ms cubic-bezier(.2, .8, .2, 1), opacity 120ms ease, padding 190ms cubic-bezier(.2, .8, .2, 1), border-color 160ms ease;
}

.card.expanded .details {
  border-top-width: 1px;
  border-top-color: var(--border);
  padding: 14px 16px 16px;
  opacity: 1;
}

.metadata {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 6px 12px;
  margin: 0 0 14px;
  font-size: 13px;
}

.metadata dd {
  margin: 0;
  color: var(--ink);
  overflow-wrap: anywhere;
}

.palette {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(92px, 1fr));
  gap: 8px;
}

.swatch {
  min-width: 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
  background: var(--panel);
}

.swatch-color {
  height: 34px;
}

.swatch-label {
  padding: 6px 7px;
  font: 12px/1.3 ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
}

.swatch-label span {
  display: block;
  overflow-wrap: anywhere;
  color: var(--muted);
}

.empty {
  padding: 40px 0;
  color: var(--muted);
  text-align: center;
}

@media (max-width: 860px) {
  .topbar,
  main {
    width: min(100% - 20px, 720px);
  }

  .topbar {
    align-items: start;
    flex-direction: column;
  }

  .topbar-actions {
    align-items: start;
  }

  .controls {
    grid-template-columns: 1fr;
  }

  .gallery {
    column-count: 2;
  }
}

@media (max-width: 560px) {
  .gallery {
    column-count: 1;
  }
}

@media (prefers-color-scheme: dark) {
  :root {
    --page: #161a1f;
    --ink: #e7ebef;
    --muted: #9ca8b4;
    --border: #303842;
    --panel: #1e242b;
    --accent: #7db4ff;
  }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    scroll-behavior: auto !important;
    transition-duration: 1ms !important;
    animation-duration: 1ms !important;
  }
}

::view-transition-old(root),
::view-transition-new(root) {
  animation-duration: 160ms;
  animation-timing-function: cubic-bezier(.2, .8, .2, 1);
}

:root[data-theme="light"] {
  color-scheme: light;
  --page: #f5f6f8;
  --ink: #1f2933;
  --muted: #64717f;
  --border: #d9dee5;
  --panel: #ffffff;
  --accent: #1b6fd8;
}

:root[data-theme="dark"] {
  color-scheme: dark;
  --page: #161a1f;
  --ink: #e7ebef;
  --muted: #9ca8b4;
  --border: #303842;
  --panel: #1e242b;
  --accent: #7db4ff;
}
"#;

const GALLERY_JS: &str = r##"const state = {
  search: "",
  system: "all",
  appearance: "all",
  pageTheme: "system",
};

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

function color(scheme, key) {
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
    scheme.author,
    scheme.system,
    scheme.variant,
    appearance(scheme),
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
  card.style.setProperty("--preview-bg", color(scheme, "base00"));
  card.style.setProperty("--preview-fg", color(scheme, "base05"));
  card.style.setProperty("--preview-muted", color(scheme, "base04"));
  card.style.setProperty("--preview-comment", color(scheme, "base03"));
  card.style.setProperty("--preview-keyword", color(scheme, "base0E"));
  card.style.setProperty("--preview-function", color(scheme, "base0D"));
  card.style.setProperty("--preview-string", color(scheme, "base0B"));
  card.style.setProperty("--preview-number", color(scheme, "base09"));
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

function setExpanded(card, details, expanded) {
  card.classList.toggle("expanded", expanded);
  details.setAttribute("aria-hidden", String(!expanded));
  details.style.maxHeight = expanded ? `${details.scrollHeight}px` : "0px";
}

function createCard(scheme) {
  const template = document.getElementById("card-template");
  const card = template.content.firstElementChild.cloneNode(true);
  const details = card.querySelector(".details");
  const metadata = card.querySelector(".metadata");

  setPreviewColors(card, scheme);
  card.querySelector("h2").textContent = scheme.name;
  card.querySelector(".card-title p").textContent = scheme.id;
  card.querySelector(".scheme-system span").textContent = scheme.system;
  card.querySelector(".scheme-appearance span").textContent = appearance(scheme);

  metadata.append(
    metadataItem("ID", scheme.id),
    metadataItem("Author", scheme.author),
    metadataItem("System", scheme.system),
    metadataItem("Variant", scheme.variant),
    metadataItem("Background L*", scheme.lightness?.background?.toFixed(2)),
    metadataItem("Foreground L*", scheme.lightness?.foreground?.toFixed(2)),
  );
  renderPalette(card.querySelector(".palette"), scheme);

  card.querySelector(".preview-button").addEventListener("click", () => {
    const expanded = !card.classList.contains("expanded");
    transitionLayout(() => setExpanded(card, details, expanded));
  });

  return card;
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
  document.documentElement.dataset.theme = theme === "system" ? "" : theme;
  if (theme === "system") {
    document.documentElement.removeAttribute("data-theme");
    setFilter("appearance", "all");
  } else {
    setFilter("appearance", theme);
  }

  document
    .querySelectorAll("[data-page-theme]")
    .forEach((candidate) => candidate.classList.toggle("active", candidate.dataset.pageTheme === theme));

  render();
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

render();
"##;
