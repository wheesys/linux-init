use crate::distro;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// ── 容器/环境检测 ─────────────────────────────────────────

/// 检测是否在容器内运行
pub fn in_container() -> bool {
    std::path::Path::new("/.dockerenv").exists()
        || std::path::Path::new("/run/.containerenv").exists()
}

/// 检测是否以 root 运行（通过 id -u 命令，零依赖）
pub fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false)
}

/// 容器内以 root 运行时创建 no-op sudo 脚本，注入 PATH 前缀
/// 所有模块的 `Command::new("sudo")` 调用自动变成直接执行
pub fn setup_container_root_env() -> anyhow::Result<()> {
    if !in_container() || !is_root() {
        return Ok(());
    }

    let script_path = "/tmp/sudo";
    std::fs::write(script_path, "#!/bin/sh\nexec \"$@\"\n")?;
    std::process::Command::new("chmod")
        .args(["+x", script_path])
        .status()?;
    // 如果系统已有 sudo，先备份
    let system_sudo = "/usr/bin/sudo";
    if std::path::Path::new(system_sudo).exists() {
        // sudo 存在时，/tmp 在 PATH 最前面，我们的包装器覆盖它
    }

    let current_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("/tmp:{}", current_path));

    Ok(())
}

// ── 数据结构 ──────────────────────────────────────────────

/// 执行 spec（从 CLI 参数 JSON 解析）
#[derive(Debug, Deserialize)]
pub struct ExecuteSpec {
    pub actions: Vec<ActionSpec>,
}

/// 单个 action 描述
#[derive(Debug, Deserialize)]
pub struct ActionSpec {
    pub id: String,
    #[serde(default)]
    pub skip: bool,
    #[serde(default = "default_true")]
    pub verify: bool,
    #[serde(default)]
    pub args: ActionArgs,
}

fn default_true() -> bool {
    true
}

/// action 参数（所有可选字段，按需使用）
#[derive(Debug, Default, Deserialize)]
pub struct ActionArgs {
    pub email: Option<String>,
    pub tools: Option<Vec<String>>,
    pub theme: Option<String>,
    pub plugins: Option<Vec<String>>,
    pub vim_plugins: Option<Vec<usize>>,
    pub vim_opts: Option<Vec<usize>>,
    pub mirror_url: Option<String>,
    pub shell: Option<String>,
}

/// 执行环境信息
struct ExecuteEnv {
    in_container: bool,
    is_root: bool,
}

/// 单个 action 执行结果
#[derive(Debug, Serialize)]
struct ActionResult {
    id: String,
    status: String, // "ok" | "skip" | "fail"
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification: Option<serde_json::Value>,
}

/// 汇总统计
#[derive(Debug, Serialize)]
struct Summary {
    total: usize,
    ok: usize,
    skip: usize,
    fail: usize,
}

/// 最终输出
#[derive(Debug, Serialize)]
struct ExecuteOutput {
    distro: String,
    in_container: bool,
    running_as_root: bool,
    results: Vec<ActionResult>,
    summary: Summary,
}

// ── 派发 ──────────────────────────────────────────────────

