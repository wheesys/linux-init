#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lang {
    Chinese,
    English,
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::Chinese => write!(f, "中文"),
            Lang::English => write!(f, "English"),
        }
    }
}

// ── Main menu ───────────────────────────────────────────────
pub fn main_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Linux Init — 环境初始化向导",
        Lang::English => "Linux Init — Environment Setup Wizard",
    }
}

pub fn main_menu(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::Chinese => &[
            ("🐚 Shell 配置", "配置 bash/zsh, oh-my-zsh"),
            ("🐳 Docker 安装", "安装 Docker 和 Docker Compose"),
            ("🔑 SSH Key 生成", "生成 SSH 密钥对"),
            ("🔧 基础工具", "安装常用开发工具"),
            ("🖥️ SSH 服务", "安装并配置 SSH 远程登录"),
            ("📝 Vim 配置", "安装 Vim, Vundle 和插件"),
            ("🟢 Node.js (nvm)", "安装 nvm 和 Node.js LTS"),
            ("🇨🇳 中文配置", "配置中文 locale、字体和输入法"),
        ],
        Lang::English => &[
            ("🐚 Shell Config", "Configure bash/zsh, oh-my-zsh"),
            ("🐳 Docker Setup", "Install Docker and Docker Compose"),
            ("🔑 SSH Key Gen", "Generate SSH key pair"),
            ("🔧 Basic Tools", "Install common dev tools"),
            ("🖥️ SSH Server", "Install and configure SSH server"),
            ("📝 Vim Config", "Install Vim, Vundle and plugins"),
            ("🟢 Node.js (nvm)", "Install nvm and Node.js LTS"),
            ("🇨🇳 Chinese Config", "Configure Chinese locale, fonts and input"),
        ],
    }
}

// ── Shell ───────────────────────────────────────────────────
pub fn shell_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Shell 配置",
        Lang::English => "Shell Configuration",
    }
}

pub fn shell_install_zsh(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Chinese => ("安装 Zsh", "现代交互式 shell"),
        Lang::English => ("Install Zsh", "Modern interactive shell"),
    }
}

pub fn shell_install_omz(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Chinese => ("安装 Oh My Zsh", "zsh 配置管理框架"),
        Lang::English => ("Install Oh My Zsh", "Zsh configuration framework"),
    }
}

pub fn shell_select_theme(lang: Lang, current: &str) -> (String, String) {
    match lang {
        Lang::Chinese => (
            "选择主题".into(),
            format!("当前: {}", current),
        ),
        Lang::English => (
            "Select Theme".into(),
            format!("Current: {}", current),
        ),
    }
}

pub fn shell_select_plugins(lang: Lang, count: usize) -> (String, String) {
    match lang {
        Lang::Chinese => (
            "选择插件".into(),
            format!("已选 {} 个", count),
        ),
        Lang::English => (
            "Select Plugins".into(),
            format!("{} selected", count),
        ),
    }
}

pub fn shell_set_default(lang: Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::Chinese => ("设为默认 Shell", "将 zsh 设为登录 shell"),
        Lang::English => ("Set Default Shell", "Set zsh as login shell"),
    }
}

// ── Themes ──────────────────────────────────────────────────
pub fn theme_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Oh My Zsh — 选择主题",
        Lang::English => "Oh My Zsh — Select Theme",
    }
}

