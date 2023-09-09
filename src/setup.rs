use crate::{utils, push};

use std::path::PathBuf;

use argh::FromArgs;
use anyhow::{Result, bail};

/// Deploy setup code to the pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name="setup")]
pub(crate) struct Args {
    /// the pi to deploy to
    #[argh(positional)]
    name: String,
    /// the script to deploy, if not present ~/.pi/<name>.sh will be used if
    /// present
    #[argh(positional)]
    script: Option<PathBuf>
}

pub(crate) fn main(args: &Args) -> Result<()> {
    let app_config = utils::app_config()?;
    let script = args.script.as_ref().cloned()
        .unwrap_or_else(|| app_config.join(format!("{}.sh", &args.name)));
    if !script.is_file() {
        bail!("{} does not exist or isn't a file", script.display())
    }
    let target = format!("{}/pi.sh", utils::PI_CONFIG);

    utils::ensure_pi_config(&args.name)?;
    push::push(&args.name, &[script], &target)?;
    ssh!(&args.name => "chmod", "+x", &target)?;
    ssh!(&args.name => "sudo", &target, "root")?;
    ssh!(&args.name => &target, &"user".to_owned())?;

    Ok(())
}
