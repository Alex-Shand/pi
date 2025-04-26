use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use argh::FromArgs;
use command_ext::CommandExt;

use crate::{identity::Identity, resolve};

/// Send a list of files to the pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "push")]
pub(crate) struct Args {
    /// the name of the pi to push to
    #[argh(positional)]
    name: String,
    #[argh(positional)]
    destination: PathBuf,
    #[argh(positional)]
    files: Vec<PathBuf>,
}

pub(crate) fn main(
    Args {
        name,
        destination,
        files,
    }: Args,
) -> Result<()> {
    push(name, &files, destination)
}

///
/// # Errors
///
pub fn push(
    name: impl AsRef<str>,
    files: &[impl AsRef<Path>],
    dst: impl AsRef<Path>,
) -> Result<()> {
    let name = name.as_ref();
    let ip = resolve(name)?;
    let mut full_dst = OsString::from(format!("pi@{ip}:"));
    full_dst.push(dst.as_ref());
    Command::new("scp")
        .arg("-i")
        .arg(Identity::private(name)?)
        .args(files.iter().map(AsRef::as_ref))
        .arg(full_dst)
        .check_status()?;
    Ok(())
}
