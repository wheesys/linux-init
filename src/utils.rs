use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// 检查命令是否存在
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 确保命令存在，不存在则安装
pub fn ensure_command(cmd: &str) -> anyhow::Result<()> {
    if command_exists(cmd) {
        return Ok(());
    }

    let package = match cmd {
        "curl" => "curl",
        "git" => "git",
        "vim" => "vim",
        _ => cmd,
    };

    crate::distro::install_packages(&[package])
}

/// 获取真实用户主目录（sudo 运行时返回原始用户目录）
pub fn get_real_home() -> anyhow::Result<PathBuf> {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let output = Command::new("getent")
            .args(["passwd", &sudo_user])
            .output()?;
        let line = String::from_utf8_lossy(&output.stdout);
        if let Some(home) = line.split(':').nth(5) {
            return Ok(PathBuf::from(home.trim()));
        }
    }
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取 home 目录"))
}

/// 下载操作日志记录器
pub struct DownloadLogger {
    log: std::fs::File,
    #[allow(dead_code)]
    log_path: PathBuf,
}

impl DownloadLogger {
    /// 创建新的下载日志记录器
    /// log_name: 日志文件名，如 "omz-install.log", "vundle-install.log"
    pub fn new(log_name: &str) -> anyhow::Result<Self> {
        let home = get_real_home()?;
        let log_dir = home.join(".config").join("linux-init");
        fs::create_dir_all(&log_dir)?;
        let log_path = log_dir.join(log_name);

        let mut log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        writeln!(log, "\n=== {} Start: {} ===", log_name, timestamp)?;
        log.flush()?;

        Ok(Self { log, log_path })
    }

    /// 写入日志
    pub fn log(&mut self, msg: &str) -> anyhow::Result<()> {
        writeln!(self.log, "{}", msg)?;
        self.log.flush()?;
        Ok(())
    }

    /// 获取日志文件路径（用于错误提示）
    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        &self.log_path
    }

    /// 检查网络连通性
    pub fn check_network(&mut self, url: &str) -> anyhow::Result<()> {
        self.log(&format!("Checking network: {}", url))?;

        let output = Command::new("curl")
            .args(["-sSL", "--max-time", "10", "-o", "/dev/null", "-w", "%{http_code}", url])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let code = String::from_utf8_lossy(&output.stdout);
                self.log(&format!("Network check passed, HTTP {}", code))?;
                Ok(())
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);
                self.log(&format!("Network check failed: exit code {}, {}", code, stderr))?;
                anyhow::bail!("无法连接到网络 (exit code: {})，请检查网络连接", code);
            }
            Err(e) => {
                self.log(&format!("Network check command failed: {}", e))?;
                anyhow::bail!("网络检测命令执行失败: {}", e);
            }
        }
    }

    /// 执行下载命令（用户可见输出）并记录日志
    pub fn run_download(
        &mut self,
        desc: &str,
        cmd: &str,
        args: &[&str],
    ) -> anyhow::Result<std::process::ExitStatus> {
        self.log(&format!("Running: {} - {} {}", desc, cmd, args.join(" ")))?;

        let status = Command::new(cmd)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        let code = status.code().unwrap_or(-1);
        self.log(&format!("Exit code: {}", code))?;
        Ok(status)
    }

    /// 以指定用户身份执行命令（sudo -u）并记录日志
    pub fn run_as_user(
        &mut self,
        desc: &str,
        user: &str,
        cmd: &str,
        args: &[&str],
    ) -> anyhow::Result<std::process::ExitStatus> {
        self.log(&format!(
            "Running as {}: {} - {} {}",
            user, desc, cmd,
            args.join(" ")
        ))?;

        let mut full_args: Vec<&str> = vec!["-u", user, cmd];
        full_args.extend_from_slice(args);

        let status = Command::new("sudo")
            .args(&full_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        let code = status.code().unwrap_or(-1);
        self.log(&format!("Exit code: {}", code))?;
        Ok(status)
    }

    /// 修复文件/目录的所有者权限
    pub fn fix_ownership(&mut self, path: &str) {
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            self.log(&format!("Fixing ownership: {}:{}", sudo_user, path)).ok();
            let _ = Command::new("chown")
                .args(["-R", &format!("{}:{}", sudo_user, sudo_user), path])
                .status();
        }
    }

    /// 写入结束标记
    pub fn finish(&mut self, success: bool) {
        let msg = if success { "=== Success ===" } else { "=== Failed ===" };
        self.log(msg).ok();
    }
}
