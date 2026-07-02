---
name: architecture
description: How to structure any system. Where files go, when a repo becomes a monorepo, when one app becomes two, and when to add (or refuse) a database, service, or queue. Use whenever laying out a new project, creating files in an unfamiliar repo, restructuring, or when structure questions come up. "where should this go", "should this be its own service", "do we need a microservice", "how should I organize this", "set up a monorepo", "split this out". Always active when creating projects, adding services, or placing new code.
user-invocable: true
disable-model-invocation: false
---

# Architecture

You are the architect. The person you're building with should never have to make a structure decision, and you should never ask them one. When a layout question comes up, check this skill, decide, and briefly say what you did. They describe what they want. You know where it goes.

Every decision below is already made. Apply it silently.

## The governing metric: "that's it?"

A finished system should make a reader go, *that's it? That's all that runs this?* That reaction is the metric of good.

It used to be the opposite. Before agents, code was bounded by how much complexity one person could hold in their head, so mastery meant holding more. That world is gone. Code is now read cold, over and over, by humans and agents who cannot hold everything either. The metric flipped: the best system is the one a cold reader reconstructs in a single pass.

This is the *why* behind everything here. It is not a coding-style guide. Architecture decides where things live and how systems relate. How a given function gets written is out of scope.

## The test every layout must pass

**Could a fresh agent, given only this repo and this skill, put the next feature in the obviously right place without asking?**

If yes, the layout is working. If no, the layout is the bug. What makes it pass is predictability: the same kind of thing always lives in the same kind of place, folders carry boring literal names, and convention beats configuration. Consistency beats cleverness every time, because the value of a layout is that a cold reader is never surprised.

## Default shape: one app

Every project starts as one app. Not a monorepo, not services. One deployable, one folder:

```
my-app/
  src/
    routes/            # thin route files: wire URLs to modules, nothing else
    jobs/              # thin scheduled entries: wire the scheduler to modules, nothing else
    modules/
      billing/
        queries.ts     # server functions for this domain
        schema.ts      # this domain's tables + validation, one source of truth
        components/    # this domain's UI
      auth/
        ...
    components/ui/     # shared primitives (buttons, dialogs, inputs)
    db/                # database client, migrations, seed data
    lib/               # small cross-cutting helpers
    env.ts             # validated environment, crashes at boot if wrong
```

The rules that keep it legible:

- **Entry points stay thin.** A route file wires a URL to a module; a job file wires the scheduler to a module. Logic lives in the module.
- **A module owns its domain.** Its data access, its validation, its domain-specific UI, its background work, colocated. Adding a feature means adding or extending a module.
- **Modules are named with the words the designer actually says.** If they say "the dashboard", the module is `dashboard/`. When ownership is ambiguous, the thing lives in the module whose name the designer would use for it. Never a `core/`, `common/`, or `misc/` module; ambiguity is resolved by naming, not a catch-all.
- **A cross-domain feature is a module too.** A dashboard, report, or admin screen that spans domains gets its own module that composes the others. Its route still wires to that one module.
- **Modules share through exports, never internals.** Another domain's data is read through what its module exports; it is written only through that module's functions, never its tables directly. Two modules that constantly import each other are one module.
- **Tables live with their module** (`schema.ts`); `db/` holds only the client, migrations, and seed data.
- **No monolithic files.** When a file can't be read in one pass, split it along a boundary that already exists: a module, a component. Never invent a new layer to hide length.
- **Reuse within the app.** Anything used twice gets factored out: if domain-specific, into the module that owns the domain, and other modules import it from there; if generic, into `components/ui` or `lib/`.
- **Everything else has one home too.** Unit tests sit next to the file they test (`queries.test.ts`); e2e specs in `e2e/` at the app root; one-off scripts in `scripts/`; static assets in `public/`; seed data in `db/seed.ts`.

## Growing: the siloed monorepo

When a real second deployable appears, the repo becomes:

```
my-repo/
  apps/
    web/               # each app is the single-app shape above, complete
    admin/             # was routes/admin in web until it needed its own deploy
    api/
```

That is the entire structure. `apps/`, then the things. Each app inside is the single-app shape, minus what it doesn't render: an API keeps `routes/` (its endpoints) but drops `components/`; a worker drops both. `modules/`, `db/`, `lib/`, and `env.ts` are universal.

**Every app is an island.** No `packages/`, no shared workspace layer, no app importing from another app. Shared packages are invisible coupling: change one thing, break three apps, and now someone is debugging a build graph instead of shipping. Siloing means any app can be understood, built, and deployed alone.

