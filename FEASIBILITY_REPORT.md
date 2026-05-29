# Linux Init — 技术可行性报告

**日期:** 2026-05-29  
**状态:** 评估完成

---

## 一、总体评估

**结论：项目完全可行。** 所有需求均有成熟的技术方案支撑，核心挑战在于多发行版差异处理和打包分发策略。

---

## 二、编程语言与 TUI 框架选型

### 候选方案对比

| 方案 | 二进制分发 | 运行时依赖 | TUI 成熟度 | 中文支持 | 开发效率 | 跨发行版打包 |
|------|-----------|-----------|-----------|---------|---------|------------|
| **Rust + ratatui** | ✅ 单静态二进制 | ✅ 零依赖 | ⭐⭐⭐⭐ | ✅ 原生 Unicode | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Go + bubbletea** | ✅ 单静态二进制 | ✅ 零依赖 | ⭐⭐⭐⭐ | ✅ 原生 Unicode | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Python + textual | ❌ 需 runtime | ❌ 需 Python3+ | ⭐⭐⭐⭐⭐ | ✅ 优秀 | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| Shell + dialog/whiptail | ❌ 需 shell 环境 | ⚠️ 需 dialog | ⭐⭐⭐ | ⚠️ 有限 | ⭐⭐ | ⭐⭐ |

### 推荐方案：**Rust + ratatui**

**理由：**
1. **零运行时依赖** — 编译为单一静态链接二进制，完美契合"最少依赖"目标
2. **原生打包支持** — `cargo-deb` 生成 .deb，`cargo-arch` / 手写 PKGBUILD 生成 Arch 包
3. **ratatui 成熟度** — 社区活跃（GitHub 12k+ stars），组件丰富（List, Table, Form, Dialog）
4. **CJK 文本渲染** — ratatui 原生支持 Unicode 宽字符，中文 TUI 无问题
5. **安全性** — 内存安全 + 无数据竞争，系统级工具尤为重要
6. **子进程调用** — `std::process::Command` 调用系统命令（apt, pacman, docker 等）非常成熟

**备选方案：Go + bubbletea**  
如果团队更熟悉 Go，bubbletea (charmbracelet) 也是非常优秀的选择，生态更丰富（lipgloss, bubbles 组件库），打包同样简单。

---

## 三、多发行版打包方案

### 3.1 目标发行版分析

| 发行版 | 包管理器 | 包格式 | 基础 | 说明 |
|--------|---------|--------|------|------|
| **Arch Linux** | pacman | .pkg.tar.zst | 独立 | 滚动更新 |
| **CachyOS** | pacman | .pkg.tar.zst | Arch 衍生 | 基于 Arch，兼容 pacman |
| **Manjaro** | pacman | .pkg.tar.zst | Arch 衍生 | 基于 Arch，有延迟更新 |
| **Ubuntu** | apt | .deb | Debian 衍生 | LTS 版本优先 |
| **Debian** | apt | .deb | 独立 | Stable 版本 |

**关键发现：** 5 个发行版只需处理 **2 种包格式**：
- `.pkg.tar.zst` → Arch / CachyOS / Manjaro（三者共享 pacman 生态）
- `.deb` → Ubuntu / Debian

### 3.2 打包工具链

```
Rust 源码
  ├── cargo build --release  → 单一二进制
  ├── cargo-deb              → .deb 包 (Ubuntu/Debian)
  └── PKGBUILD (手写/模板)   → .pkg.tar.zst (Arch/CachyOS/Manjaro)
```

- **cargo-deb**: 在 `Cargo.toml` 中配置 `[package.metadata.deb]`，自动生成包含 man page、desktop entry 的 .deb 包
- **PKGBUILD**: 标准 Arch 打包方式，可直接提交 AUR 或自建仓库分发
- **CI/CD**: GitHub Actions 可同时构建两种包，配合 release 自动发布

### 3.3 运行时发行版检测

```rust
// 读取 /etc/os-release 获取发行版信息
// 关键字段：ID, ID_LIKE
fn detect_distro() -> Distro {
    // ID=arch, ID_LIKE=arch (CachyOS, Manjaro)
    // ID=ubuntu, ID_LIKE=debian
    // ID=debian
}
```

`/etc/os-release` 是所有目标发行版的标准接口（freedesktop 规范），可靠性极高。

---

## 四、功能模块技术可行性

### 4.1 Shell 环境配置（bash / zsh）

