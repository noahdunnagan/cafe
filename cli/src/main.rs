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
        Err(e) => {
            let _ = log::error(e.to_string());
            1
        }
    };
    std::process::exit(code);
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => menu(),
        Some("install") => install(),
        Some("list" | "ls") => list(),
        Some("update") => update(),
        Some("uninstall") => uninstall(),
        Some("clean") => clean(),
        Some("-h" | "--help" | "help") => {
            help();
            Ok(())
        }
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
    for &pi in &picked {
        for &ai in &targets {
            let (n, sk) = install_plugin(&plugins[pi], &agents[ai])?;
            linked += n;
            skipped.extend(sk);
        }
    }
    sp.stop(format!("Linked {linked} item(s)."));

    if !skipped.is_empty() {
        log::warning(format!(
            "Skipped {} path(s) that exist and aren't cafe links:\n  {}",
            skipped.len(),
            skipped.join("\n  ")
        ))?;
    }
    // Self-heal: drop links whose skill was removed upstream since last install.
    let touched: Vec<PathBuf> = targets
        .iter()
        .flat_map(|&i| [agents[i].skills.clone(), agents[i].commands.clone()])
        .flatten()
        .collect();
    let pruned = stale_cafe_links(&touched, &c.root);
    for p in &pruned {
        let _ = fs::remove_file(p);
    }
    if !pruned.is_empty() {
        log::info(format!("Cleaned {} dead link(s) from removed skills.", pruned.len()))?;
    }
    if c.home.join(".cursor").is_dir() || have_bin("cursor") {
        log::remark("Cursor has no global skills dir — it's per-project only, not yet handled here.")?;
    }
    // Conductor is a runner for Claude Code & Codex, not a separate skills store —
    // installing those already reaches its / menu. Say so instead of pretending
    // there's a Conductor target to link into.
    if conductor_present(&c.home)
        && targets.iter().any(|&i| matches!(agents[i].label, "Claude Code" | "Codex CLI"))
    {
        log::info(
            "Conductor detected — it runs Claude Code & Codex, so these show up in its / menu \
             automatically. If you also added cafe via /plugin, each command appears twice; keep one.",
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
    let sp = spinner();
    sp.start("git pull…");
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(&c.root)
        .args(["pull", "--ff-only"])
        .output()?;
    if out.status.success() {
        sp.stop("Updated. Every linked agent now sees the latest skills.");
        let msg = String::from_utf8_lossy(&out.stdout);
        log::info(msg.trim())?;
        outro("Done.")?;
        Ok(())
    } else {
        sp.stop("git pull failed");
        log::error(String::from_utf8_lossy(&out.stderr).trim())?;
        Err(io::Error::other("git pull failed"))
    }
}

fn uninstall() -> io::Result<()> {
    let c = ctx()?;
    let root = fs::canonicalize(&c.root)?;
    let agents = agents(&c.home);
    let mut victims: Vec<PathBuf> = Vec::new();
    for a in &agents {
        for dir in [a.skills.as_ref(), a.commands.as_ref()].into_iter().flatten() {
            for entry in read_all(dir) {
                if is_cafe_link(&entry, &root) {
                    victims.push(entry);
                }
            }
        }
    }
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
    let c = ctx()?;
    let dirs: Vec<PathBuf> = agents(&c.home)
        .into_iter()
        .flat_map(|a| [a.skills, a.commands])
        .flatten()
        .collect();
    let stale = stale_cafe_links(&dirs, &c.root);
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
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "HOME is not set"))
}

