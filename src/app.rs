use crate::distro::Distro;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Page {
    MainMenu,
    Shell,
    ShellZshTheme,
    ShellZshPlugins,
    Docker,
    Ssh,
    Tools,
    Locale,
    Status(Box<StatusData>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusData {
    pub title: String,
    pub lines: Vec<String>,
    pub back: Page,
}

pub struct App {
    pub distro: Distro,
    pub running: bool,
    pub page: Page,
    pub status_msg: String,

    // -- main menu --
    pub menu_index: usize,

    // -- shell --
    pub shell_index: usize,
    pub zsh_installed: bool,
    pub omz_installed: bool,
    pub omz_configured: bool,
    pub default_shell_set: bool,
    pub shell_theme_index: usize,
    pub selected_theme: String,
    pub plugin_index: usize,
    pub selected_plugins: Vec<String>,

    // -- docker --
    pub docker_index: usize,
    pub docker_installed: bool,
    pub compose_installed: bool,
    pub docker_user_configured: bool,

    // -- ssh --
    pub ssh_index: usize,
    pub ssh_key_exists: bool,
    pub last_pubkey: String,

    // -- tools --
    pub tool_index: usize,
    pub selected_tools: Vec<bool>,

    // -- locale --
    pub locale_index: usize,
    pub locale_configured: bool,
    pub fonts_installed: bool,
    pub fcitx_installed: bool,
}

pub const MAIN_MENU_ITEMS: &[(&str, &str)] = &[
    ("🐚 Shell 配置", "配置 bash/zsh, oh-my-zsh"),
    ("🐳 Docker 安装", "安装 Docker 和 Docker Compose"),
    ("🔑 SSH Key 生成", "生成 SSH 密钥对"),
    ("🔧 基础工具", "安装常用开发工具"),
    ("🇨🇳 中文配置", "配置中文 locale、字体和输入法"),
];

#[allow(dead_code)]
pub const SHELL_MENU_ITEMS: &[(&str, &str)] = &[
    ("安装 Oh My Zsh", "安装 oh-my-zsh 框架"),
    ("选择主题", "选择 oh-my-zsh 主题"),
    ("选择插件", "选择 oh-my-zsh 插件"),
    ("设为默认 Shell", "将 zsh 设为默认登录 shell"),
];

pub const THEMES: &[(&str, &str)] = &[
    ("robbyrussell", "默认主题，简洁带 git 状态"),
    ("af-magic", "双行提示，显示路径和 git 分支"),
    ("agnoster", "紧凑的 powerline 风格"),
    ("bira", "双行彩色提示，信息丰富"),
    ("clean", "极简风格，仅显示路径和符号"),
    ("dst", "经典单行提示"),
    ("gallifrey", "Doctor Who 风格主题"),
    ("maran", "简洁的单行 git 提示"),
    ("minimal", "极简主义者的选择"),
    ("powerlevel10k", "高性能、高度可定制主题 (需额外安装)"),
    ("refined", "精炼的单行提示"),
    ("suvash", "简洁带 git 状态和路径"),
    ("ys", "信息丰富的双行提示"),
];

pub const PLUGINS: &[(&str, &str, &str)] = &[
    ("git", "内置", "Git 别名和补全"),
    ("zsh-autosuggestions", "第三方", "根据历史记录自动建议补全"),
    ("zsh-syntax-highlighting", "第三方", "命令语法高亮"),
    ("docker", "内置", "Docker 命令补全"),
    ("docker-compose", "内置", "Docker Compose 命令补全"),
    ("kubectl", "内置", "Kubernetes 命令补全"),
    ("sudo", "内置", "双击 ESC 自动添加 sudo"),
    ("extract", "内置", "统一解压命令 extract"),
    ("colorize", "内置", "彩色输出增强"),
    ("copypath", "内置", "复制当前路径到剪贴板"),
    ("z", "内置", "快速跳转常用目录"),
    ("fzf", "第三方", "模糊搜索增强 (需安装 fzf)"),
    ("zsh-completions", "第三方", "额外的补全定义"),
    ("you-should-use", "第三方", "提示你有可用的别名"),
];

pub const TOOLS: &[(&str, &str, &str)] = &[
    ("git", "开发工具", "版本控制系统"),
    ("curl", "开发工具", "HTTP 客户端工具"),
    ("wget", "开发工具", "文件下载工具"),
    ("htop", "系统工具", "交互式进程管理器"),
    ("neovim", "开发工具", "现代化 Vim 编辑器"),
    ("tmux", "系统工具", "终端复用器"),
    ("jq", "开发工具", "JSON 处理器"),
    ("ripgrep", "CLI 增强", "超快速搜索工具 (rg)"),
    ("fd", "CLI 增强", "更友好的 find 替代"),
    ("bat", "CLI 增强", "带语法高亮的 cat 替代"),
    ("eza", "CLI 增强", "现代化的 ls 替代"),
];

pub const DOCKER_MENU_ITEMS: &[(&str, &str)] = &[
    ("安装 Docker", "安装 Docker 引擎"),
    ("安装 Docker Compose", "安装 Docker Compose 插件"),
    ("配置非 root 用户", "将当前用户加入 docker 组"),
    ("启动 Docker 服务", "启用并启动 docker.service"),
];

pub const SSH_MENU_ITEMS: &[(&str, &str)] = &[
    ("生成 Ed25519 密钥", "推荐 - 更安全、更快"),
    ("生成 RSA 4096 密钥", "兼容性最好"),
    ("查看已有公钥", "显示当前公钥内容"),
];

pub const LOCALE_MENU_ITEMS: &[(&str, &str)] = &[
    ("配置中文 locale", "生成 zh_CN.UTF-8 locale"),
    ("安装中文字体", "安装 Noto CJK 字体"),
    ("安装 Fcitx5 输入法", "安装 fcitx5 + 拼音输入法"),
];

impl App {
    pub fn new(distro: Distro) -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let zsh_installed = crate::distro::is_package_installed("zsh");
        let omz_installed = home.join(".oh-my-zsh").exists();
        let docker_installed = crate::distro::is_package_installed("docker");
        let compose_installed = crate::distro::is_package_installed("docker-compose");
        let ssh_key_exists = home.join(".ssh/id_ed25519.pub").exists()
            || home.join(".ssh/id_rsa.pub").exists();

        let current_shell = std::env::var("SHELL").unwrap_or_default();
        let default_shell_set = current_shell.contains("zsh");

        let selected_theme = if omz_installed {
            Self::read_current_theme(&home)
        } else {
            "robbyrussell".to_string()
        };

        let selected_plugins = if omz_installed {
            Self::read_current_plugins(&home)
        } else {
            vec!["git".to_string()]
        };

        Self {
            distro,
            running: true,
            page: Page::MainMenu,
            status_msg: format!("检测到发行版: 按 ↑↓ 导航, Enter 选择, q 退出"),

            menu_index: 0,

            shell_index: 0,
            zsh_installed,
            omz_installed,
            omz_configured: omz_installed,
            default_shell_set,
            shell_theme_index: 0,
            selected_theme,
            plugin_index: 0,
            selected_plugins,

            docker_index: 0,
            docker_installed,
            compose_installed,
            docker_user_configured: false,

            ssh_index: 0,
            ssh_key_exists,
            last_pubkey: String::new(),

            tool_index: 0,
            selected_tools: vec![false; TOOLS.len()],

            locale_index: 0,
            locale_configured: false,
            fonts_installed: false,
            fcitx_installed: false,
        }
    }

    fn read_current_theme(home: &std::path::Path) -> String {
        let zshrc = home.join(".zshrc");
        if let Ok(content) = std::fs::read_to_string(&zshrc) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("ZSH_THEME=") {
                    return trimmed
                        .trim_start_matches("ZSH_THEME=")
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string();
                }
            }
        }
        "robbyrussell".to_string()
    }

    fn read_current_plugins(home: &std::path::Path) -> Vec<String> {
        let zshrc = home.join(".zshrc");
        if let Ok(content) = std::fs::read_to_string(&zshrc) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("plugins=") {
                    let raw = trimmed
                        .trim_start_matches("plugins=")
                        .trim_matches('(')
                        .trim_matches(')')
                        .trim_matches('"');
                    return raw.split_whitespace().map(|s| s.to_string()).collect();
                }
            }
        }
        vec!["git".to_string()]
    }
}
