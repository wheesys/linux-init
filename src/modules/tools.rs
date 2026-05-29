use crate::distro;
use crate::utils::get_real_home;
use std::fs;
use std::io::Write;

pub fn install_tools(selected: &[&str]) -> anyhow::Result<()> {
    let mut packages: Vec<&str> = Vec::new();

    for tool in selected {
        if let Some(pkg) = distro::package_name(tool) {
            if !distro::is_package_installed(pkg) {
                packages.push(pkg);
            }
        }
    }

    if packages.is_empty() {
        return Ok(());
    }

    distro::install_packages(&packages)?;
    Ok(())
}

pub fn get_tool_status(tool: &str) -> bool {
    if let Some(pkg) = distro::package_name(tool) {
        distro::is_package_installed(pkg)
    } else {
        false
    }
}

/// 配置命令别名
pub fn configure_aliases(installed_tools: &[&str]) -> anyhow::Result<()> {
    let home = get_real_home()?;
    
    // 构建 alias 配置
    let mut aliases = Vec::new();
    
    // 检查哪些工具已安装并添加对应的 alias
    if installed_tools.contains(&"trash-cli") {
        aliases.push(r#"alias rm='trash '"#);
    }
    if installed_tools.contains(&"procs") {
        aliases.push(r#"alias ps='procs '"#);
    }
    if installed_tools.contains(&"dust") {
        aliases.push(r#"alias du='dust '"#);
    }
    if installed_tools.contains(&"duf") {
        aliases.push(r#"alias df='duf '"#);
    }
    if installed_tools.contains(&"eza") {
        aliases.push(r#"alias la='eza -lah'"#);
    }
    
    if aliases.is_empty() {
        return Ok(());
    }
    
    // 配置 bash
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        append_aliases_to_file(&bashrc_path, &aliases, "bash")?;
    }
    
    // 配置 zsh
    let zshrc_path = home.join(".zshrc");
    if zshrc_path.exists() {
        append_aliases_to_file(&zshrc_path, &aliases, "zsh")?;
    }
    
    Ok(())
}

fn append_aliases_to_file(path: &std::path::Path, aliases: &[&str], shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;
    
    // 检查是否已经配置过
    let marker = format!("# Linux Init - {} aliases", shell);
    if content.contains(&marker) {
        return Ok(());
    }
    
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)?;
    
    writeln!(file, "\n{}", marker)?;
    for alias in aliases {
        writeln!(file, "{}", alias)?;
    }
    
    Ok(())
}

/// 配置 direnv hook
pub fn configure_direnv_hook() -> anyhow::Result<()> {
    let home = get_real_home()?;
    
    // 检查 direnv 是否已安装
    if !distro::is_package_installed(distro::package_name("direnv").unwrap_or("direnv")) {
        return Ok(());
    }
    
    // 配置 bash
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        append_direnv_hook_to_file(&bashrc_path, "bash")?;
    }
    
    // 配置 zsh
    let zshrc_path = home.join(".zshrc");
    if zshrc_path.exists() {
        append_direnv_hook_to_file(&zshrc_path, "zsh")?;
    }
    
    Ok(())
}

fn append_direnv_hook_to_file(path: &std::path::Path, shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;
    
    // 检查是否已经配置过
    let marker = format!("# Linux Init - direnv hook for {}", shell);
    if content.contains(&marker) {
        return Ok(());
    }
    
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(path)?;
    
    writeln!(file, "\n{}", marker)?;
    writeln!(file, "eval \"$(direnv hook {})\"", shell)?;
    
    Ok(())
}
