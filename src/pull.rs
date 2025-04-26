use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use argh::FromArgs;
use command_ext::CommandExt as _;

use crate::{identity::Identity, resolve};

/// Retrieve a list of files from the pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "pull")]
pub(crate) struct Args {
    /// the name of the pi to pull from
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
    pull(name, &files, destination)
}

/// Pull a list of files from the pi
///
/// # Errors
/// If the scp process exits unsuccessfully
pub fn pull(
    name: impl AsRef<str>,
    files: &[impl AsRef<Path>],
    dst: impl AsRef<Path>,
) -> Result<()> {
    let name = name.as_ref();
    let ip = resolve(name)?;
    let files = files
        .iter()
        .map(AsRef::as_ref)
        .map(Path::as_os_str)
        .collect::<Vec<_>>()
        .join(&OsString::from(" "));
    let mut full_src = OsString::from(format!("pi@{ip}:"));
    full_src.push(files);
    Command::new("scp")
        .arg("-i")
        .arg(Identity::private(name)?)
        .arg(full_src)
        .arg(dst.as_ref())
        .check_status()?;
    Ok(())
}
