use std::process::{Command, ExitStatus};

use anyhow::Result;
use argh::FromArgs;

use crate::{CommandExt as _, utils::ssh_dir};

/// SSH wrapper for managed pis
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "ssh")]
pub(crate) struct Args {
    /// the pi to connect to
    #[argh(positional)]
    name: String,
    #[argh(positional)]
    cmd: Vec<String>,
}

pub(crate) fn main(Args { name, cmd }: Args) -> Result<ExitStatus> {
    if let Some((cmd, args)) = cmd.split_first() {
        Ok(Command::new(cmd).args(args).run_on_pi(&name)?.status()?)
    } else {
        Ok(Command::new("ssh")
            .arg("-i")
            .arg(ssh_dir()?.join(&name))
            .arg(format!("alex@{name}.local"))
            .status()?)
    }
}
