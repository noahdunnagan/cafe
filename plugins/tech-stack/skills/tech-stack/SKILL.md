---
name: tech-stack
description: cafe · The canonical, opinionated technology stack for any new project — what to reach for, in what order, and what to never touch. Use this whenever a new project starts or a stack decision comes up: scaffolding an app, prototype, dashboard, landing page, or design system, or picking a framework, UI library, database, ORM, auth, hosting, animation lib, or any dependency. Triggers include "new project", "start building", "scaffold", "spin up a site/app", "what should I use for", "which framework/library/database", "set up auth", "add a backend", "where do I deploy", "build a prototype". Always active when choosing or installing technology.
user-invocable: true
disable-model-invocation: false
---

# Tech Stack

This is the stack. Every new project starts here. The choices below are not a menu — they are the defaults, already decided, so you spend your time designing and building instead of re-litigating tooling. Deviate only when a project has a concrete reason the defaults can't serve, and when you deviate, say why.

The user this skill serves builds **web app UIs and dashboards, interactive prototypes, and design systems** — frequently touching **sensitive data**, often shipping the whole thing end-to-end. So two values sit underneath every pick: **own your data** (self-host the things that touch users; no third-party data processors holding identity or files) and **reuse one mental model** (React + TypeScript everywhere it can reach, so web, mobile, and email are the same skillset).

---

## Philosophy

Six principles. Every choice below flows from one of them.

1. **Reach for the lightest thing that works.** No dedicated backend until server functions can't do the job. No database until something needs to persist. No new dependency when the stack already has the answer. The best dependency is the one you didn't add.
2. **One ecosystem, one mental model.** TanStack on the web, React Native on mobile, React for email templates. The same `Query`, `Zod`, and TypeScript instincts carry across all of them. Don't fragment the skillset.
3. **Own the data.** Anything touching users or secrets is self-hosted on infrastructure you control — your Postgres, your auth tables, your object storage. Third-party processors are a liability, not a convenience.
4. **Sensitive data lives server-side.** It is read, handled, and transformed on the server (server functions or a Rust backend) and never shipped into the client bundle. The browser sees rendered results, not raw secrets.
5. **Fail loud, early.** A missing env var crashes at boot, not in production. Validation (Zod) sits at every boundary — env, forms, API, database. Bad data never travels.
6. **Type-safe end to end.** The database schema generates the validators; the validators type the forms and the API. One source of truth, no drift between layers.

---

## The stack at a glance

| Concern | Choice | Notes |
|---|---|---|
| Package manager | **bun** | Never npm. Ever. Including lookups (`bun pm`, not `npm view`). |
| Language | **TypeScript** | Everywhere on the JS side. No plain JS. |
| Web framework | **TanStack Start** | Full-stack: SSR + server functions on Vite. |
| Routing / data / tables / forms | **TanStack Router / Query / Table / Form** | One coherent suite. |
| Client/UI state | **React state + URL params**, then Zustand | Server state stays in Query, never copied into a store. |
| Validation | **Zod** | Every boundary. `drizzle-orm/zod` derives schemas from tables. |
| Styling | **Tailwind v4** | CSS-first config, Vite plugin. |
| Components | **shadcn/ui** | Radix primitives you copy in and own. Accessible by default. |
| Toasts / notifications | **Sonner** | shadcn default; the visible-confirmation surface. |
| Motion | **Motion** (ex-Framer Motion) | Springs, gestures, layout animation. |
| Icons | **Nucleo** | The owned, licensed set. Not Lucide. |
| Fonts | **Self-hosted** | Served from your origin. No Google Fonts CDN. |
| Charts | **Recharts** | shadcn-native default; swappable per project. |
| Dates / numbers | **Native `Intl`** | No formatting library by default. |
| Lint + format | **Biome** | One fast Rust tool. Replaces ESLint + Prettier. |
| Design-system workbench | **Storybook** | Isolated component dev + docs. |
| Database | **Railway Postgres** | Self-hosted on your Railway. |
| ORM | **Drizzle** | TS-native, SQL-like, light. Zod via `drizzle-orm/zod`. |
| Auth | **Better Auth** | Self-hosted in your Postgres. Discord OAuth first. |
| Secrets / env | **Railway vars + Zod-validated env** | Nothing sensitive in the repo. |
| Dedicated backend (only if needed) | **Rust: actix-web + tokio + SeaORM** | Rust-native serving, no nginx. |
| Mobile | **Expo / React Native** | Shares React/TS/TanStack/Zod with web. |
| Desktop | **Out of scope** | Unless a project specifically demands it. |
| Hosting | **Railway** | Always. GitHub source → auto-deploy. |
| Tiny throwaway only | **Cloudflare Workers** | Never production. |
| Object storage | **Railway Object Storage** | Never R2/S3 directly. |
| AI / LLM | **OpenRouter** + **orkey** | Single key, capped keys minted per use. |
| Transactional email | **Cloudflare Email** | Routing (inbound); Email Service for outbound (beta). |
| Testing | **Vitest + Playwright** | Unit + e2e/a11y. Dial to project maturity. |
| Analytics | **None by default** | Add per project only. |

