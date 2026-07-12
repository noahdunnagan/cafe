// cafe — a friendly installer for cafe's skills across every AI coding agent.
//
// Detects each agent on the machine and symlinks skills + commands into its dir;
// browse skills with descriptions, pick a subset, update, and uninstall.
// Symlinks point back into the clone, so `cafe update` (git pull) refreshes every
// agent at once.
//
// ponytail: skills + commands only. Claude plugin hooks (e.g. plainspeak's
// SessionStart) still need `/plugin install …@cafe`; noted, not reimplemented.
// Unix-only symlinks (mac/Linux).

use std::io::{self, ErrorKind};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::{env, fs};

use cliclack::{confirm, intro, log, multiselect, note, outro, outro_cancel, select, spinner};

const MARKER: &str = ".claude-plugin/marketplace.json";

fn main() {
    let code = match run() {
        Ok(()) => 0,
        // cliclack returns Interrupted on Esc / Ctrl-C.
        Err(e) if e.kind() == ErrorKind::Interrupted => {
            let _ = outro_cancel("Cancelled — nothing changed.");
            0
        }
        // cliclack returns NotConnected when there's no interactive terminal.
        Err(e) if e.kind() == ErrorKind::NotConnected => {
            eprintln!(
                "cafe needs an interactive terminal — run it directly in one, \
                 not piped, redirected, or from CI."
            );
            1
        }
        Err(e) => {
            let _ = log::error(e.to_string());
            1
        }
    };
    std::process::exit(code);
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Help and version win from any position, so `cafe install --help` shows
    // help instead of dropping the user into the live installer.
    if args.iter().any(|a| a == "-h" || a == "--help" || a == "help") {
        help();
        return Ok(());
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("cafe {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let cmd = args.first().map(String::as_str);
    // None of the commands take arguments — reject stray ones loudly rather than
    // silently ignoring them (so `cafe list junk` doesn't look like it worked).
    if matches!(cmd, Some("install" | "list" | "ls" | "update" | "clean" | "uninstall"))
        && args.len() > 1
    {
        eprintln!("cafe: `{}` takes no arguments (got `{}`)", cmd.unwrap(), args[1..].join(" "));
        std::process::exit(2);
    }

    match cmd {
        None => menu(),
        Some("install") => install(),
        Some("list" | "ls") => list(),
        Some("update") => update(),
        Some("clean") => clean(),
        Some("uninstall") => uninstall(),
        Some(other) => {
            eprintln!("cafe: unknown command '{other}'\n");
            help();
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------- flows

fn menu() -> io::Result<()> {
    let action: String = select("cafe · what would you like to do?")
        .item("install".into(), "Install skills into your AI agents", "browse & pick")
        .item("list".into(), "List available skills", "")
        .item("update".into(), "Update everything", "git pull")
        .item("clean".into(), "Remove dead links from removed skills", "")
        .item("uninstall".into(), "Remove cafe's links", "")
        .item("quit".into(), "Quit", "")
        .interact()?;
    match action.as_str() {
        "install" => install(),
        "list" => list(),
        "update" => update(),
        "clean" => clean(),
        "uninstall" => uninstall(),
        _ => Ok(()),
    }
}

fn install() -> io::Result<()> {
    intro("cafe · install")?;
    let c = ctx()?;
    let plugins = plugins(&c.root);
    if plugins.is_empty() {
        return Err(io::Error::new(ErrorKind::NotFound, "no plugins found in this checkout"));
    }
    let agents = agents(&c.home);

    // Pick skills — everything pre-checked; deselect what you don't want.
    let mut skill_pick = multiselect("Which skills?  (space toggles · enter confirms)").required(true);
    for (i, p) in plugins.iter().enumerate() {
        skill_pick = skill_pick.item(i, &p.name, truncate(&p.desc, 66));
    }
    let picked: Vec<usize> = skill_pick.initial_values((0..plugins.len()).collect()).interact()?;

    // Pick agents — detected ones pre-checked; you can opt into others.
    let detected: Vec<usize> =
        agents.iter().enumerate().filter(|(_, a)| a.detected).map(|(i, _)| i).collect();
    let mut agent_pick = multiselect("Install into which agents?").required(true);
    for (i, a) in agents.iter().enumerate() {
        agent_pick = agent_pick.item(i, a.label, if a.detected { "detected" } else { "not detected" });
    }
    let targets: Vec<usize> = agent_pick.initial_values(detected).interact()?;

    if !confirm(format!("Link {} skill(s) into {} agent(s)?", picked.len(), targets.len()))
        .initial_value(true)
        .interact()?
    {
        return Err(io::Error::new(ErrorKind::Interrupted, "cancelled"));
    }

    let sp = spinner();
    sp.start("Linking…");
    let mut linked = 0usize;
    let mut skipped: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    // One agent's permission error shouldn't abandon the rest — collect and go on.
    for &ai in &targets {
        for &pi in &picked {
            match install_plugin(&plugins[pi], &agents[ai], &c.root) {
                Ok((n, sk)) => {
                    linked += n;
                    skipped.extend(sk);
                }
                Err(e) => failures.push(format!("{}: {e}", agents[ai].label)),
            }
        }
    }
    sp.stop(format!("Linked {linked} item(s)."));

    if !failures.is_empty() {
        log::warning(format!(
            "Couldn't link {} item(s):\n  {}",
            failures.len(),
            failures.join("\n  ")
        ))?;
    }
    if !skipped.is_empty() {
        log::warning(format!(
            "Left {} path(s) alone (a real file or a non-cafe link is already there):\n  {}",
            skipped.len(),
            skipped.join("\n  ")
        ))?;
    }
    // Self-heal: drop links whose skill was removed upstream. `collect_cafe_links`
    // with include_live=false only ever returns dangling links into this checkout,
    // so removing them without a prompt is safe (they're already broken).
    let touched: Vec<PathBuf> = targets
        .iter()
        .flat_map(|&i| [agents[i].skills.clone(), agents[i].commands.clone()])
        .flatten()
        .collect();
    let pruned = collect_cafe_links(&touched, Some(&c.root), false);
    for p in &pruned {
        let _ = fs::remove_file(p);
    }
    if !pruned.is_empty() {
        log::info(format!("Cleaned {} dead link(s) from removed skills.", pruned.len()))?;
    }

    // Flag agents the user opted into that aren't actually on this machine, so a
    // fat-fingered toggle doesn't silently provision a stray dir tree.
    let undetected: Vec<&str> =
        targets.iter().map(|&i| &agents[i]).filter(|a| !a.detected).map(|a| a.label).collect();
    if !undetected.is_empty() {
        log::warning(format!(
            "Also linked into {} agent(s) not detected here: {}. Undo with `cafe uninstall` if that wasn't intended.",
            undetected.len(),
            undetected.join(", ")
        ))?;
    }

    if c.home.join(".cursor").is_dir() || have_bin("cursor") {
        log::remark("Cursor has no global skills dir — it's per-project only, not yet handled here.")?;
    }
    // The real duplicate trap: cafe is ALSO installed as a Claude plugin, so every
    // command shows up twice. Warn only when that's actually the case (not gated on
    // Conductor, which was the wrong condition).
    let claude_or_codex =
        targets.iter().any(|&i| matches!(agents[i].label, "Claude Code" | "Codex CLI"));
    if claude_or_codex && c.home.join(".claude/plugins/marketplaces/cafe").exists() {
        log::warning(
            "cafe is also installed via /plugin, so every command will appear twice. Remove one \
             side:  /plugin marketplace remove cafe   (or `cafe uninstall`).",
        )?;
    }
    if claude_or_codex && conductor_present(&c.home) {
        log::info(
            "Conductor runs Claude Code & Codex, so these show up in its / menu automatically.",
        )?;
    }

    outro("Done.  Update everything later with:  cafe update")?;
    Ok(())
}

fn list() -> io::Result<()> {
    let c = ctx()?;
    let plugins = plugins(&c.root);
    intro("cafe · skills")?;
    for p in &plugins {
        log::step(format!("{}\n{}", p.name, p.desc))?;
    }
    outro(format!("{} skill(s).  Install with:  cafe install", plugins.len()))?;
    Ok(())
}

fn update() -> io::Result<()> {
    let c = ctx()?;
    intro("cafe · update")?;
    // A ZIP download has the manifest but no .git — the most likely update fail.
    if !c.root.join(".git").exists() {
        outro_cancel(
            "This copy of cafe isn't a git clone, so it can't update itself. \
             Re-install by cloning the repo instead of downloading the ZIP.",
        )?;
        return Ok(());
    }
    let sp = spinner();
    sp.start("git pull…");
    let out = match std::process::Command::new("git")
        .arg("-C")
        .arg(&c.root)
        .args(["pull", "--ff-only"])
        .output()
    {
        Ok(out) => out,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            sp.error("can't run git");
            return Err(io::Error::other(
                "Git isn't installed (or not on PATH). Install git, then run `cafe update` again.",
            ));
        }
        Err(e) => {
            sp.error("couldn't start git");
            return Err(e);
        }
    };
    if out.status.success() {
        sp.stop("Updated. Every linked agent now sees the latest skills.");
        log::info(String::from_utf8_lossy(&out.stdout).trim())?;
        outro("Done.")?;
        Ok(())
    } else {
        sp.error("git pull failed");
        log::error(String::from_utf8_lossy(&out.stderr).trim())?;
        log::info(
            "If you edited files inside the clone, stash or discard them and retry. \
             Still stuck? Delete the folder and re-clone.",
        )?;
        Err(io::Error::other("git pull failed"))
    }
}

fn uninstall() -> io::Result<()> {
    let home = home()?;
    // Don't require a live checkout: uninstall must still work after the user
    // deleted the clone (which leaves dangling links behind).
    let root = cafe_root().ok();
    let victims = collect_cafe_links(&agent_dirs(&home), root.as_deref(), true);
    intro("cafe · uninstall")?;
    if victims.is_empty() {
        outro("No cafe links found — nothing to remove.")?;
        return Ok(());
    }
    note(
        "These cafe links will be removed",
        victims.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n"),
    )?;
    if !confirm(format!("Remove {} link(s)?", victims.len())).interact()? {
        return Err(io::Error::new(ErrorKind::Interrupted, "cancelled"));
    }
    let mut n = 0;
    for v in &victims {
        if fs::remove_file(v).is_ok() {
            n += 1;
        }
    }
    outro(format!("Removed {n} link(s)."))?;
    Ok(())
}

/// Remove dead cafe links — orphans left after a skill was renamed or removed
/// upstream (install and update don't prune on their own).
fn clean() -> io::Result<()> {
    let home = home()?;
    // Works even if the checkout is gone — dead links are what we're after.
    let root = cafe_root().ok();
    let stale = collect_cafe_links(&agent_dirs(&home), root.as_deref(), false);
    intro("cafe · clean")?;
    if stale.is_empty() {
        outro("No dead links — nothing to clean.")?;
        return Ok(());
    }
    note(
        "Dead cafe links (their skill no longer exists)",
        stale.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n"),
    )?;
    if !confirm(format!("Remove {} dead link(s)?", stale.len())).interact()? {
        return Err(io::Error::new(ErrorKind::Interrupted, "cancelled"));
    }
    let mut n = 0;
    for p in &stale {
        if fs::remove_file(p).is_ok() {
            n += 1;
        }
    }
    outro(format!("Removed {n} dead link(s)."))?;
    Ok(())
}

fn help() {
    println!(
        "\
cafe — install cafe's skills into your AI coding agents

USAGE
  cafe                interactive menu
  cafe install        browse skills and install into your agents
  cafe list           list available skills with descriptions
  cafe update         git pull — refreshes every linked agent at once
  cafe clean          remove dead links left by removed/renamed skills
  cafe uninstall      remove cafe's links
  cafe --help         this help
  cafe --version      print the version

Skills install as symlinks back into this checkout, so one update reaches
every agent. Run inside the clone, or set CAFE_HOME to point at it."
    );
}

// ---------------------------------------------------------------- model

struct Ctx {
    root: PathBuf,
    home: PathBuf,
}

fn ctx() -> io::Result<Ctx> {
    Ok(Ctx { root: cafe_root()?, home: home()? })
}

fn home() -> io::Result<PathBuf> {
    // An empty or relative HOME (e.g. `env HOME= cafe install`) would make every
    // agent path resolve under the cwd — refuse it rather than trash the cwd.
    env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "HOME is not set to an absolute path"))
}

/// A directory is the cafe checkout if it carries the marketplace manifest and
/// that manifest names "cafe" (so an unrelated Claude marketplace isn't mistaken
/// for it, which would make `cafe update` pull the wrong repo).
fn is_cafe_checkout(dir: &Path) -> bool {
    fs::read_to_string(dir.join(MARKER))
        .ok()
        .and_then(|t| json_str(&t, "name"))
        .map(|n| n == "cafe")
        .unwrap_or(false)
}

/// Canonicalize so link targets and later link-matching use one path spelling
/// (immune to a convenience symlink like ~/cafe -> ~/dev/cafe). Falls back to the
/// raw path if canonicalize fails.
fn canonical(p: PathBuf) -> PathBuf {
    fs::canonicalize(&p).unwrap_or(p)
}

/// Locate the cafe checkout: $CAFE_HOME, then walk up from cwd, then the repo
/// this binary was built in (so `cargo install --path cli` keeps working).
fn cafe_root() -> io::Result<PathBuf> {
    // An explicit override wins — but if it's wrong, say so instead of silently
    // operating on some other checkout found by the fallbacks below.
    if let Some(p) = env::var_os("CAFE_HOME") {
        let p = PathBuf::from(p);
        if is_cafe_checkout(&p) {
            return Ok(canonical(p));
        }
        return Err(io::Error::new(
            ErrorKind::NotFound,
            format!("CAFE_HOME ({}) isn't a cafe checkout — it has no cafe {MARKER}.", p.display()),
        ));
    }
    if let Ok(mut dir) = env::current_dir() {
        loop {
            if is_cafe_checkout(&dir) {
                return Ok(canonical(dir));
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if let Some(built) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        if is_cafe_checkout(built) {
            return Ok(canonical(built.to_path_buf()));
        }
    }
    Err(io::Error::new(
        ErrorKind::NotFound,
        "cafe checkout not found. Run inside the clone, or set CAFE_HOME to it.",
    ))
}

struct Plugin {
    name: String,
    desc: String,
    dir: PathBuf,
}

fn plugins(root: &Path) -> Vec<Plugin> {
    let mut v = Vec::new();
    let Ok(entries) = fs::read_dir(root.join("plugins")) else {
        return v;
    };
    for e in entries.flatten() {
        let dir = e.path();
        let Ok(txt) = fs::read_to_string(dir.join(".claude-plugin/plugin.json")) else {
            continue;
        };
        let name =
            json_str(&txt, "name").unwrap_or_else(|| e.file_name().to_string_lossy().into_owned());
        let desc = json_str(&txt, "description").unwrap_or_default();
        v.push(Plugin { name, desc, dir });
    }
    v.sort_by(|a, b| a.name.cmp(&b.name));
    v
}

struct Agent {
    label: &'static str,
    skills: Option<PathBuf>,
    commands: Option<PathBuf>,
    detected: bool,
}

/// Agent target dirs and how each agent is detected on this machine.
fn agents(home: &Path) -> Vec<Agent> {
    let d = |rel: &str| home.join(rel);
    let has = |rels: &[&str], bins: &[&str]| {
        rels.iter().any(|r| home.join(r).is_dir()) || bins.iter().any(|b| have_bin(b))
    };
    vec![
        Agent {
            label: "Claude Code",
            skills: Some(d(".claude/skills")),
            commands: Some(d(".claude/commands")),
            detected: has(&[".claude"], &["claude"]),
        },
        Agent {
            label: "Codex CLI",
            skills: Some(d(".codex/skills")),
            commands: Some(d(".codex/prompts")),
            detected: has(&[".codex"], &["codex"]),
        },
        Agent {
            label: "Gemini CLI",
            skills: Some(d(".gemini/skills")),
            commands: None,
            detected: has(&[".gemini"], &["gemini"]),
        },
        Agent {
            label: "GitHub Copilot",
            skills: Some(d(".copilot/skills")),
            commands: None,
            detected: has(&[".copilot"], &["copilot"]),
        },
        Agent {
            label: "opencode",
            skills: Some(d(".config/opencode/skills")),
            commands: Some(d(".config/opencode/command")),
            detected: has(&[".config/opencode"], &["opencode"]),
        },
        Agent {
            label: "Cline",
            skills: Some(d(".cline/skills")),
            commands: None,
            detected: has(&[".cline"], &[]),
        },
        Agent {
            label: "Kilo Code",
            skills: Some(d(".kilocode/skills")),
            commands: None,
            detected: has(&[".kilocode"], &[]),
        },
        Agent {
            label: "OpenClaw",
            skills: Some(d(".openclaw/skills")),
            commands: None,
            detected: has(&[".openclaw"], &[]),
        },
        Agent {
            label: ".agents (Codex · Zed · Copilot)",
            skills: Some(d(".agents/skills")),
            commands: None,
            detected: has(&[".agents", ".config/zed", ".codex", ".copilot"], &["zed", "codex", "copilot"]),
        },
    ]
}

// ---------------------------------------------------------------- file ops

/// Link one plugin's skills + commands into one agent. Returns (linked, skipped).
fn install_plugin(p: &Plugin, a: &Agent, root: &Path) -> io::Result<(usize, Vec<String>)> {
    let mut linked = 0;
    let mut skipped = Vec::new();
    let mut place = |src: &Path, dest: PathBuf| -> io::Result<()> {
        match link(src, &dest, root)? {
            true => linked += 1,
            false => skipped.push(dest.display().to_string()),
        }
        Ok(())
    };

    if let Some(sdir) = &a.skills {
        for skill in read_subdirs(&p.dir.join("skills")) {
            if skill.join("SKILL.md").is_file() {
                let dest = sdir.join(file_name(&skill));
                place(&skill, dest)?;
            }
        }
    }
    if let Some(cdir) = &a.commands {
        for cmd in read_files(&p.dir.join("commands"), "md") {
            let dest = cdir.join(file_name(&cmd));
            place(&cmd, dest)?;
        }
    }
    Ok((linked, skipped))
}

/// Symlink src -> dest. Refreshes a link cafe owns, but never touches a real
/// file OR a foreign symlink (someone else's) — those return false = skipped, so
/// the caller can surface them. Errors carry the offending path.
fn link(src: &Path, dest: &Path, root: &Path) -> io::Result<bool> {
    if fs::symlink_metadata(dest).is_ok() {
        if is_cafe_owned(dest, root) {
            fs::remove_file(dest)?;
        } else {
            return Ok(false); // real file or someone else's link — leave it alone
        }
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", parent.display())))?;
    }
    symlink(src, dest).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", dest.display())))?;
    Ok(true)
}

/// A symlink cafe created into `root` — live (resolves under root) or now-dangling
/// (raw target under root but its source is gone). NOT a real file and NOT a
/// foreign symlink. Falls back to the raw target because canonicalize fails on a
/// dangling link.
fn is_cafe_owned(p: &Path, root: &Path) -> bool {
    if !fs::symlink_metadata(p).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
        return false;
    }
    if fs::canonicalize(p).map(|t| t.starts_with(root)).unwrap_or(false) {
        return true;
    }
    fs::read_link(p).map(|t| t.starts_with(root) && !t.exists()).unwrap_or(false)
}

/// A dangling target with cafe's layout (…/plugins/<x>/skills|commands/<y>). Used
/// only when the checkout is gone, so uninstall/clean can still sweep dead links
/// with no root to match against.
fn looks_cafe_shaped(target: &Path) -> bool {
    let s = target.to_string_lossy();
    s.contains("/plugins/") && (s.contains("/skills/") || s.contains("/commands/"))
}

/// Cafe symlinks across `dirs`. `include_live` adds still-good links (uninstall);
/// otherwise only dead ones (clean / install self-heal). With a `root` we match
/// precisely; without one (checkout deleted) we fall back to layout shape for
/// dead links only — always safe, since those are already broken.
fn collect_cafe_links(dirs: &[PathBuf], root: Option<&Path>, include_live: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in dirs {
        for p in read_all(dir) {
            if !fs::symlink_metadata(&p).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
                continue;
            }
            let Ok(target) = fs::read_link(&p) else { continue };
            let dangling = !p.exists();
            let live = include_live
                && !dangling
                && root
                    .map(|r| fs::canonicalize(&p).map(|t| t.starts_with(r)).unwrap_or(false))
                    .unwrap_or(false);
            let dead = dangling
                && match root {
                    Some(r) => target.starts_with(r),
                    None => looks_cafe_shaped(&target),
                };
            if live || dead {
                out.push(p);
            }
        }
    }
    out
}

/// Every agent's skills + commands dir — the search space for link cleanup.
fn agent_dirs(home: &Path) -> Vec<PathBuf> {
    agents(home).into_iter().flat_map(|a| [a.skills, a.commands]).flatten().collect()
}

// ---------------------------------------------------------------- helpers

fn json_str(txt: &str, key: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(txt).ok()?.get(key)?.as_str().map(str::to_string)
}

fn have_bin(name: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|p| p.join(name).is_file()))
        .unwrap_or(false)
}

