use crate::identity::Identity;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::ffi::OsString;
use std::borrow::Borrow;

use anyhow::{Result, bail};

pub(crate) const PI_CONFIG: &str = "/home/pi/.pi";

pub(crate) enum DefaultPrompt {
    Yes,
    No
}

pub(crate) trait CommandExt {
    fn identity(&mut self, name: &str) -> Result<&mut Self>;
    fn identity_alt(&mut self, name: &str) -> Result<&mut Self>;
}

impl CommandExt for Command {
    fn identity(&mut self, name: &str) -> Result<&mut Self> {
        Ok(self.arg("-i").arg(identity(name)?.private))
    }

    fn identity_alt(&mut self, name: &str) -> Result<&mut Self> {
        let mut identity_arg = OsString::from("IdentityFile=");
        identity_arg.push(identity(name)?.private);
        Ok(self.arg("-o").arg(identity_arg))
    }
}

pub(crate) fn home() -> Result<PathBuf> {
    if let Some(home) = home::home_dir() {
        Ok(home)
    } else {
        bail!("Unable to determine home directory")
    }
}

pub(crate) fn app_config() -> Result<PathBuf> {
    let path = home()?.join(".pi");
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    if path.is_file() {
        bail!("app config directory appears to be a file")
    }
    Ok(path)
}

pub(crate) fn ssh_cmd(name: &str, ip: &str) -> Result<Command> {
    let mut cmd = Command::new("ssh");
    let _ = cmd.identity(name)?;
    let _ = cmd.arg(format!("pi@{ip}"));
    Ok(cmd)
}

pub(crate) fn ensure_pi_config(name: &str) -> Result<()> {
    Ok(ssh!(name => "mkdir", "-p", PI_CONFIG)?)
}

pub(crate) fn read_line() -> Result<String> {
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf)?;
    Ok(String::from(buf.trim()))
}

pub(crate) fn read_prompt(default: impl Borrow<DefaultPrompt>) -> Result<bool> {
    let response = read_line()?.to_lowercase().chars().next().expect("Should at least be a newline...");
    Ok(match default.borrow() {
        DefaultPrompt::Yes => response != 'n',
        DefaultPrompt::No => response == 'y'
    })
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn truncate(x: i32) -> u8 {
    x as u8
}

fn identity(name: &str) -> Result<Identity> {
    let identity = Identity::new(name)?;
    if !identity.exists() {
        bail!("No identity file for {name}")
    }
    Ok(identity)
}