---

## Decide before you install: do you even need it?

Before adding anything, walk the ladder. Stop at the first rung that satisfies the requirement.

- **Need to persist data?** → Only then a database. A prototype with mocked data needs no Postgres.
- **Need server logic / secrets / DB access?** → A **TanStack Start server function**, not a separate backend. This is the answer for the overwhelming majority of projects.
- **Need a dedicated backend?** → Only when server functions genuinely can't carry it: heavy compute, long-running jobs, a service consumed by clients beyond this app, or workloads where Rust's performance/safety is the point. Then, and only then, stand up Rust.
- **Need to deploy?** → Railway. The exception is below, and it is narrow.

The instinct is always *down* the ladder, toward less. Adding a backend, a database, or a new dependency is a decision you justify, not a default you assume.

---

## Web — the default for almost everything

### TanStack Start

Every web project is **TanStack Start**: SSR plus server functions on Vite, with type-safe routing. This posture matters for the sensitive-data case — server functions run on the server, so secrets and raw records are read and handled there and never reach the client bundle. The browser receives rendered output.

Use the full TanStack suite as one system:

- **TanStack Router** — type-safe, file-based routing. Search params and loaders are typed.
- **TanStack Query** — all server-state fetching and caching. Don't hand-roll fetch/useEffect.
- **TanStack Table** — headless data grids. The backbone of every dashboard table.
- **TanStack Form** — forms, validated with Zod.

Scaffold **Start** (SSR) — not the router-only SPA. Confirm the exact CLI against current TanStack docs; the starter moved off the old `create-tsrouter-app` package onto the consolidated `@tanstack/cli`:

```sh
bunx @tanstack/cli create my-app    # TanStack Start, SSR by default
cd my-app && bun install
# add --router-only ONLY if you deliberately want a client-only SPA (no server functions)
```

### Validation with Zod everywhere

Zod is the validation layer at **every** boundary: environment, forms, server-function inputs/outputs, and parsed external data. With Drizzle's `drizzle-orm/zod` integration, the database schema generates the Zod schemas, so the table definition is the single source of truth that flows into your forms and API types. Never let unvalidated data cross a boundary.

### Styling — Tailwind v4 + shadcn/ui

- **Tailwind v4** for styling. v4 is CSS-first: configure via the `@tailwindcss/vite` plugin and a CSS `@theme` block, not a sprawling `tailwind.config.js`.
- **shadcn/ui** for components — Radix primitives you copy into the repo and own outright. Accessible by default (focus management, ARIA, keyboard nav come for free at the component level), and fully restyleable because the code is yours. This is the component layer; build custom pieces in the same Radix + Tailwind idiom.

```sh
bunx shadcn@latest init
bunx shadcn@latest add button dialog dropdown-menu sonner
```

### Light/dark theming

Define semantic CSS variables in the `@theme`/`:root` and `.dark` blocks (the shadcn convention) — components reference the variables, never raw colors. Toggle by setting a class on `<html>`, and **persist the choice in a cookie read during SSR** so there's no flash of the wrong theme on load. This is foundational; set it up before building screens, because everything inherits it.

### Motion

**Motion** (formerly Framer Motion) for animation — springs, gestures, layout animations, drag. This is the tool for *deliberate* interaction: gesture commits gated behind real thresholds, a tactile tick at the commit line, visible confirmation of what just happened. (See **Interaction design** below — this matters here.) Reserve CSS transitions for trivial hovers/fades; reach for Motion the moment an interaction has state or gesture.

