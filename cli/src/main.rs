// cafe — a friendly installer for cafe's skills across every AI coding agent.
//
// Detects each agent on the machine and symlinks skills + commands into its dir;
// browse skills with descriptions, pick a subset, update, and uninstall.
// Symlinks point back into the clone, so `cafe update` (git pull) refreshes every
// agent at once.
//
// Plugins that ship a hooks/hooks.json (the "always on via a SessionStart hook"
// ones) also get those hooks merged into Claude Code's settings.json — cafe used
// to link the skill and silently drop the hook, which made the skill inert.
// Unix-only symlinks (mac/Linux).

use std::io::{self, ErrorKind};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::{env, fs};

use cliclack::{confirm, intro, log, multiselect, note, outro, outro_cancel, select, spinner};
use serde_json::{Map, Value};

const MARKER: &str = ".claude-plugin/marketplace.json";
/// The one agent that reads hooks out of a settings file rather than loading them
/// from a plugin dir, so it's the only place cafe writes hooks.
const CLAUDE: &str = "Claude Code";

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
    if matches!(cmd, Some("install" | "list" | "ls" | "update" | "clean" | "uninstall" | "doctor"))
        && args.len() > 1
    {
        eprintln!("cafe: `{}` takes no arguments (got `{}`)", cmd.unwrap(), args[1..].join(" "));
        std::process::exit(2);
    }

    match cmd {
        None => menu(),
        Some("install") => install(),
        Some("list" | "ls") => list(),
        Some("doctor") => doctor(),
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
    // A missing checkout can't install/list/update; offer only what works, and say why once.
    let has_checkout = cafe_root().is_ok();
    if !has_checkout {
        log::warning(
            "cafe checkout not found — hiding install/list/update. Run inside the clone or set \
             CAFE_HOME to restore them; clean/uninstall still work without it.",
        )?;
    }
    let mut selected = if has_checkout { "install" } else { "clean" }.to_string();
    loop {
        let mut sel = select("cafe · what would you like to do?").initial_value(selected.clone());
        if has_checkout {
            sel = sel
                .item("install".into(), "Install skills into your AI agents", "browse & pick")
                .item("list".into(), "List available skills", "")
                .item("doctor".into(), "Check what actually landed", "skills + hooks")
                .item("update".into(), "Update everything", "git pull");
        }
        selected = sel
            .item("clean".into(), "Remove dead links from removed skills", "")
            .item("uninstall".into(), "Remove cafe's links", "")
            .item("quit".into(), "Quit", "")
            .interact()?;
        let outcome = match selected.as_str() {
            "install" => install(),
            "list" => list(),
            "doctor" => doctor(),
            "update" => update(),
            "clean" => clean(),
            "uninstall" => uninstall(),
            _ => return Ok(()),
        };
        // Cancelling or erroring inside an action drops back to the menu, not out of the app.
        match outcome {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::Interrupted => {
                outro_cancel("Cancelled — nothing changed.")?
            }
            Err(e) => log::error(e.to_string())?,
        }
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

    // Remember what was unchecked. It's the one fact the filesystem can't tell us
    // later: an absent plugin looks identical whether it was declined here or added
    // upstream after the fact, and `update` needs to treat those opposite ways.
    let declined: Vec<String> = plugins
        .iter()
        .enumerate()
        .filter(|(i, _)| !picked.contains(i))
        .map(|(_, p)| p.name.clone())
        .collect();
    if let Err(e) = set_declined(&c.home, &declined) {
        log::warning(format!("Couldn't record your selection ({e}); `cafe update` will link everything."))?;
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

    // Skills alone leave the always-on plugins inert — their SessionStart hook has
    // to be in settings.json or nothing injects them.
    if targets.iter().any(|&i| agents[i].label == CLAUDE) {
        let want: Vec<&Plugin> = picked.iter().map(|&i| &plugins[i]).collect();
        match sync_hooks(&c.home, &c.root, &want) {
            Ok(h) if h.changed => log::info(format!(
                "Wired {} always-on hook(s) into ~/.claude/settings.json{}.",
                h.added,
                if h.stale > 0 { format!(", dropped {} stale one(s)", h.stale) } else { String::new() }
            ))?,
            Ok(_) => {}
            Err(e) => log::warning(format!("Left settings.json alone — {e}"))?,
        }
    }

    verify_after(&c)?;

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
        sp.stop("Pulled.");
        log::info(String::from_utf8_lossy(&out.stdout).trim())?;
        relink(&c)?;
        outro("Done. Every linked agent now sees the latest skills.")?;
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

/// Re-link every plugin the user didn't decline, then sweep what's still dead.
///
/// This used to relink only the plugins an agent already had, read back off its own
/// links — which meant a plugin added upstream since your last `cafe install` was
/// never in that set and so never arrived, forever, silently. Now the set is
/// "everything except what you unchecked", so a new plugin lands on the next pull.
/// A pull can also move a command between plugins; the old link then dangles, and
/// agents skip dangling links without a word, so relinking has to happen regardless.
fn relink(c: &Ctx) -> io::Result<()> {
    let plugins = plugins(&c.root);
    let declined = declined(&c.home);
    let want: Vec<&Plugin> = plugins.iter().filter(|p| !declined.contains(&p.name)).collect();
    let sp = spinner();
    sp.start("Relinking…");
    let mut linked = 0usize;
    let mut claude = false;
    for a in agents(&c.home) {
        let dirs: Vec<PathBuf> =
            [a.skills.clone(), a.commands.clone()].into_iter().flatten().collect();
        // An agent with no cafe links was never installed into — leave it that way.
        if collect_cafe_links(&dirs, Some(&c.root), true).is_empty() {
            continue;
        }
        claude |= a.label == CLAUDE;
        // A permission error on one agent shouldn't sink the whole update.
        for p in &want {
            if let Ok((n, _)) = install_plugin(p, &a, &c.root) {
                linked += n;
            }
        }
    }
    // Whatever still dangles belongs to a skill that's gone for good.
    let dead = collect_cafe_links(&agent_dirs(&c.home), Some(&c.root), false);
    for p in &dead {
        let _ = fs::remove_file(p);
    }
    sp.stop(format!("Relinked {linked} item(s), removed {} dead link(s).", dead.len()));
    if claude {
        match sync_hooks(&c.home, &c.root, &want) {
            Ok(h) if h.changed => log::info(format!(
                "Hooks in ~/.claude/settings.json: {} live, {} stale removed.",
                h.added, h.stale
            ))?,
            Ok(_) => {}
            Err(e) => log::warning(format!("Left settings.json alone — {e}"))?,
        }
    }
    verify_after(c)?;
    Ok(())
}

fn uninstall() -> io::Result<()> {
    let home = home()?;
    // Don't require a live checkout: uninstall must still work after the user
    // deleted the clone (which leaves dangling links behind).
    let root = cafe_root().ok();
    let victims = collect_cafe_links(&agent_dirs(&home), root.as_deref(), true);
    // Hooks can only be identified by the checkout path baked into their command,
    // so with the clone gone they stay — say so rather than pretend it was clean.
    let hooks = root.as_deref().map(|r| cafe_hooks(&home, r)).unwrap_or_default();
    intro("cafe · uninstall")?;
    if victims.is_empty() && hooks.is_empty() {
        outro("No cafe links or hooks found — nothing to remove.")?;
        return Ok(());
    }
    let mut listing: Vec<String> = victims.iter().map(|p| p.display().to_string()).collect();
    listing.extend(hooks.iter().map(|(e, name)| format!("{e} hook  ({name})  ~/.claude/settings.json")));
    note("These will be removed", listing.join("\n"))?;
    if !confirm(format!("Remove {} item(s)?", victims.len() + hooks.len())).interact()? {
        return Err(io::Error::new(ErrorKind::Interrupted, "cancelled"));
    }
    let mut n = 0;
    for v in &victims {
        if fs::remove_file(v).is_ok() {
            n += 1;
        }
    }
    if let Some(r) = root.as_deref() {
        match sync_hooks(&home, r, &[]) {
            Ok(h) => n += h.stale,
            Err(e) => log::warning(format!("Left settings.json alone — {e}"))?,
        }
    }
    let _ = fs::remove_file(state_path(&home));
    if root.is_none() {
        log::warning(
            "No checkout found, so cafe hooks in ~/.claude/settings.json were left alone — \
             they're identified by the path they point at. Restore the clone and re-run to clear them.",
        )?;
    }
    outro(format!("Removed {n} item(s)."))?;
    Ok(())
}

/// Remove dead cafe links — orphans left after a skill was renamed or removed
/// upstream (install and update don't prune on their own).
fn clean() -> io::Result<()> {
    let home = home()?;
    // Works even if the checkout is gone — dead links are what we're after.
    let root = cafe_root().ok();
    let stale = collect_cafe_links(&agent_dirs(&home), root.as_deref(), false);
    // A hook whose plugin dir is gone is the settings.json twin of a dangling link:
    // Claude runs it every session and it fails silently.
    let dead_hooks: Vec<(String, String)> = root
        .as_deref()
        .map(|r| {
            cafe_hooks(&home, r)
                .into_iter()
                .filter(|(_, name)| !r.join("plugins").join(name).is_dir())
                .collect()
        })
        .unwrap_or_default();
    intro("cafe · clean")?;
    if stale.is_empty() && dead_hooks.is_empty() {
        outro("No dead links or hooks — nothing to clean.")?;
        return Ok(());
    }
    let mut listing: Vec<String> = stale.iter().map(|p| p.display().to_string()).collect();
    listing.extend(dead_hooks.iter().map(|(e, name)| format!("{e} hook  ({name})  ~/.claude/settings.json")));
    note("Dead cafe entries (whatever they point at no longer exists)", listing.join("\n"))?;
    if !confirm(format!("Remove {} dead entry/entries?", stale.len() + dead_hooks.len())).interact()?
    {
        return Err(io::Error::new(ErrorKind::Interrupted, "cancelled"));
    }
    let mut n = 0;
    for p in &stale {
        if fs::remove_file(p).is_ok() {
            n += 1;
        }
    }
    if let (Some(r), false) = (root.as_deref(), dead_hooks.is_empty()) {
        match prune_dead_hooks(&home, r) {
            Ok(k) => n += k,
            Err(e) => log::warning(format!("Left settings.json alone — {e}"))?,
        }
    }
    outro(format!("Removed {n} dead entry/entries."))?;
    Ok(())
}

/// Say, per plugin, what is and isn't actually on disk. The whole failure mode this
/// exists for was silent: a plugin listed in the manifest, linked nowhere, with its
/// always-on hook missing, and nothing anywhere saying so.
fn doctor() -> io::Result<()> {
    let c = ctx()?;
    intro("cafe · doctor")?;

    let (unlisted, missing_dirs) = manifest_drift(&c.root);
    if !unlisted.is_empty() {
        log::warning(format!(
            "In plugins/ but not in {MARKER}: {}. cafe installs them anyway; `/plugin` won't see them.",
            unlisted.join(", ")
        ))?;
    }
    if !missing_dirs.is_empty() {
        log::warning(format!(
            "Listed in {MARKER} with no plugins/ dir: {}.",
            missing_dirs.join(", ")
        ))?;
    }

    let rows = report(&c);
    if rows.is_empty() {
        outro("No agent has cafe installed — run `cafe install`.")?;
        return Ok(());
    }
    let mut agent = "";
    for r in &rows {
        if r.agent != agent {
            if !agent.is_empty() {
                println!();
            }
            agent = r.agent;
            log::info(agent)?;
        }
        println!("  {}", r.line());
    }
    println!();

    let broken = rows.iter().filter(|r| r.broken()).count();
    let mut absent: Vec<&str> = rows.iter().filter(|r| r.absent()).map(|r| r.plugin.as_str()).collect();
    absent.sort_unstable();
    absent.dedup();
    if broken > 0 {
        log::error(format!("{broken} plugin(s) half-installed or missing a hook."))?;
    }
    if !absent.is_empty() {
        log::warning(format!("Not installed anywhere: {}.", absent.join(", ")))?;
    }
    if broken == 0 && absent.is_empty() {
        outro("Every plugin is linked and every hook is wired.")?;
    } else {
        outro("Fix with:  cafe install")?;
    }
    Ok(())
}

/// Run after install/update so a partial result surfaces immediately instead of
/// waiting for someone to notice a skill never showed up.
fn verify_after(c: &Ctx) -> io::Result<()> {
    let bad: Vec<String> =
        report(c).into_iter().filter(Row::broken).map(|r| format!("{} ({})", r.plugin, r.agent)).collect();
    if !bad.is_empty() {
        log::warning(format!(
            "Didn't fully land: {}. `cafe doctor` has the detail.",
            bad.join(", ")
        ))?;
    }
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
  cafe doctor         report, per plugin, whether its skills and hooks are present
  cafe update         git pull, then relink every agent (picks up new/moved skills)
  cafe clean          remove dead links and dead hooks left by removed skills
  cafe uninstall      remove cafe's links and hooks
  cafe --help         this help
  cafe --version      print the version

Skills install as symlinks back into this checkout, so one update reaches
every agent. Plugins that ship hooks/hooks.json also get those hooks merged
into ~/.claude/settings.json — re-running never duplicates them, and nothing
cafe didn't write is touched. Run inside the clone, or set CAFE_HOME to it."
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
    // Where install.sh clones. A prebuilt binary has no build-time repo path, so this
    // is the only thing standing between a downloaded `cafe` and "checkout not found".
    if let Some(home) = env::var_os("HOME") {
        let dot = Path::new(&home).join(".cafe");
        if is_cafe_checkout(&dot) {
            return Ok(canonical(dot));
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

    let (skills, commands) = plugin_items(p);
    if let Some(sdir) = &a.skills {
        for skill in skills {
            let dest = sdir.join(file_name(&skill));
            place(&skill, dest)?;
        }
    }
    if let Some(cdir) = &a.commands {
        for cmd in commands {
            let dest = cdir.join(file_name(&cmd));
            place(&cmd, dest)?;
        }
    }
    Ok((linked, skipped))
}

/// What a plugin contributes: skill dirs that really carry a SKILL.md, and command
/// markdown. One definition, so linking and verifying can never disagree about what
/// a complete install looks like.
fn plugin_items(p: &Plugin) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let skills = read_subdirs(&p.dir.join("skills"))
        .into_iter()
        .filter(|s| s.join("SKILL.md").is_file())
        .collect();
    (skills, read_files(&p.dir.join("commands"), "md"))
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

// ---------------------------------------------------------------- hooks

fn settings_path(home: &Path) -> PathBuf {
    home.join(".claude/settings.json")
}

/// Read settings.json whole, so everything cafe doesn't understand survives the
/// round trip. A file that isn't valid JSON is an error, never something to
/// overwrite — it's the user's config and it's the only copy.
fn read_settings(path: &Path) -> io::Result<Map<String, Value>> {
    let txt = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(io::Error::new(e.kind(), format!("{}: {e}", path.display()))),
    };
    if txt.trim().is_empty() {
        return Ok(Map::new());
    }
    serde_json::from_str(&txt).map_err(|e| {
        io::Error::other(format!(
            "{} isn't a valid JSON object ({e}). Fix it and re-run; cafe won't overwrite it.",
            path.display()
        ))
    })
}

/// Temp-file + rename so an interrupted write can't truncate the config, keeping
/// one backup from the first time cafe ever touches it.
fn write_settings(path: &Path, map: &Map<String, Value>) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let backup = path.with_extension("json.cafe-bak");
    if path.exists() && !backup.exists() {
        let _ = fs::copy(path, &backup);
    }
    let body = format!("{}\n", serde_json::to_string_pretty(map).map_err(io::Error::other)?);
    let tmp = path.with_extension("json.cafe-tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)
}

/// Every string anywhere in a JSON value.
fn json_strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| json_strings(x, out)),
        Value::Object(o) => o.values().for_each(|x| json_strings(x, out)),
        _ => {}
    }
}

/// Substitute `${CLAUDE_PLUGIN_ROOT}`. Claude's plugin loader expands that; a hook
/// sitting in settings.json gets no such treatment, so copying it in verbatim would
/// produce a command that cats a file literally named `${CLAUDE_PLUGIN_ROOT}/…`.
fn expand(v: &Value, plugin_dir: &Path) -> Value {
    let dir = plugin_dir.display().to_string();
    match v {
        Value::String(s) => Value::String(
            s.replace("${CLAUDE_PLUGIN_ROOT}", &dir).replace("$CLAUDE_PLUGIN_ROOT", &dir),
        ),
        Value::Array(a) => Value::Array(a.iter().map(|x| expand(x, plugin_dir)).collect()),
        Value::Object(o) => {
            Value::Object(o.iter().map(|(k, x)| (k.clone(), expand(x, plugin_dir))).collect())
        }
        other => other.clone(),
    }
}

/// A plugin's hook entries as (event, entry), paths already resolved. Empty when
/// the plugin ships no hooks/hooks.json or it's malformed — a broken hooks file
/// must not stop the skills from linking.
fn plugin_hooks(p: &Plugin) -> Vec<(String, Value)> {
    let Ok(txt) = fs::read_to_string(p.dir.join("hooks/hooks.json")) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&txt) else {
        return Vec::new();
    };
    let Some(events) = v.get("hooks").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (event, entries) in events {
        for e in entries.as_array().into_iter().flatten() {
            out.push((event.clone(), expand(e, &p.dir)));
        }
    }
    out
}

