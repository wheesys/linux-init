use std::fs;
use std::process::{Command, Stdio};
use crate::utils::{get_real_home, DownloadLogger};

pub fn install_zsh() -> anyhow::Result<()> {
    crate::distro::install_packages(&["zsh"])
}

pub fn install_oh_my_zsh() -> anyhow::Result<()> {
    crate::utils::ensure_command("curl")?;
    crate::utils::ensure_command("git")?;

    let home = get_real_home()?;
    let omz_dir = home.join(".oh-my-zsh");

    let mut log = DownloadLogger::new("omz-install.log")?;
    log.log(&format!("Real home: {:?}", home))?;
    log.log(&format!("OMZ dir: {:?}", omz_dir))?;

    if omz_dir.exists() {
        log.log("OMZ already exists, skipping")?;
        log.finish(true);
        return Ok(());
    }

    // 下载源列表：GitHub 优先，Gitee 作为镜像
    let sources = [
        ("GitHub",
         "https://github.com/ohmyzsh/ohmyzsh/blob/master/tools/install.sh",
         "https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh"),
        ("Gitee",
         "https://gitee.com/mirrors/oh-my-zsh/blob/master/tools/install.sh",
         "https://gitee.com/mirrors/oh-my-zsh/raw/master/tools/install.sh"),
    ];

    let tmp_script = "/tmp/linux-init-omz-install.sh";
    let mut downloaded = false;

    for (name, check_url, download_url) in &sources {
        log.log(&format!("Trying {} mirror...", name))?;

        // 网络检测
        if log.check_network(check_url).is_err() {
            log.log(&format!("{} network check failed, trying next...", name))?;
            continue;
        }

        // 下载脚本
        let status = log.run_download(
            &format!("Download from {}", name),
            "curl",
            &["-fSL", "--max-time", "60", "-o", tmp_script, download_url],
        )?;

        if status.success() {
            log.log(&format!("{} download succeeded", name))?;
            downloaded = true;
            break;
        }

        let code = status.code().unwrap_or(-1);
        log.log(&format!("{} download failed (exit code: {}), trying next...", name, code))?;
    }

    if !downloaded {
        anyhow::bail!("所有下载源均失败，请检查网络连接");
    }

    // 执行安装
    let install_status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        log.run_as_user("Install OMZ", &sudo_user, "sh", &[tmp_script, "", "--unattended"])?
    } else {
        log.run_download("Install OMZ", "sh", &[tmp_script, "", "--unattended"])?
    };

    // 清理临时文件
    let _ = fs::remove_file(tmp_script);
    log.log("Cleaned up temp script")?;

    // 修复权限
    log.fix_ownership(omz_dir.to_str().unwrap_or(""));

    let installed = omz_dir.exists();
    log.log(&format!("OMZ dir exists: {}", installed))?;
    log.finish(installed && install_status.success());

    if !install_status.success() || !installed {
        anyhow::bail!("Oh My Zsh 安装失败");
    }
    Ok(())
}

pub fn set_theme(theme: &str) -> anyhow::Result<()> {
    let home = get_real_home()?;
    let zshrc_path = home.join(".zshrc");

    let content = fs::read_to_string(&zshrc_path)?;
    let mut new_content = String::new();
    let mut found = false;

    for line in content.lines() {
        if line.trim().starts_with("ZSH_THEME=") {
            new_content.push_str(&format!("ZSH_THEME=\"{}\"", theme));
            found = true;
        } else {
            new_content.push_str(line);
        }
        new_content.push('\n');
    }

    if !found {
        new_content.push_str(&format!("\nZSH_THEME=\"{}\"\n", theme));
    }

    fs::write(&zshrc_path, new_content)?;
    Ok(())
}

pub fn set_plugins(plugins: &[String]) -> anyhow::Result<()> {
    let home = get_real_home()?;
    let zshrc_path = home.join(".zshrc");

    let content = fs::read_to_string(&zshrc_path)?;
    let mut new_content = String::new();
    let mut found = false;
    let plugin_line = format!("plugins=({})", plugins.join(" "));

    for line in content.lines() {
        if line.trim().starts_with("plugins=") {
            new_content.push_str(&plugin_line);
            found = true;
        } else {
            new_content.push_str(line);
        }
        new_content.push('\n');
    }

    if !found {
        new_content.push_str(&format!("\n{}\n", plugin_line));
    }

    fs::write(&zshrc_path, new_content)?;
    Ok(())
}

