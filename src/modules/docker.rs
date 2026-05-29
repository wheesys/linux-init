use crate::distro::{self, DistroFamily};
use std::process::{Command, Stdio};

pub fn install_docker() -> anyhow::Result<()> {
    let family = distro::detect().family();
    match family {
        DistroFamily::Arch => {
            distro::install_packages(&["docker"])?;
        }
        DistroFamily::Debian => {
            distro::install_packages(&["docker.io"])?;
        }
        _ => anyhow::bail!("不支持的发行版"),
    }
    
    // 安装完成后自动启动 Docker 守护进程
    start_docker_service()?;
    
    Ok(())
}

pub fn install_compose() -> anyhow::Result<()> {
    let family = distro::detect().family();
    match family {
        DistroFamily::Arch => {
            distro::install_packages(&["docker-compose"])?;
        }
        DistroFamily::Debian => {
            distro::install_packages(&["docker-compose-v2"])?;
        }
        _ => anyhow::bail!("不支持的发行版"),
    }
    Ok(())
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
