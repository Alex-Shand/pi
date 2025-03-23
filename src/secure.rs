use std::{
    fs::{self, File},
    io::Write as _,
    path::Path,
    process::Command,
};

use anyhow::{bail, Result};
use argh::FromArgs;

use crate::{mount, push, utils};

/// Disable password ssh on the target pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "secure")]
pub(crate) struct Args {
    /// the pi to secure
    #[argh(positional)]
    name: String,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    let sshd = read_sshd(&args.name)?;

    let tempdir = tempfile::tempdir()?;
    let new_sshd_path = tempdir.path().join("sshd_config");
    let mut new_sshd = File::create(&new_sshd_path)?;
    writeln!(new_sshd, "{sshd}\nPasswordAuthentication no")?;

    utils::ensure_pi_config(&args.name)?;
    push::push(&args.name, &[new_sshd_path], utils::PI_CONFIG)?;
    ssh!(
        &args.name =>
            "sudo", "mv", &format!("{}/sshd_config", utils::PI_CONFIG), "/etc/ssh/sshd_config"
    )?;

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
