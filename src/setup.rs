use std::{path::PathBuf, process::Command};

use anyhow::{Result, bail};
use argh::FromArgs;
use command_ext::CommandExt as _;

use crate::{CommandExt as _, push, utils};

/// Deploy setup code to the pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "setup")]
pub(crate) struct Args {
    /// the pi to deploy to
    #[argh(positional)]
    name: String,
    /// the script to deploy, if not present ~/.pi/<name>.sh will be used if
    /// present
    #[argh(positional)]
    script: Option<PathBuf>,
}

pub(crate) fn main(Args { name, script }: Args) -> Result<()> {
    let app_config = utils::app_config()?;
    let script =
        script.unwrap_or_else(|| app_config.join(format!("{name}.sh")));
    if !script.is_file() {
        bail!("{} does not exist or isn't a file", script.display())
    }
    let target = format!("{}/pi.sh", utils::PI_CONFIG);

    utils::ensure_pi_config(&name)?;
    push(&name, &[script], &target)?;
    Command::new("chmod")
        .args(["+x", &target])
        .run_on_pi(&name)?
        .check_status()?;
    Command::new(&target)
        .arg("root")
        .run_as_root()
        .run_on_pi(&name)?
        .check_status()?;
    Command::new(target)
        .arg("user")
        .run_on_pi(&name)?
        .check_status()?;
    Ok(())
}
