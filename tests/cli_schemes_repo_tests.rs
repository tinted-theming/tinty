//! Integration tests for overriding the built-in schemes repository via the
//! `[schemes]` table's `path` and `revision` keys.
//!
//! The override tests are fully offline and deterministic: they build throwaway
//! **local** git repositories (addressed with `file://` URLs to exercise the
//! clone path) and plain local directories (to exercise the symlink path). Each
//! config also declares a local-directory `[[items]]` entry so that `install`
//! never falls back to cloning the default `tinted-shell` item over the network.
//!
//! The backwards-compatibility test uses the cached real schemes repo (network
//! on first run only, like the other suites).

mod utils;

use std::fs;
use std::fs::symlink_metadata;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use utils::{build_command_vec, run_command, setup, write_to_file};

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .with_context(|| format!("failed to run git {args:?} in {}", dir.display()))?;
    ensure!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

fn head(dir: &Path) -> Result<String> {
    Ok(git(dir, &["rev-parse", "HEAD"])?.trim().to_string())
}

/// A minimal, valid base16 scheme YAML for the given slug.
fn scheme_yaml(slug: &str, name: &str) -> String {
    format!(
        "system: base16\nname: {name}\nslug: {slug}\nauthor: Tinty Test\nvariant: dark\npalette:\n  base00: '#282628'\n  base01: '#403e3f'\n  base02: '#595757'\n  base03: '#71706e'\n  base04: '#8a8986'\n  base05: '#a2a29d'\n  base06: '#bbbbb5'\n  base07: '#d4d4cd'\n  base08: '#bf2546'\n  base09: '#f69622'\n  base0A: '#f99923'\n  base0B: '#19953f'\n  base0C: '#40dab9'\n  base0D: '#0666dc'\n  base0E: '#8554ac'\n  base0F: '#ac7424'\n"
    )
}

/// Writes `base16/<slug>.yaml` under `dir`, creating the system subdir.
fn write_scheme(dir: &Path, slug: &str, name: &str) -> Result<()> {
    write_to_file(
        dir.join("base16").join(format!("{slug}.yaml")),
        &scheme_yaml(slug, name),
    )
}

/// Initializes a git repo at `dir` on `main` with a single base16 scheme.
fn git_init(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    git(dir, &["init", "-q", "-b", "main"])?;
    git(dir, &["config", "user.email", "tinty@test.local"])?;
    git(dir, &["config", "user.name", "tinty test"])?;
    // Never sign throwaway test commits — commit signing (e.g. via 1Password)
    // would otherwise hang or fail in CI/dev environments.
    git(dir, &["config", "commit.gpgsign", "false"])?;
    Ok(())
}

fn init_scheme_repo(dir: &Path, slug: &str, name: &str) -> Result<()> {
    git_init(dir)?;
    write_scheme(dir, slug, name)?;
    git(dir, &["add", "-A"])?;
    git(dir, &["commit", "-q", "-m", "init"])?;
    Ok(())
}

/// A `file://` URL for a local directory, so tinty treats it as a Git remote to
/// clone rather than a local path to symlink.
fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

/// A `[[items]]` entry backed by a throwaway local git repo (addressed by
/// `file://`), so both `install` and `update` work offline without falling back
/// to cloning the default `tinted-shell` item over the network.
fn local_item_config(temp: &Path) -> Result<String> {
    let item_dir = temp.join("localitem-remote");
    // Idempotent: create the throwaway remote once, then reuse it across
    // repeated calls within a single test.
    if !item_dir.join(".git").exists() {
        git_init(&item_dir)?;
        write_to_file(item_dir.join("placeholder.txt"), "placeholder\n")?;
        git(&item_dir, &["add", "-A"])?;
        git(&item_dir, &["commit", "-q", "-m", "init"])?;
    }
    Ok(format!(
        "[[items]]\npath = \"{}\"\nname = \"localitem\"\nthemes-dir = \".\"\n",
        file_url(&item_dir)
    ))
}

fn schemes_repo_path(data_path: &Path) -> PathBuf {
    data_path.join("repos").join("schemes")
}

fn is_symlink(path: &Path) -> Result<bool> {
    Ok(symlink_metadata(path)?.file_type().is_symlink())
}

// -----------------------------------------------------------------------------
// Git URL source
// -----------------------------------------------------------------------------