**Islands include data.** A database is owned by exactly one app: the owner holds its schema and migrations, and no other app connects to it. Anything else that needs the data goes through the owner's API. If two apps both seem to need direct access to the same database, they are almost certainly one app; don't split.

**Islands talk over public APIs only.** The serving app owns its contract. The consuming app treats a sibling's responses like any other untrusted input and parses them at its own boundary, so when a contract drifts the consumer fails loudly at the seam instead of quietly corrupting downstream.

The consequence, accepted deliberately: **reuse lives inside an app, duplication lives between them.** If two apps need the same button, copy it. A copy costs a paste; a shared dependency costs coupled release cycles forever. DRY within an app, WET across apps.

## Triggers: when to add the next thing

Nothing gets added without hitting its trigger. These are checkpoints, not judgment calls. If the trigger hasn't fired, the answer is no, and no amount of "we might need it later" changes that.

| Add | Only when |
|---|---|
| A database | Something must actually persist. Prototypes mock. |
| A second app (`apps/*`) | A second deployable is forced: a different runtime, or something that genuinely cannot ship inside the existing app's deploy. A new audience is not a deployable; admin and internal surfaces start as routes in the existing app and split only when they can no longer ship with it. |
| A dedicated backend service | Server functions genuinely can't carry it: heavy compute, long-running jobs, or an API surface that has outgrown the app's own routes (several consumers, its own deploy cadence). A single webhook or one-off endpoint for an outside client is just a route in the existing app. |
| Scheduled work (cron) | Never new infrastructure. A thin entry in `jobs/`, invoked by the platform scheduler; logic stays in the owning module. |
| A queue / worker | Request-initiated work that must survive the request and needs retries, durability, or fan-out. The worker is the same app started as a second process, sharing its modules and schema; it is never a second app. Time-scheduled work is not a queue. |
| A cache | A measured bottleneck, with numbers. Never speculatively. |
| A new repo | Only when the work is a separate product. Anything asked for inside a repo lands in that repo: a module first, a new app only if the second-app trigger fires. |
| Microservices | Effectively never. Each service must independently pass the backend trigger. |

And the anti-triggers, the pressure to refuse:

- **Never split for tidiness.** Size alone never justifies a second app or service.
- **Never add a layer to feel organized.** New folders, abstractions, or indirection must earn their place by making the test above pass better, not worse.
- **Never build for imagined scale.** The system serves today's load plus one comfortable step, not a hypothetical future.

## Where logic lives

Logic and data access live on the server. Components render and dispatch, nothing else. The server/client seam is the security boundary: secrets, raw records, and business rules stay behind it, and the client receives results. One seam, in one place, that everyone can point to.

## Robust without bulk

Elegant and unbreakable are not in tension, but only if robustness comes from structure instead of volume. The old way covered every edge case with defensive code, which is exactly the complexity being killed. Instead:

- **Make bad states unrepresentable.** Types and schemas that can't hold invalid data beat runtime checks that catch it.
- **Parse at every entry point, once.** A boundary is anywhere data enters a process: env at boot, requests and forms, job payloads, a sibling app's responses. Parse there into types; everything past that point trusts the types. No re-validating internal hops.
- **Fail loud, at boot.** A misconfigured system refuses to start with a clear message. It never limps into production and fails quietly later.

The result reads as "that's it?" precisely because the edge-case handling lives in the schema, not in a thousand `if` statements.

## The never list

- **Never ask the user a structure question.** Decide from this skill, say what you did.
- **Never microservices by default.**
- **Never a `packages/` or shared workspace layer.** Apps are islands. In an inherited repo that already has one, follow its existing convention; this list governs structure you introduce, and you never restructure an existing layout to satisfy it.
- **Never reach from one app into another.**
- **Never two apps on one database.** One owner; everyone else goes through its API.
- **Never a second app or service for tidiness.** Only for a real deployable.
- **Never monolithic files.** Split along existing boundaries.
- **Never cleverness over consistency.** The reader's lack of surprise is the point.
- **Never speculative infrastructure.** No queue, cache, or service before its trigger fires.

## With the tech-stack skill

This skill decides *shape*: where code lives and when systems split. The **tech-stack** skill decides *tools*: frameworks, databases, hosting. If both are installed they compose; when a tool choice comes up, defer to tech-stack. When either skill would have you ask the user to choose, don't. Pick the default and move.
