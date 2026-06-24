#!/bin/sh
# cafe cross-agent installer — symlinks cafe's skills + commands into every
# AI coding agent on this machine. No signup, no dependencies, no network.
#
# SKILL.md is the de-facto cross-vendor format, so for skills there is zero
# conversion — just a symlink into each agent's own skills namespace. Because
# they're symlinks into this checkout, `git -C <cafe> pull` updates every agent.
#
#   ./install.sh                 install for every agent detected under $HOME
#   ./install.sh --project .     install into a repo's per-project agent dirs
#   ./install.sh --dry-run       show what would happen, touch nothing
#   ./install.sh --copy          copy instead of symlink (Windows / no symlinks)
#
# ponytail: pure file ops. The two lossy transforms research flagged (Gemini
# command TOML, Aider flatten) are skipped — skills reach those agents natively,
# and AGENTS.md covers Aider's instruction tier. Add them only if asked.

CAFE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
MODE=user; COPY=0; DRY=0; PROJECT_DIR=.; LINKED=0

usage() {
  sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
  case $1 in
    --user) MODE=user ;;
    --project) MODE=project
      case ${2:-} in -*|"") PROJECT_DIR=. ;; *) PROJECT_DIR=$2; shift ;; esac ;;
    --copy) COPY=1 ;;
    --dry-run|-n) DRY=1 ;;
    -h|--help) usage; exit 0 ;;
    *) printf 'unknown option: %s\n\n' "$1"; usage; exit 1 ;;
  esac
  shift
done

# link SRC DEST — never clobbers a real file/dir, only refreshes its own symlinks.
link() {
  src=$1; dest=$2
  if [ "$DRY" = 1 ]; then printf '  [dry] %s -> %s\n' "$dest" "$src"; LINKED=$((LINKED+1)); return; fi
  mkdir -p -- "$(dirname -- "$dest")" 2>/dev/null || return
  if [ -L "$dest" ]; then rm -f -- "$dest"
  elif [ -e "$dest" ]; then printf '  skip (exists, not a cafe link): %s\n' "$dest"; return; fi
  if [ "$COPY" = 1 ]; then
    cp -R -- "$src" "$dest" || return
  else
    ln -s -- "$src" "$dest" 2>/dev/null || { printf '  symlink failed (%s); retry with --copy\n' "$dest"; return; }
  fi
  printf '  %s\n' "$dest"; LINKED=$((LINKED+1))
}

install_skills() {  # $1 = target skills dir
  for s in "$CAFE"/plugins/*/skills/*/; do
    [ -f "${s}SKILL.md" ] || continue
    link "${s%/}" "$1/$(basename -- "$s")"
  done
}
install_commands() {  # $1 = target commands dir
  for c in "$CAFE"/plugins/*/commands/*.md; do
    [ -f "$c" ] || continue
    link "$c" "$1/$(basename -- "$c")"
  done
}

have_dir() { [ -d "$HOME/$1" ]; }
have_bin() { command -v "$1" >/dev/null 2>&1; }
present()  { have_dir "$1" || { [ -n "$2" ] && have_bin "$2"; }; }
header()   { printf '\n• %s\n' "$1"; }

if [ "$MODE" = project ]; then
  PROJ=$(CDPATH= cd -- "$PROJECT_DIR" 2>/dev/null && pwd -P) || { printf 'no such dir: %s\n' "$PROJECT_DIR"; exit 1; }
  header "Project: $PROJ  (skills-native agents read these per-repo dirs)"
  for d in .claude/skills .agents/skills .cursor/skills .gemini/skills .cline/skills .windsurf/skills .kilocode/skills .github/skills .opencode/skills; do
    install_skills "$PROJ/$d"
  done
  for d in .claude/commands .cursor/commands .opencode/command; do
    install_commands "$PROJ/$d"
  done
else
  any=0
  present .claude          claude   && { any=1; header "Claude Code";     install_skills "$HOME/.claude/skills";          install_commands "$HOME/.claude/commands"; }
  present .codex           codex    && { any=1; header "Codex CLI";       install_skills "$HOME/.codex/skills";           install_commands "$HOME/.codex/prompts"; }
  present .gemini          gemini   && { any=1; header "Gemini CLI";      install_skills "$HOME/.gemini/skills"; }
  present .cline           ""       && { any=1; header "Cline";           install_skills "$HOME/.cline/skills"; }
  present .copilot         copilot  && { any=1; header "GitHub Copilot";  install_skills "$HOME/.copilot/skills"; }
  present .openclaw        ""       && { any=1; header "OpenClaw";        install_skills "$HOME/.openclaw/skills"; }
  present .kilocode        ""       && { any=1; header "Kilo Code";       install_skills "$HOME/.kilocode/skills"; }
  present .config/opencode opencode && { any=1; header "opencode";        install_skills "$HOME/.config/opencode/skills"; install_commands "$HOME/.config/opencode/command"; }
  # Shared .agents/skills convention — read by Codex, Zed, and Copilot.
  if present .codex codex || present .config/zed zed || have_bin zed || present .copilot copilot || have_dir .agents; then
    any=1; header ".agents/skills (Codex · Zed · Copilot)"; install_skills "$HOME/.agents/skills"
  fi
  # Cursor has no global skills dir — it is per-project only.
  present .cursor cursor && printf '\n• Cursor detected — no global skills dir; run:  %s --project <repo>\n' "$0"
  [ "$any" = 1 ] || printf 'No known agents found under %s.  Use --project <dir> to install into a repo.\n' "$HOME"
fi

printf '\n%s %d item(s).' "$([ "$DRY" = 1 ] && echo 'Would link' || echo 'Linked')" "$LINKED"
[ "$COPY" = 1 ] || printf '  Update everything with:  git -C %s pull' "$CAFE"
printf '\n'
