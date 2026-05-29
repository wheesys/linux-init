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
            ("🟢 Node.js (nvm)", "安装 nvm 和 Node.js（可选版本）"),
            ("🇨🇳 中文配置", "配置中文 locale、字体和输入法（支持 Wayland/WPS）"),
        ],
        Lang::English => &[
            ("🐚 Shell Config", "Configure bash/zsh, oh-my-zsh"),
            ("🐳 Docker Setup", "Install Docker and Docker Compose"),
            ("🔑 SSH Key Gen", "Generate SSH key pair"),
            ("🔧 Basic Tools", "Install common dev tools"),
            ("🖥️ SSH Server", "Install and configure SSH server"),
            ("📝 Vim Config", "Install Vim, Vundle and plugins"),
            ("🟢 Node.js (nvm)", "Install nvm and Node.js (choose version)"),
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
            ("1. 安装 Docker", "安装 Docker 引擎"),
            ("2. 安装 Docker Compose", "安装 Docker Compose 插件"),
            ("3. 配置非 root 用户", "将当前用户加入 docker 组"),
            ("4. 启动 Docker 服务", "启用并启动 docker.service"),
            ("5. 清空 Docker", "卸载 Docker 并移除配置"),
        ],
        Lang::English => &[
            ("1. Install Docker", "Install Docker engine"),
            ("2. Install Docker Compose", "Install Docker Compose plugin"),
            ("3. Configure non-root", "Add current user to docker group"),
            ("4. Start Docker Service", "Enable and start docker.service"),
            ("5. Clear Docker", "Uninstall Docker and remove config"),
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
            ("1. 生成 Ed25519 密钥", "推荐 - 更安全、更快"),
            ("2. 生成 RSA 4096 密钥", "兼容性最好"),
            ("3. 查看已有公钥", "显示当前公钥内容"),
            ("4. 清空 SSH 密钥", "删除已生成的密钥对"),
        ],
        Lang::English => &[
            ("1. Generate Ed25519 Key", "Recommended - more secure and faster"),
            ("2. Generate RSA 4096 Key", "Best compatibility"),
            ("3. View Existing Key", "Show current public key"),
            ("4. Clear SSH Keys", "Remove generated key pairs"),
        ],
    }
}

pub fn nvm_node_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "选择 Node.js 版本",
        Lang::English => "Select Node.js Version",
    }
}

pub fn nvm_node_menu(lang: Lang) -> &'static [(&'static str, &'static str)] {
    match lang {
        Lang::Chinese => &[
            ("最新版", "安装最新稳定版 Node.js（nvm install node）"),
            ("LTS 长期支持版", "安装最新 LTS 版本 Node.js（nvm install --lts）"),
        ],
        Lang::English => &[
            ("Latest", "Install latest stable Node.js (nvm install node)"),
            ("LTS", "Install latest Long Term Support version (nvm install --lts)"),
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
        (Lang::Chinese, "trash-cli") => "安全的 rm 替代（移到回收站）",
        (Lang::Chinese, "procs") => "现代化的 ps 替代",
        (Lang::Chinese, "dust") => "现代化的 du 替代",
        (Lang::Chinese, "duf") => "现代化的 df 替代",
        (Lang::Chinese, "direnv") => "自动加载/卸载项目环境变量",
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
        (Lang::English, "trash-cli") => "Safe rm alternative (moves to trash)",
        (Lang::English, "procs") => "Modern ps replacement",
        (Lang::English, "dust") => "Modern du replacement",
        (Lang::English, "duf") => "Modern df replacement",
        (Lang::English, "direnv") => "Auto-load/unload project environment variables",
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
            ("1. 安装 OpenSSH Server", "安装 sshd 服务端"),
            ("2. 配置安全选项", "禁止 root 远程登录"),
            ("3. 启动 SSH 服务", "启用并启动 sshd 服务"),
            ("4. 清空 SSH 服务", "停止并卸载 SSH 服务"),
        ],
        Lang::English => &[
            ("1. Install OpenSSH Server", "Install sshd server"),
            ("2. Configure Security", "Disable root remote login"),
            ("3. Start SSH Service", "Enable and start sshd service"),
            ("4. Clear SSH Server", "Stop and uninstall SSH server"),
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
            ("1. 安装 Vim".into(), "终端文本编辑器".into()),
            ("2. 安装 Vundle".into(), "Vim 插件管理器".into()),
            ("3. 选择插件".into(), "选择要安装的 Vim 插件".into()),
            ("4. 优化 Vim".into(), "启用通用 Vim 优化设置".into()),
            ("5. 清空 Vim".into(), "卸载 Vim 并删除所有配置".into()),
        ],
        Lang::English => vec![
            ("1. Install Vim".into(), "Terminal text editor".into()),
            ("2. Install Vundle".into(), "Vim plugin manager".into()),
            ("3. Select Plugins".into(), "Choose Vim plugins to install".into()),
            ("4. Optimize Vim".into(), "Enable general Vim optimization settings".into()),
            ("5. Clear Vim".into(), "Uninstall Vim and remove all config".into()),
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

pub fn vim_opt_title(lang: Lang) -> &'static str {
    match lang {
        Lang::Chinese => "Vim 优化设置 (Space 切换)",
        Lang::English => "Vim Optimizations (Space to toggle)",
    }
}

pub fn vim_opt_name(lang: Lang, key: &str) -> &'static str {
    match (lang, key) {
        (Lang::Chinese, "mouse") => "启用鼠标",
        (Lang::Chinese, "scrolloff") => "滚动留白",
        (Lang::Chinese, "laststatus") => "始终显示状态行",
        (Lang::Chinese, "ignorecase") => "搜索忽略大小写",
        (Lang::Chinese, "fileformat") => "强制 Unix 换行",
        (Lang::Chinese, "cindent") => "C 风格缩进",
        (Lang::Chinese, "autoread") => "外部修改自动重载",
        (Lang::Chinese, "whichwrap") => "行首行尾光标折返",
        (Lang::Chinese, "matchtime") => "短暂高亮匹配括号",
        (Lang::Chinese, "selection") => "更好的可视选择",
        (Lang::Chinese, "guioptions") => "隐藏 GUI 滚动条",
        (Lang::Chinese, "showtabline") => "隐藏标签栏",
        (Lang::English, "mouse") => "Enable Mouse",
        (Lang::English, "scrolloff") => "Scroll Context",
        (Lang::English, "laststatus") => "Always Show Statusline",
        (Lang::English, "ignorecase") => "Ignore Case in Search",
        (Lang::English, "fileformat") => "Force Unix Line Endings",
        (Lang::English, "cindent") => "C-Style Indentation",
        (Lang::English, "autoread") => "Auto-Reload on External Change",
        (Lang::English, "whichwrap") => "Wrap Cursor at Line Boundaries",
        (Lang::English, "matchtime") => "Briefly Highlight Matching Bracket",
        (Lang::English, "selection") => "Better Visual Selection",
        (Lang::English, "guioptions") => "Hide GUI Scrollbars",
        (Lang::English, "showtabline") => "Hide Tab Line",
        _ => "",
    }
}

