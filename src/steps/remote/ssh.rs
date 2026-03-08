use color_eyre::eyre::Result;
use rust_i18n::t;

use crate::{
    command::CommandExt, error::SkipStep, execution_context::ExecutionContext, terminal::print_separator, utils,
};

fn prepare_async_ssh_command(args: &mut Vec<&str>) {
    args.insert(0, "ssh");
    args.push("--keep");
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

    let mut args = vec!["-t", hostname];

    if let Some(ssh_arguments) = ctx.config().ssh_arguments() {
        args.extend(ssh_arguments.split_whitespace());
    }

    let env = format!("TOPGRADE_PREFIX={hostname}");
    args.extend(["env", &env, "$SHELL", "-lc", &topgrade_cmd]);

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
        let mut args = vec!["-t", hostname];

        if let Some(ssh_arguments) = ctx.config().ssh_arguments() {
            args.extend(ssh_arguments.split_whitespace());
        }

        let env = format!("TOPGRADE_PREFIX={hostname}");
        args.extend(["env", &env, "$SHELL", "-lc", &topgrade_cmd]);

        print_separator(format!("Remote ({hostname})"));
        println!("{}", t!("Connecting to {hostname}...", hostname = hostname));

        ctx.execute(ssh).args(&args).status_checked()
    }
}
