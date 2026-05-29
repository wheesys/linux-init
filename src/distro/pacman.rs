use std::process::Command;

pub fn install(packages: &[&str]) -> anyhow::Result<()> {
    if packages.is_empty() {
        return Ok(());
    }
    let status = Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg("--needed")
        .args(packages)
        .status()?;

    if !status.success() {
        anyhow::bail!("pacman 安装失败");
    }
    Ok(())
}

pub fn is_installed(package: &str) -> bool {
    Command::new("pacman")
        .arg("-Qi")
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
        "btop" => Some("btop"),
        "neovim" => Some("neovim"),
        "tmux" => Some("tmux"),
        "jq" => Some("jq"),
        "ripgrep" => Some("ripgrep"),
        "fd" => Some("fd"),
        "bat" => Some("bat"),
        "eza" => Some("eza"),
        "trash-cli" => Some("trash-cli"),
        "procs" => Some("procs"),
        "dust" => Some("du-dust"),
        "duf" => Some("duf"),
        "direnv" => Some("direnv"),
        "zsh" => Some("zsh"),
        "docker" => Some("docker"),
        "docker-compose" => Some("docker-compose"),
        "noto-fonts-cjk" => Some("noto-fonts-cjk"),
        "fcitx5" => Some("fcitx5"),
        "fcitx5-chinese-addons" => Some("fcitx5-chinese-addons"),
        "fcitx5-configtool" => Some("fcitx5-configtool"),
        "vim" => Some("vim"),
        "openssh" => Some("openssh"),
        _ => None,
    }
}
