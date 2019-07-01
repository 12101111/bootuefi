use tempfile::TempDir;
mod config;
use config::*;
use failure::{format_err, Error};
use std::path::{Path, PathBuf};
use wait_timeout::ChildExt;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let uefi_path = get_uefi_path()?;
    let config = Config::read()?;
    let esp = make_esp(uefi_path.as_path())?;
    let is_test = is_test(uefi_path.as_path());
    let profile = config.build_profile(is_test, esp.path())?;
    let exit_code = run_qemu(is_test, profile)?;
    std::process::exit(exit_code)
}

fn get_uefi_path() -> Result<PathBuf, Error> {
    let mut args = std::env::args();
    let _ = args.next();
    let arg = args
        .next()
        .ok_or_else(|| format_err!("No input file!\nFor more information try -h"))?;
    if arg == "-h" {
        //TODO: make doc here
        print!(include_str!("../README.md"));
        std::process::exit(0);
    }
    Ok(arg.into())
}

fn is_test(uefi_path: &Path) -> bool {
    match uefi_path.parent() {
        None => false,
        Some(path) => path.ends_with("deps"),
    }
}

fn make_esp(uefi_path: &Path) -> Result<TempDir, Error> {
    let target_path = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .map_err(|e| format_err!("Can't run cargo metadata: {}", e))?
        .target_directory;
    // Temporary dir for ESP.
    let esp = tempfile::tempdir_in(target_path)
        .map_err(|e| format_err!("Unable to create temporary directory: {}", e))?;
    // Path to /EFI/BOOT
    let efi_boot_path = esp.path().join("EFI").join("BOOT");
    std::fs::create_dir_all(efi_boot_path.clone())
        .map_err(|e| format_err!("Unable to create /EFI/BOOT directory: {}", e))?;
    let bootx64_path = efi_boot_path.join("BOOTX64.EFI");
    std::fs::copy(uefi_path, bootx64_path)
        .map_err(|e| format_err!("Unable to copy EFI executable: {}", e))?;
    Ok(esp)
}

fn run_qemu(is_test:bool, profile:Profile)->Result<i32,Error>{
    let mut cmd = std::process::Command::new(profile.qemu);
    cmd.args(profile.args);
    let exit_code = if is_test {
        let mut child = cmd
            .spawn()
            .map_err(|e| format_err!("Failed to launch QEMU: {:?}\n{}", cmd, e))?;
        let timeout = std::time::Duration::from_secs(profile.test_timeout.into());
        match child
            .wait_timeout(timeout)
            .map_err(|e| format_err!("Failed to wait with timeout: {}", e))?
        {
            None => {
                child
                    .kill()
                    .map_err(|e| format_err!("Failed to kill QEMU: {}", e))?;
                child
                    .wait()
                    .map_err(|e| format_err!("Failed to wait for QEMU process: {}", e))?;
                return Err(format_err!("Timed Out"));
            }
            Some(exit_status) => match exit_status.code() {
                Some(code) if code == profile.test_success_exit_code => 0,
                other => other.unwrap_or(1),
            },
        }
    } else {
        let status = cmd
            .status()
            .map_err(|e| format_err!("Failed to execute `{:?}`: {}", cmd, e))?;
        status.code().unwrap_or(1)
    };
    Ok(exit_code)
}