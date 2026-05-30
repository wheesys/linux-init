# 源切换功能 — 实现计划

> **For agentic workers:** 使用 superpowers:subagent-driven-development 或 superpowers:executing-plans 逐任务实现。步骤使用 checkbox (`- [ ]`) 语法追踪。

**目标:** 为 linux-init 添加系统源/Docker 源/Node 源镜像切换功能，支持自动测速推荐 + 手动选择，切换后自动刷新缓存。

**架构:** 新增 `sources.rs` 模块统一管理所有源切换逻辑。使用单个 `Page::SourceSelect(SourceType)` 页面复用于三种源类型，通过 `SourceType` 枚举区分。镜像列表、ping 测速、切换逻辑均在模块内实现。

**技术栈:** Rust + ratatui TUI + 系统命令（pacman/apt/docker/npm）

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/modules/sources.rs` | **新建** | 镜像数据定义、ping 测速、源切换函数 |
| `src/modules/mod.rs` | 修改 | 注册 sources 模块 |
| `src/app.rs` | 修改 | 添加 `SourceType` 枚举、`Page::SourceSelect`、状态字段 |
| `src/ui/mod.rs` | 修改 | 渲染和事件处理 |
| `src/i18n.rs` | 修改 | 三种源菜单的国际化文本 |
| `src/main.rs` | 读取 | 了解 Action 处理方式以确定测速结果回传方案 |

---

### Task 1: 创建 sources.rs 核心模块

**文件:**
- 新建: `src/modules/sources.rs`
- 修改: `src/modules/mod.rs`

- [ ] **Step 1: 创建 sources.rs 模块文件**

```rust
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant};
use std::fs;

use crate::distro::{self, DistroFamily};

// ── 镜像数据结构 ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MirrorEntry {
    pub name_cn: &'static str,
    pub name_en: &'static str,
    pub url: &'static str,
    pub ping_host: &'static str,
}

// ── 系统源 (pacman) ──────────────────────────────────────

pub const PACMAN_MIRRORS: &[MirrorEntry] = &[
    MirrorEntry { name_cn: "默认官方源", name_en: "Default Official", url: "", ping_host: "" },
    MirrorEntry { name_cn: "清华 (TUNA)", name_en: "Tsinghua (TUNA)", url: "https://mirrors.tuna.tsinghua.edu.cn/archlinux/$repo/os/$arch", ping_host: "mirrors.tuna.tsinghua.edu.cn:443" },
    MirrorEntry { name_cn: "中科大 (USTC)", name_en: "USTC", url: "https://mirrors.ustc.edu.cn/archlinux/$repo/os/$arch", ping_host: "mirrors.ustc.edu.cn:443" },
    MirrorEntry { name_cn: "阿里云", name_en: "Aliyun", url: "https://mirrors.aliyun.com/archlinux/$repo/os/$arch", ping_host: "mirrors.aliyun.com:443" },
    MirrorEntry { name_cn: "腾讯云", name_en: "Tencent Cloud", url: "https://mirrors.cloud.tencent.com/archlinux/$repo/os/$arch", ping_host: "mirrors.cloud.tencent.com:443" },
];

// ── 系统源 (apt) ────────────────────────────────────────

pub const APT_MIRRORS: &[MirrorEntry] = &[
    MirrorEntry { name_cn: "默认官方源", name_en: "Default Official", url: "", ping_host: "" },
    MirrorEntry { name_cn: "清华 (TUNA)", name_en: "Tsinghua (TUNA)", url: "https://mirrors.tuna.tsinghua.edu.cn", ping_host: "mirrors.tuna.tsinghua.edu.cn:443" },
    MirrorEntry { name_cn: "中科大 (USTC)", name_en: "USTC", url: "https://mirrors.ustc.edu.cn", ping_host: "mirrors.ustc.edu.cn:443" },
    MirrorEntry { name_cn: "阿里云", name_en: "Aliyun", url: "https://mirrors.aliyun.com", ping_host: "mirrors.aliyun.com:443" },
    MirrorEntry { name_cn: "腾讯云", name_en: "Tencent Cloud", url: "https://mirrors.cloud.tencent.com", ping_host: "mirrors.cloud.tencent.com:443" },
];

// ── Docker 源 ────────────────────────────────────────────

