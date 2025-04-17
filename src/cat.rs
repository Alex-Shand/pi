use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use command_ext::CommandExt as _;

use crate::CommandExt as _;

/// Read a file on the pi
#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "cat")]
pub(crate) struct Args {
    /// the target pi
    #[argh(positional)]
    name: String,
    /// target path on the pi
    #[argh(positional)]
    path: PathBuf,
}

pub(crate) fn main(Args { name, path }: Args) -> Result<()> {
    println!("{}", cat(name, path)?);
    Ok(())
}

///
/// # Errors
///
pub fn cat(name: impl AsRef<str>, path: impl AsRef<Path>) -> Result<String> {
    Command::new("cat")
        .arg(path.as_ref())
        .run_on_pi(name.as_ref())?
        .check_output()
}