| 项目 | 技术路径 | 可行性 | 风险 |
|------|---------|--------|------|
| 安装 zsh | `pacman -S zsh` / `apt install zsh` | ✅ 完全可行 | 低 |
| 设为默认 shell | `chsh -s $(which zsh)` | ✅ 完全可行 | 低，需用户密码或 sudo |
| 安装 oh-my-zsh | `sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"` | ✅ 完全可行 | 中（需网络） |
| 选择主题 | 修改 `~/.zshrc` 中 `ZSH_THEME="xxx"` | ✅ 完全可行 | 低 |
| 选择插件 | `git clone` 插件到 `$ZSH_CUSTOM/plugins/` + 修改 `.zshrc` | ✅ 完全可行 | 中（需网络） |

**菜单层级设计：**
```
主菜单
├── Shell 配置
│   ├── 选择 Shell (bash / zsh)
│   ├── [zsh] Oh My Zsh 配置
│   │   ├── 选择主题
│   │   └── 选择插件
│   └── [zsh] 设为默认 Shell
├── Docker 安装
├── SSH Key 生成
└── 基础工具安装
```

**注意：** oh-my-zsh 配置（主题/插件）嵌套在 zsh 子菜单内，符合需求中"oh my zsh 要放在 zsh 菜单内完成"的要求。

### 4.2 Docker 安装

| 项目 | Arch 系 | Debian 系 | 可行性 |
|------|---------|----------|--------|
| 安装 docker | `pacman -S docker` | `apt install docker.io` 或官方仓库 | ✅ |
| 安装 docker compose | `pacman -S docker-compose` | `apt install docker-compose-plugin` | ✅ |
| 非 root 使用 | `usermod -aG docker $USER` | `usermod -aG docker $USER` | ✅ |
| 启动服务 | `systemctl enable --now docker` | `systemctl enable --now docker` | ✅ |

**注意事项：**
- Debian 系推荐使用 Docker 官方仓库（而非默认仓库中的旧版本），需添加 GPG key + sources.list
- `usermod -aG docker` 需要重新登录才生效，程序中需提示用户
- 需检测 systemd 是否可用（所有目标发行版默认均使用 systemd）

### 4.3 SSH Key 生成

| 项目 | 技术路径 | 可行性 |
|------|---------|--------|
| 生成 ed25519 key | `ssh-keygen -t ed25519 -C "comment" -f ~/.ssh/id_ed25519 -N ""` | ✅ 完全可行 |
| 生成 rsa key | `ssh-keygen -t rsa -b 4096 -C "comment" -f ~/.ssh/id_rsa -N ""` | ✅ 完全可行 |
| 显示公钥 | `cat ~/.ssh/id_*.pub` | ✅ 完全可行 |

**方案：** 提供密钥类型选择（推荐 ed25519）+ 可选密码 + 自动生成 + 显示公钥 + 可选复制到剪贴板。

### 4.4 基础工具安装

工具列表建议（可配置）：

| 工具 | Arch 包名 | Debian/Ubuntu 包名 | 说明 |
|------|----------|-------------------|------|
| git | git | git | 版本控制 |
| curl | curl | curl | HTTP 客户端 |
| wget | wget | wget | 下载工具 |
| htop | htop | htop | 进程管理 |
| neovim | neovim | neovim | 编辑器 |
| tmux | tmux | tmux | 终端复用 |
| jq | jq | jq | JSON 处理 |
| ripgrep | ripgrep | ripgrep | 搜索工具 |
| fd | fd | fd-find | 文件查找 |
| bat | bat | bat | 增强 cat |
| eza | eza | eza | 增强 ls |

**方案：** 提供分类列表（开发工具/系统工具/CLI增强），用户多选后批量安装。需维护一张「工具名 → 各发行版包名」的映射表。

### 4.5 中文配置支持

| 项目 | 技术路径 | 可行性 |
|------|---------|--------|
| 系统 locale 生成 | `locale-gen zh_CN.UTF-8` / `/etc/locale.gen` | ✅ |
| 中文字体安装 | `noto-fonts-cjk` (Arch) / `fonts-noto-cjk` (Debian) | ✅ |
| 输入法框架 | fcitx5 + fcitx5-chinese-addons | ✅ |

**注意：** TUI 本身显示中文只需终端支持 UTF-8（现代终端均支持），ratatui 原生处理宽字符。

---

## 五、系统架构设计建议