### Icons, fonts, charts

- **Icons: Nucleo.** The owned, *licensed* set — import from the team's Nucleo export, not Lucide or any CDN. On a fresh machine you won't have it: get access to the team's Nucleo license/export before wiring icons in.
- **Fonts: self-hosted.** Bundle and serve fonts from your own origin. No Google Fonts CDN — a third-party request on every page load leaks visitor IPs and costs a round-trip. Self-hosting is faster and keeps the privacy boundary clean.
- **Charts: Recharts** as the default, because shadcn's chart components are built on it, so charts inherit the theme for free. Swap per project (visx for fully custom viz, Tremor for instant dashboards) — but Recharts is where you start.

### Day-one defaults

The calls every project hits on the first afternoon — all chosen to stay light:

- **Client/UI state:** React state + typed URL search params (via TanStack Router) first. Lift to **Zustand** (or TanStack Store, to stay in-ecosystem) only when state is genuinely shared across distant parts of the tree. Server state is always TanStack Query — never copied into a store.
- **Toasts / notifications:** **Sonner** (the current shadcn default — added via `shadcn add sonner`). This is the visible-confirmation surface the interaction rules depend on: every async action gets a success or failure toast.
- **Dates / numbers / currency:** native `Intl.DateTimeFormat` / `Intl.NumberFormat`. No formatting library by default; reach for `date-fns` only when real date *math* (not just display) shows up.
- **Runtime errors:** TanStack Router route-level `errorComponent` and `pendingComponent` for loader/server-function failures. Show the user a generic message — never surface a raw error or secret. Pair with Query's error states for mutations.

### Tooling

- **bun** for everything — install, run, scripts, package lookups. Never npm.
- **Biome** for lint + format — one Rust-based tool, near-zero config. Not ESLint + Prettier.
- **TypeScript**, strict. No plain JS.

```sh
bunx @biomejs/biome init
```

---

## Design systems → Storybook

For the design-system workload, **Storybook** is the workbench: isolated component development, visual documentation, and the a11y + interaction-test addons. It's heavier than the alternatives, and that weight buys the completeness a real, shared design system needs. Build components in Storybook, document their variants and states there, and consume them from apps.

---

## Data & persistence

The default tier — used directly from TanStack Start server functions, no separate backend.

### Database — Railway Postgres

Postgres, self-hosted on **Railway**. You own the data and the box; no third-party data processor sits between you and your users' records. Connect the GitHub repo as the service source so pushes auto-deploy.

### ORM — Drizzle

**Drizzle.** TS-native, SQL-like, and light — no heavy runtime or codegen step. On Drizzle 1.0+, Zod schema generation is built into core: import `createInsertSchema`/`createSelectSchema` from `drizzle-orm/zod` (the standalone `drizzle-zod` package is deprecated). Either way, the table is the one source of truth that feeds validation and types. `drizzle-kit` handles migrations.

### Auth — Better Auth

**Better Auth**, self-hosted. Sessions and users live in *your* Postgres, not a third party's servers — the right call whenever data is sensitive. **Discord OAuth is the first-class provider** (Google is a hassle to set up; avoid it unless a project specifically requires it). Add email/password or magic links as needed.

### Secrets & environment

- Secrets live in **Railway variables**. Never in the repo, never committed, not even in an example file with real values.
- Validate the environment with a **Zod schema at boot**. A missing or malformed variable crashes startup with a clear message — it never fails silently in production.

---

## When you need a dedicated backend → Rust

Default to **no** dedicated backend; server functions cover most needs. When a project genuinely requires a standalone service — heavy compute, long-running work, a shared API for multiple clients, or where raw performance and memory safety are the point — build it in **Rust**:

- **actix-web** — the web framework. Rust-native serving; no nginx in front.
- **tokio** — the async runtime.
- **SeaORM** — the async ORM, against Railway Postgres.

