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
else
    root="${CAFE_HOME:-$HOME/.cafe}"
    if [ -f "$root/$MARKER" ]; then
        say "refreshing $root"
        git -C "$root" pull --ff-only || true
    else
        say "cloning cafe into $root"
        git clone "$REPO" "$root"
    fi
fi

# Ensure a working cargo. `cargo --version` rather than `command -v` — a rustup shim
# with no default toolchain is on PATH but can't build anything.
if ! cargo --version >/dev/null 2>&1 && [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi
if ! cargo --version >/dev/null 2>&1; then
    command -v curl >/dev/null 2>&1 || die "curl isn't installed — needed to fetch Rust. Install curl and re-run."
    say "Rust not found — installing it via rustup (https://rustup.rs)…"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    . "$HOME/.cargo/env"
fi

say "building the CLI…"
cargo install --path "$root/cli"

# A fresh rustup, or a shell that never had ~/.cargo/bin, leaves the binary unreachable.
# Say so — under zsh's AUTO_CD, typing `cafe` next to a cafe/ dir silently cd's there
# instead of erroring, which looks exactly like a working install that does nothing.
bin="${CARGO_INSTALL_ROOT:-$HOME/.cargo}/bin"
case ":$orig_path:" in
    *":$bin:"*) say "done — run:  cafe" ;;
    *) say "done, but $bin isn't on your PATH. Add it to your shell rc:
     export PATH=\"$bin:\$PATH\"
   then open a new shell and run:  cafe" ;;
esac