```
linux-init/
├── src/
│   ├── main.rs              # 入口，TUI 初始化
│   ├── app.rs               # 应用状态管理
│   ├── ui/                  # TUI 界面组件
│   │   ├── mod.rs
│   │   ├── menu.rs          # 主菜单
│   │   ├── shell.rs         # Shell 配置界面
│   │   ├── docker.rs        # Docker 安装界面
│   │   ├── ssh.rs           # SSH 配置界面
│   │   ├── tools.rs         # 工具安装界面
│   │   └── locale.rs        # 中文配置界面
│   ├── modules/             # 业务逻辑模块
│   │   ├── mod.rs
│   │   ├── shell.rs         # Shell/OMZ 安装与配置逻辑
│   │   ├── docker.rs        # Docker 安装逻辑
│   │   ├── ssh.rs           # SSH key 生成逻辑
│   │   ├── tools.rs         # 工具安装逻辑
│   │   └── locale.rs        # Locale 配置逻辑
│   ├── distro/              # 发行版检测与适配
│   │   ├── mod.rs
│   │   ├── detect.rs        # /etc/os-release 解析
│   │   ├── pacman.rs        # Arch 系包管理
│   │   └── apt.rs           # Debian 系包管理
│   └── utils/               # 通用工具
│       ├── mod.rs
│       ├── command.rs       # 子进程执行封装
│       └── config.rs        # 配置持久化
├── Cargo.toml
├── PKGBUILD                  # Arch 打包
└── debian/                   # Debian 打包元数据
    ├── control
    ├── rules
    └── changelog
```

**核心设计原则：**
1. **UI 与逻辑分离** — `ui/` 只负责展示和交互，`modules/` 封装所有系统操作
2. **发行版适配层** — `distro/` 抽象包管理差异，业务代码通过 trait 调用
3. **幂等性** — 每个模块执行前检测已安装状态，支持重复运行
4. **权限模型** — 仅在需要时请求 sudo（安装软件），用户级操作（配置 dotfiles）不需要

---

## 六、关键风险与缓解措施

| 风险 | 等级 | 缓解措施 |
|------|------|---------|
| 网络不可用（安装 OMZ/插件/Docker） | 🟡 中 | 每个操作前检测网络连通性，失败时友好提示 |
| sudo 权限获取 | 🟡 中 | 启动时检测权限，必要时引导用户手动执行 |
| 发行版版本差异（如 Ubuntu 20.04 vs 24.04） | 🟡 中 | 通过 os-release 版本号分支处理，CI 多版本测试 |
| 用户中断导致状态不一致 | 🟡 中 | 每步操作记录日志，支持断点恢复 |
| oh-my-zsh 上游脚本变更 | 🟢 低 | 固定 commit hash 或自建安装逻辑 |
| 终端兼容性（CJK 显示异常） | 🟢 低 | 检测 `$TERM` 和 `$LANG`，提示用户设置 UTF-8 终端 |

---

## 七、开发工作量估算

| 阶段 | 内容 | 复杂度 |
|------|------|--------|
| **Phase 1** | 项目骨架 + TUI 框架 + 菜单导航 + 发行版检测 | 中 |
| **Phase 2** | Shell 配置模块（zsh + oh-my-zsh + 主题/插件） | 中高 |
| **Phase 3** | Docker 安装模块 + SSH Key 模块 | 中 |
| **Phase 4** | 基础工具安装 + 中文配置模块 | 低 |
| **Phase 5** | 打包（deb + PKGBUILD）+ CI/CD | 中 |
| **Phase 6** | 测试（多发行版 VM 验证）+ 文档 | 中 |

---

## 八、总结与建议

| 维度 | 评估 |
|------|------|
| **技术可行性** | ✅ 完全可行，无技术阻塞点 |
| **依赖控制** | ✅ Rust 单二进制 = 零运行时依赖 |
| **多发行版覆盖** | ✅ 仅需处理 2 种包管理器（pacman/apt） |
| **TUI + 中文** | ✅ ratatui 原生支持 Unicode/CJK |
| **模块化/独立使用** | ✅ 菜单驱动 + 幂等设计天然支持 |
| **打包分发** | ✅ cargo-deb + PKGBUILD 成熟方案 |

### 推荐技术栈

```
语言:       Rust (2021 edition)
TUI 框架:   ratatui + crossterm (后端)
序列化:     serde (配置文件)
打包:       cargo-deb (Debian/Ubuntu) + PKGBUILD (Arch/CachyOS/Manjaro)
CI:         GitHub Actions (多发行版构建)
```

**项目无技术障碍，可立即启动开发。**
