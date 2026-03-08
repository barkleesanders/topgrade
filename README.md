<div align="center">
  <h1>
    <img alt="Topgrade" src="https://github.com/topgrade-rs/topgrade/blob/main/doc/topgrade_transparent.png?raw=true" width="850px">
  </h1>

<a href="https://github.com/barkleesanders/topgrade/actions"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/barkleesanders/topgrade/ci.yml?branch=main&label=CI"></a>
<a href="https://github.com/barkleesanders/topgrade/actions"><img alt="i18n" src="https://img.shields.io/github/actions/workflow/status/barkleesanders/topgrade/check_i18n.yml?branch=main&label=i18n"></a>
<a href="https://github.com/barkleesanders/topgrade/actions"><img alt="Security" src="https://img.shields.io/github/actions/workflow/status/barkleesanders/topgrade/check_security_vulnerability.yml?branch=main&label=Security"></a>
<a href="https://crates.io/crates/topgrade"><img alt="crates.io" src="https://img.shields.io/crates/v/topgrade.svg"></a>
<a href="https://aur.archlinux.org/packages/topgrade"><img alt="AUR" src="https://img.shields.io/aur/version/topgrade.svg"></a>
<a href="https://formulae.brew.sh/formula/topgrade"><img alt="Homebrew" src="https://img.shields.io/homebrew/v/topgrade.svg"></a>

  <img alt="Demo" src="https://github.com/topgrade-rs/topgrade/blob/main/doc/topgrade_demo.gif?raw=true">
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

## Acknowledgments

This fork builds on the work of:
- [r-darwish](https://github.com/r-darwish) - Original creator of Topgrade
- [topgrade-rs](https://github.com/topgrade-rs) - Maintenance team
- All community contributors who submitted the PRs and issues that this fork integrates