/// Conductor (macOS app) runs Claude Code & Codex — no skills dir of its own.
fn conductor_present(home: &Path) -> bool {
    Path::new("/Applications/Conductor.app").exists() || home.join(".conductor").is_dir()
}

fn file_name(p: &Path) -> std::ffi::OsString {
    p.file_name().unwrap_or_default().to_os_string()
}

fn read_subdirs(dir: &Path) -> Vec<PathBuf> {
    read_all(dir).into_iter().filter(|p| p.is_dir()).collect()
}

fn read_files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    read_all(dir).into_iter().filter(|p| p.extension().map(|x| x == ext).unwrap_or(false)).collect()
}

fn read_all(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).collect()
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n.saturating_sub(1)).collect::<String>())
    }
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let d = env::temp_dir().join(format!("cafe-test-{}-{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn link_refreshes_cafe_links_but_spares_real_files_and_foreign_links() {
        let base = scratch("link");
        let root = base.join("cafe");
        let src = root.join("plugins/p/skills/x");
        fs::create_dir_all(&src).unwrap();
        let rootc = fs::canonicalize(&root).unwrap();
        let out = base.join("out");
        fs::create_dir_all(&out).unwrap();

        // fresh cafe link, then a refresh of our own link
        let dest = out.join("x");
        assert!(link(&src, &dest, &rootc).unwrap(), "fresh link created");
        assert!(is_cafe_owned(&dest, &rootc));
        assert!(link(&src, &dest, &rootc).unwrap(), "cafe link refreshed in place");

        // a real file is never clobbered
        let real = out.join("real");
        fs::write(&real, "keep me").unwrap();
        assert!(!link(&src, &real, &rootc).unwrap(), "real file left alone");
        assert_eq!(fs::read_to_string(&real).unwrap(), "keep me");

        // a foreign symlink (points outside the checkout) is left alone too
        let mine = base.join("mine");
        fs::write(&mine, "user data").unwrap();
        let foreign = out.join("foreign");
        symlink(&mine, &foreign).unwrap();
        assert!(!link(&src, &foreign, &rootc).unwrap(), "foreign symlink left alone");
        assert_eq!(fs::read_link(&foreign).unwrap(), mine, "foreign target untouched");

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn install_plugin_links_real_skill_and_command_into_agent_dirs() {
        // Use this repo's own blueprint plugin (1 skill + 1 command).
        let repo = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()).unwrap();
        let plugin =
            Plugin { name: "blueprint".into(), desc: String::new(), dir: repo.join("plugins/blueprint") };
        let base = scratch("install");
        let agent = Agent {
            label: "fake",
            skills: Some(base.join("skills")),
            commands: Some(base.join("commands")),
            detected: true,
        };

        let (linked, skipped) = install_plugin(&plugin, &agent, &repo).unwrap();
        assert_eq!(skipped.len(), 0);
        assert_eq!(linked, 2, "one skill dir + one command file");

        let skill = base.join("skills/blueprint");
        assert!(skill.join("SKILL.md").is_file(), "skill symlink resolves to real SKILL.md");
        assert!(is_cafe_owned(&skill, &repo));
        assert!(base.join("commands").read_dir().unwrap().next().is_some(), "command was linked");

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn collect_cafe_links_classifies_live_dead_and_foreign() {
        let base = scratch("collect");
        let root = base.join("cafe");
        let src = root.join("plugins/x/skills/live");
        fs::create_dir_all(&src).unwrap();
        let rootc = fs::canonicalize(&root).unwrap();
        let dir = base.join("agent/skills");
        fs::create_dir_all(&dir).unwrap();

        let live = dir.join("live");
        let dead = dir.join("dead");
        let foreign = dir.join("foreign");
        symlink(&src, &live).unwrap(); // resolves under root
        symlink(rootc.join("plugins/x/skills/gone"), &dead).unwrap(); // dangling, under root
        let mine = base.join("mine");
        fs::write(&mine, "x").unwrap();
        symlink(&mine, &foreign).unwrap(); // not cafe's

        // dead only (clean / self-heal)
        assert_eq!(collect_cafe_links(&[dir.clone()], Some(&rootc), false), vec![dead.clone()]);
        // live + dead, foreign excluded (uninstall)
        let both = collect_cafe_links(&[dir.clone()], Some(&rootc), true);
        assert!(both.contains(&live) && both.contains(&dead) && !both.contains(&foreign));
        // checkout gone (root None): dead cafe-shaped link still caught by layout
        assert_eq!(collect_cafe_links(&[dir.clone()], None, false), vec![dead]);

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn json_str_reads_fields() {
        let j = r#"{"name":"blueprint","description":"quote \" and dash — ok"}"#;
        assert_eq!(json_str(j, "name").as_deref(), Some("blueprint"));
        assert_eq!(json_str(j, "description").as_deref(), Some("quote \" and dash — ok"));
        assert_eq!(json_str(j, "missing"), None);
    }
}
