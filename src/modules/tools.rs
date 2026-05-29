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

/// 确定 shell 配置目标：优先使用系统当前默认 shell（$SHELL），
/// 若无法识别则 fallback 到用户本次会话选定的 shell。
/// 返回 "bash" 或 "zsh"，两者都不确定时返回 None。
pub fn resolve_shell_for_config(selected_shell: Option<&str>) -> Option<String> {
    let current = std::env::var("SHELL").unwrap_or_default();
    if current.contains("zsh") {
        return Some("zsh".to_string());
    }
    if current.contains("bash") {
        return Some("bash".to_string());
    }
    selected_shell.map(|s| s.to_string())
}

/// 配置命令别名，只写入目标 shell 的配置文件
pub fn configure_aliases(installed_tools: &[&str], selected_shell: Option<&str>) -> anyhow::Result<()> {
    let home = get_real_home()?;

    let mut aliases = Vec::new();
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

    let target = resolve_shell_for_config(selected_shell);
    match target.as_deref() {
        Some("zsh") => {
            let path = home.join(".zshrc");
            if path.exists() {
                append_aliases_to_file(&path, &aliases, "zsh")?;
            }
        }
        _ => {
            let path = home.join(".bashrc");
            if path.exists() {
                append_aliases_to_file(&path, &aliases, "bash")?;
            }
        }
    }

    Ok(())
}

fn append_aliases_to_file(path: &std::path::Path, aliases: &[&str], shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;

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

/// 配置 direnv hook，只写入目标 shell 的配置文件
pub fn configure_direnv_hook(selected_shell: Option<&str>) -> anyhow::Result<()> {
    let home = get_real_home()?;

    if !distro::is_package_installed(distro::package_name("direnv").unwrap_or("direnv")) {
        return Ok(());
    }

    let target = resolve_shell_for_config(selected_shell);
    match target.as_deref() {
        Some("zsh") => {
            let path = home.join(".zshrc");
            if path.exists() {
                append_direnv_hook_to_file(&path, "zsh")?;
            }
        }
        _ => {
            let path = home.join(".bashrc");
            if path.exists() {
                append_direnv_hook_to_file(&path, "bash")?;
            }
        }
    }

    Ok(())
}

fn append_direnv_hook_to_file(path: &std::path::Path, shell: &str) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;

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

pub fn clear_tools(selected: &[bool]) -> anyhow::Result<()> {
    let pkg_list: Vec<&str> = selected.iter().enumerate()
        .filter(|(_, &s)| s)
        .filter_map(|(i, _)| crate::distro::package_name(crate::app::TOOLS[i].0))
        .collect();
    if !pkg_list.is_empty() {
        crate::distro::uninstall_packages(&pkg_list)?;
    }
    Ok(())
}