/// Which plugin a single hook command belongs to, read out of the checkout path in
/// the command itself — cafe's proof of ownership. Deliberately catches
/// hand-written commands pointing at the checkout: those are what a partial install
/// leaves behind, and adopting one is what stops it becoming a duplicate.
fn hook_owner(cmd: &Value, root: &Path) -> Option<String> {
    let needle = format!("{}/plugins/", root.display());
    let mut strs = Vec::new();
    json_strings(cmd, &mut strs);
    strs.iter().find_map(|s| {
        let name: String = s.split(&needle).nth(1)?.chars().take_while(|c| *c != '/').collect();
        (!name.is_empty()).then_some(name)
    })
}

/// The individual commands inside one hook entry (an entry is a matcher plus a list).
fn entry_commands(entry: &Value) -> Vec<Value> {
    entry.get("hooks").and_then(Value::as_array).cloned().unwrap_or_default()
}

/// Remove the commands cafe owns from one event's entries, dropping any entry left
/// with none. Granularity matters here: a hand-edited entry can hold a cafe command
/// right next to one the user wrote, and taking the whole entry would delete config
/// cafe never put there.
fn strip_owned(entries: &mut Vec<Value>, root: &Path, doomed: &dyn Fn(&str) -> bool) -> usize {
    let mut removed = 0;
    for entry in entries.iter_mut() {
        if let Some(inner) = entry.get_mut("hooks").and_then(Value::as_array_mut) {
            let n = inner.len();
            inner.retain(|h| !hook_owner(h, root).map(|name| doomed(&name)).unwrap_or(false));
            removed += n - inner.len();
        }
    }
    // An entry whose commands all went is an empty matcher — no reason to keep it.
    entries.retain(|e| e.get("hooks").and_then(Value::as_array).map(|a| !a.is_empty()).unwrap_or(true));
    removed
}