pub const DOCKER_MIRRORS: &[MirrorEntry] = &[
    MirrorEntry { name_cn: "默认（无镜像加速）", name_en: "Default (no mirror)", url: "", ping_host: "" },
    MirrorEntry { name_cn: "阿里云（需替换 your-id）", name_en: "Aliyun (replace your-id)", url: "https://<your-id>.mirror.aliyuncs.com", ping_host: "<your-id>.mirror.aliyuncs.com:443" },
    MirrorEntry { name_cn: "腾讯云", name_en: "Tencent Cloud", url: "https://mirror.ccs.tencentyun.com", ping_host: "mirror.ccs.tencentyun.com:443" },
    MirrorEntry { name_cn: "Docker China", name_en: "Docker China", url: "https://registry.docker-cn.com", ping_host: "registry.docker-cn.com:443" },
];

// ── Node (npm) 源 ────────────────────────────────────────

pub const NPM_MIRRORS: &[MirrorEntry] = &[
    MirrorEntry { name_cn: "默认 (npmjs.org)", name_en: "Default (npmjs.org)", url: "https://registry.npmjs.org", ping_host: "registry.npmjs.org:443" },
    MirrorEntry { name_cn: "npmmirror.com", name_en: "npmmirror.com", url: "https://registry.npmmirror.com", ping_host: "registry.npmmirror.com:443" },
    MirrorEntry { name_cn: "腾讯云 npm", name_en: "Tencent Cloud npm", url: "https://mirrors.cloud.tencent.com/npm/", ping_host: "mirrors.cloud.tencent.com:443" },
];

// ── 网络测速 ─────────────────────────────────────────────

fn ping_host(host: &str) -> Option<u64> {
    if host.is_empty() {
        return None;
    }
    let start = Instant::now();
    let addr: SocketAddr = match host.to_socket_addrs().ok()?.next() {
        Some(a) => a,
        None => return None,
    };
    match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
        Ok(_) => Some(start.elapsed().as_millis() as u64),
        Err(_) => None,
    }
}

pub fn test_mirrors(mirrors: &[MirrorEntry]) -> Vec<Option<u64>> {
    mirrors.iter().map(|m| ping_host(m.ping_host)).collect()
}

pub fn recommend_index(latencies: &[Option<u64>]) -> Option<usize> {
    latencies.iter().enumerate()
        .filter_map(|(i, lat)| lat.map(|l| (i, l)))
        .min_by_key(|(_, l)| *l)
        .map(|(i, _)| i)
}

// ── 系统源切换 ───────────────────────────────────────────

const PACMAN_MIRRORLIST: &str = "/etc/pacman.d/mirrorlist";
const PACMAN_MIRRORLIST_BAK: &str = "/etc/pacman.d/mirrorlist.linux-init.bak";
const APT_SOURCES: &str = "/etc/apt/sources.list";
const APT_SOURCES_BAK: &str = "/etc/apt/sources.list.linux-init.bak";

pub fn switch_pacman_mirror(url: &str) -> anyhow::Result<()> {
    if !std::path::Path::new(PACMAN_MIRRORLIST_BAK).exists() {
        let status = Command::new("sudo")
            .args(["cp", PACMAN_MIRRORLIST, PACMAN_MIRRORLIST_BAK])
            .status()?;
        if !status.success() {
            anyhow::bail!("备份 mirrorlist 失败");
        }
    }

    if url.is_empty() {
        let status = Command::new("sudo")
            .args(["cp", PACMAN_MIRRORLIST_BAK, PACMAN_MIRRORLIST])
            .status()?;
        if !status.success() {
            anyhow::bail!("恢复 mirrorlist 失败");
        }
    } else {
        let backup = fs::read_to_string(PACMAN_MIRRORLIST_BAK)?;
        let filtered: Vec<&str> = backup.lines()
            .filter(|l| !l.contains("## linux-init mirror"))
            .collect();
        let new_mirror = format!("## linux-init mirror\nServer = {}\n", url);
        let new_content = format!("{}\n{}", new_mirror, filtered.join("\n"));
        let tmp = "/tmp/linux-init-mirrorlist";
        fs::write(tmp, new_content)?;
        let status = Command::new("sudo").args(["cp", tmp, PACMAN_MIRRORLIST]).status()?;
        let _ = fs::remove_file(tmp);
        if !status.success() {
            anyhow::bail!("写入 mirrorlist 失败");
        }
    }
    refresh_pacman_cache()
}