pub fn theme_desc(lang: Lang, name: &str) -> &'static str {
    match (lang, name) {
        (Lang::Chinese, "robbyrussell") => "默认主题，简洁带 git 状态",
        (Lang::Chinese, "af-magic") => "双行提示，显示路径和 git 分支",
        (Lang::Chinese, "agnoster") => "紧凑的 powerline 风格",
        (Lang::Chinese, "bira") => "双行彩色提示，信息丰富",
        (Lang::Chinese, "clean") => "极简风格，仅显示路径和符号",
        (Lang::Chinese, "dst") => "经典单行提示",
        (Lang::Chinese, "gallifrey") => "Doctor Who 风格主题",
        (Lang::Chinese, "maran") => "简洁的单行 git 提示",
        (Lang::Chinese, "minimal") => "极简主义者的选择",
        (Lang::Chinese, "powerlevel10k") => "高性能、高度可定制主题 (需额外安装)",
        (Lang::Chinese, "refined") => "精炼的单行提示",
        (Lang::Chinese, "suvash") => "简洁带 git 状态和路径",
        (Lang::Chinese, "ys") => "信息丰富的双行提示",
        (Lang::English, "robbyrussell") => "Default theme, minimal with git status",
        (Lang::English, "af-magic") => "Two-line prompt with path and git branch",
        (Lang::English, "agnoster") => "Compact powerline style",
        (Lang::English, "bira") => "Two-line colorful prompt, info-rich",
        (Lang::English, "clean") => "Minimal, only path and symbol",
        (Lang::English, "dst") => "Classic single-line prompt",
        (Lang::English, "gallifrey") => "Doctor Who themed",
        (Lang::English, "maran") => "Clean single-line git prompt",
        (Lang::English, "minimal") => "Minimalist's choice",
        (Lang::English, "powerlevel10k") => "High-performance, highly customizable (extra install)",
        (Lang::English, "refined") => "Refined single-line prompt",
        (Lang::English, "suvash") => "Clean with git status and path",
        (Lang::English, "ys") => "Info-rich two-line prompt",
        _ => "",
    }
}

// ── OMZ Plugins ─────────────────────────────────────────────
pub fn plugin_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Oh My Zsh — 选择插件 (Space 切换)",
        Lang::English => "Oh My Zsh — Select Plugins (Space to toggle)",
    }
}

pub fn plugin_kind(lang: Lang, kind: &str) -> String {
    match (lang, kind) {
        (Lang::Chinese, "内置") => "内置".to_string(),
        (Lang::Chinese, "第三方") => "第三方".to_string(),
        (Lang::English, "内置") => "builtin".to_string(),
        (Lang::English, "第三方") => "3rd-party".to_string(),
        _ => kind.to_string(),
    }
}

pub fn plugin_desc(lang: Lang, name: &str) -> &'static str {
    match (lang, name) {
        (Lang::Chinese, "git") => "Git 别名和补全",
        (Lang::Chinese, "zsh-autosuggestions") => "根据历史记录自动建议补全",
        (Lang::Chinese, "zsh-syntax-highlighting") => "命令语法高亮",
        (Lang::Chinese, "docker") => "Docker 命令补全",
        (Lang::Chinese, "docker-compose") => "Docker Compose 命令补全",
        (Lang::Chinese, "kubectl") => "Kubernetes 命令补全",
        (Lang::Chinese, "sudo") => "双击 ESC 自动添加 sudo",
        (Lang::Chinese, "extract") => "统一解压命令 extract",
        (Lang::Chinese, "colorize") => "彩色输出增强",
        (Lang::Chinese, "copypath") => "复制当前路径到剪贴板",
        (Lang::Chinese, "z") => "快速跳转常用目录",
        (Lang::Chinese, "fzf") => "模糊搜索增强 (需安装 fzf)",
        (Lang::Chinese, "zsh-completions") => "额外的补全定义",
        (Lang::Chinese, "you-should-use") => "提示你有可用的别名",
        (Lang::English, "git") => "Git aliases and completions",
        (Lang::English, "zsh-autosuggestions") => "Auto-suggest completions from history",
        (Lang::English, "zsh-syntax-highlighting") => "Command syntax highlighting",
        (Lang::English, "docker") => "Docker command completions",
        (Lang::English, "docker-compose") => "Docker Compose command completions",
        (Lang::English, "kubectl") => "Kubernetes command completions",
        (Lang::English, "sudo") => "Double-ESC to prepend sudo",
        (Lang::English, "extract") => "Universal extract command",
        (Lang::English, "colorize") => "Colorized output enhancement",
        (Lang::English, "copypath") => "Copy current path to clipboard",
        (Lang::English, "z") => "Quick jump to frequent directories",
        (Lang::English, "fzf") => "Fuzzy search enhancement (requires fzf)",
        (Lang::English, "zsh-completions") => "Additional completion definitions",
        (Lang::English, "you-should-use") => "Notify you about available aliases",
        _ => "",
    }
}