This is the deliberate, opinionated backend tier. It is a step you take on purpose, not a reflex. (The TS tier uses Drizzle; the Rust tier uses SeaORM — they don't mix within one service.)

---

## Mobile → Expo / React Native

**Expo (React Native)** for native-feeling mobile. It reuses the entire web mental model — React, TypeScript, TanStack Query, Zod — so mobile is the same skillset, not a second one. One team, one set of patterns, across web and phone.

## Desktop → out of scope

Don't pin a desktop stack. Desktop is out of scope unless a specific project demands it; revisit the choice then rather than carrying an unused decision.

---

## Hosting & deploy

### Railway, always

Everything ships to **Railway**. Connect the GitHub repo as the service source (auto-deploy on push) rather than uploading local images. SSR apps, the Rust backend, Postgres, and object storage all live there — one platform, your control, your data.

### The one exception — and its hard limit

A **really tiny** throwaway site (a one-off demo, a quick static experiment) may deploy to **Cloudflare Workers**. This is a throwaway path only. **Nothing production ever runs on Cloudflare Workers.** If a "tiny" site grows real users or real data, it moves to Railway. No exceptions.

### Object storage — Railway Object Storage

Files, uploads, and assets go to **Railway Object Storage**. Never R2 or S3 directly. Keeps storage on the same owned platform as everything else. Uploads flow **through a server function** that validates type/size with Zod and streams to storage, returning only an opaque key or rendered URL — the client never holds credentials. Use presigned direct-to-storage uploads only for large, non-sensitive assets.

---

## AI / LLM

- **OpenRouter, always.** A single key routes to any model — simpler than juggling per-provider keys or gateway URLs. Pick the model per task (latest-generation Claude: a top-tier model like Opus for heavy reasoning, a fast one like Haiku for cheap/quick work — always the current generation, never a stale pin).
- **No zero-data-retention requirement here.** OpenRouter is the routing layer regardless of provider retention. The own-your-data rule governs identity, records, and files — not model traffic.
- **Never hand out the real key.** Use **`orkey`** — the Rust CLI that mints spend-capped OpenRouter keys. Hand a coding agent or a project a key with a low ceiling (e.g. `orkey mint` for a $5 cap) instead of the real management key. If it leaks or loops on an expensive model, the cap eats the damage; `orkey burn <hash>` kills it when done.

Install orkey to PATH and configure it as the way LLM keys get minted:

```sh
git clone https://github.com/noahdunnagan/orkey && cd orkey
cargo install --path .
# create a management key at openrouter.ai/settings/management-keys
mkdir -p ~/.config/orkey && cat > ~/.config/orkey/key
chmod 600 ~/.config/orkey/key
```

That `cat >` line waits silently for input — paste the management key, press Enter, then Ctrl-D to save. Nothing is echoed. (Setting `OPENROUTER_MANAGEMENT_KEY` works as a fallback, but the file keeps the key out of shell history.)

```sh
KEY=$(orkey mint)        # capped key on stdout, hash on stderr
orkey ls                 # hash  name  $used/$cap
orkey burn <hash>        # delete when done
```

---

## Transactional email → Cloudflare Email

Auth flows, magic links, and notifications go through **Cloudflare Email**. Two complementary pieces:

- **Email Routing** — inbound only. Forwards incoming mail to a destination address or to a Worker. Free on all plans.
- **Email Sending** (Cloudflare Email Service) — outbound transactional mail to arbitrary recipients, via a Workers binding, REST API, or SMTP, with SPF/DKIM/DMARC configured automatically when you add the domain.

Caveats to plan around:

1. The domain must use **Cloudflare DNS**.
2. Sending to arbitrary (non-verified) recipients needs the Workers **Paid** plan. The older `send_email` Email Workers binding only reaches addresses already verified in your account — so it *can't* email a new user a magic link.
3. Email Sending is in **public beta** as of mid-2026; confirm GA status and limits before leaning on it in production.

This path is fine even though Cloudflare Workers is throwaway-only for *hosting* — email is a discrete service, not the app's home. Templates can be authored in React (the same skillset), then sent via the Worker.

---

## Testing

**Vitest + Playwright.** Vitest for unit and logic tests (Vite-native, so it fits the toolchain with no extra config); Playwright for end-to-end flows and accessibility smoke tests in a real browser. Scale to project maturity: a throwaway prototype skips tests; a production app with sensitive data gets the full pair, with e2e coverage on every auth and data-mutation path.

**Analytics: none by default.** Add per project only when a real question needs answering — cleanest privacy posture, and the right default given sensitive data.

---

## Handling sensitive data

These projects frequently touch sensitive data, so this is not optional polish — it's the baseline. Make the right call by default:

1. **Server-side only.** Read, handle, and transform sensitive data in server functions or the Rust backend. Never ship it into the client bundle, never expose it through a public API surface, never log it. SSR exists partly so the browser sees results, not raw records.
2. **Self-host what touches users.** Auth (Better Auth → your Postgres), database (Railway Postgres), files (Railway Object Storage). No third-party data processor holds identity, records, or uploads.
3. **Secrets in Railway vars, validated at boot.** Never in the repo. A bad/missing secret fails startup loudly.
4. **Validate every boundary with Zod.** Env, forms, server-function inputs, external data. Untrusted data is validated before it travels.
5. **No third-party requests that leak users.** Self-hosted fonts (no Google Fonts CDN), no analytics by default, no CDN pinging on page load.
6. **Cap the blast radius on AI.** orkey-minted, spend-capped keys — never the real key in an app or agent.
7. **Don't leak in errors.** Route-level error components show a generic message; raw errors and secrets never reach the user or the logs.
8. **Accessible by default — but not automatically.** shadcn/Radix primitives ship correct per-component focus, ARIA, and keyboard handling. You still own route-change focus management and the keyboard model of any custom, non-Radix view — don't assume app-level focus is free.

---

## Interaction design

The owned design sensibility, encoded so built interactions match it. Motion is the tool; these are the rules:

- **Deliberate over fast.** Gesture-driven actions commit only past a real threshold — not on the faintest swipe. An *accidental* gesture commit is a cardinal sin.
- **Confirm at the commit line.** A tactile/visual tick at the moment a gesture commits, and a visible confirmation of what just happened (a Sonner toast for async results). The user is never left guessing whether an action took.
- **Always offer tap-to-open alongside swipe.** Swipe is a shortcut, never the only way in. Every gesture has a plain, discoverable equivalent.
- **No silently disabled buttons.** A disabled control says *why* it's disabled, or the UI falls back to an enabled path. Never leave a dead button sitting with no explanation.

---

## The never list

- **Never npm.** bun for everything, including package lookups.
- **Never a backend before server functions.** Default down the ladder.
- **Never a database before something needs persisting.**
- **Never Google OAuth first.** Discord OAuth is the default provider.
- **Never nginx.** Rust-native serving (actix-web) when a Rust backend exists.
- **Never a third-party data processor for identity, records, or files.** Self-host on Railway.
- **Never R2 or S3 directly.** Railway Object Storage.
- **Never production on Cloudflare Workers.** Tiny throwaways only.
- **Never the Google Fonts CDN.** Self-host fonts.
- **Never the real OpenRouter key in an app or agent.** orkey-minted capped keys.
- **Never secrets in the repo.** Railway vars, Zod-validated at boot.
- **Never ship sensitive data to the client.** Server-side only.

---

## New project checklist

The canonical order for spinning up a standard web project. (Confirm CLI invocations against current docs — scaffolders move.)

1. **Scaffold** TanStack **Start** (SSR — not the router-only SPA) with bun: `bunx @tanstack/cli create`. `cd` in, `bun install`.
2. **Biome** — `bunx @biomejs/biome init`. Strict TS.
3. **Tailwind v4** — add the `@tailwindcss/vite` plugin, set up the `@theme` block; define light/dark CSS variables and the `<html>`-class toggle (cookie-persisted, SSR-read).
4. **shadcn/ui** — `bunx shadcn@latest init`, then add the primitives you need (include `sonner` for toasts).
5. **Nucleo** icons wired in (get the team export first); **fonts self-hosted** from the start.
6. **Decide the data ladder** — does this even need persistence? If yes: Railway Postgres + Drizzle (Zod via `drizzle-orm/zod`). If no: mocks, move on.
7. **Auth** (if needed) — Better Auth, Discord OAuth, tables in your Postgres.
8. **Env** — Zod schema, validated at boot. Secrets into Railway vars.
9. **Motion** for any interaction with state or gesture; follow the interaction-design rules. Wire route-level `errorComponent`/`pendingComponent`.
10. **Deploy** — Railway, GitHub source connected for auto-deploy. (Tiny throwaway only? Cloudflare Workers — never prod.)
11. **Tests** scaled to maturity — Vitest + Playwright on prod paths, especially auth and data mutations.

A dedicated Rust backend (actix-web + tokio + SeaORM), Expo mobile, Storybook, AI via OpenRouter + orkey, and Cloudflare Email get added only when the project actually reaches for them.