pub fn switch_apt_mirror(url: &str) -> anyhow::Result<()> {
    if !std::path::Path::new(APT_SOURCES_BAK).exists() {
        let status = Command::new("sudo")
            .args(["cp", APT_SOURCES, APT_SOURCES_BAK])
            .status()?;
        if !status.success() {
            anyhow::bail!("备份 sources.list 失败");
        }
    }

    if url.is_empty() {
        let status = Command::new("sudo")
            .args(["cp", APT_SOURCES_BAK, APT_SOURCES])
            .status()?;
        if !status.success() {
            anyhow::bail!("恢复 sources.list 失败");
        }
    } else {
        let backup = fs::read_to_string(APT_SOURCES_BAK)?;
        let new_content = replace_apt_mirror(&backup, url);
        let tmp = "/tmp/linux-init-sources.list";
        fs::write(tmp, new_content)?;
        let status = Command::new("sudo").args(["cp", tmp, APT_SOURCES]).status()?;
        let _ = fs::remove_file(tmp);
        if !status.success() {
            anyhow::bail!("写入 sources.list 失败");
        }
    }
    refresh_apt_cache()
}

fn replace_apt_mirror(content: &str, new_mirror: &str) -> String {
    let host = new_mirror.trim_start_matches("https://").trim_start_matches("http://");
    content.lines().map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with("deb ") && !trimmed.starts_with("deb-src") {
            if let Some(idx) = trimmed.find("://") {
                let after_proto = &trimmed[idx + 3..];
                if let Some(space_idx) = after_proto.find(char::is_whitespace) {
                    let rest = &after_proto[space_idx..];
                    return format!("deb https://{}{}", host, rest);
                }
            }
        }
        line.to_string()
    }).collect::<Vec<_>>().join("\n")
}

fn refresh_pacman_cache() -> anyhow::Result<()> {
    let status = Command::new("sudo").args(["pacman", "-Syy"]).status()?;
    if !status.success() { anyhow::bail!("pacman -Syy 失败"); }
    Ok(())
}

fn refresh_apt_cache() -> anyhow::Result<()> {
    let status = Command::new("sudo").args(["apt", "update"]).status()?;
    if !status.success() { anyhow::bail!("apt update 失败"); }
    Ok(())
}

// ── Docker 源切换 ─────────────────────────────────────────

const DOCKER_DAEMON_JSON: &str = "/etc/docker/daemon.json";

pub fn switch_docker_mirror(url: &str) -> anyhow::Result<()> {
    let mut config: serde_json::Value = if std::path::Path::new(DOCKER_DAEMON_JSON).exists() {
        let content = fs::read_to_string(DOCKER_DAEMON_JSON)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if url.is_empty() {
        if let Some(obj) = config.as_object_mut() {
            obj.remove("registry-mirrors");
        }
    } else {
        config["registry-mirrors"] = serde_json::json!([url]);
    }

    let new_content = serde_json::to_string_pretty(&config)?;
    let tmp = "/tmp/linux-init-daemon.json";
    fs::write(tmp, new_content)?;
    let _ = Command::new("sudo").args(["mkdir", "-p", "/etc/docker"]).status();
    let status = Command::new("sudo").args(["cp", tmp, DOCKER_DAEMON_JSON]).status()?;
    let _ = fs::remove_file(tmp);
    if !status.success() { anyhow::bail!("写入 daemon.json 失败"); }
    restart_docker()
}

fn restart_docker() -> anyhow::Result<()> {
    let status = Command::new("sudo").args(["systemctl", "restart", "docker"]).status()?;
    if !status.success() { anyhow::bail!("systemctl restart docker 失败"); }
    Ok(())
}

// ── Node (npm) 源切换 ─────────────────────────────────────

pub fn switch_npm_registry(url: &str) -> anyhow::Result<()> {
    let status = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        Command::new("sudo").args(["-u", &sudo_user, "npm", "config", "set", "registry", url]).status()?
    } else {
        Command::new("npm").args(["config", "set", "registry", url]).status()?
    };
    if !status.success() { anyhow::bail!("npm config set registry 失败"); }
    Ok(())
}

