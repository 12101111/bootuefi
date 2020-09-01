use tempfile::TempDir;
mod config;
use anyhow::{anyhow, Context, Result};
use config::*;
use std::path::{Path, PathBuf};
use std::process::exit;
use wait_timeout::ChildExt;

fn main() -> Result<()> {
    let uefi_path = get_uefi_path()?;
    let config = Config::read()?;
    let esp = make_esp(uefi_path.as_path())?;
    let is_test = is_test(uefi_path.as_path());
    let profile = config.build_profile(is_test, esp.path())?;
    let code = run_qemu(is_test, profile)?;
    exit(code)
}

fn get_uefi_path() -> Result<PathBuf> {
    let mut args = std::env::args();
    let _ = args.next();
    let arg = args
        .next()
        .ok_or_else(|| anyhow!("No input file!\nFor more information try -h"))?;
    if arg == "-h" {
        println!("See document in https://github.com/12101111/bootuefi");
        exit(0);
    }
    Ok(arg.into())
}

fn is_test(uefi_path: &Path) -> bool {
    match uefi_path.parent() {
        None => false,
        Some(path) => path.ends_with("deps"),
    }
}

fn make_esp(uefi_path: &Path) -> Result<TempDir> {
    let target_path = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Can't run cargo metadata")?
        .target_directory;
    // Temporary dir for ESP.
    let esp = tempfile::tempdir_in(target_path).context("Unable to create temporary directory")?;
    // Path to /EFI/BOOT
    let efi_boot_path = esp.path().join("EFI").join("BOOT");
    std::fs::create_dir_all(efi_boot_path.clone())
        .context("Unable to create /EFI/BOOT directory")?;
    let bootx64_path = efi_boot_path.join("BOOTX64.EFI");
    std::fs::copy(uefi_path, bootx64_path).context("Unable to copy EFI executable")?;
    Ok(esp)
}

fn run_qemu(is_test: bool, profile: Profile) -> Result<i32> {
    println!("Runing: `{} {}`", profile.qemu, profile.args.join(" "));
    let mut cmd = std::process::Command::new(profile.qemu);
    cmd.args(profile.args);
    let exit_code = if is_test {
        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to launch QEMU: {:?}", cmd))?;
        let timeout = std::time::Duration::from_secs(profile.test_timeout.into());
        match child
            .wait_timeout(timeout)
            .context("Failed to wait with timeout")?
        {
            None => {
                child.kill().context("Failed to kill QEMU")?;
                child.wait().context("Failed to wait for QEMU process")?;
                return Err(anyhow!("Timed Out"));
            }
            Some(exit_status) => match exit_status.code() {
                Some(code) if code == profile.test_success_exit_code => 0,
                other => other.unwrap_or(1),
            },
        }
    } else {
        let status = cmd
            .status()
            .with_context(|| format!("Failed to execute `{:?}`", cmd))?;
        status.code().unwrap_or(1)
    };
    Ok(exit_code)
}