/// 派发 action，调用对应模块函数
fn dispatch_action(id: &str, args: &ActionArgs, env: &ExecuteEnv) -> Result<(), String> {
    match id {
        // ── System ──
        "detect_distro" => {
            let d = distro::detect();
            eprintln!("  发行版: {}", d);
            Ok(())
        }
        "refresh_package_cache" => distro::refresh_cache().map_err(|e| e.to_string()),

        // ── Shell ──
        "install_zsh" => crate::modules::shell::install_zsh().map_err(|e| e.to_string()),
        "install_oh_my_zsh" => crate::modules::shell::install_oh_my_zsh().map_err(|e| e.to_string()),
        "set_zsh_theme" => {
            let theme = args.theme.as_deref().unwrap_or("robbyrussell");
            crate::modules::shell::set_theme(theme).map_err(|e| e.to_string())
        }
        "install_plugins" => {
            let plugins: Vec<String> = args.plugins.clone().unwrap_or_default();
            if plugins.is_empty() {
                return Ok(());
            }
            crate::modules::shell::set_plugins(&plugins).map_err(|e| e.to_string())?;
            crate::modules::shell::install_selected_plugins(&plugins).map_err(|e| e.to_string())
        }
        "set_default_shell" => {
            if env.in_container {
                return Err("SKIP:chsh_not_available_in_container".into());
            }
            crate::modules::shell::set_default_shell().map_err(|e| e.to_string())
        }
        "clear_shell" => crate::modules::shell::clear_shell().map_err(|e| e.to_string()),

        // ── Docker ──
        "install_docker" => crate::modules::docker::install_docker().map_err(|e| e.to_string()),
        "install_compose" => crate::modules::docker::install_compose().map_err(|e| e.to_string()),
        "add_user_to_docker_group" => {
            if env.is_root {
                return Err("SKIP:root_no_need_docker_group".into());
            }
            crate::modules::docker::add_user_to_docker_group().map_err(|e| e.to_string())
        }
        "start_docker_service" => {
            if env.in_container {
                return Err("SKIP:no_systemd_in_container".into());
            }
            crate::modules::docker::start_docker_service().map_err(|e| e.to_string())
        }
        "clear_docker" => crate::modules::docker::clear_docker().map_err(|e| e.to_string()),

        // ── SSH ──
        "generate_ed25519_key" => {
            ensure_ssh_keygen()?;
            let email = args.email.as_deref().unwrap_or("ci@linux-init.test");
            crate::modules::ssh::generate_ed25519(email).map(|_| ()).map_err(|e| e.to_string())
        }
        "generate_rsa_key" => {
            ensure_ssh_keygen()?;
            let email = args.email.as_deref().unwrap_or("ci@linux-init.test");
            crate::modules::ssh::generate_rsa(email).map(|_| ()).map_err(|e| e.to_string())
        }
        "read_public_key" => {
            let key = crate::modules::ssh::read_public_key().map_err(|e| e.to_string())?;
            eprintln!("  公钥: {}", key.trim());
            Ok(())
        }
        "clear_ssh_keys" => crate::modules::ssh::clear_ssh_keys().map_err(|e| e.to_string()),

        // ── SSH Server ──
        "install_ssh_server" => crate::modules::ssh_server::install().map_err(|e| e.to_string()),
        "disable_root_login" => crate::modules::ssh_server::disable_root_login().map_err(|e| e.to_string()),
        "start_ssh_service" => {
            if env.in_container {
                return Err("SKIP:no_systemd_in_container".into());
            }
            crate::modules::ssh_server::start_service().map_err(|e| e.to_string())
        }
        "clear_ssh_server" => crate::modules::ssh_server::clear_ssh_server().map_err(|e| e.to_string()),

        // ── Tools ──
        "install_tools" => {
            let tools: Vec<String> = args.tools.clone().unwrap_or_else(|| {
                vec!["git".into(), "curl".into(), "wget".into()]
            });
            let refs: Vec<&str> = tools.iter().map(|s| s.as_str()).collect();
            crate::modules::tools::install_tools(&refs).map_err(|e| e.to_string())
        }
        "configure_aliases" => {
            let tools: Vec<String> = args.tools.clone().unwrap_or_default();
            let refs: Vec<&str> = tools.iter().map(|s| s.as_str()).collect();
            let shell = args.shell.as_deref();
            crate::modules::tools::configure_aliases(&refs, shell).map_err(|e| e.to_string())
        }
        "configure_direnv_hook" => {
            let shell = args.shell.as_deref();
            crate::modules::tools::configure_direnv_hook(shell).map_err(|e| e.to_string())
        }

        // ── Vim ──
        "install_vim" => crate::modules::vim::install_vim().map_err(|e| e.to_string()),
        "install_vundle" => crate::modules::vim::install_vundle().map_err(|e| e.to_string()),
        "write_vimrc" => {
            let plugins = args.vim_plugins.clone().unwrap_or_default();
            let opts = args.vim_opts.clone().unwrap_or_default();
            crate::modules::vim::write_vimrc(&plugins, &opts).map_err(|e| e.to_string())
        }

        // ── NVM ──
        "install_nvm" => crate::modules::nvm::install_nvm().map_err(|e| e.to_string()),
        "install_node_latest" => crate::modules::nvm::install_node_latest().map_err(|e| e.to_string()),
        "install_node_lts" => crate::modules::nvm::install_node_lts().map_err(|e| e.to_string()),
        "clear_nvm" => crate::modules::nvm::clear_nvm().map_err(|e| e.to_string()),

        // ── Locale ──
        "configure_locale" => crate::modules::locale::configure_locale().map_err(|e| e.to_string()),
        "install_cjk_fonts" => crate::modules::locale::install_cjk_fonts().map_err(|e| e.to_string()),
        "install_fcitx5" => crate::modules::locale::install_fcitx5().map_err(|e| e.to_string()),
        "clear_locale" => crate::modules::locale::clear_locale().map_err(|e| e.to_string()),

        // ── Sources ──
        "switch_system_mirror" => {
            let url = args.mirror_url.as_deref().unwrap_or("");
            crate::modules::sources::switch_system_mirror(url).map_err(|e| e.to_string())
        }
        "switch_docker_mirror" => {
            let url = args.mirror_url.as_deref().unwrap_or("");
            crate::modules::sources::switch_docker_mirror(url).map_err(|e| e.to_string())
        }
        "switch_npm_registry" => {
            let url = args.mirror_url.as_deref().unwrap_or("");
            crate::modules::sources::switch_npm_registry(url).map_err(|e| e.to_string())
        }

        _ => Err(format!("未知 action: {}", id)),
    }
}

