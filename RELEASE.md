# Release process

1. Make sure `## Unreleased` section exists in [CHANGELOG.md]
1. Run [Release GitHub action]. Select `minor` or `patch` as a
   dispatch value option. This follows the [semver] pattern.

   - `minor` if changes include new features or breaking changes
   - `patch` if the change only contains bug fixes

   This action does the following:

   1. Bumps `Cargo.toml` version by `minor` or `patch`
   1. Updates `Cargo.lock` with the new version
   1. Updates [CHANGELOG.md] `## Unreleased` section to the new version
   1. Regenerates [LICENSE-THIRD-PARTY.md] using `cargo-about` for dependency
      license summaries
   1. Add a Git tag to the release commit with the new
      version number
   1. Create a release under [GitHub releases] with the changes
      mentioned in `CHANGELOG`
   1. Generate the various binaries and add it to the GitHub release

1. Run the [homebrew-tinted] [Update CLI tool GitHub
   action] and specify `tinty` as the action input value. This will
   update the version for [Homebrew]

## Debugging

Have a look at which step has failed and read the logs to understand what went
wrong.

1. If the problem is before the `Tag Release` step
   1. Push a fix for the problem
   1. Re-run the [Release GitHub action] workflow.
1. If the problem is with or after the `Tag Release` step
   1. Remove the git tag with `git push --delete origin v0.x.x` (where `0.x.x`
      mirrors the Cargo version number)
   1. Push a fix for the problem
   1. Individually run the `Tag Release` workflow
   1. Individually run the workflows following the `Tag Release` workflow. This
      is required because the `Create Release-Commit` action has already
      created the commit, since we can't remove that commit we need to manually
      trigger the subsequent workflows.

[semver]: https://semver.org/
[CHANGELOG.md]: ./CHANGELOG.md
[LICENSE-THIRD-PARTY.md]: ./LICENSE-THIRD-PARTY.md
[Release GitHub action]: https://github.com/tinted-theming/tinty/actions/workflows/release.yml
[GitHub releases]: https://github.com/tinted-theming/tinty/releases
[homebrew-tinted]: https://github.com/tinted-theming/homebrew-tinted
[Update CLI tool GitHub action]: https://github.com/tinted-theming/homebrew-tinted/actions/workflows/update-cli-tool.yml
[Homebrew]: https://brew.sh/