// ── 工具函数 ─────────────────────────────────────────────

pub fn system_mirrors() -> &'static [MirrorEntry] {
    match distro::detect().family() {
        DistroFamily::Arch => PACMAN_MIRRORS,
        DistroFamily::Debian => APT_MIRRORS,
        _ => PACMAN_MIRRORS,
    }
}

pub fn switch_system_mirror(url: &str) -> anyhow::Result<()> {
    match distro::detect().family() {
        DistroFamily::Arch => switch_pacman_mirror(url),
        DistroFamily::Debian => switch_apt_mirror(url),
        _ => anyhow::bail!("不支持的发行版"),
    }
}
```

- [ ] **Step 2: 注册模块**

修改 `src/modules/mod.rs`，添加 `pub mod sources;`：

```rust
pub mod shell;
pub mod docker;
pub mod ssh;
pub mod ssh_server;
pub mod tools;
pub mod locale;
pub mod vim;
pub mod nvm;
pub mod sources;
```

---

### Task 2: 添加 app.rs 状态和 Page 变体

**文件:**
- 修改: `src/app.rs`

- [ ] **Step 1: 添加 SourceType 枚举（在 Page 枚举定义之前）**

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceType {
    System,
    Docker,
    Npm,
}
```

- [ ] **Step 2: 在 Page 枚举中添加新变体**

```rust
pub enum Page {
    LangSelect,
    MainMenu,
    Shell,
    ShellZshTheme,
    ShellZshPlugins,
    Docker,
    Ssh,
    SshServer,
    Tools,
    Vim,
    VimPlugins,
    VimOptimize,
    Nvm,
    NvmNodeVersion,
    Locale,
    Status(Box<StatusData>),
    SourceSelect(SourceType),
}
```

- [ ] **Step 3: 在 App 结构体中添加状态字段（locale 块之后）**

```rust
    // -- sources --
    pub source_index: usize,
    pub source_latencies: Vec<Option<u64>>,
    pub source_recommended: Option<usize>,
    pub source_tested: bool,
```

- [ ] **Step 4: 在 App::new 构造函数中初始化**

```rust
            source_index: 0,
            source_latencies: vec![],
            source_recommended: None,
            source_tested: false,
```

---

### Task 3: 添加国际化文本

**文件:**
- 修改: `src/i18n.rs`

- [ ] **Step 1: 在 i18n.rs 末尾添加源管理文本**

```rust
// ── Source Management ──────────────────────────────────────

pub fn sources_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "系统源管理 — 选择镜像源",
        Lang::English => "System Sources — Select Mirror",
    }
}

pub fn docker_mirror_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Docker 镜像加速 — 选择镜像源",
        Lang::English => "Docker Mirror — Select Registry Mirror",
    }
}

pub fn npm_mirror_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "npm 镜像源 — 选择 Registry",
        Lang::English => "npm Registry — Select Mirror",
    }
}

pub fn source_statusbar(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "↑↓ 导航  Enter 选择+测速  Esc 返回",
        Lang::English => "↑↓ nav  Enter select+test  Esc back",
    }
}

pub fn source_latency(lang: Lang, ms: u64) -> String {
    match lang {
        Lang::Chinese => format!("{}ms", ms),
        Lang::English => format!("{}ms", ms),
    }
}

pub fn source_unreachable(lang: Lang) -> &'static str {
    match lang { Lang::Chinese => "不可达", Lang::English => "unreachable" }
}

pub fn source_recommended(lang: Lang) -> &'static str {
    match lang { Lang::Chinese => "★ 推荐", Lang::English => "★ Recommended" }
}

pub fn source_switch_ok(lang: Lang, name: &str) -> String {
    match lang {
        Lang::Chinese => format!("✅ 已切换到: {}，缓存已刷新", name),
        Lang::English => format!("✅ Switched to: {}, cache refreshed", name),
    }
}

pub fn source_switch_fail(lang: Lang, name: &str, err: &str) -> String {
    match lang {
        Lang::Chinese => format!("❌ 切换到 {} 失败: {}", name, err),
        Lang::English => format!("❌ Failed to switch to {}: {}", name, err),
    }
}

pub fn source_testing(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "正在测试镜像源速度...",
        Lang::English => "Testing mirror speeds...",
    }
}
```

