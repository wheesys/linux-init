mod detect;
mod pacman;
mod apt;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Distro {
    Arch,
    CachyOS,
    Manjaro,
    Ubuntu(String),
    Debian(String),
    Unknown(String),
}

impl fmt::Display for Distro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Distro::Arch => write!(f, "Arch Linux"),
            Distro::CachyOS => write!(f, "CachyOS"),
            Distro::Manjaro => write!(f, "Manjaro"),
            Distro::Ubuntu(v) => write!(f, "Ubuntu {}", v),
            Distro::Debian(v) => write!(f, "Debian {}", v),
            Distro::Unknown(s) => write!(f, "Unknown ({})", s),
        }
    }
}

impl Distro {
    pub fn family(&self) -> DistroFamily {
        match self {
            Distro::Arch | Distro::CachyOS | Distro::Manjaro => DistroFamily::Arch,
            Distro::Ubuntu(_) | Distro::Debian(_) => DistroFamily::Debian,
            Distro::Unknown(_) => DistroFamily::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistroFamily {
    Arch,
    Debian,
    Unknown,
}

pub fn detect() -> Distro {
    detect::detect_distro()
}

pub fn install_packages(packages: &[&str]) -> anyhow::Result<()> {
    let distro = detect();
    match distro.family() {
        DistroFamily::Arch => pacman::install(packages),
        DistroFamily::Debian => apt::install(packages),
        DistroFamily::Unknown => anyhow::bail!("不支持的发行版: {}", distro),
    }
}

pub fn uninstall_packages(packages: &[&str]) -> anyhow::Result<()> {
    let distro = detect();
    match distro.family() {
        DistroFamily::Arch => pacman::uninstall(packages),
        DistroFamily::Debian => apt::uninstall(packages),
        DistroFamily::Unknown => anyhow::bail!("不支持的发行版: {}", distro),
    }
}

pub fn is_tool_installed(tool: &str) -> bool {
    package_name(tool)
        .map(|pkg| is_package_installed(pkg))
        .unwrap_or(false)
}

pub fn is_package_installed(package: &str) -> bool {
    let distro = detect();
    match distro.family() {
        DistroFamily::Arch => pacman::is_installed(package),
        DistroFamily::Debian => apt::is_installed(package),
        DistroFamily::Unknown => false,
    }
}

pub fn package_name(tool: &str) -> Option<&'static str> {
    let distro = detect();
    match distro.family() {
        DistroFamily::Arch => pacman::package_name(tool),
        DistroFamily::Debian => apt::package_name(tool),
        DistroFamily::Unknown => None,
    }
}
