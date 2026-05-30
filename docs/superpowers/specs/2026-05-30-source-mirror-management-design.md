# 源切换功能设计

**日期**: 2026-05-30
**状态**: 设计中

## 概述

为 linux-init 添加系统源/Docker 源/Node 源镜像切换功能，支持自动测速推荐 + 手动选择，切换后自动刷新缓存。

## 设计决策

- **三个独立功能**：系统源、Docker 源、Node 源各自独立
- **菜单位置**：系统源 → 主菜单独立入口；Docker 源 → Docker 菜单内；Node 源 → NVM 菜单内
- **网络检测**：混合模式 — 进入页面自动测速推荐最佳源，用户可手动覆盖

## 模块结构

新增 `src/modules/sources.rs`，包含：

```
├── 通用工具
│   ├── ping_mirror()       — TCP 连接测速
│   └── recommend_mirror()  — 自动推荐最优源
├── 系统源
│   ├── switch_pacman_mirror()  — Arch 系 mirrorlist 切换
│   ├── switch_apt_mirror()     — Debian 系 sources.list 切换
│   └── refresh_system_cache()  — pacman -Syy / apt update
├── Docker 源
│   ├── switch_docker_mirror()  — daemon.json 修改
│   └── restart_docker()        — systemctl restart docker
└── Node 源
    ├── switch_npm_mirror()     — npm config set registry
    └── npm 源切换后无需额外刷新
```

## 可用镜像源

### 系统源

| 区域   | Arch (pacman)                              | Debian (apt)                                  |
|--------|--------------------------------------------|-----------------------------------------------|
| 默认   | 官方源                                      | 官方源                                         |
| 清华   | mirrors.tuna.tsinghua.edu.cn                | mirrors.tuna.tsinghua.edu.cn                   |
| 中科大 | mirrors.ustc.edu.cn                         | mirrors.ustc.edu.cn                            |
| 阿里云 | mirrors.aliyun.com                          | mirrors.aliyun.com                             |
| 腾讯云 | mirrors.cloud.tencent.com                   | mirrors.cloud.tencent.com                      |

### Docker 源

- 默认（无镜像加速）
- 阿里云：`https://<user-id>.mirror.aliyuncs.com`
- 腾讯云：`https://mirror.ccs.tencentyun.com`
- Docker China：`https://registry.docker-cn.com`

### Node 源

- 默认：`https://registry.npmjs.org`
- npmmirror：`https://registry.npmmirror.com`
- 腾讯云：`https://mirrors.cloud.tencent.com/npm/`

## UI 交互

### 流程

```
进入源页面 → 自动 ping 各源 → 显示列表（延迟 + 推荐标记）→ 用户选择 → 切换 + 自动刷新 → 显示结果
```

### 页面

- `Page::Sources` — 系统源管理（主菜单进入）
- Docker 源和 Node 源无需新 Page，在各自 handle 函数中通过 Action::Execute 执行

### 状态栏

```
↑↓ 导航  Enter 选择  Esc 返回  (自动测速推荐)
```

## 需要修改的文件

| 文件 | 修改内容 |
|------|---------|
| `src/modules/sources.rs` | **新增** — 源切换核心逻辑 |
| `src/modules/mod.rs` | 添加 `pub mod sources;` |
| `src/app.rs` | `Page` 枚举添加 `Sources`；`App` 添加 `sources_index: usize` |
| `src/ui/mod.rs` | 新增 `render_sources`；`handle_sources`；`input_number_to_page` 添加 `Page::Sources` |
| `src/i18n.rs` | 三个源菜单的国际化文本 |
| `src/modules/docker.rs` | 新增 Docker 源切换相关函数 |
| `src/modules/nvm.rs` | 新增 npm 源切换相关函数 |

## 源切换后自动操作

| 源类型   | 切换后自动操作                                    |
|----------|--------------------------------------------------|
| 系统源   | `sudo pacman -Syy` (Arch) / `sudo apt update` (Debian) |
| Docker源 | `sudo systemctl restart docker`                    |
| Node源   | 无需额外操作（npm config 即生效）                   |

## 测试要点

- Arch/CachyOS/Manjaro 上 pacman 源切换 + 缓存刷新
- Ubuntu/Debian 上 apt 源切换 + update
- Docker daemon.json 不存在时的创建和权限处理
- 网络不可达时的 fallback 提示
- npm registry 切换验证