/// Every cafe-owned hook command currently in settings.json, as (event, plugin dir).
fn cafe_hooks(home: &Path, root: &Path) -> Vec<(String, String)> {
    let Ok(settings) = read_settings(&settings_path(home)) else {
        return Vec::new();
    };
    let Some(events) = settings.get("hooks").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (event, arr) in events {
        for entry in arr.as_array().into_iter().flatten() {
            for cmd in entry_commands(entry) {
                if let Some(name) = hook_owner(&cmd, root) {
                    out.push((event.clone(), name));
                }
            }
        }
    }
    out
}

struct HookSync {
    added: usize,
    stale: usize,
    changed: bool,
}

/// Make the cafe-owned slice of Claude's hooks equal exactly `want`, and leave
/// every other key and every foreign hook byte-identical.
///
/// Idempotent by construction: owned entries are dropped and rebuilt on every run,
/// so running twice yields the same file and a half-finished previous run repairs
/// itself. Writes nothing when the result already matches.
fn sync_hooks(home: &Path, root: &Path, want: &[&Plugin]) -> io::Result<HookSync> {
    let path = settings_path(home);
    let before = read_settings(&path)?;
    let mut settings = before.clone();
    let mut hooks =
        settings.get("hooks").and_then(Value::as_object).cloned().unwrap_or_else(Map::new);

    let mut dropped = 0usize;
    for entries in hooks.values_mut() {
        if let Some(arr) = entries.as_array_mut() {
            dropped += strip_owned(arr, root, &|_| true);
        }
    }
    let mut added = 0usize;
    for p in want {
        for (event, entry) in plugin_hooks(p) {
            if let Some(arr) =
                hooks.entry(event).or_insert_with(|| Value::Array(Vec::new())).as_array_mut()
            {
                // Counted in commands, matching what strip_owned removes.
                added += entry_commands(&entry).len();
                arr.push(entry);
            }
        }
    }
    // Don't leave `"SessionStart": []` behind after removing the last entry.
    hooks.retain(|_, v| !v.as_array().map(|a| a.is_empty()).unwrap_or(false));
    match hooks.is_empty() {
        true => settings.remove("hooks"),
        false => settings.insert("hooks".into(), Value::Object(hooks)),
    };

    let changed = settings != before;
    if changed {
        write_settings(&path, &settings)?;
    }
    Ok(HookSync { added, stale: dropped.saturating_sub(added), changed })
}

