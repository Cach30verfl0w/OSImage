use std::fmt::{Display, Formatter};
use std::path::{absolute, Path, PathBuf};
use std::time::SystemTime;
use cargo_toml::Manifest;
use colorful::Colorful;
use glob::glob;
use log::{debug, info};
use toml::Value;
use crate::error::Error;
use crate::validate::find_manifest_and_validate;
use colorful::Color;
use crate::arch::Architecture;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum ProjectKind {
    Kernel,
    Bootloader,
    SharedLibrary,
    StaticLibrary,
    Executable
}

impl Display for ProjectKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", match self {
            ProjectKind::Kernel => "Kernel",
            ProjectKind::Bootloader => "Bootloader",
            ProjectKind::SharedLibrary => "Shared Library",
            ProjectKind::StaticLibrary => "Static Library",
            ProjectKind::Executable => "Executable"
        })
    }
}

impl ProjectKind {
    pub fn target(&self, architecture: Architecture) -> Option<String> {
        match self {
            ProjectKind::Kernel => Some(format!("{}-unknown-none.json", String::from(architecture))),
            ProjectKind::Bootloader => Some(format!("{}-unknown-uefi", String::from(architecture))),
            ProjectKind::SharedLibrary => None,
            ProjectKind::StaticLibrary => None,
            ProjectKind::Executable => None
        }
    }

    pub fn output_file_path(&self, architecture: Architecture, project: &str) -> Option<String> {
        // TODO: All non-kernel and non-bootloader projects are ignored by the build system
        match self {
            ProjectKind::Kernel => Some(format!("target/{}-unknown-none/debug/{}", String::from(architecture), project)),
            ProjectKind::Bootloader => Some(format!("target/{}-unknown-uefi/debug/{}.efi", String::from(architecture), project)),
            ProjectKind::SharedLibrary => None,
            ProjectKind::StaticLibrary => None,
            ProjectKind::Executable => None
        }
    }

    pub fn image_target_file(&self, architecture: Architecture, _project: &str) -> Option<String> {
        // TODO: All non-kernel and non-bootloader projects are ignored by the build system
        match self {
            ProjectKind::Kernel => Some(String::from("EFI/BOOT/KERNEL.ELF")),
            ProjectKind::Bootloader => Some(architecture.efi_boot_file()),
            ProjectKind::SharedLibrary => None,
            ProjectKind::StaticLibrary => None,
            ProjectKind::Executable => None,
        }
    }

}

#[derive(Clone)]
pub struct CargoProject {
    pub manifest: Manifest<Value>,
    pub path: String,
    pub kind: ProjectKind
}

impl CargoProject {

    #[inline]
    pub fn from_manifest(path: String, manifest: Manifest<Value>) -> Self {
        // Get kind of project
        let kind = match &manifest.package().metadata {
            None => {
                if Path::new(&path).join("src/main.rs").exists() {
                    ProjectKind::Executable
                } else {
                    match &manifest.lib {
                        None => ProjectKind::StaticLibrary,
                        Some(lib) => {
                            if lib.crate_type.contains(&String::from("cdylib")) {
                                ProjectKind::SharedLibrary
                            } else if lib.crate_type.contains(&String::from("dylib")) {
                                ProjectKind::SharedLibrary
                            } else {
                                ProjectKind::StaticLibrary
                            }
                        }
                    }
                }
            }
            Some(metadata) => {
                let osimage_data = &metadata["osimage"];
                match osimage_data["kind"].as_str().unwrap() {
                    "kernel" => ProjectKind::Kernel,
                    "bootloader" => ProjectKind::Bootloader,
                    _ => panic!("Unable to get kind of project")
                }
            }
        };

        // Return structure
        Self {
            manifest,
            path,
            kind
        }
    }

}

fn member_to_paths<P: AsRef<Path>>(base_path: &P, member: &str) -> Result<Vec<PathBuf>, Error> {
    let mut member_paths = Vec::new();
    for path in glob(base_path.as_ref().join(member).to_str().unwrap())? {
        let path = path?;
        member_paths.push(path);
    }
    Ok(member_paths)
}

pub fn load_from_workspace<P: AsRef<Path>>(base_path: &P, members: Vec<String>) -> Result<Vec<CargoProject>, Error> {
    let start_time = SystemTime::now();
    let mut projects = Vec::new();
    for member in members {
        let paths = member_to_paths(base_path, &member)?;

        // Load all manifests
        info!("Found {} projects in member '{}', loading manifests from all", paths.len(), member.gradient(Color::Red));
        for path in paths {
            let manifest = find_manifest_and_validate(path.clone())?;
            debug!(" - Valid Manifest: {} ({}) => {}", manifest.package().name().gradient(Color::Green),
                manifest.package().version(), absolute(&path)?.to_str().unwrap().gradient(Color::Red));
            projects.push(CargoProject::from_manifest(path.to_string_lossy().into_owned(), manifest));
        }
    }
    let duration = SystemTime::now().duration_since(start_time).unwrap();
    info!("Loaded {} projects with manifest successfully into memory in {}ms", projects.len(),
        duration.as_millis());
    Ok(projects)
}