//! Integration tests for extra scheme repositories (`[[schemes.extras]]`) and
//! the relocation of the built-in schemes repo to `scheme-repos/`.
//!
//! These tests are fully offline and deterministic: the built-in schemes repo
//! and every extra are plain local directories (symlinked into place), and a
//! local-directory `[[items]]` template stands in for a network template repo.

mod utils;

use std::fs;
use std::path::Path;

use anyhow::{ensure, Result};
use utils::{build_command_vec, builtin_schemes_repo_path, run_command, setup, write_to_file};

/// A minimal, valid base16 scheme YAML for the given slug and name.
fn scheme_yaml(slug: &str, name: &str) -> String {
    format!(
        "system: base16\nname: {name}\nslug: {slug}\nauthor: Tinty Test\nvariant: dark\npalette:\n  base00: '#282628'\n  base01: '#403e3f'\n  base02: '#595757'\n  base03: '#71706e'\n  base04: '#8a8986'\n  base05: '#a2a29d'\n  base06: '#bbbbb5'\n  base07: '#d4d4cd'\n  base08: '#bf2546'\n  base09: '#f69622'\n  base0A: '#f99923'\n  base0B: '#19953f'\n  base0C: '#40dab9'\n  base0D: '#0666dc'\n  base0E: '#8554ac'\n  base0F: '#ac7424'\n"
    )
}

/// Writes `base16/<slug>.yaml` under `dir` (creating the system subdir).
fn write_scheme(dir: &Path, slug: &str, name: &str) -> Result<()> {
    write_to_file(
        dir.join("base16").join(format!("{slug}.yaml")),
        &scheme_yaml(slug, name),
    )
}

/// A `[schemes]` block pointing the built-in repo at a local directory, so
/// `install` symlinks it instead of cloning the real repo over the network.
fn local_builtin_block(dir: &Path) -> String {
    format!("[schemes]\npath = \"{}\"\n", dir.display())
}

/// A `[[schemes.extras]]` block backed by a local directory (symlinked).
fn extra_block(name: &str, dir: &Path) -> String {
    format!(
        "[[schemes.extras]]\nname = \"{name}\"\npath = \"{}\"\n",
        dir.display()
    )
}

// -----------------------------------------------------------------------------
// Collection & merging
// -----------------------------------------------------------------------------

#[test]
fn extra_repo_schemes_are_listed_with_builtin() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("extras_listed_with_builtin", "install", false)?;

    let builtin = temp.path().join("builtin-schemes");
    write_scheme(&builtin, "builtin-one", "Builtin One")?;
    let extra = temp.path().join("extra-schemes");
    write_scheme(&extra, "extra-one", "Extra One")?;

    let config = format!(
        "{}\n{}",
        local_builtin_block(&builtin),
        extra_block("community", &extra)
    );
    write_to_file(&config_path, &config)?;
    run_command(&command_vec)?;

    let list_vec = build_command_vec("list", &config_path, &data_path)?;
    let (stdout, _) = run_command(&list_vec)?;
    ensure!(
        stdout.contains("base16-builtin-one"),
        "expected the built-in scheme to be listed.\nGot: {stdout}"
    );
    ensure!(
        stdout.contains("base16-extra-one"),
        "expected the extra scheme to be listed.\nGot: {stdout}"
    );

    Ok(())
}

#[test]
fn duplicate_scheme_extra_wins_over_builtin() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("extras_duplicate_extra_wins", "install", false)?;

    // Both the built-in repo and the extra define `base16-dup`, with distinct
    // names so we can tell which one survived the merge. The extra is listed
    // after the built-in, so it overrides it (last-listed wins).
    let builtin = temp.path().join("builtin-schemes");
    write_scheme(&builtin, "dup", "BuiltinDup")?;
    let extra = temp.path().join("extra-schemes");
    write_scheme(&extra, "dup", "ExtraDup")?;

    let config = format!(
        "{}\n{}",
        local_builtin_block(&builtin),
        extra_block("community", &extra)
    );
    write_to_file(&config_path, &config)?;
    run_command(&command_vec)?;

    // `list --json` embeds each scheme's name; the extra copy must win.
    let list_vec = build_command_vec("list --json", &config_path, &data_path)?;
    let (stdout, stderr) = run_command(&list_vec)?;
    ensure!(
        stdout.contains("ExtraDup") && !stdout.contains("BuiltinDup"),
        "the extra scheme must override the built-in duplicate.\nstdout: {stdout}"
    );
    // The shadowing is surfaced (not silent) on stderr.
    ensure!(
        stderr.contains("base16-dup") && stderr.contains("community"),
        "expected a duplicate-scheme note naming the override.\nstderr: {stderr}"
    );

    Ok(())
}

