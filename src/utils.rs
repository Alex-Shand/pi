use std::{borrow::Borrow, sync::LazyLock};

use anyhow::{Result, anyhow};
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};

pub(crate) enum Prompt {
    Yes,
    No,
}

impl Prompt {
    pub(crate) fn is_no(&self) -> bool {
        matches!(self, Prompt::No)
    }
}

pub(crate) fn home() -> Result<&'static Path> {
    static HOME: LazyLock<Result<PathBuf>> = LazyLock::new(|| {
        let home = home::home_dir()
            .ok_or_else(|| anyhow!("Cannot determine home directory"))?;
        Ok(PathBuf::try_from(home)?)
    });
    HOME.as_deref().map_err(|e| anyhow!(e))
}

pub(crate) fn ssh_dir() -> Result<&'static Path> {
    static SSH: LazyLock<Result<PathBuf>> =
        LazyLock::new(|| Ok(home()?.join(".ssh")));
    SSH.as_deref().map_err(|e| anyhow!(e))
}

fn read_line() -> Result<String> {
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