// ── Docker ──────────────────────────────────────────────────
pub fn docker_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Docker 安装与配置",
        Lang::English => "Docker Setup",
    }
}

pub fn docker_menu(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::Chinese => &[
            ("安装 Docker", "安装 Docker 引擎"),
            ("安装 Docker Compose", "安装 Docker Compose 插件"),
            ("配置非 root 用户", "将当前用户加入 docker 组"),
            ("启动 Docker 服务", "启用并启动 docker.service"),
        ],
        Lang::English => &[
            ("Install Docker", "Install Docker engine"),
            ("Install Docker Compose", "Install Docker Compose plugin"),
            ("Configure non-root", "Add current user to docker group"),
            ("Start Docker Service", "Enable and start docker.service"),
        ],
    }
}

// ── SSH Key ─────────────────────────────────────────────────
pub fn ssh_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "SSH Key 生成",
        Lang::English => "SSH Key Generation",
    }
}

pub fn ssh_pubkey_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => " 公钥 (Esc 关闭) ",
        Lang::English => " Public Key (Esc to close) ",
    }
}

pub fn ssh_menu(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::Chinese => &[
            ("生成 Ed25519 密钥", "推荐 - 更安全、更快"),
            ("生成 RSA 4096 密钥", "兼容性最好"),
            ("查看已有公钥", "显示当前公钥内容"),
        ],
        Lang::English => &[
            ("Generate Ed25519 Key", "Recommended - more secure and faster"),
            ("Generate RSA 4096 Key", "Best compatibility"),
            ("View Existing Key", "Show current public key"),
        ],
    }
}

// ── Tools ───────────────────────────────────────────────────
pub fn tools_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "基础工具安装 (Space 切换, a 全选)",
        Lang::English => "Basic Tools (Space to toggle, a to select all)",
    }
}

pub fn tool_category(lang: Lang, cat: &str) -> String {
    match (lang, cat) {
        (Lang::Chinese, "开发工具") => "开发工具".to_string(),
        (Lang::Chinese, "系统工具") => "系统工具".to_string(),
        (Lang::Chinese, "CLI 增强") => "CLI 增强".to_string(),
        (Lang::English, "开发工具") => "dev".to_string(),
        (Lang::English, "系统工具") => "system".to_string(),
        (Lang::English, "CLI 增强") => "CLI".to_string(),
        _ => cat.to_string(),
    }
}

pub fn tool_desc(lang: Lang, name: &str) -> &'static str {
    match (lang, name) {
        (Lang::Chinese, "git") => "版本控制系统",
        (Lang::Chinese, "curl") => "HTTP 客户端工具",
        (Lang::Chinese, "wget") => "文件下载工具",
        (Lang::Chinese, "btop") => "现代化系统资源监控工具",
        (Lang::Chinese, "neovim") => "现代化 Vim 编辑器",
        (Lang::Chinese, "tmux") => "终端复用器",
        (Lang::Chinese, "jq") => "JSON 处理器",
        (Lang::Chinese, "ripgrep") => "超快速搜索工具 (rg)",
        (Lang::Chinese, "fd") => "更友好的 find 替代",
        (Lang::Chinese, "bat") => "带语法高亮的 cat 替代",
        (Lang::Chinese, "eza") => "现代化的 ls 替代",
        (Lang::English, "git") => "Version control system",
        (Lang::English, "curl") => "HTTP client tool",
        (Lang::English, "wget") => "File download tool",
        (Lang::English, "btop") => "Modern system resource monitor",
        (Lang::English, "neovim") => "Modern Vim editor",
        (Lang::English, "tmux") => "Terminal multiplexer",
        (Lang::English, "jq") => "JSON processor",
        (Lang::English, "ripgrep") => "Ultra-fast search tool (rg)",
        (Lang::English, "fd") => "User-friendly find alternative",
        (Lang::English, "bat") => "Cat clone with syntax highlighting",
        (Lang::English, "eza") => "Modern ls replacement",
        _ => "",
    }
}

