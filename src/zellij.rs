use std::env;
use std::process::Command;

use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;

use crate::command::CommandExt;
use crate::utils::which;

use rust_i18n::t;
#[cfg(unix)]
use std::os::unix::process::CommandExt as _;

/// Run topgrade inside a new zellij session.
pub fn run_in_zellij() -> Result<()> {
    let zellij = which("zellij").ok_or_else(|| eyre!("Could not find zellij"))?;

    let mut command = vec![
        String::from("env"),
        String::from("TOPGRADE_KEEP_END=1"),
        String::from("TOPGRADE_INSIDE_TMUX=1"),
    ];
    command.extend(env::args());
    let topgrade_cmd = shell_words::join(&command);

    // Create a new zellij session running topgrade
    let session_name = "topgrade";

    // Check if we're already inside zellij
    let is_inside_zellij = env::var("ZELLIJ").is_ok();

    if is_inside_zellij {
        // If inside zellij, open a new pane with the topgrade command
        #[allow(clippy::disallowed_methods)]
        let err = Command::new(&zellij)
            .args(["run", "--", "sh", "-c", &topgrade_cmd])
            .exec();
        return Err(eyre!("{err}")).context("Failed to exec zellij run");
    }

    // If not inside zellij, create a new session
    #[allow(clippy::disallowed_methods)]
    let err = Command::new(&zellij)
        .args(["--session", session_name, "--", "sh", "-c", &topgrade_cmd])
        .exec();

    Err(eyre!("{err}")).context("Failed to exec zellij")
}
