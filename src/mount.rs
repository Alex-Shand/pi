use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
    process::Command,
};

use anyhow::{bail, Result};
use argh::FromArgs;

use crate::{resolve, utils::CommandExt as _};

/// Mount a directory from a pi locally
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "mount")]
pub(crate) struct Args {
    /// the pi to connect to
    #[argh(positional)]
    name: String,
    /// the source directory on the pi
    #[argh(positional)]
    source: PathBuf,
    /// the mount point on the local machine
    #[argh(positional)]
    mount: PathBuf,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    mount(&args.name, &args.source, &args.mount)
}

/// Mount a directory from the pi locally using sshfs
pub fn mount(
    name: &str,
    src: impl AsRef<OsStr>,
    dst: impl AsRef<OsStr>,
) -> Result<()> {
    let ip = resolve(name)?;
    let mut full_src = OsString::from(format!("pi@{ip}:"));
    full_src.push(src);
    let success = Command::new("sshfs")
        .identity_alt(name)?
        .arg(full_src)
        .arg(dst)
        .status()?
        .success();
    if !success {
        bail!("sshfs failed")
    }
    Ok(())
}
