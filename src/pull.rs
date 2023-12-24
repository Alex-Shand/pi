use crate::{resolve, utils::CommandExt as _};

use std::path::PathBuf;
use std::process::Command;
use std::ffi::{OsStr, OsString};

use argh::FromArgs;
use anyhow::{Result, bail};

/// Retrieve a list of files from the pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name="pull")]
pub(crate) struct Args {
    /// the name of the pi to pull from
    #[argh(positional)]
    name: String,
    #[argh(positional)]
    destination: PathBuf,
    #[argh(positional)]
    files: Vec<PathBuf>
}

pub(crate) fn main(args: &Args) -> Result<()> {
    pull(&args.name, &args.files, &args.destination)
}

/// Pull a list of files from the pi
/// 
/// # Errors
/// If the scp process exits unsuccessfully
pub fn pull(
    name: &str,
    files: &[impl AsRef<OsStr>],
    dst: impl AsRef<OsStr>
) -> Result<()> {
    let ip = resolve(name)?;
    let files = files.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(&OsString::from(" "));
    let mut full_src = OsString::from(format!("pi@{ip}:"));
    full_src.push(files);
    let success = Command::new("scp")
        .identity(name)?
        .arg(full_src).arg(dst)
        .status()?.success();
    if !success {
        bail!("scp failed")
    }
    Ok(())
}
