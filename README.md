# Linux Init

> 一键初始化 Linux 开发环境 — 基于 TUI 的交互式配置向导

Linux Init 是一个零依赖的命令行工具，通过图形化菜单引导用户快速完成 Linux 环境初始化，包括 shell 配置、Docker 安装、SSH 密钥生成、常用工具安装和中文环境配置。

## 功能特性

- 🌐 **多语言支持** — 启动时选择中文或英文界面
- 🐚 **Shell 配置** — 安装 zsh + Oh My Zsh，选择主题和插件，设为默认 shell，支持清除还原
- 🐳 **Docker 安装** — 安装 Docker 引擎和 Docker Compose，配置非 root 使用
- 🔑 **SSH Key 生成** — 一键生成 Ed25519 或 RSA 密钥对
- 🔧 **基础工具** — 批量安装常用开发工具，支持别名配置和 direnv 集成
- 🖥️ **SSH 服务** — 安装 OpenSSH Server，配置安全选项（禁止 root 登录），启动服务
- 📝 **Vim 配置** — 安装 Vim、Vundle 插件管理器，选择 12+ 实用插件
- 🟢 **Node.js (nvm)** — 安装 nvm 版本管理器，可选安装 Node.js Latest/LTS
- 🇨🇳 **中文配置** — 配置中文 locale、CJK 字体和 Fcitx5 输入法（支持 Wayland/WPS）
- 🔁 **系统源管理** — 切换 pacman/apt/Docker/NPM 镜像源，支持自动测速选优

## 支持的发行版

| 发行版 | 包管理器 | 说明 |
|--------|----------|------|
| Arch Linux | pacman | 原生支持 |
| CachyOS | pacman | Arch 衍生版 |
| Manjaro | pacman | Arch 衍生版 |
| Ubuntu | apt | 推荐 LTS 版本 |
| Debian | apt | 推荐 Stable 版本 |

## 快速开始

### 运行

```bash
# 从源码编译
cargo build --release

# 运行（需要 sudo 权限安装软件包）
sudo ./target/release/linux-init
```

### 安装（待实现）

```bash
# Arch Linux / CachyOS / Manjaro
yay -S linux-init

# Ubuntu / Debian
sudo dpkg -i linux-init_*.deb
```

## 使用指南

启动后会进入 TUI 界面。所有操作通过键盘完成，无需鼠标。

### 主菜单结构

```
语言选择 (中文 / English)

主菜单
├── 🐚 Shell 配置
│   ├── 安装 Zsh
│   ├── 安装 Oh My Zsh (支持 GitHub/Gitee 镜像自动回退)
│   ├── 选择主题 (13+ 内置主题，带预览)
│   ├── 选择插件 (14+ 可选插件，自动下载第三方插件)
│   ├── 设为默认 Shell
│   └── 🗑️ 清除 Shell 配置（还原为 bash）
├── 🐳 Docker 安装
│   ├── 安装 Docker 引擎 (Debian/Ubuntu 使用官方脚本)
│   ├── 安装 Docker Compose (已内置在官方脚本中)
│   ├── 配置非 root 用户
│   ├── 启动 Docker 服务
│   └── 🗑️ 清除 Docker 配置
├── 🔑 SSH Key 生成
│   ├── 生成 Ed25519 密钥 (推荐)
│   ├── 生成 RSA 4096 密钥
│   ├── 查看已有公钥
│   └── 🗑️ 清除已有密钥
├── 🔧 基础工具 (多选安装)
│   ├── 开发工具: git, curl, wget, neovim, jq, direnv
│   ├── 系统监控: btop, dust, duf, procs
│   ├── CLI 增强: ripgrep, fd, bat, eza, zoxide, fzf
│   ├── 配置别名 (bat → batcat 等，自动适配)
│   ├── 配置 direnv shell hook
│   └── 🗑️ 清除选中的工具
├── 🖥️ SSH 服务
│   ├── 安装 OpenSSH Server
│   ├── 配置安全选项 (禁止 root 远程登录)
│   ├── 启动 SSH 服务
│   └── 🗑️ 清除 SSH 服务
├── 📝 Vim 配置
│   ├── 安装 Vim
│   ├── 安装 Vundle 插件管理器
│   ├── 选择插件 (12+ 实用插件，含功能描述)
│   └── 🗑️ 清除 Vim 配置
├── 🟢 Node.js (nvm)
│   ├── 安装 nvm (Node Version Manager)
│   ├── 安装 Node.js Latest 版本
│   ├── 安装 Node.js LTS 版本
│   ├── 配置 shell 集成
│   └── 🗑️ 清除 nvm
├── 🇨🇳 中文配置
│   ├── 配置中文 locale
│   ├── 安装中文字体 (Noto CJK + WPS 兼容字体)
│   ├── 安装 Fcitx5 输入法
│   └── 🗑️ 清除中文配置
└── 🔁 系统源管理
    ├── 系统源 (pacman/apt) — 清华/中科大/阿里云/腾讯云 + 自动测速
    ├── Docker 镜像源 — 国内镜像加速器
    └── NPM 镜像源 — npmmirror/淘宝源
```

