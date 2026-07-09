//! Integration tests for `allow-dirty-update`.
//!
//! These exercise `tinty update` against an item whose local copy has
//! uncommitted changes. Unlike the other update tests, they use a throwaway
//! **local** git repository as the remote, so they are fully deterministic and
//! require no network access: we control exactly which files the "upstream"
//! revision changes and can assert that the user's work is preserved.

mod utils;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use tempfile::TempDir;
use utils::{run_command, run_command_with_env, setup, write_to_file};

const ITEM_NAME: &str = "localtheme";

/// Runs `tinty update` in the ambient locale. Conflict detection must not
/// depend on the locale, so the tests deliberately do not force one (see also
/// `conflict_is_detected_regardless_of_git_locale`).
fn run_update(command_vec: &[String]) -> Result<(String, String)> {
    run_command(command_vec)
}

/// A throwaway remote repo plus an installed clone of it under the tinty data
/// directory, ready for a `tinty update` invocation.
struct Fixture {
    remote: PathBuf,
    clone: PathBuf,
    config_path: PathBuf,
    command_vec: Vec<String>,
    _temp: TempDir,
}

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

/// Creates a remote repo (with `theme-a.txt`/`theme-b.txt` on `main`) and
/// clones it into the tinty data dir to simulate an installed item.
fn fixture(name: &str) -> Result<Fixture> {
    let (config_path, data_path, command_vec, temp) = setup(name, "update", false)?;

    let remote = temp.path().join("remote");
    fs::create_dir_all(&remote)?;
    git(&remote, &["init", "-q", "-b", "main"])?;
    git(&remote, &["config", "user.email", "tinty@test.local"])?;
    git(&remote, &["config", "user.name", "tinty test"])?;
    fs::write(remote.join("theme-a.txt"), "a1\n")?;
    fs::write(remote.join("theme-b.txt"), "b1\n")?;
    git(&remote, &["add", "-A"])?;
    git(&remote, &["commit", "-q", "-m", "init"])?;

    let clone = data_path.join("repos").join(ITEM_NAME);
    fs::create_dir_all(clone.parent().unwrap())?;
    let status = Command::new("git")
        .args([
            "clone",
            "-q",
            remote.to_str().unwrap(),
            clone.to_str().unwrap(),
        ])
        .status()?;
    ensure!(status.success(), "failed to clone local remote");

    Ok(Fixture {
        remote,
        clone,
        config_path,
        command_vec,
        _temp: temp,
    })
}

/// `[[items]]` config for the fixture, optionally setting the per-item flag.
fn item_config(remote: &Path, item_allow: Option<bool>) -> String {
    let mut config = format!(
        "[[items]]\npath = \"{}\"\nname = \"{ITEM_NAME}\"\nthemes-dir = \".\"\n",
        remote.display()
    );
    if let Some(value) = item_allow {
        config.push_str(&format!("allow-dirty-update = {value}\n"));
    }
    config
}

/// Adds a commit on the remote's `main` that changes `file` to `contents`.
fn remote_commit(remote: &Path, file: &str, contents: &str, message: &str) -> Result<()> {
    fs::write(remote.join(file), contents)?;
    git(remote, &["add", "-A"])?;
    git(remote, &["commit", "-q", "-m", message])?;
    Ok(())
}

#[test]
fn update_carries_non_overlapping_local_changes_forward() -> Result<()> {
    let f = fixture("allow_dirty_carry_forward")?;

    // Local uncommitted edit to theme-a; upstream only advances theme-b.
    fs::write(f.clone.join("theme-a.txt"), "my local edit\n")?;
    remote_commit(&f.remote, "theme-b.txt", "b2\n", "advance b")?;

    write_to_file(&f.config_path, &item_config(&f.remote, Some(true)))?;
    let (stdout, _) = run_update(&f.command_vec)?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME} up to date (local changes preserved)")),
        "Expected a preserved-changes success message.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-a.txt"))? == "my local edit\n",
        "Local uncommitted edit should have been carried forward untouched."
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-b.txt"))? == "b2\n",
        "Upstream change to theme-b should have been applied."
    );

    Ok(())
}

#[test]
fn update_refuses_and_preserves_work_on_overlapping_conflict() -> Result<()> {
    let f = fixture("allow_dirty_overlap_conflict")?;

    // Both the user and upstream change the same file.
    fs::write(f.clone.join("theme-a.txt"), "my local edit\n")?;
    remote_commit(&f.remote, "theme-a.txt", "upstream edit\n", "advance a")?;
    let before = head(&f.clone)?;

    write_to_file(&f.config_path, &item_config(&f.remote, Some(true)))?;
    let (stdout, _) = run_update(&f.command_vec)?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME}: could not update")),
        "Expected a friendly could-not-update message.\nGot: {stdout}"
    );
    ensure!(
        stdout.contains("theme-a.txt"),
        "Expected the conflicting file to be named.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-a.txt"))? == "my local edit\n",
        "The user's uncommitted work must be left untouched."
    );
    ensure!(
        head(&f.clone)? == before,
        "HEAD must not move when the update is refused."
    );

    Ok(())
}

#[test]
fn update_refuses_when_incoming_file_collides_with_untracked() -> Result<()> {
    let f = fixture("allow_dirty_untracked_collision")?;

    // Upstream adds a new file that already exists locally as untracked.
    remote_commit(&f.remote, "theme-c.txt", "upstream c\n", "add c")?;
    fs::write(f.clone.join("theme-c.txt"), "my untracked c\n")?;

    write_to_file(&f.config_path, &item_config(&f.remote, Some(true)))?;
    let (stdout, _) = run_update(&f.command_vec)?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME}: could not update")),
        "Expected a friendly could-not-update message.\nGot: {stdout}"
    );
    ensure!(
        stdout.contains("theme-c.txt"),
        "Expected the colliding untracked file to be named.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-c.txt"))? == "my untracked c\n",
        "The untracked local file must be left untouched."
    );

    Ok(())
}

