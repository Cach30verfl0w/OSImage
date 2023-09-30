use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use log::info;
use crate::Arguments;
use crate::error::Error;
use crate::project::CargoProject;

// https://stackoverflow.com/questions/37498864/finding-executable-in-path-with-rust/37499032#37499032
fn find_in_path<P>(name: P) -> Option<PathBuf> where P: AsRef<Path> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join(&name);
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    })
}

pub(crate) fn run_build_on(project: CargoProject, args: Arguments) -> Result<(), Error> {
    Err(Error::InvalidCargoFile("".to_string()))
}

pub(crate) fn build_image(args: &Arguments, projects: Vec<CargoProject>, image_file: &String, iso_file: &String) -> Result<(), Error> {
    info!("Build all in-memory loaded Rust projects");
    let cargo_path = find_in_path("cargo").ok_or(Error::ExecutableNotFound(String::from("cargo")))?;

    for project in projects.clone() {
        let project_name = project.manifest.package().name();
        info!("Run build task on {} over `cargo build`", project_name);

        // Execute `cargo build`
        let mut command = Command::new(&cargo_path);
        command.arg("build")
            .arg("--package")
            .arg(project_name);
        command.current_dir(project.path);

        if let Some(target) = project.kind.target(args.target_arch) {
            command.arg("--target").arg(target);
        }

        // Validate exit code
        let exit_status = command.status()?;
        if !exit_status.success() {
            return Err(Error::BuildFailed(String::from(project_name), exit_status.code().unwrap()));
        }
    }
    Ok(())
}