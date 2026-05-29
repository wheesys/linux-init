该项目目标是帮助用户初始化linux环境
使用方式要尽可能少的依赖,最理想的方案是打包成各发行版可直接安装的软件.
使用TUI方式引导用户选择.
支持的发行版需要包括arch,cachyos,manjaro,ubuntu,debian
需要支持中文配置
shell环境支持bash,zsh, 由用户选择是否将zsh设置为默认shell环境
zsh 默认安装oh my zsh,帮助用户选择主题,在用户选择后修改配置文件
帮助用户选择oh my zsh插件, 选定后自动下载插件,修改配置
帮助用户安装docker,docker compose, 并将docker设定为非root用户使用
帮助用户生成ssh key
帮助用户安装基础工具
类似安装docker, 配置shell环境这种都可以独立使用.但比如配置oh my zsh要放在zsh菜单内完成.注意菜单层级.