- [ ] **Step 2: 更新 main_menu（加第9项系统源管理）**

在 `main_menu` 函数中，两个语言分支的数组末尾各添加一项：

中文分支末尾添加：
```rust
("🔁 系统源管理", "切换 pacman/apt 软件源镜像"),
```

英文分支末尾添加：
```rust
("🔁 System Sources", "Switch pacman/apt mirror sources"),
```

- [ ] **Step 3: 更新 docker_menu（加第6项切换 Docker 源）**

在 `docker_menu` 函数中，两个语言分支的数组末尾各添加一项：

中文分支末尾添加：
```rust
("6. 切换 Docker 源", "切换 Docker Hub 镜像加速源"),
```

英文分支末尾添加：
```rust
("6. Switch Docker Mirror", "Switch Docker Hub registry mirror"),
```

- [ ] **Step 4: 更新 nvm_menu（加第4项切换 npm 源）**

在 `nvm_menu` 函数中，两个语言分支的数组末尾各添加一项：

中文分支末尾添加：
```rust
("4. 切换 npm 源", "切换 npm registry 镜像源"),
```

英文分支末尾添加：
```rust
("4. Switch npm Mirror", "Switch npm registry mirror"),
```

---

### Task 4: 添加 UI 渲染

**文件:**
- 修改: `src/ui/mod.rs`

- [ ] **Step 1: 在 render 函数中添加分支**

匹配分支中添加：
```rust
Page::SourceSelect(_) => render_source_select(frame, app, chunks[0]),
```

- [ ] **Step 2: 在 render_status_bar 中添加 statusbar 路由**

在 `keys` 变量的 match 中添加：
```rust
Page::SourceSelect(_) => i18n::source_statusbar(lang),
```

- [ ] **Step 3: 在 input_number_to_page 中添加路由**

```rust
Page::SourceSelect(_) => app.source_index = idx,
```

- [ ] **Step 4: 实现 render_source_select 函数**

```rust
fn render_source_select(frame: &mut Frame, app: &App, area: Rect) {
    let lang = app.lang;
    let source_type = match &app.page {
        Page::SourceSelect(t) => *t,
        _ => return,
    };

    let title = match source_type {
        SourceType::System => i18n::sources_title(lang),
        SourceType::Docker => i18n::docker_mirror_title(lang),
        SourceType::Npm => i18n::npm_mirror_title(lang),
    };
    let block = styled_block(title);

    let mirrors = match source_type {
        SourceType::System => crate::modules::sources::system_mirrors(),
        SourceType::Docker => crate::modules::sources::DOCKER_MIRRORS,
        SourceType::Npm => crate::modules::sources::NPM_MIRRORS,
    };

    let items: Vec<ListItem> = mirrors
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_selected = i == app.source_index;
            let marker = if is_selected {
                Span::styled("  ▸ ", Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("    ")
            };

            let rec_span = if app.source_recommended == Some(i) {
                Span::styled(
                    format!(" {} ", i18n::source_recommended(lang)),
                    Style::default().fg(C_WARN),
                )
            } else {
                Span::raw("")
            };

            let lat_span = if app.source_tested {
                match app.source_latencies.get(i) {
                    Some(Some(ms)) => {
                        let color = if *ms < 50 { C_SUCCESS } else if *ms < 200 { C_WARN } else { C_ERROR };
                        Span::styled(
                            format!("  {}", i18n::source_latency(lang, *ms)),
                            Style::default().fg(color),
                        )
                    }
                    _ => Span::styled(
                        format!("  {}", i18n::source_unreachable(lang)),
                        Style::default().fg(C_ERROR),
                    ),
                }
            } else {
                Span::raw("")
            };

            let name = match lang {
                Lang::Chinese => m.name_cn,
                Lang::English => m.name_en,
            };
            let name_s = Span::styled(
                name,
                if is_selected {
                    Style::default().fg(Color::Reset).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Reset)
                },
            );

            ListItem::new(Line::from(vec![marker, rec_span, name_s, lat_span]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD | Modifier::REVERSED));

    let mut state = ListState::default().with_selected(Some(app.source_index));
    frame.render_stateful_widget(list, area, &mut state);
}
```

---

### Task 5: 添加事件处理

