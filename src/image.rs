use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
};

use ::nix::unistd::Uid;
use anyhow::{Context, Result, anyhow, bail, ensure};
use argh::FromArgs;
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};
use indicatif::{ProgressBar, ProgressStyle};
use zstd::stream::Decoder;

use crate::{
    nix,
    utils::{self, Prompt},
};

/// Build a custom PI image and write it to an attached SDCard
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "image")]
pub(crate) struct Args {
    /// the name of the PI (must be defined in flake.nix)
    #[argh(positional)]
    name: String,
}

pub(crate) fn main(Args { name }: Args) -> Result<()> {
    if !Uid::effective().is_root() {
        bail!("The image subcommand requires root permissions")
    }

    let disk = find_disk().ok_or_else(|| {
        anyhow!("Unable to determine which device to write to. Is the SDCard plugged in?")
    })?;

    prompt!(
        "WARNING: This command will overwrite the SDCard in {disk}. Continue? [y/N]: "
    );
    if utils::read_prompt(Prompt::No)?.is_no() {
        bail!("Aborted image operation")
    }

    println!("Building image...");
    let system: String =
        nix::Eval::expr("builtins.currentSystem").impure().eval()?;
    let image = find_file_under(
        &nix::build(format_args!("packages.{system}.{name}"))?.join("sd-image"),
    )?;

    write_with_progress(&image, &disk, "Writing image")?;
    Ok(())
}

fn find_disk() -> Option<PathBuf> {
    [PathBuf::from("/dev/sda"), PathBuf::from("/dev/sdb")]
        .into_iter()
        .find(|disk| {
            if !disk.exists() {
                return false;
            }
            File::open(disk).is_ok()
        })
}

fn find_file_under(path: &Path) -> Result<PathBuf> {
    let err = || anyhow!("Failed to read {path}");
    let files = fs::read_dir(path)
        .with_context(err)?
        .collect::<Result<Vec<_>, _>>()
        .with_context(err)?;
    let count = files.len();
    ensure!(count == 1, "Expected one file under {path}, got {count}");
    let result = files.into_iter().next().unwrap().path();
    Ok(Path::from_path(&result)
        .with_context(|| anyhow!("`{}` is not valid utf8", result.display()))?
        .to_owned())
}

fn write_with_progress(
    src: &Path,
    dst: &Path,
    msg: &'static str,
) -> Result<()> {
    let err = || anyhow!("Failed to read {src}");
    let size = fs::metadata(src).with_context(err)?.len();
    let bar = ProgressBar::new(size).with_message(msg).with_style(
            ProgressStyle::with_template("{msg} [{elapsed_precise}] {wide_bar} {binary_bytes}/{binary_total_bytes} ({bytes_per_sec})").unwrap(),
        );

    let mut reader = Decoder::with_buffer(
        bar.wrap_read(BufReader::new(File::open(src).with_context(err)?)),
    )
    .with_context(err)?;
    let mut writer = BufWriter::new(
        File::create(dst)
            .with_context(|| anyhow!("Failed to open {dst} for writing"))?,
    );

    let err = || anyhow!("Error occured while writing {src} to {dst}");
    let _ = io::copy(&mut reader, &mut writer).with_context(err)?;
    writer.flush().with_context(err)?;
    bar.finish();
    Ok(())
}
