use std::{borrow::Borrow, fs, path::PathBuf, process::Command};

use anyhow::{Result, bail};
use command_ext::CommandExt as _;

use crate::CommandExt as _;

pub(crate) const PI_CONFIG: &str = "/home/pi/.pi";

pub(crate) enum Prompt {
    Yes,
    No,
}

impl Prompt {
    pub(crate) fn is_yes(&self) -> bool {
        matches!(self, Prompt::Yes)
    }

    pub(crate) fn is_no(&self) -> bool {
        matches!(self, Prompt::No)
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

pub(crate) fn ensure_pi_config(name: &str) -> Result<()> {
    Command::new("mkdir")
        .arg("-p")
        .arg(PI_CONFIG)
        .run_on_pi(name)?
        .check_status()?;
    Ok(())
}

pub(crate) fn read_line() -> Result<String> {
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf)?;
    Ok(String::from(buf.trim()))
}

pub(crate) fn read_prompt(default: impl Borrow<Prompt>) -> Result<Prompt> {
    let response = read_line()?
        .to_lowercase()
        .chars()
        .next()
        .expect("Should at least be a newline...");
    let is_yes = match default.borrow() {
        Prompt::Yes => response != 'n',
        Prompt::No => response == 'y',
    };
    Ok(if is_yes { Prompt::Yes } else { Prompt::No })
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn truncate(x: i32) -> u8 {
    x as u8
}