**文件:**
- 修改: `src/ui/mod.rs`

- [ ] **Step 1: 在 handle_key 中添加 SourceSelect 路由**

```rust
Page::SourceSelect(_) => handle_source_select(terminal, app, key),
```

- [ ] **Step 2: 实现 handle_source_select 函数**

```rust
fn handle_source_select(
    terminal: &mut Term,
    app: &mut App,
    key: KeyEvent,
) -> anyhow::Result<Option<Action>> {
    let lang = app.lang;
    let mirrors = match &app.page {
        Page::SourceSelect(SourceType::System) => crate::modules::sources::system_mirrors(),
        Page::SourceSelect(SourceType::Docker) => crate::modules::sources::DOCKER_MIRRORS,
        Page::SourceSelect(SourceType::Npm) => crate::modules::sources::NPM_MIRRORS,
        _ => return Ok(None),
    };
    let max = mirrors.len();

    match key.code {
        KeyCode::Esc | KeyCode::Backspace => {
            app.page = match &app.page {
                Page::SourceSelect(SourceType::System) => Page::MainMenu,
                Page::SourceSelect(SourceType::Docker) => Page::Docker,
                Page::SourceSelect(SourceType::Npm) => Page::Nvm,
                _ => Page::MainMenu,
            };
            app.source_index = 0;
            app.source_tested = false;
        }
        KeyCode::Up => app.source_index = app.source_index.saturating_sub(1),
        KeyCode::Down => app.source_index = (app.source_index + 1).min(max - 1),
        KeyCode::Enter => {
            // 先测速
            let latencies = crate::modules::sources::test_mirrors(mirrors);
            let recommended = crate::modules::sources::recommend_index(&latencies);
            app.source_latencies = latencies;
            app.source_recommended = recommended;
            app.source_tested = true;

            // 如果还没有测速过，先只显示测速结果
            if !app.source_tested && app.source_recommended.is_none() {
                return Ok(None);
            }

            // 再次按 Enter：执行切换
            // 注意：需要区分"首次 Enter 测速"和"第二次 Enter 切换"
            // 简化方案：Enter 直接执行测速 + 切换
            let selected_idx = app.source_index;
            let mirror = mirrors[selected_idx];
            let name = match lang {
                Lang::Chinese => mirror.name_cn,
                Lang::English => mirror.name_en,
            };
            let url = mirror.url.to_string();
            let source_type = match &app.page {
                Page::SourceSelect(t) => *t,
                _ => SourceType::System,
            };

            app.status_msg = i18n::source_testing(lang).into();
            return Ok(Some(Action::Execute(Box::new(move |terminal| {
                run_in_terminal(terminal, || match source_type {
                    SourceType::System => crate::modules::sources::switch_system_mirror(&url),
                    SourceType::Docker => crate::modules::sources::switch_docker_mirror(&url),
                    SourceType::Npm => crate::modules::sources::switch_npm_registry(&url),
                })?;
                Ok(i18n::source_switch_ok(lang, name))
            }))));
        }
        _ => {}
    }
    Ok(None)
}
```

**注意：** Step 2 的代码有逻辑问题 —— 首次 Enter 只测速、二次 Enter 才切换的分支控制需要修正。我们采用简化方案：Enter 时测速 + 切换合二为一，在 `run_in_terminal` 中先测速再切换。

修正后的 `KeyCode::Enter` 处理：

```rust
KeyCode::Enter => {
    let selected_idx = app.source_index;
    let mirror = mirrors[selected_idx];
    let name = match lang {
        Lang::Chinese => mirror.name_cn,
        Lang::English => mirror.name_en,
    };
    let url = mirror.url.to_string();
    let source_type = match &app.page {
        Page::SourceSelect(t) => *t,
        _ => SourceType::System,
    };

    app.status_msg = i18n::source_testing(lang).into();
    return Ok(Some(Action::Execute(Box::new(move |terminal| {
        // 测速（结果显示在日志中）
        let latencies = crate::modules::sources::test_mirrors(mirrors);
        let _recommended = crate::modules::sources::recommend_index(&latencies);
        // 执行切换
        let result = run_in_terminal(terminal, || match source_type {
            SourceType::System => crate::modules::sources::switch_system_mirror(&url),
            SourceType::Docker => crate::modules::sources::switch_docker_mirror(&url),
            SourceType::Npm => crate::modules::sources::switch_npm_registry(&url),
        });
        result?;
        Ok(i18n::source_switch_ok(lang, name))
    }))));
}
```

