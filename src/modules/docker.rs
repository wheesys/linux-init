use crate::distro::{self, DistroFamily};
use crate::utils;
use std::process::{Command, Stdio};

/// docker 安装
/// - Arch 系: pacman -S docker
/// - Debian 系: Docker 官方安装脚本 (get.docker.com)，自动包含 compose 插件
pub fn install_docker() -> anyhow::Result<()> {
    let family = distro::detect().family();
    match family {
        DistroFamily::Arch => {
            distro::install_packages(&["docker"])?;
        }
        DistroFamily::Debian => {
            install_docker_via_official_script()?;
        }
        _ => anyhow::bail!("不支持的发行版"),
    }

    // 安装完成后自动启动 Docker 守护进程
    start_docker_service()?;

    Ok(())
}

/// 使用 Docker 官方脚本安装（仅 Debian 系）
fn install_docker_via_official_script() -> anyhow::Result<()> {
    utils::ensure_command("curl")?;
    let tmp_script = "/tmp/linux-init-get-docker.sh";
    let status = Command::new("curl")
        .args(["-fsSL", "https://get.docker.com", "-o", tmp_script])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        anyhow::bail!("Docker 官方脚本下载失败，请检查网络连接");
    }
    let status = Command::new("sudo")
        .args(["sh", tmp_script])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    let _ = std::fs::remove_file(tmp_script);
    if !status.success() {
        anyhow::bail!("Docker 官方脚本执行失败");
    }
    Ok(())
}

/// docker-compose 安装
/// - Arch 系: pacman -S docker-compose
/// - Debian 系: 官方脚本已自带 compose 插件 (docker compose)，检查后决定是否装独立版
pub fn install_compose() -> anyhow::Result<()> {
    let family = distro::detect().family();
    match family {
        DistroFamily::Arch => {
            distro::install_packages(&["docker-compose"])?;
        }
        DistroFamily::Debian => {
            // 官方脚本安装后 compose 插件已可用
            if is_compose_plugin_available() {
                return Ok(());
            }
            // 兜底：apt 安装独立版本
            distro::install_packages(&["docker-compose-v2"])?;
        }
        _ => anyhow::bail!("不支持的发行版"),
    }
    Ok(())
}

fn is_compose_plugin_available() -> bool {
    Command::new("docker")
        .args(["compose", "version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn add_user_to_docker_group() -> anyhow::Result<()> {
    let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
    let status = Command::new("sudo")
        .args(["usermod", "-aG", "docker", &user])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        anyhow::bail!("usermod 命令失败");
    }
    Ok(())
}

pub fn start_docker_service() -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .args(["systemctl", "enable", "--now", "docker"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        anyhow::bail!("systemctl 命令失败");
    }
    Ok(())
}

pub fn is_docker_running() -> bool {
    Command::new("systemctl")
        .arg("is-active")
        .arg("--quiet")
        .arg("docker")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn is_user_in_docker_group() -> bool {
    let user = std::env::var("USER").unwrap_or_default();
    Command::new("groups")
        .arg(&user)
        .output()
        .map(|o| {
            let groups = String::from_utf8_lossy(&o.stdout);
            groups.contains("docker")
        })
        .unwrap_or(false)
}

pub fn clear_docker() -> anyhow::Result<()> {
    let _ = std::process::Command::new("sudo").args(["systemctl", "stop", "docker"]).status();
    let _ = std::process::Command::new("sudo").args(["gpasswd", "-d", &std::env::var("SUDO_USER").unwrap_or_default(), "docker"]).status();
    crate::distro::uninstall_packages(&["docker", "docker-compose"])?;
    Ok(())
}