/// Drop only the cafe hooks whose plugin dir is gone, leaving live ones in place.
fn prune_dead_hooks(home: &Path, root: &Path) -> io::Result<usize> {
    let path = settings_path(home);
    let before = read_settings(&path)?;
    let mut settings = before.clone();
    let Some(mut hooks) = settings.get("hooks").and_then(Value::as_object).cloned() else {
        return Ok(0);
    };
    let mut removed = 0usize;
    for entries in hooks.values_mut() {
        if let Some(arr) = entries.as_array_mut() {
            removed += strip_owned(arr, root, &|name| !root.join("plugins").join(name).is_dir());
        }
    }
    hooks.retain(|_, v| !v.as_array().map(|a| a.is_empty()).unwrap_or(false));
    match hooks.is_empty() {
        true => settings.remove("hooks"),
        false => settings.insert("hooks".into(), Value::Object(hooks)),
    };
    if settings != before {
        write_settings(&path, &settings)?;
    }
    Ok(removed)
}

// ---------------------------------------------------------------- state

/// The plugins the user unchecked at install time.
///
/// The only fact not readable off the filesystem: an absent plugin looks identical
/// whether it was declined or added upstream after the last install. Without this,
/// `update` has to pick one — never picking up new plugins (the old bug) or
/// resurrecting declined ones on every pull.
fn state_path(home: &Path) -> PathBuf {
    home.join(".config/cafe/state.json")
}