// ── SSH Server ──────────────────────────────────────────────
pub fn ssh_server_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "SSH 服务配置",
        Lang::English => "SSH Server Configuration",
    }
}

pub fn ssh_server_menu(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::Chinese => &[
            ("安装 OpenSSH Server", "安装 sshd 服务端"),
            ("配置安全选项", "禁止 root 远程登录"),
            ("启动 SSH 服务", "启用并启动 sshd 服务"),
        ],
        Lang::English => &[
            ("Install OpenSSH Server", "Install sshd server"),
            ("Configure Security", "Disable root remote login"),
            ("Start SSH Service", "Enable and start sshd service"),
        ],
    }
}

// ── Vim ─────────────────────────────────────────────────────
pub fn vim_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Vim 配置",
        Lang::English => "Vim Configuration",
    }
}

pub fn vim_plugin_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Vim 插件选择 (Space 切换)",
        Lang::English => "Vim Plugins (Space to toggle)",
    }
}

pub fn vim_menu(lang: Lang) -> Vec<(String, String)> {
    match lang {
        Lang::Chinese => vec![
            ("安装 Vim".into(), "终端文本编辑器".into()),
            ("安装 Vundle".into(), "Vim 插件管理器".into()),
            ("选择插件".into(), "选择要安装的 Vim 插件".into()),
        ],
        Lang::English => vec![
            ("Install Vim".into(), "Terminal text editor".into()),
            ("Install Vundle".into(), "Vim plugin manager".into()),
            ("Select Plugins".into(), "Choose Vim plugins to install".into()),
        ],
    }
}

pub fn vim_plugin_desc(lang: Lang, name: &str) -> &'static str {
    match (lang, name) {
        (Lang::Chinese, "nerdtree") => "侧边栏文件浏览器，可视化管理目录结构",
        (Lang::Chinese, "vim-airline") => "底部状态栏美化，显示行号、模式、分支等信息",
        (Lang::Chinese, "vim-fugitive") => "在 Vim 内使用 Git 命令，查看状态、diff、blame",
        (Lang::Chinese, "nerdcommenter") => "快速注释/取消注释代码块 (gcc 注释当前行)",
        (Lang::Chinese, "vim-surround") => "快速增删改括号/引号 (cs\"' 把双引号换单引号)",
        (Lang::Chinese, "vim-commentary") => "轻量注释插件 (gc 注释选中区域)",
        (Lang::Chinese, "auto-pairs") => "自动补全成对括号和引号",
        (Lang::Chinese, "vim-gitgutter") => "在行号区显示 Git 修改标记 (+/-/~)",
        (Lang::Chinese, "vim-easymotion") => "高效光标跳转，按前缀字符快速定位到任意位置",
        (Lang::Chinese, "ctrlp.vim") => "模糊搜索文件 (Ctrl+P 快速打开文件)",
        (Lang::Chinese, "vim-markdown") => "Markdown 语法高亮和折叠",
        (Lang::Chinese, "tagbar") => "侧边栏显示代码结构/函数列表 (需 ctags)",
        (Lang::English, "nerdtree") => "Sidebar file explorer for directory navigation",
        (Lang::English, "vim-airline") => "Beautified status/tabline with mode, branch info",
        (Lang::English, "vim-fugitive") => "Git wrapper - status, diff, blame inside Vim",
        (Lang::English, "nerdcommenter") => "Quickly comment/uncomment code blocks (gcc)",
        (Lang::English, "vim-surround") => "Add/change/delete surrounding brackets/quotes",
        (Lang::English, "vim-commentary") => "Lightweight comment toggle (gc on selection)",
        (Lang::English, "auto-pairs") => "Auto-insert paired brackets and quotes",
        (Lang::English, "vim-gitgutter") => "Show Git diff markers in gutter (+/-/~)",
        (Lang::English, "vim-easymotion") => "Fast cursor jump with leader + character prefix",
        (Lang::English, "ctrlp.vim") => "Fuzzy file finder (Ctrl+P to open files)",
        (Lang::English, "vim-markdown") => "Markdown syntax highlighting and folding",
        (Lang::English, "tagbar") => "Sidebar code structure/function list (needs ctags)",
        _ => "",
    }
}