### 键盘操作

**全局快捷键：**

| 按键 | 功能 |
|------|------|
| `↑` `↓` / `j` `k` | 上下导航 |
| `数字键 1-9` | 快速跳转到对应菜单项 |
| `Enter` | 确认选择 / 执行操作 |
| `Space` | 切换选中状态（多选列表） |
| `a` | 全选 / 取消全选（多选列表） |
| `Esc` | 返回上级菜单 |
| `q` | 退出程序 |

**子页面操作说明：**

- **多选列表**（工具、插件等）：`Space` 勾选/取消，`a` 全选/取消全选，`Enter` 确认安装
- **数字输入框**（SSH 邮箱等）：直接输入文本，`Enter` 确认，`Esc` 取消
- **源管理页面**：进入后自动测速各镜像延迟，选择延迟最低的按 `Enter` 切换
- **清除功能**：各模块最后一项为 🗑️ 清除项，选择后二次确认执行

## 依赖

Linux Init 在运行时会使用以下系统工具和命令行程序：

### 系统自带工具（无需额外安装）

| 工具 | 用途 |
|------|------|
| sudo | 权限提升，执行安装和管理操作 |
| sh / bash | 执行 shell 脚本 |
| chown | 修改文件所有者权限 |
| getent | 获取用户主目录等账户信息 |
| systemctl | 启用/启动/查询系统服务 |
| groups | 查询用户所属组 |
| locale | 查询系统语言环境 |
| dpkg | 查询 Debian/Ubuntu 系软件包状态 |
| pacman | Arch 系软件包管理 |
| ssh-keygen | 生成 SSH 密钥对 |
| which | 检测命令是否存在 |
| chsh | 修改用户默认登录 shell |
| tee | 写入需要 root 权限的系统文件 |

### 按需自动安装的外部依赖

以下工具仅在用户使用对应功能时才会检查并自动安装：

| 工具 | 用途 | 触发功能 |
|------|------|----------|
| curl | 下载安装脚本 | Shell (Oh My Zsh)、Node.js (nvm)、Docker (官方脚本) |
| git | 克隆插件仓库 | Shell (Oh My Zsh 插件)、Vim (Vundle 及插件) |
| vim | 执行 `:PluginInstall` | Vim (插件安装) |
| snap | Ubuntu 备用包管理器 | 基础工具 (apt 不可用时自动回退) |


### 功能模块所需软件包

各功能模块通过系统包管理器自动安装的软件：

| 模块 | 安装的软件包 |
|------|------------|
| Shell 配置 | zsh |
| Docker 安装 | docker, docker-compose (Debian/Ubuntu 使用 Docker 官方脚本) |
| SSH 服务 | openssh (Arch) 或 openssh-server (Debian) |
| 中文配置 | noto-fonts-cjk (或 fonts-noto-cjk), fcitx5, fcitx5-chinese-addons, fcitx5-configtool |
| 基础工具 | 用户选择的工具，支持 apt → snap → GitHub Release 三级回退 |
| Vim 配置 | vim |
| Node.js (nvm) | nvm (curl 安装脚本) + 可选 Node.js Latest/LTS |

> **Ubuntu/Debian 工具安装策略：** 优先使用 apt 安装；apt 缺失时自动回退到 snap；snap 也失败时从 GitHub Release 下载二进制。已安装检测覆盖全部三种方式。

## 技术栈

- **语言：** Rust
- **TUI 框架：** ratatui + crossterm
- **特性：** 单一静态链接二进制，零运行时依赖，编译后仅 784KB

## 项目结构