/// 批量安装选中的第三方插件
pub fn install_selected_plugins(selected: &[String]) -> anyhow::Result<()> {
    let third_party = [
        "zsh-autosuggestions",
        "zsh-syntax-highlighting",
        "zsh-completions",
        "fzf",
        "you-should-use",
    ];

    for name in selected {
        if third_party.contains(&name.as_str()) {
            install_third_party_plugin(name)?;
        }
    }

    // fzf 插件需要 fzf 二进制
    if selected.contains(&"fzf".to_string()) {
        if !crate::utils::command_exists("fzf") {
            let _ = crate::distro::install_packages(&["fzf"]);
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn install_third_party_plugin(name: &str) -> anyhow::Result<()> {
    crate::utils::ensure_command("git")?;

    let home = get_real_home()?;
    let custom_plugins = home.join(".oh-my-zsh/custom/plugins");
    let plugin_dir = custom_plugins.join(name);

    let mut log = DownloadLogger::new(&format!("plugin-{}.log", name))?;
    log.log(&format!("Plugin: {}, Dir: {:?}", name, plugin_dir))?;

    if plugin_dir.exists() {
        log.log("Plugin already exists, skipping")?;
        log.finish(true);
        return Ok(());
    }

    let sources: Vec<(&str, &str)> = match name {
        "zsh-autosuggestions" => vec![
            ("GitHub", "https://github.com/zsh-users/zsh-autosuggestions.git"),
            ("Gitee", "https://gitee.com/mirrors/zsh-autosuggestions.git"),
        ],
        "zsh-syntax-highlighting" => vec![
            ("GitHub", "https://github.com/zsh-users/zsh-syntax-highlighting.git"),
            ("Gitee", "https://gitee.com/mirrors/zsh-syntax-highlighting.git"),
        ],
        "zsh-completions" => vec![
            ("GitHub", "https://github.com/zsh-users/zsh-completions.git"),
            ("Gitee", "https://gitee.com/mirrors/zsh-completions.git"),
        ],
        "fzf" => vec![
            ("GitHub", "https://github.com/chitoku-k/fzf-zsh-completions.git"),
        ],
        "you-should-use" => vec![
            ("GitHub", "https://github.com/MichaelAqworka/zsh-you-should-use.git"),
        ],
        _ => {
            log.log(&format!("Unknown plugin: {}", name))?;
            return Ok(());
        }
    };

    let mut cloned = false;
    for (mirror_name, repo_url) in &sources {
        log.log(&format!("Trying {} mirror: {}", mirror_name, repo_url))?;
        if log.check_network(repo_url).is_err() {
            log.log(&format!("{} check failed, trying next...", mirror_name))?;
            continue;
        }

        let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            log.run_as_user(
                &format!("Clone {} from {}", name, mirror_name),
                &sudo_user,
                "git",
                &["clone", repo_url, plugin_dir.to_str().unwrap()],
            )?
        } else {
            log.run_download(
                &format!("Clone {} from {}", name, mirror_name),
                "git",
                &["clone", repo_url, plugin_dir.to_str().unwrap()],
            )?
        };

        if status.success() {
            cloned = true;
            break;
        }
        log.log(&format!("{} clone failed, trying next...", mirror_name))?;
        // Clean up partial clone
        let _ = std::fs::remove_dir_all(&plugin_dir);
    }

    log.fix_ownership(plugin_dir.to_str().unwrap_or(""));

    let installed = plugin_dir.exists();
    log.finish(installed && cloned);

    if !cloned {
        anyhow::bail!("插件 {} 下载失败，所有镜像均失败", name);
    }
    Ok(())
}

pub fn set_default_shell() -> anyhow::Result<()> {
    let which_zsh = Command::new("which").arg("zsh").output()?;
    let zsh_path = String::from_utf8_lossy(&which_zsh.stdout).trim().to_string();

    if zsh_path.is_empty() {
        anyhow::bail!("未找到 zsh");
    }

    let status = Command::new("chsh")
        .args(["-s", &zsh_path])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("chsh 命令失败");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn is_third_party_plugin_installed(name: &str) -> bool {
    let home = match get_real_home() {
        Ok(h) => h,
        Err(_) => return false,
    };
    home.join(".oh-my-zsh/custom/plugins").join(name).exists()
}

#[allow(dead_code)]
pub fn get_bundled_theme_list() -> Vec<(String, String)> {
    let home = match get_real_home() {
        Ok(h) => h,
        Err(_) => return vec![],
    };
    let themes_dir = home.join(".oh-my-zsh/themes");
    if !themes_dir.exists() {
        return vec![];
    }
    let mut themes = vec![];
    if let Ok(entries) = fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "zsh-theme").unwrap_or(false) {
                if let Some(name) = path.file_stem() {
                    themes.push((name.to_string_lossy().to_string(), String::new()));
                }
            }
        }
    }
    themes.sort_by(|a, b| a.0.cmp(&b.0));
    themes
}

pub fn clear_shell() -> anyhow::Result<()> {
    let home = get_real_home()?;
    let omz_dir = home.join(".oh-my-zsh");
    if omz_dir.exists() {
        std::fs::remove_dir_all(&omz_dir)?;
    }
    crate::distro::uninstall_packages(&["zsh"])?;
    Ok(())
}