fn declined(home: &Path) -> Vec<String> {
    fs::read_to_string(state_path(home))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            Some(
                v.get("declined")?
                    .as_array()?
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect(),
            )
        })
        .unwrap_or_default()
}

fn set_declined(home: &Path, names: &[String]) -> io::Result<()> {
    let path = state_path(home);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = serde_json::json!({ "declined": names });
    let txt = serde_json::to_string_pretty(&body).map_err(io::Error::other)?;
    fs::write(&path, format!("{txt}\n")).map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
}

// ---------------------------------------------------------------- verify

enum Hook {
    /// Plugin ships no hooks, or this agent doesn't take them.
    NotApplicable,
    Wired,
    Missing,
}

struct Row {
    agent: &'static str,
    plugin: String,
    skills: (usize, usize),
    commands: (usize, usize),
    hook: Hook,
}

impl Row {
    fn wanted(&self) -> usize {
        self.skills.1 + self.commands.1
    }
    fn have(&self) -> usize {
        self.skills.0 + self.commands.0
    }
    /// Nothing of this plugin is here. Not a fault — you may have declined it — but
    /// it's the state that went unreported for token-efficiency.
    fn absent(&self) -> bool {
        self.wanted() > 0 && self.have() == 0
    }
    /// Installed, but not completely: some links missing, or the hook never landed.
    fn broken(&self) -> bool {
        self.have() > 0 && (self.have() < self.wanted() || matches!(self.hook, Hook::Missing))
    }
    fn line(&self) -> String {
        let mark = if self.broken() {
            "✗"
        } else if self.absent() {
            "·"
        } else {
            "✓"
        };
        let mut bits = Vec::new();
        if self.skills.1 > 0 {
            bits.push(format!("skills {}/{}", self.skills.0, self.skills.1));
        }
        if self.commands.1 > 0 {
            bits.push(format!("cmds {}/{}", self.commands.0, self.commands.1));
        }
        match self.hook {
            Hook::Wired => bits.push("hook wired".into()),
            Hook::Missing => bits.push("HOOK MISSING".into()),
            Hook::NotApplicable => {}
        }
        format!("{mark} {:<18} {}", self.plugin, bits.join("  "))
    }
}