pub fn vim_opt_desc(lang: Lang, key: &str) -> &'static str {
    match (lang, key) {
        (Lang::Chinese, "mouse") => "在所有模式中使用鼠标（普通/可视/插入/命令）",
        (Lang::Chinese, "scrolloff") => "光标上下始终保持 5 行可见上下文",
        (Lang::Chinese, "laststatus") => "即使只有一个窗口也显示状态行",
        (Lang::Chinese, "ignorecase") => "搜索时默认忽略大小写",
        (Lang::Chinese, "fileformat") => "默认使用 LF 换行符，避免 Windows 换行问题",
        (Lang::Chinese, "cindent") => "类 C 语言的自动缩进（比 smartindent 更精确）",
        (Lang::Chinese, "autoread") => "文件被外部程序修改后自动重新加载",
        (Lang::Chinese, "whichwrap") => "方向键在行首/行尾可跳到上一行/下一行",
        (Lang::Chinese, "matchtime") => "输入匹配括号时短暂高亮 0.5 秒",
        (Lang::Chinese, "selection") => "可视选择排除最后一个字符 + 鼠标/键盘可视模式",
        (Lang::Chinese, "guioptions") => "移除 GVim 右侧/左侧滚动条和底部工具栏",
        (Lang::Chinese, "showtabline") => "只有一个标签页时隐藏标签栏",
        (Lang::English, "mouse") => "Enable mouse in all modes (normal/visual/insert/command)",
        (Lang::English, "scrolloff") => "Keep 5 visible lines above/below cursor while scrolling",
        (Lang::English, "laststatus") => "Always display status line even with a single window",
        (Lang::English, "ignorecase") => "Case-insensitive search by default",
        (Lang::English, "fileformat") => "Default to Unix LF line endings, avoid CRLF issues",
        (Lang::English, "cindent") => "C-style auto-indentation (more precise than smartindent)",
        (Lang::English, "autoread") => "Auto-reload file when modified externally",
        (Lang::English, "whichwrap") => "Allow cursor keys to wrap across line boundaries",
        (Lang::English, "matchtime") => "Briefly highlight matching bracket for 0.5 seconds",
        (Lang::English, "selection") => "Visual selection excludes last char + mouse/key visual mode",
        (Lang::English, "guioptions") => "Remove right/left scrollbars and bottom toolbar in GVim",
        (Lang::English, "showtabline") => "Hide tab line when only one tab page exists",
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
            ("1. 配置中文 locale", "生成 zh_CN.UTF-8 locale"),
            ("2. 安装中文字体", "Noto CJK + 文泉驿微米黑/正黑（兼容 WPS）"),
            ("3. 安装 Fcitx5 输入法", "拼音输入法，支持 Wayland/X11"),
            ("4. 清空中文环境", "卸载字体、输入法并删除配置"),
        ],
        Lang::English => &[
            ("1. Configure Chinese Locale", "Generate zh_CN.UTF-8 locale"),
            ("2. Install Chinese Fonts", "Noto CJK + WQY Micro Hei/Zen Hei (WPS compatible)"),
            ("3. Install Fcitx5 Input", "Pinyin input, supports Wayland/X11"),
            ("4. Clear Chinese Config", "Uninstall fonts/input and remove configs"),
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
            ("1. 安装 nvm", "Node Version Manager，管理多版本 Node.js"),
            ("2. 安装 Node.js", "选择并安装最新版或 LTS 长期支持版"),
            ("3. 清空 nvm", "卸载 nvm 并删除所有 Node.js 版本"),
        ],
        Lang::English => &[
            ("1. Install nvm", "Node Version Manager - manage multiple Node.js versions"),
            ("2. Install Node.js", "Choose between latest or LTS version"),
            ("3. Clear nvm", "Uninstall nvm and remove all Node.js versions"),
        ],
    }
}
