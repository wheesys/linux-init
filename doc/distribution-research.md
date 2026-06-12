# linux-init 跨发行版分发调研报告

> 调研日期: 2026-06-12
> 目的: 将 linux-init (Rust CLI) 分发到 Arch、Debian/Ubuntu、Fedora 的官方/社区仓库
> 方法: 多源交叉验证 (24 个来源, 25 条声明对抗验证, 14 条确认)

---

## 总体结论

| 发行版系列 | 推荐路径 | 门槛 | 维护成本 | 自动化 |
|------------|---------|------|---------|--------|
| Arch / CachyOS / Manjaro | **AUR** | 最低 | 低 | ✅ GitHub Actions |
| Fedora | **COPR** | 低 | 低-中 | ✅ SCM 自动构建 |
| Ubuntu / Debian | **cargo-deb + GitHub Release** | 中 | 中 | ✅ GitHub Actions |
| Ubuntu / Debian | PPA (Launchpad) | 高 | 高 | ❌ 需手动 |
| Fedora | 官方仓库 | 最高 | 最高 | ❌ 需审核 |

**推荐优先级: AUR → COPR → GitHub Release .deb → PPA → 官方仓库**

---

## 1. Arch 系: AUR

### 1.1 概述

AUR (Arch User Repository) 是社区维护的包仓库，所有 Arch 衍生版 (CachyOS/Manjaro) 通用。只需提交 PKGBUILD 脚本，无需审核。

### 1.2 PKGBUILD 结构

linux-init 是预编译二进制分发，按规范应使用 `-bin` 后缀 → 包名: **`linux-init-bin`**

四阶段流程（Arch Wiki Rust 打包指南）：

| 阶段 | 函数 | 作用 |
|------|------|------|
| prepare | `cargo fetch --locked` | 锁定依赖版本 |
| build | `cargo build --frozen --release` | 编译 |
| check | `cargo test` (非 --release) | 保留调试断言 |
| package | `install -Dm755` → `/usr/bin/` | 安装到系统 |

对于 `-bin` 包，实际上只需 `package()` 函数，直接下载 GitHub Release 中的二进制。

### 1.3 辅助工具

- **cargo-aur** (fosskers/cargo-aur, v1.7.1, 2024-03): 从 Cargo.toml 的 `[package.metadata.aur]` 段读取 depends/optdepends 自动生成 PKGBUILD
- 注意: v1.6.0 后配置段从 `[package.metadata]` 迁移到 `[package.metadata.aur]`

### 1.4 发布步骤

1. 编写 PKGBUILD
2. `makepkg --printsrcinfo > .SRCINFO`
3. 推送到 AUR git 仓库 (`ssh://aur@aur.archlinux.org/linux-init-bin.git`)
4. 可自动化: GitHub Actions 监听 tag push → 更新 PKGBUILD → 推送 AUR

### 1.5 命名规则

| 后缀 | 含义 |
|------|------|
| `-bin` | 预编译二进制（本项目适用） |
| 无后缀 | 版本化源码构建 |
| `-git` | VCS 最新构建 |

禁止与官方仓库重复；变体用 `conflicts` 数组声明。

---

## 2. Fedora 系: COPR → 官方仓库

### 2.1 COPR (推荐第一站)

COPR 是 Fedora 的社区构建服务，类似 AUR 但构建在服务器端完成。

**关键优势:**
- **不强制遵循 Fedora Packaging Guidelines** (只需行为准则 + 许可证合规)
- 支持 **Git SCM 自动构建** — 只需提供包含 `.spec` 文件的仓库
- 构建在 COPR 服务器完成，用户无需本地环境

**SCM 构建所需字段:**
- Clone URL (GitHub 仓库地址)
- Committish (分支/tag，如 `v0.2.1`)
- Subdirectory (`.spec` 文件所在子目录)
- Spec File (`.spec` 文件名)

**工具链:**
- `copr-cli` — 命令行管理 COPR 项目
- `cargo-rpm` — **已归档** (2022-07)，不可用
- 替代: `cargo-generate-rpm` 或手写 `.spec`

**注意:** SCM 默认使用 `rpkg` 方法，要求 `.spec` 使用 `{{{ }}}` 模板语法；传统 `.spec` 需切换到 `tito` 或 `make srpm` 方法。

