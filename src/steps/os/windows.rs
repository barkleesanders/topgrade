use std::fmt::Write as _;
use std::path::Path;
use std::{ffi::OsStr, process::Command};

use color_eyre::eyre::Result;
use etcetera::base_strategy::BaseStrategy;
use rust_i18n::t;
use tracing::{debug, info};

use crate::command::CommandExt;
use crate::config::{SdioDriverpackPolicy, UpdatesAutoReboot};
use crate::execution_context::ExecutionContext;
use crate::step::Step;
use crate::terminal::{print_info, print_separator, print_warning, prompt_yesno};
use crate::utils::{require, which};
use crate::{error::SkipStep, steps::git::RepoStep};

pub fn run_chocolatey(ctx: &ExecutionContext) -> Result<()> {
    let choco = require("choco")?;
    let yes = ctx.config().yes(Step::Chocolatey);

    print_separator("Chocolatey");

    let sudo = ctx.require_sudo()?;

    let mut command = sudo.execute(ctx, &choco)?;
    command.args(["upgrade", "all"]);

    if yes {
        command.arg("--yes");
    }

    command.status_checked()
}

pub fn run_winget(ctx: &ExecutionContext) -> Result<()> {
    let winget = require("winget")?;

    print_separator("winget");

    ctx.execute(&winget).args(["source", "update"]).status_checked()?;

    let mut command = if ctx.config().winget_use_sudo() {
        let sudo = ctx.require_sudo()?;
        sudo.execute(ctx, &winget)?
    } else {
        ctx.execute(winget)
    };

    let mut args = vec!["upgrade", "--all"];
    if ctx.config().winget_silent_install() {
        args.push("--silent");
    }

    command.args(args).status_checked()?;

    Ok(())
}

pub fn run_scoop(ctx: &ExecutionContext) -> Result<()> {
    let scoop = require("scoop")?;

    print_separator("Scoop");

    ctx.execute(&scoop).args(["update"]).status_checked()?;
    ctx.execute(&scoop).args(["update", "*"]).status_checked()?;

    if ctx.config().cleanup() {
        ctx.execute(&scoop).args(["cleanup", "*"]).status_checked()?;
        ctx.execute(&scoop).args(["cache", "rm", "-a"]).status_checked()?
    }
    Ok(())
}

pub fn update_wsl(ctx: &ExecutionContext) -> Result<()> {
    if !is_wsl_installed()? {
        return Err(SkipStep(t!("WSL not installed").to_string()).into());
    }

    let wsl = require("wsl")?;

    print_separator(t!("Update WSL"));

    let mut wsl_command = ctx.execute(wsl);
    wsl_command.args(["--update"]);

    if ctx.config().wsl_update_pre_release() {
        wsl_command.args(["--pre-release"]);
    }

    if ctx.config().wsl_update_use_web_download() {
        wsl_command.args(["--web-download"]);
    }
    wsl_command.status_checked()?;
    Ok(())
}

