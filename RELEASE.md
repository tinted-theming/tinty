# Release process

1. Create a release commit. This includes specific updated files:

   1. Cargo.toml (Bump the version using [semver] standards)
   1. Cargo.lock (This is usually updated automatically with the `cargo`
      CLI tool)
   1. Make sure the changes for the new release exist under
      `CHANGELOG.md`. This should already exist since changes done in
      GitHub Pull Requests should include updates to the `CHANGELOG.md`.
      If this doesn't exist, the release will fail.
   1. Change `CHANGELOG.md` `## Unreleased` to `## [x.x.x] - YYYY-MM-DD`
      where `x.x.x` is the new Tinty version specified in the
      `Cargo.toml` file and the link to `[x.x.x]` at the bottom of the
      `CHANGELOG.md` file and compare it with the previously released
      version, eg:

      ```md
      [0.22.0]: https://github.com/tinted-theming/tinty/compare/v0.21.1...v0.22.0
      ```

   1. Run `cargo about generate about.hbs > LICENSES-THIRD-PARTY.md` to
      update the third party licenses. (`cargo install cargo-about` if
      you don't have it installed)
   1. Create a commit with the 3 changed files titled `Release x.x.x`
      where `x.x.x` is the new Tinty version specified in the
      `Cargo.toml` file

1. Push the commit or create a Pull Request and merge
1. Once the CI tests have passed, run the [Release GitHub action]. This
   will automatically do the following:

   1. Add a Git tag to the release commit with the new
      version number
   1. Create a release under [GitHub releases] with the changes
      mentioned in `CHANGELOG`
   1. Generate the various binaries and add it to the GitHub release
1. Run the [homebrew-tinted] [Update CLI tool GitHub
   action] and specify `tinty` as the action input value. This will
   update the version for [Homebrew]

[semver]: https://semver.org/
[Release GitHub action]: https://github.com/tinted-theming/tinty/actions/workflows/release.yml
[GitHub releases]: https://github.com/tinted-theming/tinty/releases
[homebrew-tinted]: https://github.com/tinted-theming/homebrew-tinted
[Update CLI tool GitHub action]: https://github.com/tinted-theming/homebrew-tinted/actions/workflows/update-cli-tool.yml
[Homebrew]: https://brew.sh/
