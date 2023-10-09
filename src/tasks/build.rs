use std::fs::{create_dir, remove_dir, remove_file};
use std::path::Path;
use std::process::Command;
use colorful::{Color, Colorful};
use log::{debug, info};
use crate::Arguments;
use crate::error::Error;
use crate::image::Image;
use crate::project::CargoProject;
use crate::utils::find_in_path;

pub(crate) fn build_image(args: &Arguments, projects: Vec<CargoProject>, image_file: &String,
                          iso_file: &String, block_size: &u16, block_count: &u32) -> Result<(), Error> {
    info!("Build all in-memory loaded Rust projects");
    let cargo_path = find_in_path("cargo").ok_or(Error::ExecutableNotFound(String::from("cargo")))?;

    // Generate image
    let image_path = Path::new(&args.workspace_path).join(".image");
    if !image_path.exists() {
        debug!("Directory {} not found! Creating it...", image_path.to_str().unwrap());
        create_dir(&image_path)?;
    }

    let image_path = image_path.join(image_file);
    let image = Image::new(Path::new(&args.workspace_path).join(&image_path).to_str()
                               .unwrap(), *block_size, *block_count)?;

    for project in projects.clone() {
        let project_name = project.manifest.package().name();

        // Execute `cargo build`
        let mut command = Command::new(&cargo_path);
        command.arg("build")
            .arg("--package")
            .arg(project_name);
        command.current_dir(&args.workspace_path);

        if let Some(target) = project.kind.target(args.target_arch) {
            command
                .arg("--target").arg(&target)
                .arg("-Zbuild-std=core,alloc,compiler_builtins")
                .arg("-Zbuild-std-features=compiler-builtins-mem");
            info!("Run build task on {} ({}) with `{}` ({})", project_name.color(Color::Green),
            project.kind.to_string().color(Color::Orange3), "cargo build".color(Color::Red), target.color(Color::Green));
        } else {
            info!("Run build task on {} ({}) with `{}`", project_name.color(Color::Green),
            project.kind.to_string().color(Color::Orange3), "cargo build".color(Color::Red));
        }

        // Validate exit code
        let exit_status = command.status()?;
        if !exit_status.success() {
            return Err(Error::BuildFailed(String::from(project_name), exit_status.code().unwrap()));
        }

        // Move file into image
        if let Some(output_path) = project.kind.output_file_path(args.target_arch, project_name) {
            let image_path = project.kind.image_target_file(args.target_arch, project_name).unwrap();
            image.copy_into(Path::new(&args.workspace_path).join(output_path), image_path)?;
        }
    }

    // Create ISO file
    info!("Generate ISO file");
    let xorriso_path = find_in_path("xorriso").ok_or(Error::ExecutableNotFound(String::from("cargo")))?;
    let mut command = Command::new(xorriso_path);
    command
        .arg("-as").arg("mkisofs")
        .arg("-V").arg("EFI_ISO_BOOT")
        .arg("-e").arg(image_file)
        .arg("-no-emul-boot")
        .arg("-o").arg(iso_file)
        .arg(".image/");
    command.current_dir(&args.workspace_path);

    let exit_status = command.status()?;
    if !exit_status.success() {
        return Err(Error::ProcessFailed(String::from("xorriso"), exit_status.code().unwrap()));
    }

    // Cleanup
    remove_file(&image_path)?;
    remove_dir(image_path.parent().unwrap())?;
    Ok(())
}