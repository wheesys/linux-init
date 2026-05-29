use crate::distro::{self, DistroFamily};
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

pub fn install_cjk_fonts() -> anyhow::Result<()> {
    distro::install_packages(&["noto-fonts-cjk"])?;
    Ok(())
}

pub fn install_fcitx5() -> anyhow::Result<()> {
    distro::install_packages(&["fcitx5", "fcitx5-chinese-addons", "fcitx5-configtool"])?;

    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))?;
    let profile_path = home.join(".pam_environment");

    let env_vars = r#"GTK_IM_MODULE=fcitx5
QT_IM_MODULE=fcitx5
XMODIFIERS=@im=fcitx
SDL_IM_MODULE=fcitx
"#;

    fs::write(&profile_path, env_vars)?;

    let xprofile = home.join(".xprofile");
    if !xprofile.exists() {
        fs::write(&xprofile, "fcitx5 &\n")?;
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

pub fn is_fcitx_installed() -> bool {
    distro::is_package_installed("fcitx5")
}