// ── 验证 ──────────────────────────────────────────────────

/// 执行完 action 后验证结果
fn run_verification(id: &str, _args: &ActionArgs) -> Option<serde_json::Value> {
    let result = match id {
        "install_zsh" => {
            let installed = distro::is_package_installed("zsh");
            let binary = std::process::Command::new("which")
                .arg("zsh")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
            serde_json::json!({
                "package_installed": installed,
                "binary": binary,
            })
        }
        "install_oh_my_zsh" => {
            let home = crate::utils::get_real_home().unwrap_or_default();
            serde_json::json!({
                "oh_my_zsh_dir": home.join(".oh-my-zsh").exists(),
                "zshrc_exists": home.join(".zshrc").exists(),
            })
        }
        "set_zsh_theme" => {
            let home = crate::utils::get_real_home().unwrap_or_default();
            let zshrc = home.join(".zshrc");
            let has_theme = std::fs::read_to_string(&zshrc)
                .map(|c| c.contains("ZSH_THEME="))
                .unwrap_or(false);
            serde_json::json!({ "zshrc_has_theme": has_theme })
        }
        "install_plugins" => {
            let home = crate::utils::get_real_home().unwrap_or_default();
            let zshrc = home.join(".zshrc");
            let has_plugins = std::fs::read_to_string(&zshrc)
                .map(|c| c.contains("plugins=("))
                .unwrap_or(false);
            serde_json::json!({ "zshrc_has_plugins": has_plugins })
        }
        "set_default_shell" => {
            let shell = std::env::var("SHELL").unwrap_or_default();
            serde_json::json!({ "current_shell": shell, "is_zsh": shell.contains("zsh") })
        }

        "install_docker" => {
            let installed = distro::is_tool_installed("docker");
            let binary = std::process::Command::new("which")
                .arg("docker")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
            serde_json::json!({ "tool_installed": installed, "binary": binary })
        }
        "install_compose" => {
            let binary = std::process::Command::new("which")
                .arg("docker-compose")
                .output()
                .ok()
                .map(|o| {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    !s.is_empty()
                })
                .unwrap_or(false);
            let plugin = std::process::Command::new("docker")
                .args(["compose", "version"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            serde_json::json!({ "docker_compose_binary": binary, "compose_plugin": plugin })
        }
        "add_user_to_docker_group" => {
            serde_json::json!({ "in_docker_group": crate::modules::docker::is_user_in_docker_group() })
        }
        "start_docker_service" => {
            serde_json::json!({ "docker_running": crate::modules::docker::is_docker_running() })
        }

        "generate_ed25519_key" => {
            serde_json::json!({ "key_exists": crate::modules::ssh::has_ed25519_key() })
        }
        "generate_rsa_key" => {
            serde_json::json!({ "key_exists": crate::modules::ssh::has_rsa_key() })
        }

        "install_ssh_server" => {
            let installed = crate::modules::ssh_server::is_installed();
            serde_json::json!({ "sshd_installed": installed })
        }
        "disable_root_login" => {
            let disabled = crate::modules::ssh_server::is_root_login_disabled();
            serde_json::json!({ "root_login_disabled": disabled })
        }
        "start_ssh_service" => {
            let running = crate::modules::ssh_server::is_running();
            serde_json::json!({ "sshd_running": running })
        }

        "install_tools" => {
            let tools = _args.tools.clone().unwrap_or_default();
            let mut statuses = serde_json::Map::new();
            for t in &tools {
                // 用 get_tool_status 正确检测（处理 ripgrep→rg, fd→fdfind 等映射）
                statuses.insert(t.clone(), serde_json::json!(crate::modules::tools::get_tool_status(t.as_str())));
            }
            serde_json::Value::Object(statuses)
        }

        "install_vim" => {
            let installed = distro::is_package_installed("vim");
            serde_json::json!({ "vim_installed": installed })
        }
        "install_vundle" => {
            let home = crate::utils::get_real_home().unwrap_or_default();
            let exists = home.join(".vim/bundle/Vundle.vim").exists();
            serde_json::json!({ "vundle_dir_exists": exists })
        }
        "write_vimrc" => {
            let home = crate::utils::get_real_home().unwrap_or_default();
            let exists = home.join(".vimrc").exists();
            serde_json::json!({ "vimrc_exists": exists })
        }

        "install_nvm" => {
            let home = crate::utils::get_real_home().unwrap_or_default();
            let nvm_sh = home.join(".nvm/nvm.sh");
            serde_json::json!({ "nvm_installed": nvm_sh.exists() })
        }
        "install_node_latest" | "install_node_lts" => {
            let version = crate::modules::nvm::installed_node_version();
            serde_json::json!({ "node_version": version })
        }

        "configure_locale" => {
            serde_json::json!({ "locale_configured": crate::modules::locale::is_locale_configured() })
        }
        "install_cjk_fonts" => {
            serde_json::json!({ "cjk_fonts_installed": crate::modules::locale::is_cjk_fonts_installed() })
        }
        "install_fcitx5" => {
            serde_json::json!({ "fcitx5_installed": crate::modules::locale::is_fcitx_installed() })
        }

        _ => return None,
    };
    Some(result)
}

// ── 主运行器 ──────────────────────────────────────────────

/// 确保 ssh-keygen 命令可用（按发行版安装对应包）
fn ensure_ssh_keygen() -> Result<(), String> {
    if crate::utils::command_exists("ssh-keygen") {
        return Ok(());
    }
    let pkg = match crate::distro::detect().family() {
        crate::distro::DistroFamily::Arch => "openssh",
        crate::distro::DistroFamily::Debian => "openssh-client",
        crate::distro::DistroFamily::Fedora => "openssh",
        _ => "openssh",
    };
    crate::distro::install_packages(&[pkg]).map_err(|e| e.to_string())
}

pub fn run_execute(json_spec: &str) -> anyhow::Result<()> {
    let spec: ExecuteSpec = serde_json::from_str(json_spec)?;
    let container = in_container();
    let root = is_root();
    let distro_info = distro::detect();

    let env = ExecuteEnv {
        in_container: container,
        is_root: root,
    };

    eprintln!("╔══════════════════════════════════════════╗");
    eprintln!("║  linux-init --execute (non-interactive)  ║");
    eprintln!("╠══════════════════════════════════════════╣");
    eprintln!("║  发行版: {:32} ║", distro_info.to_string());
    eprintln!("║  容器:   {:32} ║", if container { "是 ✓" } else { "否" });
    eprintln!("║  root:   {:32} ║", if root { "是 ✓" } else { "否" });
    eprintln!("╚══════════════════════════════════════════╝");
    eprintln!();

    let mut results: Vec<ActionResult> = Vec::new();

    for action_def in &spec.actions {
        if action_def.skip {
            eprintln!("[skip] {} (flagged)", action_def.id);
            results.push(ActionResult {
                id: action_def.id.clone(),
                status: "skip".into(),
                reason: Some("flagged".into()),
                duration_ms: 0,
                error: None,
                verification: None,
            });
            continue;
        }

        eprint!("[{:>3}/{}] {} ... ", results.len() + 1, spec.actions.len(), action_def.id);

        let start = Instant::now();
        match dispatch_action(&action_def.id, &action_def.args, &env) {
            Ok(()) => {
                let duration = start.elapsed().as_millis() as u64;
                let v = if action_def.verify {
                    run_verification(&action_def.id, &action_def.args)
                } else {
                    None
                };
                eprintln!("✔ ok ({}ms)", duration);
                results.push(ActionResult {
                    id: action_def.id.clone(),
                    status: "ok".into(),
                    reason: None,
                    duration_ms: duration,
                    error: None,
                    verification: v,
                });
            }
            Err(e) => {
                let duration = start.elapsed().as_millis() as u64;
                if let Some(reason) = e.strip_prefix("SKIP:") {
                    eprintln!("⏭ skip ({})", reason);
                    results.push(ActionResult {
                        id: action_def.id.clone(),
                        status: "skip".into(),
                        reason: Some(reason.to_string()),
                        duration_ms: duration,
                        error: None,
                        verification: None,
                    });
                } else {
                    eprintln!("✖ FAIL");
                    eprintln!("      {}", e);
                    results.push(ActionResult {
                        id: action_def.id.clone(),
                        status: "fail".into(),
                        reason: None,
                        duration_ms: duration,
                        error: Some(e),
                        verification: None,
                    });
                }
            }
        }
    }

    let ok = results.iter().filter(|r| r.status == "ok").count();
    let skip = results.iter().filter(|r| r.status == "skip").count();
    let fail = results.iter().filter(|r| r.status == "fail").count();

    let output = ExecuteOutput {
        distro: distro_info.to_string(),
        in_container: container,
        running_as_root: root,
        summary: Summary {
            total: results.len(),
            ok,
            skip,
            fail,
        },
        results,
    };

    let json_output = serde_json::to_string_pretty(&output)?;
    println!("{}", json_output);

    eprintln!();
    eprintln!("═══════════════════════════════════════");
    eprintln!("  合计: {} | ✔ ok: {} | ⏭ skip: {} | ✖ fail: {}",
        output.summary.total, ok, skip, fail);

    if fail > 0 {
        eprintln!("═══════════════════════════════════════");
        std::process::exit(1);
    }

    eprintln!("  全部通过 ✓");
    eprintln!("═══════════════════════════════════════");
    Ok(())
}
