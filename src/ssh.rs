use crate::{utils, resolve};

use std::process::ExitStatus;
use std::ffi::OsStr;
use std::iter;

use argh::FromArgs;
use anyhow::Result;

/// SSH wrapper for managed pis
#[derive(Debug, FromArgs)]
#[argh(subcommand, name="ssh")]
pub(crate) struct Args {
    /// the pi to connect to
    #[argh(positional)]
    name: String,
    #[argh(positional)]
    cmd: Vec<String>
}

pub(crate) fn main(args: &Args) -> Result<ExitStatus> {
    ssh_internal(&args.name, &args.cmd)
}

/// Run a command on the pi via ssh
///
/// # Errors
/// If the ssh process can't be launched
pub fn ssh<O: AsRef<OsStr>>(
    name: &str,
    cmd: O,
    args: &[O]
) -> Result<ExitStatus> {
    ssh_internal(name, &iter::once(&cmd).chain(args).collect::<Vec<_>>())
}

fn ssh_internal(name: &str, args: &[impl AsRef<OsStr>]) -> Result<ExitStatus> {
    let ip = resolve(name)?;
    let mut cmd = utils::ssh_cmd(name, &ip)?;
    if !args.is_empty() {
        let _ = cmd.args(args);
    }
    Ok(cmd.status()?)
}
