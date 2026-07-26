#!/bin/sh
# Idiot-proof cafe installer: gets Rust if it's missing, then builds the `cafe` CLI.
#   curl -fsSL https://raw.githubusercontent.com/noahdunnagan/cafe/main/install.sh | sh
#   ./install.sh   # from inside a checkout
set -eu

REPO="https://github.com/noahdunnagan/cafe"
MARKER=".claude-plugin/marketplace.json"
say() { printf '\033[1;36m☕️ %s\033[0m\n' "$1"; }
die() { printf '\033[1;31m☕️ %s\033[0m\n' "$1" >&2; exit 1; }

# Remember the PATH we were called with: sourcing .cargo/env below adds ~/.cargo/bin to
# ours, which would mask the case where the user's own shell still can't see the binary.
orig_path="$PATH"

command -v git >/dev/null 2>&1 || die "git isn't installed — get it first (macOS: xcode-select --install), then re-run."

# Where's the checkout? Inside one already, else clone/refresh at ~/.cafe (or $CAFE_HOME).
if [ -f "$MARKER" ]; then
    root="$(pwd)"
    in_checkout=1
else
    in_checkout=0
    root="${CAFE_HOME:-$HOME/.cafe}"
    if [ -f "$root/$MARKER" ]; then
        say "refreshing $root"
        git -C "$root" pull --ff-only || true
    else
        say "cloning cafe into $root"
        git clone "$REPO" "$root"
    fi
fi

# `cargo --version` rather than `command -v` — a rustup shim with no default toolchain
# is on PATH but can't build anything. Checked here because it decides the next branch.
if ! cargo --version >/dev/null 2>&1 && [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

# Prefer a prebuilt binary — nobody should need a Rust toolchain to run a CLI.
# Exception: running ./install.sh inside a checkout, with cargo already there, means a
# contributor who wants *their* tree installed, not whatever's tagged latest.
case "$(uname -s)-$(uname -m)" in
    Darwin-arm64)              target="aarch64-apple-darwin" ;;
    Darwin-x86_64)             target="x86_64-apple-darwin" ;;
    Linux-x86_64)              target="x86_64-unknown-linux-musl" ;;
    Linux-aarch64|Linux-arm64) target="aarch64-unknown-linux-musl" ;;
    *)                         target="" ;;
esac
if [ "$in_checkout" = 1 ] && cargo --version >/dev/null 2>&1; then
    say "building your checkout (cargo is here) rather than downloading a release."
    target=""
fi

bin=""
if [ -n "$target" ] && command -v curl >/dev/null 2>&1 && command -v tar >/dev/null 2>&1; then
    # Braced: bash 3.2 (macOS /bin/sh) reads the ellipsis bytes as part of the name.
    say "downloading the cafe binary for ${target}…"
    tmp="$(mktemp -d)"
    # To a file, not a pipe: BSD tar exits 0 on the empty stream a 404 produces, so a
    # missing release would look like a successful extract. The -f test is the real check.
    url="$REPO/releases/latest/download/cafe-$target.tar.gz"
    if curl -fsSL -o "$tmp/cafe.tar.gz" "$url" 2>/dev/null &&
       tar xzf "$tmp/cafe.tar.gz" -C "$tmp" 2>/dev/null && [ -f "$tmp/cafe" ]; then
        bin="${CAFE_BIN:-$HOME/.local/bin}"
        mkdir -p "$bin"
        mv "$tmp/cafe" "$bin/cafe"
        chmod +x "$bin/cafe"
    else
        say "no prebuilt binary for this platform yet — building from source."
    fi
    rm -rf "$tmp"
fi

if [ -z "$bin" ]; then
    if ! cargo --version >/dev/null 2>&1; then
        command -v curl >/dev/null 2>&1 || die "curl isn't installed — needed to fetch Rust. Install curl and re-run."
        say "Rust not found — installing it via rustup (https://rustup.rs)…"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        . "$HOME/.cargo/env"
    fi

    say "building the CLI…"
    cargo install --path "$root/cli"
    bin="${CARGO_INSTALL_ROOT:-$HOME/.cargo}/bin"
fi

# Does typing `cafe` actually reach what we just installed? Two ways it doesn't:
# the dir isn't on PATH, or an older cafe shadows it — the download path installs to
# ~/.local/bin, so anyone who previously built from source has one in ~/.cargo/bin too.
# Worth being noisy about: under zsh's AUTO_CD, typing `cafe` next to a cafe/ dir
# silently cd's there instead of erroring, which looks like an install that does nothing.
found="$(PATH="$orig_path" command -v cafe 2>/dev/null || true)"
if [ -n "$found" ] && [ "$found" -ef "$bin/cafe" ]; then
    say "done — run:  cafe"
elif [ -n "$found" ]; then
    say "done, but an older cafe at $found shadows the one just installed
   in $bin. Remove the stale one:
     rm \"$found\"
   then open a new shell and run:  cafe"
else
    say "done, but $bin isn't on your PATH. Add it to your shell rc:
     export PATH=\"$bin:\$PATH\"
   then open a new shell and run:  cafe"
fi
