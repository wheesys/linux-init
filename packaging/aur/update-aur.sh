#!/bin/bash
set -euo pipefail

VERSION="${VERSION:?required}"
SHA256="${SHA256:?required}"
AUR_REPO="ssh://aur@aur.archlinux.org/linux-init-bin.git"
WORKDIR="$(mktemp -d)"

# Clone AUR repo (retry without --depth 1 if repo is empty)
git clone --depth 1 "$AUR_REPO" "$WORKDIR" 2>/dev/null || \
    git clone "$AUR_REPO" "$WORKDIR"

# Read template from project repo and substitute placeholders
sed -e "s/__VERSION__/${VERSION}/g" \
    -e "s/__SHA256__/${SHA256}/g" \
    packaging/aur/PKGBUILD > "$WORKDIR/PKGBUILD"

cd "$WORKDIR"

# Generate .SRCINFO by parsing the substituted PKGBUILD
pkgver="$(grep '^pkgver=' PKGBUILD | head -1 | cut -d= -f2)"
pkgrel="$(grep '^pkgrel=' PKGBUILD | head -1 | cut -d= -f2)"
source_="$(grep '^source=' PKGBUILD | head -1 | sed 's/^source=//')"
sha256_="$(grep '^sha256sums=' PKGBUILD | head -1 | cut -d"'" -f2)"

cat > .SRCINFO << EOF
pkgbase = linux-init-bin
	pkgdesc = Linux 环境初始化工具 - TUI 向导
	pkgver = ${pkgver}
	pkgrel = ${pkgrel}
	url = https://github.com/wheesys/linux-init
	arch = x86_64
	license = MIT
	depends =
	provides = linux-init
	conflicts = linux-init
	source = ${source_}
	sha256sums = ${sha256_}

pkgname = linux-init-bin
EOF

# Commit and push
git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add PKGBUILD .SRCINFO
if git diff --staged --quiet; then
    echo "No changes to push (already at ${VERSION})"
else
    git commit -m "Update to ${VERSION}"
    git push
    echo "Pushed ${VERSION} to AUR"
fi