#[test]
fn duplicate_across_two_extras_last_listed_wins() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("extras_duplicate_last_wins", "install", false)?;

    let builtin = temp.path().join("builtin-schemes");
    write_scheme(&builtin, "builtin-only", "Builtin Only")?;
    let first = temp.path().join("first-extra");
    write_scheme(&first, "shared", "FirstShared")?;
    let second = temp.path().join("second-extra");
    write_scheme(&second, "shared", "SecondShared")?;

    let config = format!(
        "{}\n{}\n{}",
        local_builtin_block(&builtin),
        extra_block("first", &first),
        extra_block("second", &second)
    );
    write_to_file(&config_path, &config)?;
    run_command(&command_vec)?;

    let list_vec = build_command_vec("list --json", &config_path, &data_path)?;
    let (stdout, _) = run_command(&list_vec)?;
    ensure!(
        stdout.contains("SecondShared") && !stdout.contains("FirstShared"),
        "the last-listed extra must win the duplicate.\nstdout: {stdout}"
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Relocation / migration
// -----------------------------------------------------------------------------

#[test]
fn legacy_schemes_repo_is_migrated_on_next_command() -> Result<()> {
    // No install: seed the *old* location by hand, then run any command and
    // confirm it was relocated and announced.
    let (config_path, data_path, _command_vec, _temp) = setup("extras_migration", "list", false)?;

    let legacy = data_path.join("repos").join("schemes");
    write_scheme(&legacy, "migrated", "Migrated")?;
    write_to_file(&config_path, "")?;

    let list_vec = build_command_vec("list", &config_path, &data_path)?;
    let (stdout, stderr) = run_command(&list_vec)?;

    let new_location = builtin_schemes_repo_path(&data_path);
    ensure!(
        new_location.join("base16").join("migrated.yaml").exists(),
        "the schemes repo should have moved to {}",
        new_location.display()
    );
    ensure!(
        !legacy.exists(),
        "the legacy schemes repo location should be gone after migration"
    );
    ensure!(
        stdout.contains("base16-migrated"),
        "the relocated scheme should still be listable.\nstdout: {stdout}"
    );
    ensure!(
        stderr.contains("scheme-repos"),
        "the relocation should be announced on stderr.\nstderr: {stderr}"
    );

    Ok(())
}

// -----------------------------------------------------------------------------
// Applying a scheme sourced from an extra repo
// -----------------------------------------------------------------------------

#[test]
fn apply_scheme_from_extra_repo_builds_on_the_fly() -> Result<()> {
    let (config_path, data_path, command_vec, temp) =
        setup("extras_apply_builds", "install", false)?;

    // Built-in repo (local) plus an extra repo (local) that defines the scheme
    // we will apply. The extra scheme has no pre-built theme anywhere, so apply
    // must build it from YAML into the template item.
    let builtin = temp.path().join("builtin-schemes");
    write_scheme(&builtin, "builtin-one", "Builtin One")?;
    let extra = temp.path().join("extra-schemes");
    write_scheme(&extra, "extra-apply", "Extra Apply")?;

    // A minimal local template item that writes one file per scheme.
    let template = temp.path().join("template-item");
    write_to_file(
        template.join("templates").join("config.yaml"),
        "base16-out:\n  filename: output/base16-{{ scheme-slug }}.txt\n  supported-systems: [base16]\n",
    )?;
    write_to_file(
        template.join("templates").join("base16-out.mustache"),
        "{{scheme-name}} {{scheme-slug}}\n",
    )?;

    let config = format!(
        "{}\n{}\n[[items]]\nname = \"local-template\"\npath = \"{}\"\nthemes-dir = \"output\"\nsupported-systems = [\"base16\"]\n",
        local_builtin_block(&builtin),
        extra_block("community", &extra),
        template.display(),
    );
    write_to_file(&config_path, &config)?;
    run_command(&command_vec)?;

    // Apply the extra-repo scheme.
    let apply_vec = build_command_vec("apply base16-extra-apply", &config_path, &data_path)?;
    let (_stdout, stderr) = run_command(&apply_vec)?;

    let current =
        fs::read_to_string(data_path.join("artifacts").join("current_scheme")).unwrap_or_default();
    ensure!(
        current.trim() == "base16-extra-apply",
        "expected the applied scheme to be recorded as current.\ncurrent: {current}\nstderr: {stderr}"
    );

    // The build-on-the-fly should have produced the theme file for the extra
    // scheme in the template item's output dir.
    ensure!(
        template
            .join("output")
            .join("base16-extra-apply.txt")
            .exists(),
        "expected the extra scheme to be built into the template item.\nstderr: {stderr}"
    );

    Ok(())
}