```
linux-init/
├── src/
│   ├── main.rs           # 入口和事件循环
│   ├── app.rs            # 应用状态管理 (App 结构体、页面枚举、事件处理)
│   ├── config.rs         # 配置文件管理 (~/.config/linux-init/config.json)
│   ├── i18n.rs           # 国际化 (中英文所有界面文本)
│   ├── utils.rs          # 工具函数 (获取真实用户、shell配置路径等)
│   ├── distro/           # 发行版检测与包管理适配
│   │   ├── mod.rs        # Distro 结构体、发行版检测
│   │   ├── pacman.rs     # Arch 系包管理 (pacman)
│   │   └── apt.rs        # Debian 系包管理 (apt + snap + GitHub fallback)
│   ├── modules/          # 业务逻辑模块
│   │   ├── mod.rs        # 模块注册
│   │   ├── shell.rs      # Shell 配置 (zsh/omz/主题/插件/默认shell)
│   │   ├── docker.rs     # Docker 安装 (引擎/compose/用户组/服务)
│   │   ├── ssh.rs        # SSH 密钥生成 (Ed25519/RSA)
│   │   ├── ssh_server.rs # SSH 服务配置
│   │   ├── tools.rs      # 工具安装 (20+ 工具、别名、direnv)
│   │   ├── vim.rs        # Vim 配置 (Vim/Vundle/插件)
│   │   ├── nvm.rs        # Node.js (nvm + Node 版本安装)
│   │   ├── locale.rs     # 中文配置 (locale/字体/输入法)
│   │   └── sources.rs    # 镜像源管理 (pacman/apt/Docker/NPM + 测速)
│   └── ui/               # TUI 界面渲染
│       └── mod.rs        # 所有界面的渲染和事件处理
├── Cargo.toml
├── README.md
└── LICENSE
```

## 配置文件

程序运行后自动在 `~/.config/linux-init/config.json` 保存安装状态，下次启动时可看到各模块的完成情况。

```json
{
  "language": "zh",
  "completed": {
    "zsh_installed": true,
    "omz_installed": true,
    "zsh_theme": "agnoster",
    "zsh_plugins": ["git", "zsh-autosuggestions"],
    "zsh_default": true,
    "docker_installed": true,
    "docker_compose_installed": true,
    "docker_user_configured": true,
    "docker_service_running": true,
    "ssh_key_generated": true,
    "ssh_key_type": "ed25519",
    "tools_installed": ["git", "neovim", "ripgrep", "fd", "bat", "eza"],
    "nvm_installed": true,
    "node_installed": true,
    "...": "..."
  }
}
```

## 脚本化安装（非交互模式）

使用 `--execute` 参数传入 JSON 配置，可以在不启动 TUI 的情况下完成批量安装，适合 CI/CD 和自动化脚本：

```bash
sudo ./target/release/linux-init --execute '{
  "actions": [
    {"id": "install_zsh"},
    {"id": "install_oh_my_zsh"},
    {"id": "set_zsh_theme", "args": {"theme": "agnoster"}},
    {"id": "install_plugins", "args": {"plugins": ["git", "zsh-autosuggestions"]}},
    {"id": "install_tools", "args": {"tools": ["git","curl","ripgrep","bat","eza","fd"]}},
    {"id": "generate_ed25519_key", "args": {"email": "me@example.com"}}
  ]
}'
```

执行后 stdout 输出 JSON 结果，exit code 0 表示全部成功。

## 跨发行版自动化测试

项目使用 GitHub Actions + Docker 容器在 6 个发行版上自动测试，参见 `.github/workflows/cross-distro-test.yml`。

本地手动测试：

```bash
# 在 Arch Linux 容器中测试
docker run --rm -v "$(pwd):/build" -w /build archlinux:latest bash -c '
  pacman -Syu --noconfirm && pacman -S --noconfirm base-devel rustup && rustup default stable && \
  cargo build --release && \
  ./target/release/linux-init --execute '\''{"actions":[{"id":"install_zsh"},{"id":"install_tools","args":{"tools":["git","curl","ripgrep"]}}]}'\'
```

## 开发

```bash
# 克隆仓库
git clone git@github.com:wheesys/linux-init.git
cd linux-init

# 开发模式运行
cargo run

# 编译发布版本
cargo build --release

# 检查代码
cargo clippy

# 格式化
cargo fmt
```

## 许可证

MIT License - 详见 [LICENSE](LICENSE)
