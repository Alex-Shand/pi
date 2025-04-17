use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use anyhow::Result;

use super::push;

/// Send a file to the pi
#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "send")]
pub(crate) struct Args {
    /// the pi to send to
    #[argh(positional)]
    name: String,
    /// path on the pi to write to
    #[argh(positional)]
    path: PathBuf,
    /// content to write
    #[argh(positional)]
    contents: String,
}

pub(crate) fn main(
    Args {
        name,
        path,
        contents,
    }: Args,
) -> Result<()> {
    send(name, path, contents)
}

///
/// # Errors
///
pub fn send(
    name: impl AsRef<str>,
    path: impl AsRef<Path>,
    contents: impl AsRef<str>,
) -> Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    write!(temp.as_file_mut(), "{}", contents.as_ref())?;
    push(name, &[temp.path()], path)
}
