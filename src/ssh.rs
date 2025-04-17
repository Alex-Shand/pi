use std::process::{Command, ExitStatus};

use anyhow::Result;
use argh::FromArgs;

use crate::CommandExt as _;

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
    let mut cmd = if let Some((cmd, args)) = cmd.split_first() {
        let mut cmd = Command::new(cmd);
        let _ = cmd.args(args);
        cmd
    } else {
        Command::new("sh")
    };
    Ok(cmd.run_on_pi(&name)?.status()?)
}
