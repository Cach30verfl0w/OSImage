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
    pub fn target(&self, project: &CargoProject, architecture: Architecture) -> Option<String> {
        project.target.clone().map(|value| Some(value)).unwrap_or(match self {
            ProjectKind::Kernel => Some(format!("{}-unknown-none.json", String::from(architecture))),
            ProjectKind::Bootloader => Some(format!("{}-unknown-uefi", String::from(architecture))),
            ProjectKind::SharedLibrary => None,
            ProjectKind::StaticLibrary => None,
            ProjectKind::Executable => None
        })
    }

    pub fn output_file_path(&self, project: &CargoProject, architecture: Architecture, name: &str) -> Option<String> {
        // Handle user-defined target
        if let Some(target) = project.target.as_ref() {
            let i = target.find('-').unwrap();
            return match &target[i..target.len()] {
                "unknown-uefi" => Some(format!("target/{}-unknown-uefi/debug/{}.efi", String::from(architecture), name)),
                "unknown-none" => Some(format!("target/{}-unknown-none/debug/{}.efi", String::from(architecture), name)),
                _ => None
            };
        }

        // Handle if no user-defined target is specified
        // TODO: All non-kernel and non-bootloader projects are ignored by the build system
        match self {
            ProjectKind::Kernel => Some(format!("target/{}-unknown-none/debug/{}", String::from(architecture), name)),
            ProjectKind::Bootloader => Some(format!("target/{}-unknown-uefi/debug/{}.efi", String::from(architecture), name)),
            ProjectKind::SharedLibrary => None,
            ProjectKind::StaticLibrary => None,
            ProjectKind::Executable => None
        }
    }

    pub fn image_target_file(&self, project: &CargoProject, architecture: Architecture, _project: &str) -> Option<String> {
        // TODO: All non-kernel and non-bootloader projects are ignored by the build system
        project.image_path.clone().map(|string| Some(string)).unwrap_or(match self {
            ProjectKind::Kernel => Some(String::from("EFI/BOOT/KERNEL.ELF")),
            ProjectKind::Bootloader => Some(architecture.efi_boot_file()),
            ProjectKind::SharedLibrary => None,
            ProjectKind::StaticLibrary => None,
            ProjectKind::Executable => None,
        })
    }

}

#[derive(Clone)]
pub struct CargoProject {
    pub manifest: Manifest<Value>,
    pub path: String,
    pub kind: ProjectKind,
    pub target: Option<String>,
    pub image_path: Option<String>
}

impl CargoProject {

    #[inline]
    pub fn from_manifest(path: String, manifest: Manifest<Value>) -> Self {
        // Get kind of project

        let mut target = None;
        let mut image_path = None;
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

                // Load target
                match osimage_data.get("target") {
                    None => {}
                    Some(value) => target = Some(value.as_str().unwrap().to_owned())
                }

                // Load image path
                match osimage_data.get("image_path") {
                    None => {}
                    Some(value) => image_path = Some(value.as_str().unwrap().to_owned())
                }

                // Load kind
                match osimage_data["kind"].as_str().unwrap() {
                    "kernel" => ProjectKind::Kernel,
                    "bootloader" => ProjectKind::Bootloader,
                    _ => panic!("Unable to get kind of project")
                }
            }
        };

        // Return structure
        Self {
            path,
            kind,
            target,
            image_path,
            manifest
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