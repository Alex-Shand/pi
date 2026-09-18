use std::{fmt, process::Command};

use anyhow::{Context, Result, anyhow};
use camino::Utf8PathBuf as PathBuf;
use command_ext::CommandExt as _;
use serde::Deserialize;

static PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/nix");

pub(crate) fn build(target: impl fmt::Display) -> Result<PathBuf> {
    let path = Command::new("nix")
        .current_dir(PATH)
        .args([
            "build",
            &format!("path:.#{target}"),
            "--accept-flake-config",
            "--print-out-paths",
        ])
        .check_output()?;
    Ok(PathBuf::from(path.trim()))
}

pub(crate) struct Eval {
    argument: Arg,
    impure: bool,
}

enum Arg {
    Expr(String),
}

impl Eval {
    pub(crate) fn expr(expr: impl Into<String>) -> Self {
        Self {
            argument: Arg::Expr(expr.into()),
            impure: false,
        }
    }

    pub(crate) fn eval<T: for<'a> Deserialize<'a>>(self) -> Result<T> {
        let mut cmd = Command::new("nix");
        let _ = cmd.args(["eval", "--json"]);
        if self.impure {
            let _ = cmd.arg("--impure");
        }
        match self.argument {
            Arg::Expr(expr) => {
                let _ = cmd.arg("--expr").arg(expr);
            }
        }
        let result = cmd.check_output()?;
        serde_json::from_str(&result)
            .with_context(|| anyhow!("Failed to deserialize eval result"))
    }

    pub(crate) fn impure(mut self) -> Self {
        self.impure = true;
        self
    }
}