/// Detect if WSL is installed or not.
///
/// For WSL, we cannot simply check if command `wsl` is installed as on newer
/// versions of Windows (since windows 10 version 2004), this command is
/// installed by default.
///
/// If the command is installed and the user hasn't installed any Linux distros
/// on it, command `wsl -l` would print a help message and exit with failure, we
/// use this to check whether WSL is install or not.
fn is_wsl_installed() -> Result<bool> {
    if let Some(wsl) = which("wsl") {
        // Don't use `output_checked` as an execution failure log is not wanted
        #[allow(clippy::disallowed_methods)]
        let output = Command::new(wsl).arg("-l").output()?;
        let status = output.status;

        if status.success() {
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_wsl_distributions(ctx: &ExecutionContext, wsl: &Path) -> Result<Vec<String>> {
    let output = ctx
        .execute(wsl)
        .always()
        .args(["--list", "-q"])
        .output_checked_utf8()?
        .stdout;
    Ok(output
        .lines()
        .map(|x| x.replace(['\u{0}', '\r'], "").trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect())
}

fn upgrade_wsl_distribution(wsl: &Path, dist: &str, ctx: &ExecutionContext) -> Result<()> {
    let topgrade = ctx
        .execute(wsl)
        .always()
        .args(["-d", dist, "bash", "-lc", "which topgrade"])
        .output_checked_utf8()
        .map_err(|_| SkipStep(t!("Could not find Topgrade installed in WSL").to_string()))?
        .stdout // The normal output from `which topgrade` appends a newline, so we trim it here.
        .trim_end()
        .to_owned();

    let mut command = ctx.execute(wsl);

    // The `arg` method automatically quotes its arguments.
    // This means we can't append additional arguments to `topgrade` in WSL
    // by calling `arg` successively.
    //
    // For example:
    //
    // ```rust
    // command
    //  .args(["-d", dist, "bash", "-lc"])
    //  .arg(format!("TOPGRADE_PREFIX={dist} exec {topgrade}"));
    // ```
    //
    // creates a command string like:
    // > `C:\WINDOWS\system32\wsl.EXE -d Ubuntu bash -lc 'TOPGRADE_PREFIX=Ubuntu exec /bin/topgrade'`
    //
    // Adding the following:
    //
    // ```rust
    // command.arg("-v");
    // ```
    //
    // appends the next argument like so:
    // > `C:\WINDOWS\system32\wsl.EXE -d Ubuntu bash -lc 'TOPGRADE_PREFIX=Ubuntu exec /bin/topgrade' -v`
    // which means `-v` isn't passed to `topgrade`.
    // All flags must be included in the single string passed to `bash -lc`
    // because WSL treats each `arg()` call as a separate argument to wsl.exe,
    // not to the inner topgrade command (see comment above).
    let mut topgrade_args = Vec::new();
    if ctx.config().verbose() {
        topgrade_args.push("-v");
    }
    if ctx.config().yes(Step::Wsl) {
        topgrade_args.push("-y");
    }
    if ctx.config().cleanup() {
        topgrade_args.push("--cleanup");
    }
    let args = topgrade_args.join(" ");

    command
        .args(["-d", dist, "bash", "-lc"])
        .arg(format!("TOPGRADE_PREFIX={dist} exec {topgrade} {args}"));

    command.status_checked()
}

pub fn run_wsl_topgrade(ctx: &ExecutionContext) -> Result<()> {
    if !is_wsl_installed()? {
        return Err(SkipStep(t!("WSL not installed").to_string()).into());
    }

    let wsl = require("wsl")?;
    let wsl_distributions = get_wsl_distributions(ctx, &wsl)?;
    let mut ran = false;

    debug!("WSL distributions: {:?}", wsl_distributions);

    for distribution in wsl_distributions {
        let result = upgrade_wsl_distribution(&wsl, &distribution, ctx);
        debug!("Upgrading {:?}: {:?}", distribution, result);
        if let Err(e) = result
            && e.is::<SkipStep>()
        {
            continue;
        }
        ran = true
    }

    if ran {
        Ok(())
    } else {
        Err(SkipStep(t!("Could not find Topgrade in any WSL distribution").to_string()).into())
    }
}

pub fn windows_update(ctx: &ExecutionContext) -> Result<()> {
    let powershell = ctx.require_powershell()?;

    print_separator(t!("Windows Update"));

    if !powershell.has_module(ctx, "PSWindowsUpdate") {
        print_warning(t!(
            "The PSWindowsUpdate PowerShell module isn't installed so Topgrade can't run Windows Update.\nInstall PSWindowsUpdate by running `Install-Module PSWindowsUpdate` in PowerShell."
        ));

        return Err(SkipStep(t!("PSWindowsUpdate is not installed").to_string()).into());
    }

    let mut cmd = "Import-Module PSWindowsUpdate; Install-WindowsUpdate -Verbose".to_string();

    if ctx.config().accept_all_windows_updates() {
        cmd.push_str(" -AcceptAll");
    }

    match ctx.config().windows_updates_auto_reboot() {
        UpdatesAutoReboot::Yes => cmd.push_str(" -AutoReboot"),
        UpdatesAutoReboot::No => cmd.push_str(" -IgnoreReboot"),
        UpdatesAutoReboot::Ask => (), // Prompting is the default for Install-WindowsUpdate
    }

    powershell.build_command(ctx, &cmd, true)?.status_checked()
}

pub fn microsoft_store(ctx: &ExecutionContext) -> Result<()> {
    print_separator(t!("Microsoft Store"));

    // Try the `store` CLI first (Microsoft Store CLI tool)
    if let Ok(store_cli) = require("store") {
        debug!("Found Microsoft Store CLI at {}", store_cli.display());
        println!("{}", t!("Checking for updates via store CLI..."));
        ctx.execute(&store_cli).arg("updates").status_checked()?;
        return Ok(());
    }

    // Fall back to PowerShell CIM method
    let powershell = ctx.require_powershell()?;

    println!("{}", t!("Scanning for updates..."));

    // Scan for updates using the MDM UpdateScanMethod
    // This method is also available for non-MDM devices
    let cmd = r#"(Get-CimInstance -Namespace "Root\cimv2\mdm\dmmap" -ClassName "MDM_EnterpriseModernAppManagement_AppManagement01" | Invoke-CimMethod -MethodName UpdateScanMethod).ReturnValue"#;

    powershell
        .build_command(ctx, cmd, true)?
        .output_checked_with_utf8(|output| {
            if !output.status.success() {
                return Err(());
            }
            let ret_val = output.stdout.trim();
            debug!("Command return value: {}", ret_val);
            if ret_val == "0" { Ok(()) } else { Err(()) }
        })?;
    println!(
        "{}",
        t!("Success, Microsoft Store apps are being updated in the background")
    );
    Ok(())
}

pub fn reboot(ctx: &ExecutionContext) -> Result<()> {
    // If this works, it won't return, but if it doesn't work, it may return a useful error
    // message.
    ctx.execute("shutdown.exe").args(["/R", "/T", "0"]).status_checked()
}

pub fn insert_startup_scripts(ctx: &ExecutionContext, git_repos: &mut RepoStep) -> Result<()> {
    let startup_dir = crate::WINDOWS_DIRS
        .data_dir()
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
    for entry in std::fs::read_dir(&startup_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(OsStr::to_str) == Some("lnk")
            && let Ok(lnk) = parselnk::Lnk::try_from(Path::new(&path))
        {
            debug!("Startup link: {:?}", lnk);
            if let Some(path) = lnk.relative_path() {
                git_repos.insert_if_repo(ctx, startup_dir.join(path));
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// SDIO (Snappy Driver Installer Origin) step
// ---------------------------------------------------------------------------

/// Updates drivers using Snappy Driver Installer Origin (SDIO).
///
/// SDIO is a free open-source tool for downloading and installing drivers.
/// It will be executed in script mode to automatically download missing driver packs
/// and install missing drivers with restore point creation when possible.
///
/// **Important**: This step requires explicit opt-in via the `enable_sdio = true`
/// configuration setting due to the critical nature of driver updates.
///
/// **Interactive Mode** (without --yes): Shows available driver updates and asks for user confirmation
/// **Automatic Mode** (with --yes): Installs drivers automatically without user interaction
pub fn run_sdio(ctx: &ExecutionContext) -> Result<()> {
    // Check if SDIO is explicitly enabled by the user
    if !ctx.config().enable_sdio() {
        return Err(SkipStep(
            t!("SDIO driver updates are disabled. Enable with 'enable_sdio = true' in [windows] section").to_string(),
        )
        .into());
    }

    let sdio = if let Some(configured_path) = ctx.config().sdio_path() {
        // Use configured path first (expand Windows env vars like %USERPROFILE%)
        let expanded = expand_env_vars_windows(configured_path);
        require(&expanded)?
    } else {
        // Try to detect SDIO automatically using various methods
        detect_sdio()?
    };

    let yes = ctx.config().yes(Step::Sdio);
    let interactive = !ctx.run_type().dry() && !yes;

    print_separator(t!("Snappy Driver Installer Origin"));

    // Create dedicated temp directory for SDIO operations
    let sdio_work_dir = std::env::temp_dir().join("topgrade_sdio");
    std::fs::create_dir_all(&sdio_work_dir).ok();

    let config = ctx.config();

    // Create a dynamic SDIO script based on run mode and user preferences
    let verbose_output = config.verbose();

    let script_options = SdioScriptOptions {
        selection_filters: config.sdio_selection_filters(),
        driverpack_policy: config.sdio_driverpack_policy(),
        fetch_indexes: config.sdio_fetch_indexes(),
        fetch_updates: config.sdio_fetch_updates(),
        prefetch_in_analysis: config.sdio_prefetch_driverpacks_in_analysis(),
        debug_logging: config.sdio_debug_logging(verbose_output),
        verbose_level: config.sdio_verbose_level(verbose_output),
        emit_echo: config.sdio_emit_echo(verbose_output),
        keep_tempfiles: config.sdio_keep_tempfiles(verbose_output),
        restore_point: config.sdio_restore_point_enabled(),
        restore_point_description: config.sdio_restore_point_description(),
    };

    let primary_mode = if ctx.run_type().dry() {
        ScriptMode::DryAnalysis
    } else if yes {
        ScriptMode::AutomaticInstall
    } else {
        ScriptMode::InteractiveAnalysis
    };

    let script_content = build_sdio_script(&sdio_work_dir, &script_options, primary_mode);

    // Write the script to temp directory
    let script_path = sdio_work_dir.join("topgrade_sdio_script.txt");
    std::fs::write(&script_path, script_content).map_err(|e| {
        SkipStep(format!(
            "Failed to create SDIO script at {}: {}",
            script_path.display(),
            e
        ))
    })?;

    // Log the command being executed for transparency and security auditing
    debug!("SDIO command: {:?} -script {:?}", sdio, script_path);
    info!("Running SDIO script: {}", script_path.display());
    info!("SDIO working directory: {}", sdio_work_dir.display());
    info!("SDIO binary location: {}", sdio.display());

    let mut command = match ctx.sudo() {
        Some(sudo) => sudo.execute(ctx, &sdio)?,
        None => ctx.execute(&sdio),
    };
    // Pass -script and script path as separate args to handle spaces safely
    command.arg("-script").arg(&script_path);
    command.current_dir(&sdio_work_dir);

    announce_script_start(primary_mode, verbose_output);
    let mut result = command.status_checked();
    announce_script_finish(primary_mode, verbose_output, result.is_ok());

    // If interactive: ask the user whether to proceed with installation and run a second script
    if interactive && result.is_ok() {
        let report_path = sdio_work_dir.join("selected_device_report.txt");
        let mut should_prompt = true;

        match count_selected_drivers(&report_path) {
            Ok(0) => {
                print_info(t!(
                    "SDIO analysis found no drivers to install; keeping this run in analysis mode."
                ));
                info!(
                    "{}",
                    t!("SDIO analysis found no drivers to install; keeping this run in analysis mode.")
                );
                should_prompt = false;
            }
            Ok(count) => {
                debug!("SDIO analysis selected {} driver(s) for installation", count);
            }
            Err(err) => {
                debug!(
                    "Unable to inspect SDIO selection report at {}: {}",
                    report_path.display(),
                    err
                );
            }
        }

        if should_prompt {
            if let Ok(true) = prompt_yesno(&t!(
                "Proceed to install selected drivers now? This will create a restore point first. (y/N)"
            )) {
                // Build an installation script similar to --yes flow
                let install_mode = ScriptMode::InteractiveInstall;
                let install_script = build_sdio_script(&sdio_work_dir, &script_options, install_mode);

                let install_script_path = sdio_work_dir.join("topgrade_sdio_install_script.txt");
                std::fs::write(&install_script_path, install_script).map_err(|e| {
                    SkipStep(format!(
                        "Failed to create SDIO install script at {}: {}",
                        install_script_path.display(),
                        e
                    ))
                })?;

                debug!("SDIO command (install): {:?} -script {:?}", sdio, install_script_path);
                info!("Running SDIO install script: {}", install_script_path.display());
                let mut install_cmd = match ctx.sudo() {
                    Some(sudo) => sudo.execute(ctx, &sdio)?,
                    None => ctx.execute(&sdio),
                };
                install_cmd.arg("-script").arg(&install_script_path);
                install_cmd.current_dir(&sdio_work_dir);
                announce_script_start(install_mode, verbose_output);
                result = install_cmd.status_checked();
                announce_script_finish(install_mode, verbose_output, result.is_ok());
            } else {
                info!("User declined SDIO installation; analysis-only run completed.");
            }
        }
    }

    // Best-effort cleanup of the temporary workdir on success
    if result.is_ok() && !script_options.keep_tempfiles {
        let _ = std::fs::remove_dir_all(&sdio_work_dir);
    }

    result
}

/// Detects SDIO installation using multiple strategies based on SDIO documentation
fn detect_sdio() -> Result<std::path::PathBuf> {
    let is_64bit = std::env::consts::ARCH == "x86_64";

    // Strategy 1: Try PATH-based executables with priority order
    if let Some(exe) = detect_sdio_in_path(is_64bit) {
        return Ok(exe);
    }

    // Strategy 2: Check common installation locations
    if let Some(exe) = detect_sdio_in_common_locations(is_64bit) {
        return Ok(exe);
    }

    Err(SkipStep(t!("SDIO (Snappy Driver Installer Origin) not found").to_string()).into())
}

/// Detects SDIO executables in PATH with architecture-aware priority
fn detect_sdio_in_path(is_64bit: bool) -> Option<std::path::PathBuf> {
    let executable_names = get_sdio_executable_names(is_64bit);

    for name in &executable_names {
        if let Some(exe) = which(name) {
            return Some(exe);
        }
    }
    None
}

/// Returns SDIO executable patterns in priority order
fn get_sdio_executable_names(is_64bit: bool) -> Vec<&'static str> {
    if is_64bit {
        vec![
            "SDIO_auto.bat",
            "SDIO_x64.exe",
            "SDIO.exe",
            "sdio",
        ]
    } else {
        vec!["SDIO_auto.bat", "SDIO.exe", "sdio"]
    }
}

/// Detects SDIO in common installation locations
fn detect_sdio_in_common_locations(is_64bit: bool) -> Option<std::path::PathBuf> {
    let locations = get_common_sdio_locations();

    for location in locations {
        let base_path = std::path::PathBuf::from(&location);
        if !base_path.exists() {
            continue;
        }

        if base_path.is_file() {
            return Some(base_path);
        }

        // Try SDIO_auto.bat first (recommended)
        let auto_bat = base_path.join("SDIO_auto.bat");
        if auto_bat.exists() {
            return Some(auto_bat);
        }

        // Try versioned executables
        if let Some(exe) = find_best_executable_in_dir(&base_path, is_64bit) {
            return Some(exe);
        }
    }
    None
}

fn count_selected_drivers(report_path: &Path) -> std::io::Result<usize> {
    let data = std::fs::read(report_path)?;
    let content = String::from_utf8_lossy(&data);

    Ok(content.lines().filter(|line| is_marked_selected(line)).count())
}

fn is_marked_selected(line: &str) -> bool {
    let mut parts = line.split([':', '=']);
    let key = match parts.next() {
        Some(key) => key.trim(),
        None => return false,
    };

    if !key.eq_ignore_ascii_case("selected") {
        return false;
    }

    let value = parts.next().map(|value| value.trim()).unwrap_or_default();
    let token = value.split_whitespace().next().unwrap_or("");

    if let Ok(num) = token.parse::<i32>() {
        return num > 0;
    }

    matches!(token.to_ascii_lowercase().as_str(), "true" | "yes")
}

#[derive(Clone, Copy)]
enum ScriptMode {
    DryAnalysis,
    InteractiveAnalysis,
    AutomaticInstall,
    InteractiveInstall,
}

struct SdioScriptOptions {
    selection_filters: Vec<String>,
    driverpack_policy: SdioDriverpackPolicy,
    fetch_indexes: bool,
    fetch_updates: bool,
    prefetch_in_analysis: bool,
    debug_logging: bool,
    verbose_level: u16,
    emit_echo: bool,
    keep_tempfiles: bool,
    restore_point: bool,
    restore_point_description: String,
}

fn build_sdio_script(work_dir: &Path, options: &SdioScriptOptions, mode: ScriptMode) -> String {
    let mut script = String::new();

    match mode {
        ScriptMode::DryAnalysis => {
            append_script_header(
                &mut script,
                "Topgrade SDIO Analysis Script",
                "This script analyzes the system for driver updates without installing",
                work_dir,
                options,
            );

            script.push_str("enableinstall off\n\n");

            push_echo_line(
                &mut script,
                options.emit_echo,
                "Topgrade: starting SDIO dry-run analysis...",
            );
            push_command_with_onerror(&mut script, "init");

            script.push_str("# Generate device analysis report before selection\n");
            script.push_str("writedevicelist device_analysis_before.txt\n\n");

            push_selection_command(&mut script, &options.selection_filters);

            script.push_str("# Generate device analysis report after selection\n");
            script.push_str("writedevicelist device_analysis_after.txt\n\n");

            push_echo_line(
                &mut script,
                options.emit_echo,
                "Topgrade: SDIO dry-run analysis complete; no drivers installed.",
            );

            append_script_footer(&mut script, options, "End without installation");
        }
        ScriptMode::InteractiveAnalysis => {
            append_script_header(
                &mut script,
                "Topgrade SDIO Interactive Analysis Script",
                "This script analyzes available driver updates and exits without installing",
                work_dir,
                options,
            );

            script.push_str("enableinstall off\n\n");

            push_echo_line(&mut script, options.emit_echo, "Topgrade: running SDIO analysis...");

            if should_run_checkupdates(mode, options) {
                push_command_with_onerror(&mut script, "checkupdates");
            }

            if should_fetch_indexes(mode, options) {
                push_command_with_onerror(&mut script, "get indexes");
            }

            push_command_with_onerror(&mut script, "init");

            script.push_str("# Generate initial device report\n");
            script.push_str("writedevicelist initial_device_report.txt\n\n");

            push_selection_command(&mut script, &options.selection_filters);

            script.push_str("# Generate selected devices report (what would be changed)\n");
            script.push_str("writedevicelist selected_device_report.txt\n\n");

            if should_download_driverpacks(mode, options) {
                if let Some(arg) = driverpack_policy_argument(options.driverpack_policy) {
                    let _ = writeln!(script, "get driverpacks {}", arg);
                    script.push_str("onerror goto end\n\n");
                }
            }

            push_echo_line(
                &mut script,
                options.emit_echo,
                "Topgrade: SDIO analysis complete; review reports for details.",
            );

            append_script_footer(&mut script, options, "End script");
        }
        ScriptMode::AutomaticInstall | ScriptMode::InteractiveInstall => {
            let (title, start_message, finish_message) = match mode {
                ScriptMode::AutomaticInstall => (
                    "Topgrade SDIO Automatic Installation Script",
                    "Topgrade: starting SDIO automatic installation...",
                    "Topgrade: SDIO installation finished; review reports for details.",
                ),
                ScriptMode::InteractiveInstall => (
                    "Topgrade SDIO Installation Script (interactive-confirmed)",
                    "Topgrade: starting SDIO installation...",
                    "Topgrade: SDIO installation complete; review reports for details.",
                ),
                _ => unreachable!(),
            };

            append_script_header(
                &mut script,
                title,
                "This script installs the selected drivers with safety measures",
                work_dir,
                options,
            );

            script.push_str("enableinstall on\n\n");

            push_echo_line(&mut script, options.emit_echo, start_message);

            if should_run_checkupdates(mode, options) {
                push_command_with_onerror(&mut script, "checkupdates");
            }

            if should_fetch_indexes(mode, options) {
                push_command_with_onerror(&mut script, "get indexes");
            }

            push_command_with_onerror(&mut script, "init");

            script.push_str("# Generate initial device report\n");
            script.push_str("writedevicelist initial_device_report.txt\n\n");

            if options.restore_point {
                let description = escape_for_quotes(&options.restore_point_description);
                let _ = writeln!(script, "restorepoint \"{}\"", description);
                script.push_str("onerror echo Warning: Failed to create restore point, continuing anyway...\n\n");
            }

            push_selection_command(&mut script, &options.selection_filters);

            script.push_str("# Record planned driver changes\n");
            script.push_str("writedevicelist selected_device_report.txt\n\n");

            if should_download_driverpacks(mode, options) {
                if let Some(arg) = driverpack_policy_argument(options.driverpack_policy) {
                    let _ = writeln!(script, "get driverpacks {}", arg);
                    script.push_str("onerror goto end\n\n");
                }
            }

            script.push_str("install\n");
            script.push_str("onerror echo Warning: Some drivers may have failed to install\n\n");

            script.push_str("# Generate final device report\n");
            script.push_str("writedevicelist final_device_report.txt\n\n");

            push_echo_line(&mut script, options.emit_echo, finish_message);

            append_script_footer(&mut script, options, "End script");
        }
    }

    script
}

fn append_script_header(
    script: &mut String,
    title: &str,
    description: &str,
    work_dir: &Path,
    options: &SdioScriptOptions,
) {
    let _ = writeln!(script, "# {title}");
    if !description.is_empty() {
        let _ = writeln!(script, "# {description}");
    }
    script.push('\n');
    script.push_str("# Configure directories (quoted for safety)\n");
    let _ = writeln!(script, "extractdir \"{}\"", work_dir.display());
    let _ = writeln!(script, "logdir \"{}\"", work_dir.join("logs").display());
    script.push('\n');
    script.push_str("# Configure logging and verbosity\n");
    script.push_str("logging on\n");
    if options.debug_logging {
        script.push_str("debug on\n");
    } else {
        script.push_str("debug off\n");
    }
    let _ = writeln!(script, "verbose {}", options.verbose_level);
    if options.keep_tempfiles {
        script.push_str("keeptempfiles on\n");
    } else {
        script.push_str("keeptempfiles off\n");
    }
    script.push('\n');
}

fn append_script_footer(script: &mut String, options: &SdioScriptOptions, end_comment: &str) {
    script.push_str(":end\n");
    script.push_str("logging off\n");
    if options.debug_logging {
        script.push_str("debug off\n");
    }
    let _ = writeln!(script, "# {end_comment}");
    script.push_str("end\n");
}

fn push_echo_line(script: &mut String, emit: bool, message: &str) {
    if emit {
        let _ = writeln!(script, "echo {}", message);
    }
}

fn push_command_with_onerror(script: &mut String, command: &str) {
    script.push_str(command);
    script.push('\n');
    script.push_str("onerror goto end\n\n");
}

fn push_selection_command(script: &mut String, filters: &[String]) {
    if filters.is_empty() {
        script.push_str("select missing newer better\n\n");
        return;
    }

    script.push_str("select");
    for filter in filters {
        script.push(' ');
        script.push_str(filter);
    }
    script.push('\n');
    script.push('\n');
}

fn driverpack_policy_argument(policy: SdioDriverpackPolicy) -> Option<&'static str> {
    match policy {
        SdioDriverpackPolicy::None => None,
        SdioDriverpackPolicy::Selected => Some("selected"),
        SdioDriverpackPolicy::Missing => Some("missing"),
        SdioDriverpackPolicy::Updates => Some("updates"),
        SdioDriverpackPolicy::All => Some("all"),
    }
}

fn should_run_checkupdates(mode: ScriptMode, options: &SdioScriptOptions) -> bool {
    if matches!(mode, ScriptMode::DryAnalysis) {
        return false;
    }

    if options.fetch_updates || options.fetch_indexes {
        return true;
    }

    matches!(mode, ScriptMode::AutomaticInstall | ScriptMode::InteractiveInstall)
        && driverpack_policy_argument(options.driverpack_policy).is_some()
}

fn should_fetch_indexes(mode: ScriptMode, options: &SdioScriptOptions) -> bool {
    !matches!(mode, ScriptMode::DryAnalysis) && options.fetch_indexes
}

fn should_download_driverpacks(mode: ScriptMode, options: &SdioScriptOptions) -> bool {
    if !options.fetch_updates {
        return false;
    }

    if driverpack_policy_argument(options.driverpack_policy).is_none() {
        return false;
    }

    match mode {
        ScriptMode::DryAnalysis => false,
        ScriptMode::InteractiveAnalysis => options.prefetch_in_analysis,
        ScriptMode::AutomaticInstall | ScriptMode::InteractiveInstall => true,
    }
}

fn escape_for_quotes(input: &str) -> String {
    input.replace('"', "\\\"")
}

fn announce_script_start(mode: ScriptMode, verbose: bool) {
    let message = match mode {
        ScriptMode::DryAnalysis => t!("Running SDIO dry-run analysis..."),
        ScriptMode::InteractiveAnalysis => t!("Running SDIO analysis..."),
        ScriptMode::AutomaticInstall => t!("Running SDIO automatic installation..."),
        ScriptMode::InteractiveInstall => t!("Running SDIO installation..."),
    };

    if verbose {
        debug!("{message}");
    } else {
        print_info(message);
    }
}

fn announce_script_finish(mode: ScriptMode, verbose: bool, succeeded: bool) {
    if !succeeded {
        return;
    }

    let message = match mode {
        ScriptMode::DryAnalysis => t!("SDIO dry-run analysis complete."),
        ScriptMode::InteractiveAnalysis => t!("SDIO analysis complete."),
        ScriptMode::AutomaticInstall => t!("SDIO automatic installation complete."),
        ScriptMode::InteractiveInstall => t!("SDIO installation complete."),
    };

    if verbose {
        debug!("{message}");
    } else {
        print_info(message);
    }
}

/// Expand Windows-style environment variables (e.g., %USERPROFILE%) in a path string.
/// Unknown variables are replaced with an empty string (matching common shell behavior).
fn expand_env_vars_windows(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut i = 0usize;
    let bytes = input.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'%' {
            // find the next '%'
            if let Some(end) = input[i + 1..].find('%') {
                let var_name = &input[i + 1..i + 1 + end];
                // Move index past the closing '%'
                i += end + 2;
                if !var_name.is_empty() {
                    if let Ok(val) = std::env::var(var_name) {
                        result.push_str(&val);
                    }
                    continue;
                } else {
                    // Handle literal '%%'
                    result.push('%');
                    continue;
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Returns common SDIO installation locations
fn get_common_sdio_locations() -> Vec<String> {
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();

    let mut locations = vec![
        // Scoop installation in user profile
        format!("{user_profile}\\scoop\\apps\\snappy-driver-installer-origin\\current"),
        // Common program files locations
        "C:\\Program Files\\SDIO".to_string(),
        "C:\\Program Files (x86)\\SDIO".to_string(),
        // Portable installations
        "C:\\SDIO".to_string(),
        format!("{user_profile}\\SDIO"),
    ];

    if !user_profile.is_empty() {
        locations.push(user_profile.clone());
        locations.push(format!("{user_profile}\\Desktop"));
        locations.push(format!("{user_profile}\\Downloads"));
    }

    let program_data = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    locations.push(format!(
        "{program_data}\\chocolatey\\lib\\snappy-driver-installer-origin\\tools"
    ));
    locations.push(format!(
        "{program_data}\\chocolatey\\lib\\snappy-driver-installer-origin\\tools\\SDIO"
    ));

    locations
}

/// Finds the best SDIO executable in a directory based on architecture
fn find_best_executable_in_dir(dir: &Path, is_64bit: bool) -> Option<std::path::PathBuf> {
    use std::fs;

    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };

    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.starts_with("SDIO") && name.ends_with(".exe") {
            let path = entry.path();
            let priority = get_executable_priority(&name, is_64bit);
            candidates.push((priority, path));
        }
    }

    // Sort by priority (lower number = higher priority)
    candidates.sort_by_key(|(priority, _)| *priority);
    candidates.into_iter().map(|(_, path)| path).next()
}

/// Assigns priority to SDIO executables (lower number = higher priority)
fn get_executable_priority(name: &str, is_64bit: bool) -> u32 {
    match (name, is_64bit) {
        (name, true) if name.contains("x64") && name.starts_with("SDIO_x64_R") => 1,
        (name, true) if name.contains("x64") => 2,
        (name, _) if name.starts_with("SDIO_R") => 3,
        ("SDIO.exe", _) => 4,
        _ => 5,
    }
}
