# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.21] - 2026-07-19

### Changed
- **Container Cleanup**: Renamed misleading `Containerfile.ubi` to standard `Containerfile` (the build target remains optimized Alpine Linux).

## [1.0.20] - 2026-07-19

### Changed
- **RFC Compliance**: Updated `shared-assets` to `v3.0.32` to inherit standard RFC 7235 compliance (`WWW-Authenticate` headers for unauthorized responses).

## [1.0.19] - 2026-07-19

### Fixed
- **Tokio Thread-Blocking in Async Contexts**: Fixed two runtime performance issues that could starve the Tokio executor:
  1. Converted `UpstreamRateLimiter::acquire` into an async/non-blocking rate limiter using `tokio::time::sleep`, resolving a critical TOCTOU race condition under concurrency.
  2. Changed WHOIS DNS validation from synchronous `std::net::ToSocketAddrs` to asynchronous `tokio::net::lookup_host`.
  3. Optimized ASN lookup route to perform rate limiting concurrently inside the individual futures.

## [1.0.18] - 2026-07-19

### Changed
- **Code Review & Splitting**: Partitioned large monolithic files (including the sh admin console) into logical modules to ensure no single file exceeds 250 lines of code.
- **UI Links Cleanup**: Updated shared assets dependency to remove the version link in the header and the GitHub/Coffee links in the footer.


## [1.0.17] - 2026-07-19

### Changed
- **Aesthetic TUI Dashboard**: Redesigned the terminal user interface console with Unicode double-line frames, glowing cyan/purple theme colors, emoji icons, selection highlights, and clean command headers.


## [1.0.16] - 2026-07-19

### Security
- **CI Workflow Security**: Declared GITHUB_TOKEN read-only content permissions in the GitHub Actions workflow to resolve CodeQL scanning warnings.


## [1.0.15] - 2026-07-19

### Changed
- **Favicon update**: Synchronized the web application favicon with the new 2-color neon squircle icon.


## [1.0.14] - 2026-07-19

### Fixed
- **TUI execution fix**: Resolved argument routing issue in the main entry point of the admin tool, enabling the "tui" parameter to launch the interactive dashboard successfully.


## [1.0.13] - 2026-07-19

### Changed
- **Uniform Rounded Icon**: Applied a rounded corner mask with white-filled borders to make all application icons perfectly uniform.


## [1.0.12] - 2026-07-19

### Changed
- **Simple Bright Icon**: Replaced application icon with a simple high-contrast 2-color flat-art neon cyan and purple vector illustration on a dark navy blue background.


## [1.0.11] - 2026-07-19

### Fixed
- **Warning fix**: Removed unused mutable keyword on server command spawn to prevent warning compilation failures in CI runners.


## [1.0.10] - 2026-07-19

### Changed
- **Release build bump**: Preparing new version release to trigger automated package container compilation on GHCR.


## [1.0.9] - 2026-07-19

### Changed
- **Slim Branding Banner**: Replaced the repository header banner with a slim, flat-art twilight landscape of Cheney, WA (home of the server) featuring rolling hills, Ponderosa pines, and a soaring neon eagle.


## [1.0.8] - 2026-07-19

### Changed
- **Containerized Admin Console integration**: Named the admin tool after the application (`trace`) and copied it to the container's system path `/usr/local/bin/trace`. The TUI can now be launched by simply executing `trace tui` (or `trace`) inside the container shell.
- **Documentation Modernization**: Rewrote `README.md` to remove CI details, format CLI commands as tables, and purge local development guides.


## [1.0.7] - 2026-07-19

### Changed
- **Containerized Admin Console integration**: Named the admin tool after the application (`trace`) and copied it to the container's system path `/usr/local/bin/trace`. The TUI can now be launched by simply executing `trace tui` (or `trace`) inside the container shell.
- **Documentation Modernization**: Rewrote `README.md` to remove CI details, format CLI commands as tables, and purge local development guides.


## [1.0.6] - 2026-07-19

### Changed
- Update README, clean file tree, and remove contributing/license files.


