use std::path::Path;
use std::process::Command;
use log::debug;
use crate::Arguments;
use crate::error::Error;
use crate::utils::find_in_path;

pub(crate) fn run_qemu(args: &Arguments, iso_file: &String, debugging: bool, debug_port: u16) -> Result<(), Error> {
    let qemu_path = find_in_path(format!("qemu-system-{}", String::from(args.target_arch)))
        .ok_or(Error::ExecutableNotFound(String::from("cargo")))?;

    let mut command = Command::new(qemu_path);
    command.arg("-bios").arg("OVMF.fd");
    command.arg("-cdrom").arg(Path::new(&args.workspace_path).join(iso_file).to_str().unwrap());
    command.arg("-m").arg("512");

    if debugging {
        debug!("QEMU Debugging enabled, set debug arguments");
        command.arg("-gdb").arg(format!("tcp::{}", debug_port));
        command.arg("-S");
    }

    let exit_status = command.status()?;
    if !exit_status.success() {
        return Err(Error::ProcessFailed(String::from("QEMU"), exit_status.code().unwrap()));
    }
    Ok(())
}