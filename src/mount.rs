use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use argh::FromArgs;
use command_ext::CommandExt;

use crate::{identity::Identity, resolve};

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

pub(crate) fn main(
    Args {
        name,
        source,
        mount,
    }: Args,
) -> Result<()> {
    self::mount(name, source, mount)
}

/// Mount a directory from the pi locally using sshfs
///
/// # Errors
///
pub fn mount(
    name: impl AsRef<str>,
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
) -> Result<()> {
    let name = name.as_ref();
    let ip = resolve(name)?;
    let mut full_src = OsString::from(format!("pi@{ip}:"));
    full_src.push(src.as_ref());
    Command::new("sshfs")
        .arg("-o")
        .arg(identity(name)?)
        .arg(full_src)
        .arg(dst.as_ref())
        .check_status()?;
    Ok(())
}

fn identity(name: &str) -> Result<OsString> {
    let mut arg = OsString::from("IdentityFile=");
    arg.push(Identity::private(name)?);
    Ok(arg)
}
