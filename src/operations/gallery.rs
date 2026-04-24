use crate::{
    constants::ARTIFACTS_DIR,
    operations::list::{scheme_entries_json, schemes_dir_path},
    utils::{ensure_directory_exists, write_to_file},
};
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const GALLERY_DIR_NAME: &str = "gallery";
const LOGO_BYTES: &[u8] = include_bytes!("../../assets/tinted-theming-logo.png");
const FAVICON_BYTES: &[u8] = include_bytes!("../../assets/favicon.png");

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
    write_binary_file(assets_dir.join("tinted-theming-logo.png"), LOGO_BYTES)?;
    write_binary_file(assets_dir.join("favicon.png"), FAVICON_BYTES)?;

    Ok(())
}

fn write_binary_file(path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
    let mut file = File::create(path.as_ref())
        .map_err(anyhow::Error::new)
        .with_context(|| format!("Unable to create file: {}", path.as_ref().display()))?;

    file.write_all(contents)?;

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
  <link rel="icon" type="image/png" sizes="32x32" href="assets/favicon.png">
  <link rel="stylesheet" href="assets/gallery.css">
</head>
<body>
  <header class="topbar">
    <div class="brand">
      <img class="brand-logo" src="assets/tinted-theming-logo.png" alt="Tinted Theming">
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

  <div id="sheet-backdrop" class="sheet-backdrop" hidden></div>
  <aside id="detail-sheet" class="detail-sheet" aria-hidden="true" aria-labelledby="sheet-title">
    <div class="sheet-handle" aria-hidden="true"></div>
    <header class="sheet-header">
      <div class="sheet-title-group">
        <span class="sheet-title-icon" aria-hidden="true">
          <svg class="icon" viewBox="0 0 24 24"><path d="m12 3 8 4.5-8 4.5-8-4.5Z"></path><path d="m4 12 8 4.5 8-4.5"></path><path d="m4 16.5 8 4.5 8-4.5"></path></svg>
        </span>
        <h2 id="sheet-title"></h2>
      </div>
      <button type="button" id="sheet-close" class="icon-button" aria-label="Close details">
        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18"></path><path d="m6 6 12 12"></path></svg>
      </button>
    </header>
    <section id="sheet-preview" class="sheet-preview" aria-label="Code preview">
      <div class="section-label">
        <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m16 18 6-6-6-6"></path><path d="m8 6-6 6 6 6"></path></svg>
        Preview
      </div>
      <div class="preview-toolbar" aria-label="Preview language">
        <button type="button" class="chip active" data-preview-language="rust"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m12 2 9 5v10l-9 5-9-5V7Z"></path><path d="M12 8v8"></path><path d="M8 12h8"></path></svg> Rust</button>
        <button type="button" class="chip" data-preview-language="kotlin"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 4h16L4 20Z"></path><path d="M4 4v16l8-8Z"></path></svg> Kotlin</button>
        <button type="button" class="chip" data-preview-language="javascript"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="4" width="16" height="16" rx="2"></rect><path d="M9 15c.5 1 1.2 1.5 2.2 1.5 1.2 0 1.8-.7 1.8-1.7V9"></path><path d="M15 15.5c.5.7 1.1 1 1.8 1 .8 0 1.2-.4 1.2-.9 0-.6-.5-.8-1.4-1.1-1-.3-1.6-.8-1.6-1.8 0-1 .8-1.7 2-1.7.8 0 1.4.2 1.9.7"></path></svg> JavaScript</button>
        <button type="button" class="chip" data-preview-language="lisp"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M9 5c-3 2-4.5 4.4-4.5 7S6 17 9 19"></path><path d="M15 5c3 2 4.5 4.4 4.5 7S18 17 15 19"></path><circle cx="12" cy="12" r="1.5"></circle></svg> Lisp</button>
        <button type="button" class="chip" data-preview-language="zsh"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="m4 7 5 5-5 5"></path><path d="M11 17h9"></path></svg> Zsh</button>
      </div>
      <pre class="code-preview sheet-code-preview"><code id="sheet-code"></code></pre>
    </section>
    <div class="command-row">
      <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"></rect><path d="m8 9 3 3-3 3"></path><path d="M13 15h3"></path></svg>
      <code id="sheet-command"></code>
      <button type="button" id="copy-command" class="chip"><svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><rect x="8" y="8" width="11" height="11" rx="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v1"></path></svg> Copy</button>
    </div>
    <div class="section-label">
      <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16"></path><path d="M4 12h16"></path><path d="M4 17h16"></path><path d="M8 7v10"></path></svg>
      Properties
    </div>
    <dl id="sheet-metadata" class="metadata"></dl>
    <div class="section-label">
      <svg class="icon" viewBox="0 0 24 24" aria-hidden="true"><circle cx="13.5" cy="6.5" r="2.5"></circle><circle cx="17.5" cy="13.5" r="2.5"></circle><circle cx="8.5" cy="14.5" r="2.5"></circle><path d="M11.3 8.3 9.7 12.3"></path><path d="m15.5 8.7 1.1 2.4"></path></svg>
      Palette
    </div>
    <div id="sheet-palette" class="palette"></div>
  </aside>

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

.brand {
  display: flex;
  align-items: center;
  gap: 14px;
}

.brand-logo {
  width: clamp(42px, 5vw, 68px);
  height: clamp(42px, 5vw, 68px);
  object-fit: contain;
  flex: 0 0 auto;
}

.topbar-actions {
  display: flex;
  align-items: end;
  flex-direction: column;
  gap: 10px;
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
.meta-pill,
.icon-button,
.command-row,
.sheet-title-group,
.section-label {
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

.filter-group,
.theme-switcher,
.preview-toolbar {
  display: flex;
  flex-wrap: wrap;
  overflow: hidden;
  width: fit-content;
  max-width: 100%;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: color-mix(in srgb, var(--panel) 88%, var(--ink));
}

.chip {
  min-height: 40px;
  border: 0;
  border-left: 1px solid var(--border);
  border-radius: 0;
  padding: 8px 11px;
  background: transparent;
  color: inherit;
  font: inherit;
  cursor: pointer;
  transition: background-color 120ms ease, border-color 120ms ease, color 120ms ease, transform 120ms ease;
}

.chip:first-child {
  border-left: 0;
}

.chip:hover {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.chip.active {
  background: color-mix(in srgb, var(--accent) 16%, transparent);
  color: var(--accent);
}

.gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  padding: 20px 0 40px;
}

.card {
  overflow: hidden;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  transition: border-color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
  view-transition-name: match-element;
}

.card:hover,
.card:focus-within {
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

.metadata {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 6px 12px;
  margin: 0 0 14px;
  font-size: 13px;
}

.metadata dt {
  color: var(--ink);
  font-weight: 760;
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

.sheet-backdrop {
  position: fixed;
  inset: 0;
  z-index: 20;
  background: rgb(0 0 0 / 26%);
  backdrop-filter: blur(8px) saturate(110%);
  opacity: 0;
  transition: opacity 170ms ease;
}

.sheet-backdrop.open {
  opacity: 1;
}

@supports not (backdrop-filter: blur(1px)) {
  .sheet-backdrop {
    background: rgb(0 0 0 / 42%);
  }
}

.detail-sheet {
  position: fixed;
  left: 50%;
  bottom: 0;
  z-index: 21;
  width: min(920px, calc(100% - 24px));
  max-height: calc(100vh - 18px);
  overflow: auto;
  padding: 10px 20px 22px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-bottom: 0;
  border-radius: 12px 12px 0 0;
  box-shadow: 0 -18px 48px rgb(0 0 0 / 20%);
  transform: translate(-50%, calc(100% + 18px));
  transition: transform 220ms cubic-bezier(.2, .8, .2, 1);
}

.detail-sheet.open {
  transform: translate(-50%, 0);
}

.sheet-handle {
  width: 48px;
  height: 4px;
  margin: 0 auto 14px;
  border-radius: 999px;
  background: var(--border);
}

.sheet-header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 12px;
}

.sheet-header h2 {
  overflow-wrap: anywhere;
  font-size: clamp(22px, 3vw, 34px);
  font-weight: 760;
  line-height: 1.05;
}

.sheet-title-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  flex: 0 0 36px;
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
}

.icon-button {
  min-width: 40px;
  min-height: 40px;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.command-row {
  justify-content: start;
  width: 100%;
  margin-bottom: 12px;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: color-mix(in srgb, var(--panel) 86%, var(--ink));
}

.command-row code {
  flex: 1 1 auto;
  overflow: auto;
  font: 12px/1.35 ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
}

.command-row .chip {
  min-height: 30px;
  padding: 4px 9px;
}

.sheet-preview {
  margin-bottom: 10px;
}

.section-label {
  margin: 0 0 8px;
  color: var(--muted);
  font-size: 12px;
  font-weight: 760;
  text-transform: uppercase;
}

.preview-toolbar {
  margin-bottom: 0;
  border-radius: 8px 8px 0 0;
}

.sheet-code-preview {
  min-height: 210px;
  border-radius: 0 0 8px 8px;
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

  .brand {
    gap: 10px;
  }

  .topbar-actions {
    align-items: start;
  }

  .controls {
    grid-template-columns: 1fr;
  }

  .filter-group,
  .theme-switcher,
  .preview-toolbar {
    width: 100%;
  }

  .chip {
    flex: 1 1 auto;
    justify-content: center;
  }

  .gallery {
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  }

  .detail-sheet {
    width: 100%;
    max-height: calc(100vh - 8px);
    padding-inline: 14px;
  }
}

@media (max-width: 560px) {
  .gallery {
    grid-template-columns: 1fr;
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

SCHEMES.sort((a, b) => a.id.localeCompare(b.id));

const previewSnippets = {
  rust: `<span class="comment">// preview.rs</span>
<span class="keyword">fn</span> <span class="function">render_scheme</span>() {
    <span class="keyword">let</span> name = <span class="string">"tinty"</span>;
    <span class="keyword">let</span> colors = <span class="number">16</span>;
    <span class="function">apply</span>(name, colors);
}`,
  kotlin: `<span class="comment">// Preview.kt</span>
<span class="keyword">fun</span> <span class="function">renderScheme</span>() {
    <span class="keyword">val</span> name = <span class="string">"tinty"</span>
    <span class="keyword">val</span> colors = <span class="number">16</span>
    <span class="function">apply</span>(name, colors)
}`,
  javascript: `<span class="comment">// preview.js</span>
<span class="keyword">function</span> <span class="function">renderScheme</span>() {
  <span class="keyword">const</span> name = <span class="string">"tinty"</span>;
  <span class="keyword">const</span> colors = <span class="number">16</span>;
  <span class="function">apply</span>(name, colors);
}`,
  lisp: `<span class="comment">;; preview.lisp</span>
(<span class="keyword">defun</span> <span class="function">render-scheme</span> ()
  (<span class="keyword">let</span> ((name <span class="string">"tinty"</span>)
        (colors <span class="number">16</span>))
    (<span class="function">apply</span> name colors)))`,
  zsh: `<span class="comment"># preview.zsh</span>
<span class="keyword">function</span> <span class="function">render_scheme</span>() {
  <span class="keyword">local</span> name=<span class="string">"tinty"</span>
  <span class="keyword">local</span> colors=<span class="number">16</span>
  <span class="function">apply</span> <span class="string">"$name"</span> <span class="string">"$colors"</span>
}`,
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

function setPreviewLanguage(language) {
  document.getElementById("sheet-code").innerHTML = previewSnippets[language] || previewSnippets.rust;
  document
    .querySelectorAll("[data-preview-language]")
    .forEach((candidate) => candidate.classList.toggle("active", candidate.dataset.previewLanguage === language));
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

function openSheet(scheme) {
  const sheet = document.getElementById("detail-sheet");
  const backdrop = document.getElementById("sheet-backdrop");
  const command = `tinty apply ${scheme.id}`;

  setPreviewColors(sheet, scheme);
  setPreviewLanguage("rust");
  document.getElementById("sheet-title").textContent = scheme.name;
  document.getElementById("sheet-command").textContent = command;
  document.getElementById("copy-command").dataset.command = command;

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

  backdrop.hidden = false;
  requestAnimationFrame(() => {
    backdrop.classList.add("open");
    sheet.classList.add("open");
    sheet.setAttribute("aria-hidden", "false");
  });
}

function closeSheet() {
  const sheet = document.getElementById("detail-sheet");
  const backdrop = document.getElementById("sheet-backdrop");

  sheet.classList.remove("open");
  backdrop.classList.remove("open");
  sheet.setAttribute("aria-hidden", "true");
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
  card.querySelector("h2").textContent = scheme.slug;
  card.querySelector(".card-title p").textContent = scheme.name;
  card.querySelector(".scheme-system span").textContent = scheme.system;
  card.querySelector(".scheme-appearance span").textContent = appearance(scheme);

  card.querySelector(".preview-button").addEventListener("click", () => {
    openSheet(scheme);
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

document.querySelectorAll("[data-preview-language]").forEach((button) => {
  button.addEventListener("click", () => {
    setPreviewLanguage(button.dataset.previewLanguage);
  });
});

document.getElementById("sheet-close").addEventListener("click", closeSheet);
document.getElementById("sheet-backdrop").addEventListener("click", closeSheet);
document.getElementById("copy-command").addEventListener("click", async (event) => {
  const button = event.currentTarget;
  const originalText = button.textContent;

  try {
    await navigator.clipboard.writeText(button.dataset.command);
    button.textContent = "Copied";
    window.setTimeout(() => {
      button.textContent = originalText;
    }, 1100);
  } catch (_error) {
    button.textContent = "Copy failed";
    window.setTimeout(() => {
      button.textContent = originalText;
    }, 1400);
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeSheet();
  }
});

render();
"##;
