use std::io;

use color_eyre::eyre::{Context, Result};
use rust_i18n::t;
use tracing::debug;

use crate::{error::SkipStep, execution_context::ExecutionContext, terminal::print_separator, utils};

fn prepare_async_ssh_command(args: &mut Vec<&str>) {
    args.insert(0, "ssh");
    args.push("--keep");
}

/// Build the remote shell invocation command.
///
/// Uses `$SHELL -lc` for login shell support, with a fallback to `$SHELL -c`
/// for shells that don't support `-l` with `-c` (e.g. FreeBSD's /bin/sh).
fn remote_shell_command(topgrade_cmd: &str, hostname: &str) -> String {
    let env = format!("TOPGRADE_PREFIX={hostname}");
    // Try `$SHELL -lc` first; if the shell doesn't support `-l` with `-c`,
    // fall back to `$SHELL -c` (POSIX-compatible).
    format!(
        "env {env} $SHELL -lc {cmd} 2>/dev/null || env {env} $SHELL -c {cmd}",
        env = shell_words::quote(&env),
        cmd = shell_words::quote(topgrade_cmd),
    )
}

pub fn ssh_step(ctx: &ExecutionContext, hostname: &str) -> Result<()> {
    let ssh = utils::require("ssh")?;

    let topgrade_path = ctx.config().remote_topgrade_path();
    // Build the remote topgrade command, forwarding --verbose if set
    let topgrade_cmd = if ctx.config().verbose() {
        format!("{topgrade_path} --verbose")
    } else {
        topgrade_path.to_string()
    };

    let remote_cmd = remote_shell_command(&topgrade_cmd, hostname);
    debug!("Remote command for {hostname}: {remote_cmd}");

    let mut args = vec!["-t", hostname];

    if let Some(ssh_arguments) = ctx.config().ssh_arguments() {
        args.extend(ssh_arguments.split_whitespace());
    }

    args.push(&remote_cmd);

    if ctx.config().run_in_tmux() && !ctx.run_type().dry() {
        #[cfg(unix)]
        {
            prepare_async_ssh_command(&mut args);
            crate::tmux::run_command(ctx, hostname, &shell_words::join(args))?;
            Err(SkipStep(String::from(t!("Remote Topgrade launched in Tmux"))).into())
        }

        #[cfg(not(unix))]
        unreachable!("Tmux execution is only implemented in Unix");
    } else if ctx.config().open_remotes_in_new_terminal() && !ctx.run_type().dry() && cfg!(windows) {
        prepare_async_ssh_command(&mut args);
        ctx.execute("wt").args(&args).spawn()?;
        Err(SkipStep(String::from(t!("Remote Topgrade launched in an external terminal"))).into())
    } else {
        print_separator(format!("Remote ({hostname})"));
        println!("{}", t!("Connecting to {hostname}...", hostname = hostname));

        // Use status_checked_with_codes_returning to detect exit code 2 (user quit).
        // When a remote topgrade exits with code 2, propagate the quit to the
        // local instance instead of continuing with the remaining steps.
        let status = ctx.execute(ssh).args(&args).status_checked_with_codes_returning(&[2])?;

        if status.code() == Some(2) {
            return Err(io::Error::from(io::ErrorKind::Interrupted))
                .context("Remote topgrade quit by user (exit code 2)");
        }

        Ok(())
    }
}