// ── Locale ──────────────────────────────────────────────────
pub fn locale_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "中文环境配置",
        Lang::English => "Chinese Environment Setup",
    }
}

pub fn locale_menu(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::Chinese => &[
            ("配置中文 locale", "生成 zh_CN.UTF-8 locale"),
            ("安装中文字体", "安装 Noto CJK 字体"),
            ("安装 Fcitx5 输入法", "安装 fcitx5 + 拼音输入法"),
        ],
        Lang::English => &[
            ("Configure Chinese Locale", "Generate zh_CN.UTF-8 locale"),
            ("Install Chinese Fonts", "Install Noto CJK fonts"),
            ("Install Fcitx5 Input", "Install fcitx5 + pinyin input method"),
        ],
    }
}

// ── Language selection ──────────────────────────────────────
pub fn lang_select_title() -> &'static str {
    "Select Language / 选择语言"
}

// ── Status bar ──────────────────────────────────────────────
pub fn statusbar_nav(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "↑↓ 导航  Enter 选择  Esc 返回  q 退出",
        Lang::English => "↑↓ nav  Enter select  Esc back  q quit",
    }
}

pub fn statusbar_main(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "↑↓ 导航  Enter 选择  q 退出",
        Lang::English => "↑↓ nav  Enter select  q quit",
    }
}

pub fn statusbar_multi(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "↑↓ 导航  Space 切换  Enter 确认  Esc 返回",
        Lang::English => "↑↓ nav  Space toggle  Enter confirm  Esc back",
    }
}

pub fn statusbar_tools(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "↑↓ 导航  Space 切换  a 全选  Enter 安装  Esc 返回",
        Lang::English => "↑↓ nav  Space toggle  a select all  Enter install  Esc back",
    }
}

pub fn statusbar_status(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "↑↓ 滚动  Esc 返回",
        Lang::English => "↑↓ scroll  Esc back",
    }
}

// ── Messages ────────────────────────────────────────────────
pub fn msg_press_esc(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "按 Esc 返回主菜单",
        Lang::English => "Press Esc to return to main menu",
    }
}

pub fn msg_installing(lang: Lang, name: &str) -> String {
    match lang {
        Lang::Chinese => format!("正在安装 {}...", name),
        Lang::English => format!("Installing {}...", name),
    }
}

pub fn msg_success(lang: Lang, name: &str) -> String {
    match lang {
        Lang::Chinese => format!("✅ {} 安装成功", name),
        Lang::English => format!("✅ {} installed successfully", name),
    }
}

pub fn msg_fail(lang: Lang, name: &str, err: &str) -> String {
    match lang {
        Lang::Chinese => format!("❌ {} 安装失败: {}", name, err),
        Lang::English => format!("❌ {} failed: {}", name, err),
    }
}

// ── NVM ─────────────────────────────────────────────────────
pub fn nvm_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Node.js 环境 (nvm)",
        Lang::English => "Node.js Environment (nvm)",
    }
}

pub fn nvm_menu(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::Chinese => &[
            ("安装 nvm", "Node Version Manager，管理多版本 Node.js"),
            ("安装 Node.js LTS", "安装最新长期支持版本的 Node.js"),
            ("配置 Shell 集成", "确保 .bashrc/.zshrc 中包含 nvm 加载脚本"),
        ],
        Lang::English => &[
            ("Install nvm", "Node Version Manager - manage multiple Node.js versions"),
            ("Install Node.js LTS", "Install latest Long Term Support version of Node.js"),
            ("Configure Shell Integration", "Ensure nvm load script is in .bashrc/.zshrc"),
        ],
    }
}
