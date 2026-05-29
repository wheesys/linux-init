use std::process::Command;

pub fn install(packages: &[&str]) -> anyhow::Result<()> {
    if packages.is_empty() {
        return Ok(());
    }

    let update_status = Command::new("sudo")
        .arg("apt")
        .arg("update")
        .status()?;

    if !update_status.success() {
        anyhow::bail!("apt update 失败");
    }

    let status = Command::new("sudo")
        .arg("apt")
        .arg("install")
        .arg("-y")
        .args(packages)
        .status()?;

    if !status.success() {
        anyhow::bail!("apt 安装失败");
    }
    Ok(())
}

pub fn is_installed(package: &str) -> bool {
    Command::new("dpkg")
        .arg("-s")
        .arg(package)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn package_name(tool: &str) -> Option<&'static str> {
    match tool {
        "git" => Some("git"),
        "curl" => Some("curl"),
        "wget" => Some("wget"),
        "htop" => Some("htop"),
        "neovim" => Some("neovim"),
        "tmux" => Some("tmux"),
        "jq" => Some("jq"),
        "ripgrep" => Some("ripgrep"),
        "fd" => Some("fd-find"),
        "bat" => Some("bat"),
        "eza" => Some("eza"),
        "zsh" => Some("zsh"),
        "docker" => Some("docker.io"),
        "docker-compose" => Some("docker-compose-v2"),
        "noto-fonts-cjk" => Some("fonts-noto-cjk"),
        "fcitx5" => Some("fcitx5"),
        "fcitx5-chinese-addons" => Some("fcitx5-chinese-addons"),
        "fcitx5-configtool" => Some("fcitx5-configtool"),
        "vim" => Some("vim"),
        "openssh-server" => Some("openssh-server"),
        _ => None,
    }
}
