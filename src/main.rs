#![feature(absolute_path)]

use std::fmt::{Display, Formatter};
use std::path;
use std::process::exit;
use clap::{Parser, Subcommand, ValueEnum};
use colorful::{Color, Colorful};
use log::{error, info, Level};
use crate::arch::Architecture;
use crate::error::{EXIT_BUILD_ERROR, EXIT_INVALID_WORKSPACE, EXIT_QEMU_ERROR};
use crate::project::{CargoProject, load_from_workspace};
use crate::tasks::build::build_image;
use crate::tasks::qemu::run_qemu;
use crate::validate::find_manifest_and_validate;

pub(crate) mod validate;
pub(crate) mod error;
pub(crate) mod project;
pub(crate) mod arch;
pub(crate) mod tasks;
pub(crate) mod image;
pub(crate) mod utils;

#[derive(ValueEnum, Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum ImageType {
    UEFI,
    BIOS
}

impl Display for ImageType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

#[derive(Parser, Clone)]
#[command(author, version)]
pub(crate) struct Arguments {
    /// The path to the Rust workspace or project
    #[arg(long, short, default_value = "./")]
    workspace_path: String,

    /// The type of image that is wanted to generate (Currently only UEFI is supported)
    #[arg(long, short, default_value = "uefi")]
    image_type: ImageType,

    /// The target architecture of the Operating System
    #[arg(long, short, default_value_t = Architecture::system())]
    target_arch: Architecture,

    /// Set the log level
    #[arg(long, short)]
    level: Option<Level>,

    #[command(subcommand)]
    command: SubCommand
}

#[derive(Subcommand, Clone)]
enum SubCommand {
    /// Build the image file with this Rust project or workspace
    BuildImage {
        /// The name of the image file that should be built by this tool
        #[arg(long, default_value = "image.img")]
        image_file: String,

        /// The name of the ISO file that should be built by this tool
        #[arg(long, default_value = "image.iso")]
        iso_file: String,

        #[arg(long, default_value_t = 512)]
        block_size: u16,

        #[arg(long, default_value_t = 93750)]
        block_count: u32
    },

    /// Run the built image in QEMU
    RunQEMU {
        /// The name of the ISO file that should be built by this tool
        #[arg(long, default_value = "image.iso")]
        iso_file: String,

        /// Should QEMU be started with debugger enabled (QEMU will wait for the connection before
        /// running the image code)
        #[arg(long, short, default_value_t = false)]
        debugging: bool,

        /// If debugging is enabled, the port of the GDB server for qemu
        #[arg(long, default_value_t = 1337)]
        debug_port: u16,

        /// If this is enabled, QEMU will print register and exception information to stdout
        #[arg(long, short, default_value_t = false)]
        exception_info: bool
    }
}

fn main() {
    simple_logger::init_with_env().unwrap();

    // Print header
    info!("{}", "        ____  _____    ____                             ".gradient(Color::Red));
    info!("{}", "       / __ \\/ ___/   /  _/___ ___  ____ _____ ____    ".gradient(Color::Red));
    info!("{}", "      / / / /\\__ \\    / // __ `__ \\/ __ `/ __ `/ _ \\".gradient(Color::Red));
    info!("{}", "     / /_/ /___/ /  _/ // / / / / / /_/ / /_/ /  __/    ".gradient(Color::Red));
    info!("{}", "     \\____//____/  /___/_/ /_/ /_/\\__,_/\\__, /\\___/ ".gradient(Color::Red));
    info!("{}", "                                       /____/           ".gradient(Color::Red));
    info!("        {} Creation Tool by {}", "OS Image".gradient(Color::Red), "Cach30verfl0w"
        .gradient(Color::Green));
    let args = Arguments::parse();
    log::set_max_level(args.level.unwrap_or(Level::Info).to_level_filter());
    info!("Targeting {} architecture ({})", args.target_arch, if args.target_arch.is64bit() { "64-bit" }
        else { "32-bit" });

    // Locate and read manifest file from Workspace
    let manifest = match find_manifest_and_validate(&args.workspace_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            error!("Unable to find and parse manifest of specified workspace => {}", error);
            exit(EXIT_INVALID_WORKSPACE);
        }
    };

    let is_workspace = manifest.workspace.is_some();
    info!("Located {} manifest file in directory {}",
        if is_workspace { "Workspace" } else { "project" },
        path::absolute(&args.workspace_path).unwrap()
        .to_str().unwrap_or(&args.workspace_path).gradient(Color::Blue));

    // Convert workspace in projects, if project is workspace. If project is not a workspace, insert
    // the only existing project into the list.
    let mut projects = Vec::new();
    if is_workspace {
        projects.append(&mut match load_from_workspace(&args.workspace_path, manifest.workspace.unwrap().members) {
            Ok(values) => values,
            Err(error) => {
                error!("Unable to find and parse manifest of workspace => {}", error);
                exit(EXIT_INVALID_WORKSPACE);
            }
        });
    } else {
        projects.push(CargoProject::from_manifest(args.workspace_path.clone(), manifest));
    }

    // Switch to selected command
    match &args.command {
        SubCommand::BuildImage { iso_file, image_file, block_size, block_count  } => {
            match build_image(&args, projects, image_file, iso_file, block_size, block_count) {
                Ok(()) => {}
                Err(error) => {
                    error!("Unable to build Operating System image => {}", error);
                    exit(EXIT_BUILD_ERROR);
                }
            }
        },
        SubCommand::RunQEMU { iso_file, debugging, debug_port, exception_info } => {
            match run_qemu(&args, iso_file, *debugging, *debug_port, *exception_info) {
                Ok(()) => {}
                Err(error) => {
                    error!("Unable to run image in QEMU => {}", error);
                    exit(EXIT_QEMU_ERROR);
                }
            }
        }
    }
}