#[test]
fn schemes_url_source_is_cloned_not_symlinked() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_url_source_is_cloned", "install", false)?;

    let remote = temp.path().join("custom-schemes");
    init_scheme_repo(&remote, "custom-red", "Custom Red")?;

    let config = format!(
        "[schemes]\npath = \"{}\"\n\n{}",
        file_url(&remote),
        local_item_config(temp.path())?
    );
    write_to_file(&config_path, &config)?;

    let (_stdout, _stderr) = run_command(&command_vec)?;

    let schemes = schemes_repo_path(&data_path);
    ensure!(
        !is_symlink(&schemes)?,
        "A Git URL source must be a clone, not a symlink."
    );
    ensure!(
        schemes.join(".git").exists(),
        "The cloned schemes repo should contain a .git directory."
    );

    // The custom scheme is discoverable via `tinty list`.
    let list_vec = build_command_vec("list", &config_path, &data_path)?;
    let (stdout, _) = run_command(&list_vec)?;
    ensure!(
        stdout.contains("base16-custom-red"),
        "Expected the custom scheme to be listed.\nGot: {stdout}"
    );

    Ok(())
}

#[test]
fn schemes_revision_is_checked_out() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_revision_is_checked_out", "install", false)?;

    // main advances past a tagged/branched revision that has a distinct scheme.
    let remote = temp.path().join("custom-schemes");
    init_scheme_repo(&remote, "pinned-scheme", "Pinned")?;
    let pinned_sha = head(&remote)?;
    git(&remote, &["branch", "stable"])?;
    // Advance main so it differs from `stable`.
    write_scheme(&remote, "moving-scheme", "Moving")?;
    git(&remote, &["add", "-A"])?;
    git(&remote, &["commit", "-q", "-m", "advance main"])?;

    let config = format!(
        "[schemes]\npath = \"{}\"\nrevision = \"stable\"\n\n{}",
        file_url(&remote),
        local_item_config(temp.path())?
    );
    write_to_file(&config_path, &config)?;

    run_command(&command_vec)?;

    let schemes = schemes_repo_path(&data_path);
    ensure!(
        head(&schemes)? == pinned_sha,
        "The schemes repo should be checked out at the pinned revision."
    );
    ensure!(
        !schemes.join("base16").join("moving-scheme.yaml").exists(),
        "The revision must not include content added after it on main."
    );

    Ok(())
}

#[test]
fn schemes_update_pulls_from_url_source() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_update_pulls_from_url_source", "install", false)?;

    let remote = temp.path().join("custom-schemes");
    init_scheme_repo(&remote, "first-scheme", "First")?;

    let config = format!(
        "[schemes]\npath = \"{}\"\n\n{}",
        file_url(&remote),
        local_item_config(temp.path())?
    );
    write_to_file(&config_path, &config)?;

    run_command(&command_vec)?;

    // Advance the remote's main branch with a new scheme, then update.
    write_scheme(&remote, "second-scheme", "Second")?;
    git(&remote, &["add", "-A"])?;
    git(&remote, &["commit", "-q", "-m", "add second"])?;

    let update_vec = build_command_vec("update", &config_path, &data_path)?;
    let (stdout, _) = run_command(&update_vec)?;
    ensure!(
        stdout.contains("schemes up to date"),
        "Expected the schemes repo to report an update.\nGot: {stdout}"
    );
    ensure!(
        schemes_repo_path(&data_path)
            .join("base16")
            .join("second-scheme.yaml")
            .exists(),
        "The update should have pulled the newly-added scheme."
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Local path source (symlink)
// -----------------------------------------------------------------------------

#[test]
fn schemes_local_path_is_symlinked_and_revision_ignored() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_local_path_symlinked", "install", false)?;

    // A plain local directory (NOT a git repo) holding schemes.
    let local_schemes = temp.path().join("local-schemes");
    write_scheme(&local_schemes, "local-green", "Local Green")?;

    // A revision is set but must be ignored for a local-path source.
    let config = format!(
        "[schemes]\npath = \"{}\"\nrevision = \"does-not-exist\"\n\n{}",
        local_schemes.display(),
        local_item_config(temp.path())?
    );
    write_to_file(&config_path, &config)?;

    let (_stdout, stderr) = run_command(&command_vec)?;
    ensure!(
        stderr.is_empty() || !stderr.contains("does-not-exist"),
        "The revision must be ignored for a local-path source.\nstderr: {stderr}"
    );

    let schemes = schemes_repo_path(&data_path);
    ensure!(
        is_symlink(&schemes)?,
        "A local-path source must be installed as a symlink."
    );

    // The local scheme is discoverable via `tinty list` — no git required.
    let list_vec = build_command_vec("list", &config_path, &data_path)?;
    let (stdout, _) = run_command(&list_vec)?;
    ensure!(
        stdout.contains("base16-local-green"),
        "Expected the local scheme to be listed.\nGot: {stdout}"
    );

    Ok(())
}

