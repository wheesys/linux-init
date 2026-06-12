Name:           linux-init
Version:        0.2.0
Release:        1%{?dist}
Summary:        Linux 环境初始化工具 - TUI 向导

License:        MIT
URL:            https://github.com/wheesys/linux-init
Source0:        https://github.com/wheesys/linux-init/releases/download/v%{version}/linux-init-x86_64-unknown-linux-gnu.tar.gz

%description
Linux Init 是一个零依赖的命令行工具，通过 TUI 图形化菜单
引导用户快速完成 Linux 环境初始化，包括 shell 配置、
Docker 安装、SSH 密钥生成、常用工具安装和中文环境配置。

%prep
%setup -q -c

%install
install -Dpm 755 linux-init %{buildroot}%{_bindir}/linux-init

%files
%license LICENSE
%{_bindir}/linux-init

%changelog
* Fri Jun 12 2026 wheesys - 0.2.0-1
- Initial package