/// What's actually on disk right now, per agent × plugin. Only agents cafe has been
/// installed into are reported — the rest aren't broken, they're just not customers.
fn report(c: &Ctx) -> Vec<Row> {
    let plugins = plugins(&c.root);
    // Compare at command level, not entry level: a hook is wired if the command is
    // in the file, wherever it ended up being grouped.
    let live: Vec<Value> = read_settings(&settings_path(&c.home))
        .ok()
        .and_then(|s| s.get("hooks").and_then(Value::as_object).cloned())
        .map(|m| {
            m.values()
                .filter_map(Value::as_array)
                .flatten()
                .flat_map(entry_commands)
                .collect()
        })
        .unwrap_or_default();

    let mut rows = Vec::new();
    for a in agents(&c.home) {
        let dirs: Vec<PathBuf> =
            [a.skills.clone(), a.commands.clone()].into_iter().flatten().collect();
        if collect_cafe_links(&dirs, Some(&c.root), true).is_empty() {
            continue;
        }
        for p in &plugins {
            let (skills, commands) = plugin_items(p);
            let want_hooks = plugin_hooks(p);
            // Nothing this agent can take (a commands-only plugin on a skills-only
            // agent) — reporting it as fine would just be noise.
            if count_for(&skills, a.skills.as_deref(), &c.root).1
                + count_for(&commands, a.commands.as_deref(), &c.root).1
                == 0
            {
                continue;
            }
            let hook = if want_hooks.is_empty() || a.label != CLAUDE {
                Hook::NotApplicable
            } else if want_hooks
                .iter()
                .flat_map(|(_, e)| entry_commands(e))
                .all(|cmd| live.contains(&cmd))
            {
                Hook::Wired
            } else {
                Hook::Missing
            };
            rows.push(Row {
                agent: a.label,
                plugin: p.name.clone(),
                // An agent with no commands dir (Gemini, .agents, …) isn't missing
                // those commands — it can't take them. Count only what it accepts.
                skills: count_for(&skills, a.skills.as_deref(), &c.root),
                commands: count_for(&commands, a.commands.as_deref(), &c.root),
                hook,
            });
        }
    }
    rows
}

/// (present, expected) for `srcs` in `dir`. No dir means this agent takes none of
/// them, so nothing is expected either. A dangling link counts as missing, which is
/// the whole point — agents skip those in silence.
fn count_for(srcs: &[PathBuf], dir: Option<&Path>, root: &Path) -> (usize, usize) {
    let Some(dir) = dir else { return (0, 0) };
    let n = srcs
        .iter()
        .filter(|s| {
            let dest = dir.join(file_name(s));
            is_cafe_owned(&dest, root)
                && dest.exists()
                && fs::canonicalize(&dest).ok() == fs::canonicalize(s).ok()
        })
        .count();
    (n, srcs.len())
}