#[test]
fn schemes_local_path_update_is_noop() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_local_path_update_noop", "install", false)?;

    let local_schemes = temp.path().join("local-schemes");
    write_scheme(&local_schemes, "local-blue", "Local Blue")?;

    let config = format!(
        "[schemes]\npath = \"{}\"\n\n{}",
        local_schemes.display(),
        local_item_config(temp.path())?
    );
    write_to_file(&config_path, &config)?;

    run_command(&command_vec)?;

    let update_vec = build_command_vec("update", &config_path, &data_path)?;
    let (stdout, _) = run_command(&update_vec)?;
    ensure!(
        stdout.contains("schemes up to date (local directory)"),
        "A local-path schemes source should report a local-directory no-op on update.\nGot: {stdout}"
    );
    ensure!(
        is_symlink(&schemes_repo_path(&data_path))?,
        "The schemes symlink should still be in place after update."
    );

    Ok(())
}

#[test]
fn schemes_source_switch_reconciles_slot_kind() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_switch_slot_kind", "install", false)?;
    let schemes = schemes_repo_path(&data_path);

    let local_schemes = temp.path().join("local-schemes");
    write_scheme(&local_schemes, "sw-local", "Switch Local")?;
    let remote = temp.path().join("custom-schemes");
    init_scheme_repo(&remote, "sw-remote", "Switch Remote")?;

    let local_cfg = format!(
        "[schemes]\npath = \"{}\"\n\n{}",
        local_schemes.display(),
        local_item_config(temp.path())?
    );
    let url_cfg = format!(
        "[schemes]\npath = \"{}\"\n\n{}",
        file_url(&remote),
        local_item_config(temp.path())?
    );

    // Local path => symlink.
    write_to_file(&config_path, &local_cfg)?;
    run_command(&command_vec)?;
    ensure!(
        is_symlink(&schemes)?,
        "expected a symlink after a local-path install"
    );

    // Switch to a Git URL => the symlink is replaced by a clone.
    write_to_file(&config_path, &url_cfg)?;
    run_command(&command_vec)?;
    ensure!(
        !is_symlink(&schemes)? && schemes.join(".git").exists(),
        "expected a clone after switching to a URL source"
    );

    // Switch back to a local path => the clone is replaced by a symlink.
    write_to_file(&config_path, &local_cfg)?;
    run_command(&command_vec)?;
    ensure!(
        is_symlink(&schemes)?,
        "expected a symlink after switching back to a local path"
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Circular reference
// -----------------------------------------------------------------------------

#[test]
fn schemes_circular_reference_is_rejected() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_circular_reference", "install", false)?;

    // Pre-create the managed schemes slot so it passes the "existing directory"
    // config validation and reaches the circular-reference check.
    let schemes = schemes_repo_path(&data_path);
    fs::create_dir_all(&schemes)?;

    let config = format!(
        "[schemes]\npath = \"{}\"\n\n{}",
        schemes.display(),
        local_item_config(temp.path())?
    );
    write_to_file(&config_path, &config)?;

    let (_stdout, stderr) = run_command(&command_vec)?;
    ensure!(
        stderr.contains("circular reference"),
        "Pointing [schemes].path at the managed schemes dir must be rejected.\nGot: {stderr}"
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Backwards compatibility
// -----------------------------------------------------------------------------

#[test]
fn default_schemes_repo_is_backwards_compatible() -> Result<()> {
    // `cache: true` seeds the real schemes repo from the shared cache.
    let (config_path, data_path, command_vec, temp) =
        setup("schemes_default_backwards_compat", "install", true)?;

    // A local item avoids a network clone of the default tinted-shell item.
    write_to_file(&config_path, &local_item_config(temp.path())?)?;

    run_command(&command_vec)?;

    let schemes = schemes_repo_path(&data_path);
    ensure!(
        !is_symlink(&schemes)?,
        "The default schemes repo must be a clone, not a symlink."
    );

    // The built-in schemes are listable.
    let list_vec = build_command_vec("list", &config_path, &data_path)?;
    let (stdout, _) = run_command(&list_vec)?;
    ensure!(
        stdout.contains("base16-"),
        "Expected built-in base16 schemes to be listed.\nGot: {}",
        stdout.lines().take(3).collect::<Vec<_>>().join(", ")
    );

    Ok(())
}
