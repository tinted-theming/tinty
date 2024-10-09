# Contributing

## Setup

Ensure you have make installed. Run `command -v make`. If there is no
stdout, search for ways to install `make` on your system.

```sh
make install # installs rust/cargo and cargo crate deps
```

## Building

To generate a release binary to `./target/release/tinty`, run:

```sh
make build
```

## Testing

```sh
make test
```

## PRs

Include Rust related changes under `## Unreleased` (Copy the format from
the existing releases).

Changes should be included under `### Added`, `### Changed` and `###
Removed`. Bullet point information should be written in present tense
and should describe what the change does.
