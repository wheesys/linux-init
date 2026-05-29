use std::fs;
use std::process::Command;
use crate::utils::get_real_home;

pub fn generate_ed25519(email: &str) -> anyhow::Result<String> {
    let home = get_real_home()?;
    let ssh_dir = home.join(".ssh");
    let key_path = ssh_dir.join("id_ed25519");

    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir)?;
    }

    if key_path.exists() {
        anyhow::bail!("密钥已存在: {}", key_path.display());
    }

    let output = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-C")
        .arg(email)
        .arg("-f")
        .arg(key_path.to_str().unwrap())
        .arg("-N")
        .arg("")
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "ssh-keygen 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let pub_key = fs::read_to_string(format!("{}.pub", key_path.display()))?;
    Ok(pub_key)
}

pub fn generate_rsa(email: &str) -> anyhow::Result<String> {
    let home = get_real_home()?;
    let ssh_dir = home.join(".ssh");
    let key_path = ssh_dir.join("id_rsa");

    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir)?;
    }

    if key_path.exists() {
        anyhow::bail!("密钥已存在: {}", key_path.display());
    }

    let output = Command::new("ssh-keygen")
        .arg("-t")
        .arg("rsa")
        .arg("-b")
        .arg("4096")
        .arg("-C")
        .arg(email)
        .arg("-f")
        .arg(key_path.to_str().unwrap())
        .arg("-N")
        .arg("")
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "ssh-keygen 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let pub_key = fs::read_to_string(format!("{}.pub", key_path.display()))?;
    Ok(pub_key)
}

pub fn read_public_key() -> anyhow::Result<String> {
    let home = get_real_home()?;
    let ssh_dir = home.join(".ssh");

    let ed25519 = ssh_dir.join("id_ed25519.pub");
    let rsa = ssh_dir.join("id_rsa.pub");

    if ed25519.exists() {
        Ok(fs::read_to_string(ed25519)?)
    } else if rsa.exists() {
        Ok(fs::read_to_string(rsa)?)
    } else {
        anyhow::bail!("未找到 SSH 公钥文件")
    }
}

pub fn has_ssh_key() -> bool {
    let home = match get_real_home() {
        Ok(h) => h,
        Err(_) => return false,
    };
    home.join(".ssh/id_ed25519.pub").exists() || home.join(".ssh/id_rsa.pub").exists()
}
