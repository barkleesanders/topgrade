use crate::command::CommandExt;
use crate::execution_context::ExecutionContext;
use crate::step::Step;
use crate::terminal::print_separator;
use color_eyre::eyre::Result;
use rust_i18n::t;

pub fn upgrade_packages(ctx: &ExecutionContext) -> Result<()> {
    print_separator(t!("DragonFly BSD Packages"));

    let sudo = ctx.require_sudo()?;
    let mut cmd = sudo.execute(ctx, "/usr/local/sbin/pkg")?;
    cmd.arg("upgrade");
    if ctx.config().yes(Step::System) {
        cmd.arg("-y");
    }
    cmd.status_checked()
}

pub fn audit_packages(ctx: &ExecutionContext) -> Result<()> {
    print_separator(t!("DragonFly BSD Audit"));

    let sudo = ctx.require_sudo()?;
    // Exit code 1 means the audit ran successfully but vulnerable packages remain.
    // Other non-zero exit codes indicate actual errors and should be propagated.
    sudo.execute(ctx, "/usr/local/sbin/pkg")?
        .args(["audit", "-Fr"])
        .status_checked_with(|status| {
            if status.code() == Some(1) {
                println!(
                    "{}",
                    t!("The package audit was successful, but vulnerable packages still remain on the system")
                );
                Ok(())
            } else if status.success() {
                Ok(())
            } else {
                Err(())
            }
        })
}
