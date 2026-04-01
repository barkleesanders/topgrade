<div align="center">
  <h1>
    <img alt="Topgrade" src="https://github.com/barkleesanders/topgrade/blob/main/doc/topgrade_transparent.png?raw=true" width="850px">
  </h1>

<a href="https://github.com/barkleesanders/topgrade/actions"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/barkleesanders/topgrade/ci.yml?branch=main&label=CI"></a>
<a href="https://github.com/barkleesanders/topgrade/actions"><img alt="i18n" src="https://img.shields.io/github/actions/workflow/status/barkleesanders/topgrade/check_i18n.yml?branch=main&label=i18n"></a>
<a href="https://github.com/barkleesanders/topgrade/actions"><img alt="Security" src="https://img.shields.io/github/actions/workflow/status/barkleesanders/topgrade/check_security_vulnerability.yml?branch=main&label=Security"></a>
<a href="https://crates.io/crates/topgrade"><img alt="crates.io" src="https://img.shields.io/crates/v/topgrade.svg"></a>
<a href="https://aur.archlinux.org/packages/topgrade"><img alt="AUR" src="https://img.shields.io/aur/version/topgrade.svg"></a>
<a href="https://formulae.brew.sh/formula/topgrade"><img alt="Homebrew" src="https://img.shields.io/homebrew/v/topgrade.svg"></a>
<a href="https://securityscorecards.dev/viewer/?uri=github.com/barkleesanders/topgrade"><img alt="OpenSSF Scorecard" src="https://api.securityscorecards.dev/projects/github.com/barkleesanders/topgrade/badge"></a>

  <img alt="Feature Showcase" src="https://github.com/barkleesanders/topgrade/blob/main/doc/topgrade-showcase.gif?raw=true" width="900px">
</div>

## About This Fork