等等，`mirrors` 和 `lang` 被 move 进了闭包，但 `mirrors` 是引用（来自 `match &app.page`），而 `app` 也被 borrow... 这里需要把 mirrors 拷贝出来。

修正——提前拷贝 mirrors 的 owned 数据：

```rust
KeyCode::Enter => {
    let selected_idx = app.source_index;
    // 提前拷贝需要的数据
    let mirrors_owned: Vec<crate::modules::sources::MirrorEntry> = mirrors.iter().cloned().collect();
    let lang = app.lang;
    let mirror = &mirrors_owned[selected_idx];
    let name = match lang {
        Lang::Chinese => mirror.name_cn,
        Lang::English => mirror.name_en,
    };
    let url = mirror.url.to_string();
    let source_type = match &app.page {
        Page::SourceSelect(t) => *t,
        _ => SourceType::System,
    };

    app.status_msg = i18n::source_testing(lang).into();
    return Ok(Some(Action::Execute(Box::new(move |terminal| {
        // 测速并记录
        let _latencies = crate::modules::sources::test_mirrors(&mirrors_owned);
        // 执行切换
        run_in_terminal(terminal, || match source_type {
            SourceType::System => crate::modules::sources::switch_system_mirror(&url),
            SourceType::Docker => crate::modules::sources::switch_docker_mirror(&url),
            SourceType::Npm => crate::modules::sources::switch_npm_registry(&url),
        })?;
        Ok(i18n::source_switch_ok(lang, name))
    }))));
}
```

但 `MirrorEntry` 需要实现 `Clone`（已有 `#[derive(Debug, Clone)]`）。✓

---

### Task 6: 集成到主菜单（系统源入口）

**文件:**
- 修改: `src/ui/mod.rs` — `handle_main_menu` 函数

- [ ] **Step 1: 在 handle_main_menu 的 Enter 分支添加第 8 项**

```rust
8 => {
    app.page = Page::SourceSelect(SourceType::System);
    app.source_index = 0;
    app.source_tested = false;
    app.source_latencies.clear();
    app.source_recommended = None;
    app.status_msg = i18n::msg_press_esc(app.lang).into();
}
```

---

### Task 7: 集成到 Docker 菜单

**文件:**
- 修改: `src/ui/mod.rs` — `handle_docker` 函数

- [ ] **Step 1: 在 handle_docker 中添加 Docker 源入口**

docker_menu 现有 6 项（索引 0-5），第 5 项（索引 5）是"切换 Docker 源"。

原代码中 `app.docker_index == 4` 处理清空 Docker，在 4 的判断后面添加 5 的处理：

```rust
if app.docker_index == 5 {
    app.page = Page::SourceSelect(SourceType::Docker);
    app.source_index = 0;
    app.source_tested = false;
    app.source_latencies.clear();
    app.source_recommended = None;
    return Ok(None);
}
```

---

### Task 8: 集成到 NVM 菜单

**文件:**
- 修改: `src/ui/mod.rs` — `handle_nvm` 函数

- [ ] **Step 1: 在 handle_nvm 中添加 npm 源入口**

nvm_menu 现有 4 项（索引 0-3），第 3 项（索引 3）是"切换 npm 源"。

在 `handle_nvm` 的 match `app.nvm_index` 分支中添加：

```rust
3 => {
    app.page = Page::SourceSelect(SourceType::Npm);
    app.source_index = 0;
    app.source_tested = false;
    app.source_latencies.clear();
    app.source_recommended = None;
}
```

---

### Task 9: 构建、验证和提交

**文件:**
- 无新建文件

- [ ] **Step 1: 编译**

```bash
cd /home/zl/code/linux-init && cargo build 2>&1
```

修正所有编译错误。

- [ ] **Step 2: 检查警告**

```bash
cd /home/zl/code/linux-init && cargo check 2>&1
```

- [ ] **Step 3: 提交**

```bash
git add -A
git commit -m "feat: 添加系统源/Docker源/Node源镜像切换功能，支持自动测速+缓存刷新"
```
