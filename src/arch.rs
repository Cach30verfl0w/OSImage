use clap::ValueEnum;
use log::error;
use std::{
    env,
    fmt::{
        Display,
        Formatter,
    },
    process::exit,
};

#[derive(ValueEnum, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub(crate) enum Architecture {
    X86_64,
    X86,
    ARM,
    ARM64,
    RISCV32,
    RISCV64
}

impl<'a> From<&'a Architecture> for String {
    #[inline]
    fn from(value: &'a Architecture) -> Self {
        String::from(*value)
    }
}

impl From<Architecture> for String {
    fn from(value: Architecture) -> Self {
        match value {
            Architecture::X86_64 => "x86_64",
            Architecture::X86 => "x86",
            Architecture::ARM => "arm",
            Architecture::ARM64 => "aarch64",
            Architecture::RISCV32 => "riscv32",
            Architecture::RISCV64 => "riscv64"
        }.to_string()
    }
}

impl Display for Architecture {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", String::from(self).replace("_", "-"))
    }
}

impl Architecture {
    pub(crate) fn system() -> Architecture {
        match env::consts::ARCH {
            "x86" => Self::X86,
            "x86_64" => Self::X86_64,
            "arm" => Self::ARM,
            "aarch64" => Self::ARM64,
            "riscv32" => Self::RISCV32,
            "riscv64" => Self::RISCV64,
            arch => {
                error!("Unable to get system architecture => Unsupported architecture {}", arch);
                exit(-1);
            }
        }
    }

    #[inline]
    pub(crate) fn is64bit(&self) -> bool {
        match self {
            Architecture::X86_64 | Architecture::ARM64 | Architecture::RISCV64 => true,
            Architecture::X86 | Architecture::ARM | Architecture::RISCV32 => false,
        }
    }

    pub(crate) fn efi_boot_file(&self) -> String {
        format!("EFI/BOOT/{}", match self {
            Architecture::X86_64 => "BOOTX64.EFI",
            Architecture::X86 => "BOOTIA32.EFI",
            Architecture::ARM => "BOOTARM.EFI",
            Architecture::ARM64 => "BOOTAA64.EFI",
            Architecture::RISCV32 => "BOOTRISCV32.EFI",
            Architecture::RISCV64 => "BOOTRISCV64.EFI"
        })
    }
}