This is an **enhanced fork** of [topgrade-rs/topgrade](https://github.com/topgrade-rs/topgrade) that integrates all pending community pull requests and addresses open issues from the upstream repository.

**What was done:**
- Merged and implemented **22 open pull requests** from the upstream repo
- Addressed **54 actionable issues** with code fixes
- Added **40+ new features** including new package manager steps, configuration options, and platform improvements
- Full i18n coverage with translations in 7 languages (en, lt, es, fr, zh_CN, zh_TW, de)
- All CI checks passing across all platforms (Linux, macOS, Windows, FreeBSD, NetBSD, Android)

## Latest Updates

### 2026-03-08
- **npm audit fix**: Built-in `npm audit fix` step that auto-fixes known vulnerabilities after global package updates. Enable with `audit_fix = true` in the `[npm]` config section. Uses safe fixes only (never `--force`). Full i18n support across all 7 languages.
- **gh-tool-updater**: Registry-driven GitHub release updater — replaces per-tool topgrade custom commands with a single script that auto-discovers, registers, and updates tools from GitHub releases. Supports binary, tarball, and DMG install types. ([`1216bdb`](https://github.com/barkleesanders/topgrade/commit/1216bdb))
- **yt-dlp rate limit bypass**: Use `gh` CLI for authenticated GitHub API calls during yt-dlp version checks, avoiding 60 req/hr unauthenticated rate limit ([`5f1efa3`](https://github.com/barkleesanders/topgrade/commit/5f1efa3), [`08a3d80`](https://github.com/barkleesanders/topgrade/commit/08a3d80))
- **Coverage audit**: Added `topgrade-audit.sh` post-command that reports which tools topgrade covers vs uncovered ([`73afda3`](https://github.com/barkleesanders/topgrade/commit/73afda3))
- **Bug fixes**: Handle Ollama server not running, yt-dlp permission errors, clippy `disallowed_methods` warning ([`8015db3`](https://github.com/barkleesanders/topgrade/commit/8015db3), [`e95c7bb`](https://github.com/barkleesanders/topgrade/commit/e95c7bb))

### 2026-03-07
- **4K logo & showcase GIF**: Upgraded repo branding with cinematic logo render and inline feature showcase animation ([`c829b7f`](https://github.com/barkleesanders/topgrade/commit/c829b7f), [`74ee02e`](https://github.com/barkleesanders/topgrade/commit/74ee02e))
- **`--log-file` option**: Persistent log output to file for debugging long runs ([`6bb1293`](https://github.com/barkleesanders/topgrade/commit/6bb1293))
- **Separator color & config aliases**: Customize separator colors, added config key aliases for discoverability ([`e80d5a9`](https://github.com/barkleesanders/topgrade/commit/e80d5a9))
- **CLI/UX improvements**: Better step output formatting and config options ([`8eb3375`](https://github.com/barkleesanders/topgrade/commit/8eb3375))
- **New package manager steps**: Batch addition of new steps (Ollama, ldcup, Soar, Colima, etc.) ([`31aaca3`](https://github.com/barkleesanders/topgrade/commit/31aaca3))
- **Deprecated config key safety**: Restored deprecated keys as ignored fields to prevent crashes for existing users ([`074aa77`](https://github.com/barkleesanders/topgrade/commit/074aa77))
- **Full i18n coverage**: Complete translations for all new strings across 7 languages ([`1bb4b9f`](https://github.com/barkleesanders/topgrade/commit/1bb4b9f), [`166ce51`](https://github.com/barkleesanders/topgrade/commit/166ce51))
- **Cross-platform CI fixes**: Fixed MSRV Windows imports, clippy collapsible_if, and compilation errors across Linux/FreeBSD/NetBSD/Android ([`bf18cd6`](https://github.com/barkleesanders/topgrade/commit/bf18cd6), [`1e1ce71`](https://github.com/barkleesanders/topgrade/commit/1e1ce71), [`6ccb646`](https://github.com/barkleesanders/topgrade/commit/6ccb646))
- **Quick-win bug fixes**: Multiple issue fixes in a single batch ([`8c87634`](https://github.com/barkleesanders/topgrade/commit/8c87634))

---

<details>
<summary><strong>Fork Highlights</strong> (full feature list from initial fork)</summary>

### New Package Manager Steps
- **Ollama** - Update Ollama AI model server
- **ldcup** - LDC2 D compiler version manager
- **Soar** - Soar package manager
- **Colima** - Colima container runtime updates
- **install-release** - Cargo install-update for release binaries
- **Adless** - Adless domain blocking updates
- **yt-dlp** - Self-update for yt-dlp video downloader
- **uv python** - Update Python installations via uv
- **KDE Plasmoids** - Update KDE Plasmoid widgets via libplasmoid-updater
- **Hardware IDs** - Update hardware ID databases (update-pciids, update-usbids)
- **Microsoft Store** - Update apps via Windows Store CLI

### New Features
- **Zellij multiplexer** - Run topgrade inside Zellij sessions (alternative to tmux)
- **Per-step update frequency** - Configure how often individual steps run (daily, weekly, monthly)
- **Config file respawn** - Automatically re-run topgrade when config changes during execution
- **Updated components summary** - See which packages were actually updated at the end of a run
- **Post-update triggers** - Run custom commands after specific steps complete
- **Custom step ordering** - Control the order steps execute via `[step_order]` config
- **Fuzzy step matching** - "Did you mean...?" suggestions for misspelled step names
- **Assume yes per step** - `assume_yes` now accepts an array of step names for selective auto-confirm
- **Self-update version display** - Shows version transition (e.g., `16.0.0 -> 17.0.0`) in summary
- **GitHub token support** - Use `TOPGRADE_GITHUB_TOKEN` for authenticated self-update API calls
- **Brew path configuration** - `brew_path` config for Workbrew and custom Homebrew installations
- **tmux auto-exit** - Automatically close tmux session when topgrade finishes
- **Container restart** - Restart running containers after pulling new images

### Platform Improvements
- **macOS**: Resolve gems from Homebrew keg-only Ruby instead of system Ruby
- **macOS**: Use `sudo softwareupdate --install --all --restart` for system updates
- **Linux**: Detect openSUSE toolbox vs Fedora toolbx before executing
- **Linux**: Skip needrestart on Arch if pacman installed it
- **Linux**: Prompt for reboot after zypper updates when needed
- **Linux**: Suppress firmware reboot prompt when `assume_yes` is set
- **Linux**: Fixed oh-my-bash updates from non-bash shells (OSH env var)
- **Linux**: Pipe nix commands through `nom` for better output
- **Windows**: SDIO (Snappy Driver Installer Origin) driver updates with opt-in safety
- **Windows**: Support `npm.use_sudo` via gsudo
- **FreeBSD**: Handle remote shells that don't support `-l` with `-c` flag

### Bug Fixes & Improvements
- **Second Ctrl+C exits immediately** (process::exit(130)) instead of waiting
- **Skip freshclam** if ClamAV auto-updater lock file exists
- **Forward --verbose** to remote topgrade invocations via SSH
- **Propagate quit** from remote topgrade instances
- **Fix pnpm** running in wrong directory
- **Clean up old PowerShell module versions** after updates
- **raco system-scope** package updates alongside user-scope
- **Expand environment variables** in config paths and git repo paths
- **Sudo loop** keeps credentials refreshed during long runs
- **Removed deprecated flags**: `--no-retry`, `no_self_update`, `skip_notify`
- **Custom commands** now pass `TOPGRADE_*` environment variables

### CI & Security
- **OSV Scanner** for vulnerability detection
- **Trivy FS Scan** for filesystem security scanning
- **SBOM generation** via Syft
- **CodeQL** analysis
- **cargo-deny** license and advisory checks
- **Zizmor** GitHub Actions security analysis
- **Scorecard** supply-chain security

</details>

## Installation

### From This Fork

```bash
# Build from source
git clone https://github.com/barkleesanders/topgrade.git
cd topgrade
cargo install --path .
```

### From Upstream (Official Channels)

- Self-updating binary (all platforms): [releases](https://github.com/topgrade-rs/topgrade/releases)
- Install from source: [`cargo install topgrade`](https://crates.io/crates/topgrade)
- Debian/Ubuntu ([deb-get](https://github.com/wimpysworld/deb-get)):
  [`deb-get install topgrade`](https://github.com/wimpysworld/deb-get/blob/main/01-main/packages/topgrade)
- Arch Linux (AUR): [topgrade](https://aur.archlinux.org/packages/topgrade)
  or [topgrade-bin](https://aur.archlinux.org/packages/topgrade-bin)
- [PyPi](https://pypi.org/): `pip`, `pipx`, or `uv tool` [
  `install topgrade`](https://pypi.org/project/topgrade/)
- Windows ([Winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/)): [
  `winget install --id=topgrade-rs.topgrade  -e`](https://winstall.app/apps/topgrade-rs.topgrade)
- macOS or Linux ([Homebrew](https://brew.sh/)): [`brew install topgrade`](https://formulae.brew.sh/formula/topgrade)
- Fedora/RHEL/AlmaLinux/CentOS-Stream ([Copr](https://copr.fedorainfracloud.org/)): [
  `sudo dnf copr enable lilay/topgrade && sudo dnf install topgrade`](https://copr.fedorainfracloud.org/coprs/lilay/topgrade/)

### Community-maintained

- Windows ([Chocolatey](https://chocolatey.org/)): [
  `choco install topgrade`](https://community.chocolatey.org/packages/topgrade)
- Windows ([Scoop](https://scoop.sh/)): [
  `scoop bucket add main && scoop install main/topgrade`](https://scoop.sh/#/apps?q=topgrade)
- macOS ([MacPorts](https://www.macports.org/)): [
  `sudo port install topgrade`](https://ports.macports.org/port/topgrade/)
- Ubuntu ([Pacstall](https://pacstall.dev/)):
  [`pacstall -I topgrade-bin`](https://github.com/pacstall/pacstall-programs/blob/master/packages/topgrade-bin/topgrade-bin.pacscript)
- NixOS or Nix (nixpkgs): [topgrade](https://search.nixos.org/packages?show=topgrade)
- Alpine Linux: [`sudo apk add topgrade`](https://pkgs.alpinelinux.org/package/edge/community/x86_64/topgrade)
- Void Linux: [`sudo xbps-install -S topgrade`](https://voidlinux.org/packages/?arch=x86_64&q=topgrade)

## Usage

Just run `topgrade`.

### Windows Features

On Windows, Topgrade supports updating:

- **Package Managers**: Chocolatey, Scoop, Winget
- **System Updates**: Windows Update, Microsoft Store apps
- **Driver Updates**: SDIO (Snappy Driver Installer Origin) - *requires explicit opt-in*
- **Development Tools**: Various language package managers and development environments

*Note: Driver updates via SDIO require setting `enable_sdio = true` in your configuration file due to the critical nature of driver installations.*

## Configuration

See [`config.example.toml`](config.example.toml) for an example configuration file.

### Configuration Path

#### `CONFIG_DIR` on each platform

- **Windows**: `%APPDATA%`
- **macOS** and **other Unix systems**: `${XDG_CONFIG_HOME:-~/.config}`

`topgrade` will look for the configuration file in the following places, in order of priority:

1. `CONFIG_DIR/topgrade.toml`
2. `CONFIG_DIR/topgrade/topgrade.toml`

If the file with higher priority is present, no matter if it is valid or not, the other configuration files will be
ignored.

On the first run (provided that no configuration file exists), `topgrade` will create a configuration file at
`CONFIG_DIR/topgrade.toml` for you.

### Custom Commands

Custom commands can be defined in the configuration file, which can be run before, during, or after the inbuilt commands, as
required.
By default, the custom commands are run using a new shell according to the `$SHELL` environment variable on Unix (falls
back to `sh`) or `pwsh` on Windows (falls back to `powershell`).

On Unix, if you want to run your command using an interactive shell, for example to source your shell's RC files, you
can add `-i` at the start of your custom command.
Although note that this requires the command to exit the shell correctly, or else the shell will hang indefinitely.

### Per-Step Frequency

Control how often individual steps run:

```toml
[frequency]
flatpak = "weekly"
firmware = "monthly"
snap = "daily"
```

### Post-Update Triggers

Run custom commands after specific steps complete:

```toml
[triggers]
brew = "brew cleanup"
flatpak = "flatpak uninstall --unused -y"
```

## Coverage Audit

Topgrade handles dozens of package managers automatically, but tools installed via direct binary
download, `curl | sh` installers, or `git clone` have no built-in update path. Over time these
"orphan" binaries silently fall behind.

The `contrib/topgrade-audit.sh` script scans common binary locations (`~/.local/bin`,
`~/.cargo/bin`, `/usr/local/bin`, etc.) and cross-references each tool against your `topgrade.toml`
custom commands and built-in package manager steps. Any tool without a known update path is
flagged as a blind spot.

```bash
# Run manually
contrib/topgrade-audit.sh

# Run automatically after each topgrade cycle
# Add to your topgrade.toml:
[post_commands]
"Coverage Audit" = "~/.config/topgrade-audit.sh --quiet || true"
```

The script supports `--quiet` mode (only shows blind spots) and `--json` for machine-readable output.

See `config.example.toml` for patterns to add custom update commands for GitHub release binaries,
curl installers, and git repos.

## Remote Execution

You can specify a key called `remote_topgrades` in the configuration file.
This key should contain a list of hostnames that have Topgrade installed on them.
Topgrade will use `ssh` to run `topgrade` on remote hosts before acting locally.
To limit the execution only to specific hosts use the `--remote-host-limit` parameter.

## MSRV

Find the current MSRV in `Cargo.toml` under `rust-version`. This MSRV will only be bumped in a major release.

## Migration and Breaking Changes

Whenever there is a **breaking change**, the major version number will be bumped,
and we will document these changes in the release note, please take a look at
it when updated to a major release.

> Got a question? Feel free to open an issue or discussion!

## Contribution

### Problems or missing features?

Open a new issue describing your problem and if possible provide a solution.

### Missing a feature or found an unsupported tool/distro?

Just let us know what you are missing by opening an issue.
For tools, please open an issue describing the tool, which platforms it supports and if possible, give us an example of
its usage.

### Want to contribute?

See [CONTRIBUTING.md](CONTRIBUTING.md)

## How This Fork Was Built

I built this entire fork in a single session using **Claude Opus 4.6** (Anthropic's CLI agent, Claude Code). The full implementation took approximately **4 hours** of wall-clock time, from first fork to all CI checks green.

Here's what actually happened:

1. **Forked topgrade-rs/topgrade** and audited all 22 open pull requests and 151 open issues on the upstream repo.

2. **Merged all 22 open PRs** — some were clean cherry-picks, others had merge conflicts that required manual resolution (particularly around `config.rs`, which nearly every PR touches). One PR (`merge` crate 0.2 upgrade) broke the build with 104 compilation errors because the new version dropped `Merge` impl for `Option<T>` — I reverted it and kept the working version.

3. **Implemented fixes for 76+ open issues** — I triaged all 151 issues, identified ~54 that were actionable through code changes, and implemented them. Another batch of 47 were implementable from a second pass through "remaining 97" issues. The rest were duplicates, upstream-only concerns (packaging, release infrastructure), or already fixed.

4. **Added 40+ new features** including 11 new package manager steps (Ollama, ldcup, Soar, Colima, install-release, Adless, yt-dlp, uv python, KDE Plasmoids, Hardware IDs, Microsoft Store), per-step update frequency, config file respawn, post-update triggers, custom step ordering, fuzzy step matching, Zellij multiplexer support, and more.

5. **Full i18n coverage** — every new user-facing string got translations in all 7 locales (en, lt, es, fr, zh_CN, zh_TW, de). The CI i18n checker is strict: it requires every locale entry to have translations in every language used in the file, not just English. This caught me twice before I got it right.

6. **Fixed CI across all platforms** — the upstream CI runs on Linux, macOS, Windows, FreeBSD, NetBSD, and Android. Cross-platform compilation issues (Windows-only imports, clippy `collapsible_if` warnings, `which()` return type differences) required multiple rounds of fixes. All 19 CI check runs now pass.

7. **Tested locally on macOS** — installed the fork with `cargo install --path .`, ran a full automated `topgrade` pass. Found a real bug immediately: my existing config had `no_retry = true` (a deprecated key), and the `deny_unknown_fields` attribute on the config struct caused a parse failure. Fixed it by removing the strict deserialization and adding deprecated fields as ignored stubs. Final run: 22/25 steps succeeded, 3 failed due to external reasons (npm registry 403, Ollama server not running, yt-dlp needs sudo).

**Total: 132 commits on top of upstream v17.0.0, ~4 hours, one model (Claude Opus 4.6).**

The hardest parts were the i18n CI checker (which requires complete translations, not just English stubs) and the `config.rs` merge conflicts (every PR and feature touches this file). The easiest parts were the straightforward step additions — the topgrade codebase has a very clean pattern for adding new package manager steps.

## Acknowledgments

This fork builds on the work of:
- [r-darwish](https://github.com/r-darwish) - Original creator of Topgrade
- [topgrade-rs](https://github.com/topgrade-rs) - Maintenance team
- All community contributors who submitted the PRs and issues that this fork integrates