#[test]
fn update_refuses_and_preserves_staged_work_on_conflict() -> Result<()> {
    let f = fixture("allow_dirty_staged_conflict")?;

    // Stage an edit to theme-a; upstream changes the same file.
    fs::write(f.clone.join("theme-a.txt"), "my staged edit\n")?;
    git(&f.clone, &["add", "theme-a.txt"])?;
    remote_commit(&f.remote, "theme-a.txt", "upstream edit\n", "advance a")?;
    let before = head(&f.clone)?;

    write_to_file(&f.config_path, &item_config(&f.remote, Some(true)))?;
    let (stdout, _) = run_update(&f.command_vec)?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME}: could not update")),
        "A staged conflict must be reported as a preserved conflict, not a hard error.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-a.txt"))? == "my staged edit\n",
        "The staged content must be left untouched."
    );
    // The change must remain staged, not silently unstaged.
    ensure!(
        git(&f.clone, &["diff", "--cached", "--name-only"])?.contains("theme-a.txt"),
        "The change must still be staged after a refused update."
    );
    ensure!(
        head(&f.clone)? == before,
        "HEAD must not move when the update is refused."
    );

    Ok(())
}

#[test]
fn update_carries_staged_changes_forward_without_conflict() -> Result<()> {
    let f = fixture("allow_dirty_staged_carry_forward")?;

    // Stage an edit to theme-a; upstream only advances theme-b.
    fs::write(f.clone.join("theme-a.txt"), "my staged edit\n")?;
    git(&f.clone, &["add", "theme-a.txt"])?;
    remote_commit(&f.remote, "theme-b.txt", "b2\n", "advance b")?;

    write_to_file(&f.config_path, &item_config(&f.remote, Some(true)))?;
    let (stdout, _) = run_update(&f.command_vec)?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME} up to date (local changes preserved)")),
        "Expected a preserved-changes success message.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-a.txt"))? == "my staged edit\n",
        "The staged edit must be carried forward untouched."
    );
    ensure!(
        git(&f.clone, &["diff", "--cached", "--name-only"])?.contains("theme-a.txt"),
        "The change must remain staged after a carry-forward update."
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-b.txt"))? == "b2\n",
        "Upstream change to theme-b should have been applied."
    );

    Ok(())
}

#[test]
fn update_is_blocked_when_flag_is_absent() -> Result<()> {
    let f = fixture("allow_dirty_blocked_default")?;

    fs::write(f.clone.join("theme-a.txt"), "my local edit\n")?;
    remote_commit(&f.remote, "theme-b.txt", "b2\n", "advance b")?;

    // No flag anywhere: defaults to the strict skip-if-dirty behavior.
    write_to_file(&f.config_path, &item_config(&f.remote, None))?;
    let (stdout, _) = run_update(&f.command_vec)?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME} contains uncommitted changes")),
        "Expected the strict skip message.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-b.txt"))? == "b1\n",
        "Upstream change must NOT be applied when the update is skipped."
    );

    Ok(())
}

#[test]
fn schemes_setting_does_not_apply_to_items() -> Result<()> {
    let f = fixture("allow_dirty_schemes_scope")?;

    fs::write(f.clone.join("theme-a.txt"), "my local edit\n")?;
    remote_commit(&f.remote, "theme-b.txt", "b2\n", "advance b")?;

    // `[schemes]` leniency is on, but the item does not opt in. The `[schemes]`
    // setting governs only the built-in schemes repo, never `[[items]]`.
    let config = format!(
        "[schemes]\nallow-dirty-update = true\n\n{}",
        item_config(&f.remote, None)
    );
    write_to_file(&f.config_path, &config)?;
    let (stdout, _) = run_update(&f.command_vec)?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME} contains uncommitted changes")),
        "Item must not inherit the [schemes] setting.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-b.txt"))? == "b1\n",
        "Item update must be skipped when the item itself does not opt in."
    );

    Ok(())
}

#[test]
fn conflict_is_detected_regardless_of_git_locale() -> Result<()> {
    let f = fixture("allow_dirty_locale_independent")?;

    fs::write(f.clone.join("theme-a.txt"), "my local edit\n")?;
    remote_commit(&f.remote, "theme-a.txt", "upstream edit\n", "advance a")?;
    let before = head(&f.clone)?;

    write_to_file(&f.config_path, &item_config(&f.remote, Some(true)))?;

    // Force a non-English locale. Even if git translates its overwrite message
    // (or the locale is not installed and git falls back to English), the
    // conflict must still be detected — detection relies on a plumbing exit
    // code, not on the message text.
    let (stdout, _) = run_command_with_env(
        &f.command_vec,
        &[("LC_ALL", "de_DE.UTF-8"), ("LANGUAGE", "de")],
    )?;

    ensure!(
        stdout.contains(&format!("{ITEM_NAME}: could not update")),
        "Conflict should be detected under a non-English locale.\nGot: {stdout}"
    );
    ensure!(
        fs::read_to_string(f.clone.join("theme-a.txt"))? == "my local edit\n",
        "The user's uncommitted work must be left untouched."
    );
    ensure!(
        head(&f.clone)? == before,
        "HEAD must not move when the update is refused."
    );

    Ok(())
}
