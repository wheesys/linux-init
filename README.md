# Linux Init

> 一键初始化 Linux 开发环境 — 基于 TUI 的交互式配置向导

Linux Init 是一个零依赖的命令行工具，通过图形化菜单引导用户快速完成 Linux 环境初始化，包括 shell 配置、Docker 安装、SSH 密钥生成、常用工具安装和中文环境配置。

## 功能特性

- 🐚 **Shell 配置** — 安装 zsh + Oh My Zsh，选择主题和插件，设为默认 shell
- 🐳 **Docker 安装** — 安装 Docker 引擎和 Docker Compose，配置非 root 使用
- 🔑 **SSH Key 生成** — 一键生成 Ed25519 或 RSA 密钥对
- 🔧 **基础工具** — 批量安装常用开发工具（git, neovim, tmux, ripgrep 等）
- 🇨🇳 **中文配置** — 配置中文 locale、CJK 字体和 Fcitx5 输入法

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

启动后会进入 TUI 界面，使用方向键导航：

```
主菜单
├── 🐚 Shell 配置
│   ├── 安装 Zsh
│   ├── 安装 Oh My Zsh
│   ├── 选择主题 (13+ 内置主题)
│   ├── 选择插件 (14+ 可选插件)
│   └── 设为默认 Shell
├── 🐳 Docker 安装
│   ├── 安装 Docker 引擎
│   ├── 安装 Docker Compose
│   ├── 配置非 root 用户
│   └── 启动 Docker 服务
├── 🔑 SSH Key 生成
│   ├── 生成 Ed25519 密钥 (推荐)
│   ├── 生成 RSA 4096 密钥
│   └── 查看已有公钥
├── 🔧 基础工具 (多选安装)
│   ├── 开发工具: git, curl, wget, neovim, jq
│   ├── 系统工具: htop, tmux
│   └── CLI 增强: ripgrep, fd, bat, eza
└── 🇨🇳 中文配置
    ├── 配置中文 locale
    ├── 安装中文字体 (Noto CJK)
    └── 安装 Fcitx5 输入法
```

**操作快捷键：**
- `↑` `↓` — 上下导航
- `Enter` — 确认选择
- `Space` — 切换选中状态（多选列表）
- `a` — 全选/取消全选
- `Esc` — 返回上级菜单
- `q` — 退出程序

## 技术栈

- **语言：** Rust
- **TUI 框架：** ratatui + crossterm
- **特性：** 单一静态链接二进制，零运行时依赖，编译后仅 735KB

## 项目结构

```
linux-init/
├── src/
│   ├── main.rs           # 入口和事件循环
│   ├── app.rs            # 应用状态管理
│   ├── distro/           # 发行版检测与包管理适配
│   │   ├── detect.rs     # /etc/os-release 解析
│   │   ├── pacman.rs     # Arch 系包管理
│   │   └── apt.rs        # Debian 系包管理
│   ├── modules/          # 业务逻辑模块
│   │   ├── shell.rs      # Shell 配置
│   │   ├── docker.rs     # Docker 安装
│   │   ├── ssh.rs        # SSH 密钥生成
│   │   ├── tools.rs      # 工具安装
│   │   └── locale.rs     # 中文配置
│   └── ui/               # TUI 界面渲染
│       └── mod.rs        # 界面组件和事件处理
├── Cargo.toml
└── README.md
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
