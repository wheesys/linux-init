use crate::distro::{self, DistroFamily};
use crate::utils::get_real_home;
use std::fs;
use std::process::{Command, Stdio};

pub fn configure_locale() -> anyhow::Result<()> {
    let family = distro::detect().family();

    match family {
        DistroFamily::Arch => {
            configure_locale_arch()?;
        }
        DistroFamily::Debian => {
            configure_locale_debian()?;
        }
        _ => anyhow::bail!("不支持的发行版"),
    }

    Ok(())
}

fn configure_locale_arch() -> anyhow::Result<()> {
    let locale_gen = "/etc/locale.gen";
    let content = fs::read_to_string(locale_gen)?;
    let mut new_content = String::new();
    let mut found = false;

    for line in content.lines() {
        if line.trim() == "#zh_CN.UTF-8 UTF-8" || line.trim() == "zh_CN.UTF-8 UTF-8" {
            new_content.push_str("zh_CN.UTF-8 UTF-8\n");
            found = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    if !found {
        new_content.push_str("zh_CN.UTF-8 UTF-8\n");
    }

    // Write locale.gen via sudo tee
    let mut child = Command::new("sudo")
        .args(["tee", locale_gen])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        stdin.write_all(new_content.as_bytes())?;
    }
    drop(child.stdin.take());
    let write_status = child.wait()?;
    if !write_status.success() {
        anyhow::bail!("写入 locale.gen 失败");
    }

    let status = Command::new("sudo")
        .arg("locale-gen")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("locale-gen 失败");
    }
    Ok(())
}

fn configure_locale_debian() -> anyhow::Result<()> {
    configure_locale_arch()
}

/// 需要安装的中文字体包列表
/// - noto-fonts-cjk: Google Noto CJK 全系列（中日韩统一覆盖）
/// - wqy-microhei: 文泉驿微米黑（WPS Office 依赖，微软雅黑替代）
/// - wqy-zenhei: 文泉驿正黑（补充字体，宋体风格）
const CJK_FONT_PACKAGES: &[&str] = &["noto-fonts-cjk", "wqy-microhei", "wqy-zenhei"];

pub fn install_cjk_fonts() -> anyhow::Result<()> {
    // 检查是否已有中文字体
    if is_cjk_fonts_installed() {
        return Ok(());
    }

    // 逐包安装，忽略已安装的
    for pkg in CJK_FONT_PACKAGES {
        if distro::is_package_installed(pkg) {
            continue;
        }
        distro::install_packages(&[pkg])?;
    }
    Ok(())
}

pub fn install_fcitx5() -> anyhow::Result<()> {
    distro::install_packages(&["fcitx5", "fcitx5-chinese-addons", "fcitx5-configtool"])?;

    let home = get_real_home()?;

    // X11 / XWayland: .pam_environment（登录时 source，对 X11 会话生效）
    let env_vars = r#"GTK_IM_MODULE_DEFAULT=fcitx5
QT_IM_MODULE_DEFAULT=fcitx5
XMODIFIERS_DEFAULT=@im=fcitx
SDL_IM_MODULE_DEFAULT=fcitx
"#;

    let pam_path = home.join(".pam_environment");
    fs::write(&pam_path, env_vars)?;

    // Wayland: ~/.config/environment.d/im.conf（Wayland 合成器读取）
    let env_dir = home.join(".config/environment.d");
    fs::create_dir_all(&env_dir)?;
    let wayland_conf = env_dir.join("fcitx5.conf");
    fs::write(&wayland_conf, r#"INPUT_METHOD=fcitx5
GTK_IM_MODULE=fcitx5
QT_IM_MODULE=fcitx5
XMODIFIERS=@im=fcitx5
"#)?;

    // 自启动：.xprofile（X11）和 .config/autostart（XDG 兼容/Wayland）
    let xprofile = home.join(".xprofile");
    if !xprofile.exists() {
        fs::write(&xprofile, "fcitx5 -d &\n")?;
    }

    Ok(())
}

pub fn is_locale_configured() -> bool {
    Command::new("locale")
        .arg("-a")
        .output()
        .map(|o| {
            let locales = String::from_utf8_lossy(&o.stdout);
            locales.contains("zh_CN.utf8") || locales.contains("zh_CN.UTF-8")
        })
        .unwrap_or(false)
}

/// 检查是否有中文字体已安装（通过 fc-list 查询）
pub fn is_cjk_fonts_installed() -> bool {
    Command::new("fc-list")
        .arg(":lang=zh")
        .output()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false)
}

pub fn is_fcitx_installed() -> bool {
    distro::is_package_installed("fcitx5")
}

pub fn clear_locale() -> anyhow::Result<()> {
    use crate::utils::get_real_home;
    // 卸载字体和输入法
    crate::distro::uninstall_packages(&["noto-fonts-cjk", "wqy-microhei", "wqy-zenhei", "fcitx5", "fcitx5-chinese-addons", "fcitx5-configtool"])?;
    // 删除配置文件
    if let Ok(home) = get_real_home() {
        let _ = std::fs::remove_file(home.join(".pam_environment"));
        let _ = std::fs::remove_file(home.join(".xprofile"));
        let env_conf = home.join(".config/environment.d/fcitx5.conf");
        let _ = std::fs::remove_file(&env_conf);
    }
    Ok(())
}
