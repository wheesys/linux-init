use crate::distro::{self, DistroFamily};
use std::process::{Command, Stdio};
use std::fs;

pub fn is_installed() -> bool {
    match distro::detect().family() {
        DistroFamily::Arch => distro::is_package_installed("openssh"),
        DistroFamily::Debian => distro::is_package_installed("openssh-server"),
        _ => false,
    }
}

pub fn install() -> anyhow::Result<()> {
    match distro::detect().family() {
        DistroFamily::Arch => distro::install_packages(&["openssh"]),
        DistroFamily::Debian => distro::install_packages(&["openssh-server"]),
        _ => anyhow::bail!("不支持的发行版"),
    }
}

pub fn is_root_login_disabled() -> bool {
    if let Ok(content) = fs::read_to_string("/etc/ssh/sshd_config") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "PermitRootLogin no" {
                return true;
            }
        }
    }
    false
}

pub fn disable_root_login() -> anyhow::Result<()> {
    let config_path = "/etc/ssh/sshd_config";
    let content = fs::read_to_string(config_path)?;

    let mut new_content = String::new();
    let mut found = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("PermitRootLogin") || trimmed.starts_with("#PermitRootLogin") {
            new_content.push_str("PermitRootLogin no\n");
            found = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    if !found {
        new_content.push_str("\nPermitRootLogin no\n");
    }

    // Write via sudo tee
    let mut child = Command::new("sudo")
        .args(["tee", config_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        stdin.write_all(new_content.as_bytes())?;
    }
    drop(child.stdin.take());
    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("写入 sshd_config 失败");
    }

    Ok(())
}

pub fn clear_ssh_server() -> anyhow::Result<()> {
    let _ = std::process::Command::new("sudo").args(["systemctl", "stop", "sshd"]).status();
    crate::distro::uninstall_packages(&["openssh-server"])?;
    Ok(())
}

pub fn start_service() -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .args(["systemctl", "enable", "--now", "sshd"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        anyhow::bail!("systemctl 命令失败");
    }
    Ok(())
}

pub fn is_running() -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", "sshd"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
