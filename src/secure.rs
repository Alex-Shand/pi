use std::{
    fs::{self, File},
    io::Write as _,
    path::Path,
    process::Command,
};

use anyhow::{Result, bail};
use argh::FromArgs;
use command_ext::CommandExt as _;

use crate::{CommandExt as _, mount, push, utils};

/// Disable password ssh on the target pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "secure")]
pub(crate) struct Args {
    /// the pi to secure
    #[argh(positional)]
    name: String,
}

pub(crate) fn main(Args { name }: Args) -> Result<()> {
    let sshd = read_sshd(&name)?;

    let tempdir = tempfile::tempdir()?;
    let new_sshd_path = tempdir.path().join("sshd_config");
    let mut new_sshd = File::create(&new_sshd_path)?;
    writeln!(new_sshd, "{sshd}\nPasswordAuthentication no")?;

    utils::ensure_pi_config(&name)?;
    push(&name, &[new_sshd_path], utils::PI_CONFIG)?;
    Command::new("mv")
        .arg(format!("{}/sshd_config", utils::PI_CONFIG))
        .arg("/etc/ssh/sshd_config")
        .run_as_root()
        .run_on_pi(&name)?
        .check_status()?;
    Ok(())
}

fn read_sshd(name: &str) -> Result<String> {
    let tempdir = tempfile::tempdir()?;
    mount(name, "/etc/ssh", tempdir.path())?;
    let _unmount = defer::defer(|| unmount(tempdir.path()));

    let sshd_config = tempdir.path().join("sshd_config");
    Ok(fs::read_to_string(sshd_config)?)
}

fn unmount(path: impl AsRef<Path>) {
    fn aux(path: &Path) -> Result<()> {
        let success = Command::new("fusermount")
            .arg("-u")
            .arg(path)
            .status()?
            .success();
        if success {
            Ok(())
        } else {
            bail!("fusermount -u {path:?} failed")
        }
    }
    let path = path.as_ref();
    aux(path).unwrap_or_else(|e| panic!("{e}"));
}