### 2.2 Fedora 官方仓库

**要求:**
- 严格遵循 [Rust Packaging Guidelines](https://docs.fedoraproject.org/en-US/packaging-guidelines/Rust/)
- 非 crates.io 应用按 "Non-crate Rust projects" 规则打包
- 禁止源码安装到 `%{cargo_registry}` 或提供 `-devel` 子包
- **必须通过 Package Review** (数周至数月)
- **首次提交者需 sponsor 担保** (主要非技术障碍)

---

## 3. Debian/Ubuntu 系

### 3.1 cargo-deb: 快速方案

**cargo-deb** (kornelski/cargo-deb, 570 stars, 2026-05 活跃):
- `cargo deb` 单命令生成 `.deb`
- 输出: `target/debian/linux-init_*.deb`
- 支持 `--install` 直接安装

**局限性 (不能替代 PPA 基础设施):**
- ❌ 不生成源码包 (`.dsc`)
- ❌ 不对包签名 (GPG)
- ❌ 不集成 apt 仓库
- ✅ 适合 GitHub Release 附带 `.deb` 分发

### 3.2 PPA (Launchpad)

**额外工作量约是 cargo-deb 的 3-5 倍:**
1. 创建 `debian/` 目录 (control, changelog, rules, copyright, compat 等)
2. `dh_make` → 生成模板
3. `debchange` → 维护 changelog
4. `debuild -S` → 生成源码包
5. `dput` → 上传到 Launchpad
6. GPG 签名 (必须)

---

## 4. 综合推荐方案

按实施优先级排序:

### Step 1: AUR (`linux-init-bin`) — 约 1-2 天

```
GitHub Release (v*) → CI 更新 PKGBUILD → 推送 AUR
```

- [ ] 编写 PKGBUILD (预编译二进制，-bin 后缀)
- [ ] 配置 `[package.metadata.aur]` in Cargo.toml
- [ ] 添加 GitHub Actions job: `aur-publish`

### Step 2: COPR — 约 1-2 天

```
GitHub Release (v*) → SCM 触发 COPR 构建 → Fedora 用户可用
```

- [ ] 编写 `.spec` 文件
- [ ] 创建 COPR 项目
- [ ] 配置 SCM 自动构建

### Step 3: GitHub Release 附带 .deb — 约 0.5 天

- [ ] 添加 `cargo-deb` 配置
- [ ] Release workflow 增加 `.deb` 构建 + 上传

### Step 4 (可选): PPA — 约 3-5 天

### Step 5 (可选): Fedora 官方仓库 — 数周至数月

---

## 5. 已验证的关键声明

| 声明 | 置信度 | 来源 |
|------|--------|------|
| AUR 通过 PKGBUILD 分发，package() 为唯一强制函数 | 3-0 ✅ | Arch Wiki |
| Rust PKGBUILD 四阶段: prepare→build→check→package | 3-0 ✅ | Arch Wiki Rust guidelines |
| 预编译二进制包名必须以 `-bin` 后缀 | 2-1 ✅ | AUR submission guidelines |
| cargo-deb 单命令生成 .deb 但不支持源码包/PGP/apt仓库 | 2-1 ✅ | cargo-deb README |
| cargo-rpm 已于 2022-07 归档不维护 | 3-0 ✅ | GitHub cargo-rpm |
| COPR 不强制遵循 Fedora Packaging Guidelines | 3-0 ✅ | COPR 官方 FAQ |
| COPR 支持 Git SCM 自动构建 | 2-1 ✅ | COPR 文档 |
| 维护成本: AUR < COPR < PPA < Fedora 官方仓库 | 综合评估 | 多源交叉验证 |

---

## 6. 待进一步调研的问题

1. PPA 具体如何将 cargo-deb 的二进制 .deb 转化为 Launnchpad 要求的源码包？
2. linux-init 涉及用户配置文件写入，各发行版 FHS 有何特殊要求？
3. GitHub Actions 能否单次 tag push 同时触发 AUR 推送 + COPR 构建 + .deb 生成？
4. 当前 `cargo-generate-rpm` 成熟度如何？是否满足 COPR SCM 构建要求？

---

## 7. 参考来源

- [Arch Wiki: Rust package guidelines](https://wiki.archlinux.org/title/Rust_package_guidelines)
- [Arch Wiki: AUR submission guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines)
- [Arch Wiki: Creating packages](https://wiki.archlinux.org/title/Creating_packages)
- [cargo-aur (GitHub)](https://github.com/fosskers/cargo-aur)
- [cargo-deb (GitHub)](https://github.com/kornelski/cargo-deb)
- [cargo-rpm (Archived)](https://github.com/iqlusioninc/cargo-rpm)
- [Fedora Rust Packaging Guidelines](https://docs.fedoraproject.org/en-US/packaging-guidelines/Rust/)
- [COPR User Documentation](https://docs.copr.fedorainfracloud.org/user_documentation.html)
- [Fedora Discussion: COPR with Rust](https://discussion.fedoraproject.org/t/how-do-i-use-copr-with-rust-and-dependencies/91676)

---

## 8. ⚠️ 手动操作清单（待执行）

> 代码已就绪 (2026-06-12)，以下步骤需要手动操作才能激活 AUR 和 COPR。

### 8.1 AUR 首次配置

#### 8.1.1 生成专用 SSH Key

```bash
# 在本机执行（不要加密码 -N ""）
ssh-keygen -t ed25519 -C "aur-bot@linux-init" -N "" -f ~/.ssh/aur-bot-key

# 查看公钥（贴到 AUR 账户设置用）
cat ~/.ssh/aur-bot-key.pub

# 查看私钥（贴到 GitHub Secrets 用）
cat ~/.ssh/aur-bot-key
```

#### 8.1.2 注册 AUR 账户

1. 访问 https://aur.archlinux.org/register 注册账户
2. 登录后进入 https://aur.archlinux.org/account/用户名/edit
3. 在 "SSH Public Key" 字段粘贴 `aur-bot-key.pub` 的内容
4. 保存

#### 8.1.3 添加 GitHub Secret

1. 访问 https://github.com/wheesys/linux-init/settings/secrets/actions
2. 点击 "New repository secret"
3. Name: `AUR_SSH_PRIVATE_KEY`
4. Value: 粘贴 `aur-bot-key`（私钥）的全部内容
5. 点击 "Add secret"

#### 8.1.4 首次手动推送 AUR 包

```bash
# 克隆 AUR 空仓库
git clone ssh://aur@aur.archlinux.org/linux-init-bin.git /tmp/aur-linux-init-bin
cd /tmp/aur-linux-init-bin

# 从项目仓库复制 PKGBUILD 并替换占位符
cp /home/zl/code/linux-init/packaging/aur/PKGBUILD .

# 手动写入当前版本号和 sha256（以 v0.2.1 为例）
# 先从 GitHub Release 获取 sha256:
curl -fsSL https://github.com/wheesys/linux-init/releases/download/v0.2.1/linux-init-x86_64-unknown-linux-gnu.tar.gz.sha256

# 替换版本号和 sha256
sed -i 's/pkgver=__VERSION__/pkgver=0.2.1/' PKGBUILD
sed -i 's/__SHA256__/<实际sha256值>/' PKGBUILD

# 生成 .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# 提交推送
git add PKGBUILD .SRCINFO
git commit -m "Initial import: 0.2.1"
git push
```

> 首次推送后，后续每次 Release 时 `aur-publish.yml` 会自动更新。

### 8.2 COPR 首次配置

1. 用 Fedora 账户登录 https://copr.fedorainfracloud.org
2. 点击 "New Project"
3. Project name: `linux-init`
4. 在 "Packages" → "New package" → 选择 "SCM" 方式
5. Clone URL: `https://github.com/wheesys/linux-init`
6. Committish: `master`
7. Subdirectory: `packaging/copr`
8. Spec File: `linux-init.spec`
9. 保存

> SCM 构建会在 GitHub 推送后自动触发。也可在 COPR 页面手动点击 "Rebuild"。

### 8.3 验证

```bash
# AUR 验证（Arch 环境）
yay -S linux-init-bin

# COPR 验证（Fedora 环境）
sudo dnf copr enable wheesys/linux-init
sudo dnf install linux-init

# 后续更新
yay -Syu linux-init-bin   # Arch
sudo dnf update linux-init # Fedora
```
