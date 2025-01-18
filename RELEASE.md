# Release process

1. Make sure `## Unreleased` section exists in [CHANGELOG.md]
1. Run [Release Commit GitHub action]. Select `minor` or `patch` as a
   dispatch value option. This follows the [semver] pattern.

   - `minor` if changes include new features or breaking changes
   - `patch` if the change only contains bug fixes

   This action does the following:

   1. Bumps `Cargo.toml` version by `minor` or `patch`
   1. Updates `Cargo.lock` with the new version
   1. Updates [CHANGELOG.md] `## Unreleased` section to the new version
   1. Updates [LICENSE-THIRD-PARTY.md]

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
[CHANGELOG.md]: ./CHANGELOG.md
[LICENSE-THIRD-PARTY.md]: ./LICENSE-THIRD-PARTY.md
[Release Commit GitHub action]: https://github.com/tinted-theming/tinty/actions/workflows/release-commit.yml
[Release GitHub action]: https://github.com/tinted-theming/tinty/actions/workflows/release.yml
[GitHub releases]: https://github.com/tinted-theming/tinty/releases
[homebrew-tinted]: https://github.com/tinted-theming/homebrew-tinted
[Update CLI tool GitHub action]: https://github.com/tinted-theming/homebrew-tinted/actions/workflows/update-cli-tool.yml
[Homebrew]: https://brew.sh/