/// Locate the cafe checkout: $CAFE_HOME, then walk up from cwd, then the repo
/// this binary was built in (so `cargo install --path cli` keeps working).
fn cafe_root() -> io::Result<PathBuf> {
    if let Some(p) = env::var_os("CAFE_HOME") {
        let p = PathBuf::from(p);
        if p.join(MARKER).exists() {
            return Ok(p);
        }
    }
    if let Ok(mut dir) = env::current_dir() {
        loop {
            if dir.join(MARKER).exists() {
                return Ok(dir);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if let Some(built) = Path::new(env!("CARGO_MANIFEST_DIR")).parent() {
        if built.join(MARKER).exists() {
            return Ok(built.to_path_buf());
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
fn install_plugin(p: &Plugin, a: &Agent) -> io::Result<(usize, Vec<String>)> {
    let mut linked = 0;
    let mut skipped = Vec::new();
    let mut place = |src: &Path, dest: PathBuf| -> io::Result<()> {
        match link(src, &dest)? {
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

/// Symlink src -> dest. Refreshes an existing symlink; never clobbers a real
/// file/dir (returns false = skipped).
fn link(src: &Path, dest: &Path) -> io::Result<bool> {
    if let Ok(meta) = fs::symlink_metadata(dest) {
        if meta.file_type().is_symlink() {
            fs::remove_file(dest)?;
        } else {
            return Ok(false);
        }
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    symlink(src, dest)?;
    Ok(true)
}

/// A symlink whose resolved target lives inside the cafe checkout.
fn is_cafe_link(p: &Path, root: &Path) -> bool {
    fs::symlink_metadata(p).map(|m| m.file_type().is_symlink()).unwrap_or(false)
        && fs::canonicalize(p).map(|t| t.starts_with(root)).unwrap_or(false)
}

/// Cafe symlinks pointing into this checkout whose source no longer exists —
/// orphans from an upstream rename/removal. Uses the raw link target (not
/// canonicalize, which fails on a dangling link) so dead links are caught.
fn stale_cafe_links(dirs: &[PathBuf], root: &Path) -> Vec<PathBuf> {
    let mut stale = Vec::new();
    for dir in dirs {
        for entry in read_all(dir) {
            let is_link =
                fs::symlink_metadata(&entry).map(|m| m.file_type().is_symlink()).unwrap_or(false);
            if !is_link {
                continue;
            }
            if let Ok(target) = fs::read_link(&entry) {
                if target.starts_with(root) && !target.exists() {
                    stale.push(entry);
                }
            }
        }
    }
    stale
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
    fn link_creates_and_refreshes_but_never_clobbers_real_files() {
        let base = scratch("link");
        let src = base.join("src");
        fs::create_dir_all(&src).unwrap();

        // fresh link
        let dest = base.join("out/link");
        assert!(link(&src, &dest).unwrap(), "fresh link should be created");
        assert!(fs::symlink_metadata(&dest).unwrap().file_type().is_symlink());

        // re-linking an existing symlink refreshes it (still true = linked)
        assert!(link(&src, &dest).unwrap(), "existing symlink should refresh");

        // a real file at the destination must never be clobbered
        let real = base.join("real.md");
        fs::write(&real, "keep me").unwrap();
        assert!(!link(&src, &real).unwrap(), "real file should be skipped");
        assert_eq!(fs::read_to_string(&real).unwrap(), "keep me");

        // is_cafe_link only matches symlinks pointing inside root
        assert!(is_cafe_link(&dest, &fs::canonicalize(&base).unwrap()));
        assert!(!is_cafe_link(&real, &fs::canonicalize(&base).unwrap()));

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn install_plugin_links_real_skill_and_command_into_agent_dirs() {
        // Use this repo's own blueprint plugin (1 skill + 1 command).
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let plugin = Plugin {
            name: "blueprint".into(),
            desc: String::new(),
            dir: repo.join("plugins/blueprint"),
        };
        let base = scratch("install");
        let agent = Agent {
            label: "fake",
            skills: Some(base.join("skills")),
            commands: Some(base.join("commands")),
            detected: true,
        };

        let (linked, skipped) = install_plugin(&plugin, &agent).unwrap();
        assert_eq!(skipped.len(), 0);
        assert_eq!(linked, 2, "one skill dir + one command file");

        let skill = base.join("skills/blueprint");
        assert!(skill.join("SKILL.md").is_file(), "skill symlink resolves to real SKILL.md");
        assert!(is_cafe_link(&skill, &fs::canonicalize(repo).unwrap()));
        assert!(base.join("commands").read_dir().unwrap().next().is_some(), "command was linked");

        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn stale_cafe_links_flags_only_dead_links() {
        let base = scratch("stale");
        let root = base.join("cafe");
        let src = root.join("plugins/x/skills/live");
        fs::create_dir_all(&src).unwrap();
        let agentdir = base.join("agent/skills");
        fs::create_dir_all(&agentdir).unwrap();

        // one live link (source exists) and one dead link (source removed)
        let live = agentdir.join("live");
        let dead = agentdir.join("dead");
        symlink(&src, &live).unwrap();
        symlink(root.join("plugins/x/skills/gone"), &dead).unwrap();

        let stale = stale_cafe_links(&[agentdir.clone()], &root);
        assert_eq!(stale, vec![dead], "only the dead link is flagged");
        assert!(live.exists(), "live link untouched");

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
