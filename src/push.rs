use crate::{resolve, utils::CommandExt as _};

use std::path::PathBuf;
use std::process::Command;
use std::ffi::{OsStr, OsString};

use argh::FromArgs;
use anyhow::{Result, bail};

/// Send a list of files to the pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name="push")]
pub(crate) struct Args {
    /// the name of the pi to push to
    #[argh(positional)]
    name: String,
    #[argh(positional)]
    destination: PathBuf,
    #[argh(positional)]
    files: Vec<PathBuf>
}

pub(crate) fn main(args: &Args) -> Result<()> {
    push(&args.name, &args.files, &args.destination)
}

pub(crate) fn push(
    name: &str,
    files: &[impl AsRef<OsStr>],
    dst: impl AsRef<OsStr>
) -> Result<()> {
    let ip = resolve(name)?;
    let mut full_dst = OsString::from(format!("pi@{ip}:"));
    full_dst.push(dst);
    let success = Command::new("scp")
        .identity(name)?
        .args(files).arg(full_dst)
        .status()?.success();
    if !success {
        bail!("scp failed")
    }
    Ok(())
}