/// Plugin dirs missing from marketplace.json, and manifest entries with no dir.
/// cafe installs from the dirs — strictly the safer source, since forgetting a
/// manifest edit can't make a plugin vanish — so drift is a publishing bug rather
/// than an install one, but it's invisible until someone installs via `/plugin`.
fn manifest_drift(root: &Path) -> (Vec<String>, Vec<String>) {
    let listed: Vec<String> = fs::read_to_string(root.join(MARKER))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            Some(
                v.get("plugins")?
                    .as_array()?
                    .iter()
                    .filter_map(|p| p.get("name")?.as_str().map(str::to_string))
                    .collect(),
            )
        })
        .unwrap_or_default();
    let dirs: Vec<String> = read_subdirs(&root.join("plugins"))
        .iter()
        .filter(|d| d.join(".claude-plugin/plugin.json").is_file())
        .map(|d| file_name(d).to_string_lossy().into_owned())
        .collect();
    (
        dirs.iter().filter(|d| !listed.contains(d)).cloned().collect(),
        listed.iter().filter(|l| !dirs.contains(l)).cloned().collect(),
    )
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

    /// A plugin dir with a SessionStart hook, for the settings.json tests.
    fn hooked_plugin(root: &Path, name: &str) -> Plugin {
        let dir = root.join("plugins").join(name);
        fs::create_dir_all(dir.join("hooks")).unwrap();
        fs::create_dir_all(dir.join("skills").join(name)).unwrap();
        fs::write(dir.join("skills").join(name).join("SKILL.md"), "# skill").unwrap();
        fs::write(
            dir.join("hooks/hooks.json"),
            r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command",
               "command":"cat \"${CLAUDE_PLUGIN_ROOT}/skills/NAME/SKILL.md\"","timeout":10}]}]}}"#
                .replace("NAME", name),
        )
        .unwrap();
        Plugin { name: name.into(), desc: String::new(), dir }
    }

    // The /pr case: a command moves from plugin `a` to plugin `b` upstream. The old
    // link dangles, and agents skip dangling links silently — relinking must repoint it.
    #[test]
    fn relinking_takes_over_a_dead_link_left_by_a_moved_command() {
        let base = scratch("moved");
        let root = base.join("cafe");
        let live = root.join("plugins/b/commands/push.md");
        fs::create_dir_all(live.parent().unwrap()).unwrap();
        fs::write(&live, "# push").unwrap();
        let rootc = fs::canonicalize(&root).unwrap();
        let cmds = base.join("out/commands");
        fs::create_dir_all(&cmds).unwrap();
        // a/commands/pr.md was deleted upstream — this link is dead
        symlink(rootc.join("plugins/a/commands/pr.md"), cmds.join("pr.md")).unwrap();
        symlink(&live, cmds.join("push.md")).unwrap();

        // b now owns pr.md — relinking b must take over the dead link.
        fs::write(root.join("plugins/b/commands/pr.md"), "# pr").unwrap();
        let p = Plugin { name: "b".into(), desc: String::new(), dir: rootc.join("plugins/b") };
        let a = Agent { label: "t", skills: None, commands: Some(cmds.clone()), detected: true };
        install_plugin(&p, &a, &rootc).unwrap();
        assert_eq!(fs::read_to_string(cmds.join("pr.md")).unwrap(), "# pr");
        assert!(collect_cafe_links(&[cmds], Some(&rootc), false).is_empty(), "no dead links left");

        fs::remove_dir_all(&base).unwrap();
    }

    // The bug this whole change exists for: a plugin's SessionStart hook has to reach
    // settings.json, exactly once, without disturbing anything already in the file.
    #[test]
    fn sync_hooks_is_idempotent_and_leaves_foreign_config_alone() {
        let base = scratch("hooks");
        let home = base.join("home");
        let root = base.join("cafe");
        fs::create_dir_all(root.join("plugins")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let rootc = fs::canonicalize(&root).unwrap();
        let settings = home.join(".claude/settings.json");
        fs::write(
            &settings,
            r#"{"model":"opus","hooks":{"SessionStart":[{"hooks":[{"type":"command",
               "command":"clawd SessionStart"}]}],"Stop":[{"hooks":[{"type":"command",
               "command":"clawd Stop"}]}]},"statusLine":{"command":"mine.sh"}}"#,
        )
        .unwrap();
        let p = hooked_plugin(&rootc, "token-efficiency");

        let first = sync_hooks(&home, &rootc, &[&p]).unwrap();
        assert!(first.changed && first.added == 1);
        let after = fs::read_to_string(&settings).unwrap();
        assert!(after.contains("token-efficiency/SKILL.md"), "hook landed");
        assert!(!after.contains("CLAUDE_PLUGIN_ROOT"), "plugin root expanded to a real path");
        assert!(after.contains("clawd SessionStart") && after.contains("clawd Stop"));
        assert!(after.contains("\"model\"") && after.contains("mine.sh"), "unrelated keys kept");
        // preserve_order: the user's key order survives a rewrite.
        assert!(after.find("\"model\"").unwrap() < after.find("\"hooks\"").unwrap());

        // Re-running must be a no-op, not a second copy of the same hook.
        let again = sync_hooks(&home, &rootc, &[&p]).unwrap();
        assert!(!again.changed, "second run rewrote the file");
        assert_eq!(fs::read_to_string(&settings).unwrap(), after);
        assert_eq!(after.matches("token-efficiency/SKILL.md").count(), 1);

        // Dropping the plugin takes its hook back out and leaves the rest.
        let off = sync_hooks(&home, &rootc, &[]).unwrap();
        assert!(off.changed && off.stale == 1);
        let cleared = fs::read_to_string(&settings).unwrap();
        assert!(!cleared.contains("token-efficiency"));
        assert!(cleared.contains("clawd Stop") && cleared.contains("mine.sh"));

        fs::remove_dir_all(&base).unwrap();
    }

    // A hand-patched hook pointing into the checkout is cafe's to adopt — otherwise
    // repairing a partial install leaves the user with the same hook twice.
    #[test]
    fn sync_hooks_adopts_a_hand_written_entry_instead_of_duplicating_it() {
        let base = scratch("adopt");
        let home = base.join("home");
        let root = base.join("cafe");
        fs::create_dir_all(root.join("plugins")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let rootc = fs::canonicalize(&root).unwrap();
        let p = hooked_plugin(&rootc, "plainspeak");
        let settings = home.join(".claude/settings.json");
        fs::write(
            &settings,
            serde_json::to_string(&serde_json::json!({
                "hooks": {"SessionStart": [{"hooks": [{"type": "command",
                    "command": format!("cat {}/plugins/plainspeak/skills/plainspeak/SKILL.md", rootc.display())}]}]}
            }))
            .unwrap(),
        )
        .unwrap();

        sync_hooks(&home, &rootc, &[&p]).unwrap();
        let after = fs::read_to_string(&settings).unwrap();
        assert_eq!(after.matches("plainspeak/SKILL.md").count(), 1, "adopted, not duplicated");

        // And a hook whose plugin was deleted upstream is dead weight clean removes.
        fs::remove_dir_all(&p.dir).unwrap();
        assert_eq!(prune_dead_hooks(&home, &rootc).unwrap(), 1);
        assert!(!fs::read_to_string(&settings).unwrap().contains("plainspeak"));

        fs::remove_dir_all(&base).unwrap();
    }

    // Straight off a real machine: one SessionStart entry holding a hand-written echo
    // next to a hand-written `cat` into the checkout. Cafe owns the second command and
    // nothing else — taking the whole entry would silently delete the user's echo.
    #[test]
    fn sync_hooks_splits_a_shared_entry_and_keeps_the_users_command() {
        let base = scratch("shared");
        let home = base.join("home");
        let root = base.join("cafe");
        fs::create_dir_all(root.join("plugins")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let rootc = fs::canonicalize(&root).unwrap();
        let p = hooked_plugin(&rootc, "token-efficiency");
        let settings = home.join(".claude/settings.json");
        fs::write(
            &settings,
            serde_json::to_string(&serde_json::json!({"hooks": {"SessionStart": [{"hooks": [
                {"type": "command", "command": "echo 'my own note'"},
                {"type": "command", "command": format!("cat {}/plugins/token-efficiency/skills/token-efficiency/SKILL.md", rootc.display())}
            ]}]}}))
            .unwrap(),
        )
        .unwrap();

        sync_hooks(&home, &rootc, &[&p]).unwrap();
        let after = fs::read_to_string(&settings).unwrap();
        assert!(after.contains("my own note"), "user's command survived");
        // Count commands, not substrings — the plugin name appears twice in one path.
        assert_eq!(cafe_hooks(&home, &rootc), vec![("SessionStart".into(), "token-efficiency".into())]);

        // And it stays that way: no second copy, no lost note.
        sync_hooks(&home, &rootc, &[&p]).unwrap();
        assert_eq!(fs::read_to_string(&settings).unwrap(), after);

        fs::remove_dir_all(&base).unwrap();
    }

    // Corrupt settings.json is the user's only copy — refuse it, never overwrite.
    #[test]
    fn sync_hooks_refuses_to_touch_unparseable_settings() {
        let base = scratch("corrupt");
        let home = base.join("home");
        let root = base.join("cafe");
        fs::create_dir_all(root.join("plugins")).unwrap();
        fs::create_dir_all(home.join(".claude")).unwrap();
        let settings = home.join(".claude/settings.json");
        fs::write(&settings, "{ this is not json").unwrap();
        assert!(sync_hooks(&home, &root, &[]).is_err());
        assert_eq!(fs::read_to_string(&settings).unwrap(), "{ this is not json");
        fs::remove_dir_all(&base).unwrap();
    }

    // `update` used to relink only what an agent already had, so a plugin added
    // upstream was skipped every single time. Declines are the one thing it respects.
    #[test]
    fn declined_round_trips_and_defaults_to_nothing() {
        let base = scratch("state");
        let home = base.join("home");
        fs::create_dir_all(&home).unwrap();
        assert!(declined(&home).is_empty(), "no state file means nothing is declined");
        set_declined(&home, &["glm".to_string()]).unwrap();
        assert_eq!(declined(&home), vec!["glm".to_string()]);
        set_declined(&home, &[]).unwrap();
        assert!(declined(&home).is_empty());
        fs::remove_dir_all(&base).unwrap();
    }

    // Every plugin this repo ships that claims an always-on hook must actually have
    // one cafe can read — the claim lives in a description, the hook in a file.
    #[test]
    fn always_on_plugins_in_this_repo_ship_a_readable_hook() {
        let repo =
            fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()).unwrap();
        for p in plugins(&repo).iter().filter(|p| p.desc.contains("SessionStart hook")) {
            let hooks = plugin_hooks(p);
            assert!(!hooks.is_empty(), "{} claims a SessionStart hook but ships none", p.name);
            assert!(hooks.iter().any(|(e, _)| e == "SessionStart"), "{}: wrong event", p.name);
            for (_, entry) in &hooks {
                let mut strs = Vec::new();
                json_strings(entry, &mut strs);
                assert!(
                    !strs.iter().any(|s| s.contains("CLAUDE_PLUGIN_ROOT")),
                    "{}: plugin root left unexpanded",
                    p.name
                );
                assert!(hook_owner(entry, &repo).is_some(), "{}: hook isn't recognisably cafe's", p.name);
            }
        }
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
