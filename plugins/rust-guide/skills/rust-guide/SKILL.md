---
name: rust-guide
description: ☕️ cafe · Opinionated Rust style guide — makes AI-written Rust code look like a human wrote it. Always active when writing Rust.
user-invocable: false
disable-model-invocation: false
---

# Rust Style Guide

This skill shapes how you write Rust code. Follow every rule in this document whenever you write, modify, or suggest Rust code. These are not suggestions — they are requirements.

---

## Philosophy

Five principles. Internalize them — every rule below flows from one of these.

1. **Clarity over cleverness.** Write code that reads like prose, not a puzzle. If someone has to re-read a line to understand it, rewrite it.
2. **Minimal abstraction.** Don't build a framework for a one-time operation. Three similar lines of code are better than a premature abstraction. No traits with one impl. No generics for one type.
3. **Own your errors.** Never panic in production. Every failure path is intentional and handled.
4. **Let the compiler work.** Lean on type inference, lifetime elision, and exhaustive matching. Only annotate what the compiler can't figure out on its own.
5. **Flat is better than nested.** Early returns, guard clauses, and `let-else` keep indentation shallow. If you're three levels deep, refactor.

---

## Formatting

Use `rustfmt` with this configuration:

```toml
max_width = 100
tab_spaces = 4
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
reorder_modules = true
fn_params_layout = "Tall"
use_try_shorthand = true
```

- **Line length:** 100 characters hard limit. Break before hitting it, not after.
- **Trailing commas:** Always on multi-line constructs — struct fields, enum variants, function args, match arms. This produces cleaner diffs.

---

## Naming

| What | Convention | Examples |
|------|-----------|---------|
| Variables | `snake_case`, descriptive | `staff_roles`, `reminder_ids` |
| Functions | `snake_case`, verb-first | `get_user_roles`, `has_staff_access`, `is_dev` |
| Types | `PascalCase` | `ReminderStatus`, `PostCursor`, `AIService` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_REMINDERS_PER_USER`, `CACHE_TTL_SECS` |
| Modules | `snake_case` | `ai_service`, `postgres_service` |
| Type aliases | `PascalCase` | `AppSchema`, `ApiResult<T>` |

**Booleans** always get a verb prefix: `is_`, `has_`, `should_`, `can_`.

```rust
pub fn is_active(&self) -> bool
pub fn has_staff_access(&self) -> bool
```

---

## Project Structure

Use the **`mod.rs` convention** — a directory with `mod.rs` inside it.

Organize by **domain**, not by type. Each feature gets its own directory. One logical concern per file.

```
src/
  commands/
    mod.rs
    ping.rs
    remind.rs
  events/
    mod.rs
    message_send.rs
  modules/
    mod.rs
    api/
      mod.rs
      routes.rs
      types/
        mod.rs
        error.rs
        response.rs
  types/
    mod.rs
    reminder.rs
  config.rs
  main.rs
```

---

## Imports

**All imports go at the top of the file.** Never use function-scoped imports.

**Order:** Standard library first, then external crates, then internal modules. The `rustfmt` config handles this automatically with `group_imports = "StdExternalCrate"`.

**Group imports from the same crate** using `imports_granularity = "Crate"`:

```rust
// Correct
use crate::types::reminder::{Reminder, ReminderStatus, RecurringSchedule};

// Wrong — one per line from the same crate
use crate::types::reminder::Reminder;
use crate::types::reminder::ReminderStatus;
use crate::types::reminder::RecurringSchedule;
```

---

## Error Handling

### No Panics in Production

Never use `unwrap()` or `expect()` in production code. Every fallible operation uses `?` or is handled explicitly.

```rust
// Wrong
let config = serde_json::from_str(&data).unwrap();

// Correct
let config = serde_json::from_str(&data)?;
```

### When to Use `thiserror` vs `anyhow`

- **`thiserror`** — API boundaries, public-facing types, anywhere callers match on the error.
- **`anyhow`** — Internal plumbing, CLI tools, scripts, glue code where you just propagate.

### Custom Error Enums

Model errors as enums. Map to HTTP status codes. Never leak internal details to users:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Db(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        self.log_internal_error();
        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: self.kind(),
            message: self.safe_message(),
        })
    }
}
```

### Guard Clauses with Early Returns

Flatten your logic. Don't nest:

```rust
// Correct — flat
let Some(guild_id) = msg.guild_id else {
    return Ok(());
};

// Wrong — nested
if let Some(guild_id) = msg.guild_id {
    // entire function body indented one level
}
```

---

## Returns

Use **implicit returns** for the final expression. Use **explicit `return`** only for early exits:

```rust
// Correct — implicit
pub fn is_active(&self) -> bool {
    self.status == Status::Active
}

// Correct — explicit early return only
pub async fn process(&self, msg: &Message) -> Result<(), Error> {
    let Some(guild_id) = msg.guild_id else {
        return Ok(());
    };
    // ...
    Ok(())
}

// Wrong — explicit return at end
pub fn is_active(&self) -> bool {
    return self.status == Status::Active;
}
```

---

## Structs

### Use `pub` Fields on Data Types

Data types and DTOs get `pub` fields. No getter/setter boilerplate:

```rust
// Correct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub creator_id: u64,
    pub message: String,
    pub status: ReminderStatus,
}

// Wrong — this isn't Java
impl Reminder {
    pub fn id(&self) -> &str { &self.id }
    pub fn creator_id(&self) -> u64 { self.creator_id }
}
```

**Exception:** Service wrappers (DB clients, API wrappers) use `pub(crate)` fields:

```rust
#[derive(Clone)]
pub struct PostgresService {
    pub(crate) connection: DatabaseConnection,
    pub(crate) cache: Option<RedisService>,
}
```

### Builders for Complex Construction

When a struct has optional fields or 4+ parameters, use a fluent builder:

```rust
impl AnalyticsEvent {
    pub fn new(event_type: EventType, guild_id: u64) -> Self {
        Self {
            event_type: event_type.as_str().to_string(),
            guild_id,
            user_id: 0,
            channel_id: 0,
            metadata: String::new(),
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    pub fn user(mut self, user_id: u64) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn channel(mut self, channel_id: u64) -> Self {
        self.channel_id = channel_id;
        self
    }

    pub fn meta(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = metadata.into();
        self
    }
}

// Usage
data.track(
    AnalyticsEvent::new(EventType::MessageSend, guild_id)
        .user(author_id)
        .channel(channel_id)
        .meta(json!({"command": name}).to_string()),
);
```

Simple structs with 2-3 required fields get a `new()` constructor instead:

```rust
impl RedisService {
    pub async fn new(uri: &str) -> RedisResult<Self> {
        let client = redis::Client::open(uri)?;
        let connection_manager = ConnectionManager::new(client).await?;
        Ok(Self { connection_manager })
    }
}
```

### Layer Conversions with `From`

Use `From` implementations to convert between layers (DB model to API type, etc.):

```rust
impl From<entity::post::Model> for Post {
    fn from(model: entity::post::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            source: model.source,
            media_url: media_url_for(model.id),
        }
    }
}
```

---

## Enums

### State as Enums, Never Strings

Model state as typed enums. Never use raw strings or integers for status:

```rust
// Correct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReminderStatus {
    Pending,
    Sent,
    Cancelled,
}

// Wrong
let status = "pending";
```

### Tagged Serialization

Use `#[serde(tag = "type")]` for enums with associated data:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecurringSchedule {
    Interval {
        interval_seconds: i64,
    },
    FixedTime {
        hour: u8,
        minute: u8,
        timezone: String,
    },
}
```

### Convenience Methods on the Type

```rust
impl Reminder {
    pub fn is_pending(&self) -> bool {
        self.status == ReminderStatus::Pending
    }

    pub fn is_recurring(&self) -> bool {
        self.recurring.is_some()
    }
}
```

### `Display` for User-Facing Enums

```rust
impl std::fmt::Display for GiveawayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "Active",
            Self::Ended => "Ended",
            Self::Cancelled => "Cancelled",
        };
        write!(f, "{}", s)
    }
}
```

---

## Functions

### Always Name Return Types

Never use `impl Trait` in return position. Spell out the concrete type:

```rust
// Correct
pub async fn get_posts(&self, limit: u64) -> Result<Vec<PostModel>, AppError>

// Wrong
pub async fn get_posts(&self, limit: u64) -> impl Future<Output = Result<Vec<PostModel>, AppError>>
```

### Generic Bounds as `where` Clauses

Use `where` clauses for readability when there are generic bounds:

```rust
pub async fn get_user_roles<C>(
    http: &C,
    guild_id: GuildId,
    user_id_str: &str,
) -> Result<Vec<String>, Error>
where
    C: CacheHttp,
{
    // ...
}
```

Single simple bounds inline are acceptable: `pub fn diff<T: Eq + Hash + Clone>(...)`

### String Parameters — Choose by Ownership

| The function... | Parameter type |
|----------------|---------------|
| Only reads | `&str` |
| Needs to own it | `impl Into<String>` |
| Caller gives ownership explicitly | `String` |

```rust
// Reading only
pub fn validate_url(value: &str, field_name: &str) -> Result<(), AppError>

// Needs ownership — flexible for callers
pub fn meta(mut self, metadata: impl Into<String>) -> Self {
    self.metadata = metadata.into();
    self
}
```

---

## Pattern Matching

### `match` for 2+ Arms, `if let` for Singles

```rust
// match — two or more branches
match self.status {
    UserTier::Anonymous => Some(50),
    UserTier::Free => Some(100),
    UserTier::Premium | UserTier::Admin => None,
}

// if let — checking a single variant
if let Some(cache) = &self.cache {
    cache.del(&key).await?;
}

// let-else — guard clause
let Some(mut reminder) = data.redis.claim_reminder(id).await? else {
    return Ok(());
};
```

### Exhaustive Matching on Your Own Enums

Never use wildcard `_` to catch variants you control. The compiler should warn when new variants are added:

```rust
// Correct — every variant listed
match self {
    Self::MessageSend => "message_send",
    Self::MessageEdit => "message_edit",
    Self::MessageDelete => "message_delete",
    Self::MemberJoin => "member_join",
}

// Wrong — swallows new variants silently
match self {
    Self::MessageSend => "message_send",
    _ => "unknown",
}
```

### Tuple Matching for Combined Conditions

```rust
let expired = match (reminder.expires_at, next_at) {
    (Some(expires), Some(next)) => next > expires,
    (Some(expires), None) => now >= expires,
    _ => false,
};
```

---

## Iterator Chains

### One Method Per Line

Always break chained method calls onto separate lines:

```rust
// Correct
let staff_roles: Vec<u64> = env_var
    .split(',')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(|s| s.parse()?)
    .collect();

// Wrong
let staff_roles: Vec<u64> = env_var.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
```

### Iterators Over Loops

Prefer `.map()`, `.filter()`, `.any()`, `.find()` over manual loops when logic is straightforward:

```rust
// Iterator — clean
Ok(member
    .roles
    .iter()
    .map(|role_id| role_id.to_string())
    .collect())

// .any() for boolean checks
user_roles.iter().any(|role| staff_roles.contains(role))
```

Use a manual loop only when you need async operations or complex mutable state inside the loop body:

```rust
let mut result = Vec::new();
for post in posts {
    if db.is_post_visible(post.id, &tier).await? {
        result.push(post);
    }
}
```

---

## Cloning

**Context-dependent.** Clone is not inherently bad, but be intentional:

- **Fine** in startup, config, and handler-level code where performance doesn't matter.
- **Fine** for `Arc::clone()` — it's cheap (reference count bump).
- **Avoid** in hot paths, tight loops, and request handlers where a reference would work.

```rust
// Fine — startup, happens once
let http = ctx.http.clone();

// Fine — Arc is cheap
let data = data.clone();
tokio::spawn(async move { job.run(data).await });

// Wrong — unnecessary clone in a loop
for item in &items {
    process(item.clone()); // just take &Item
}
```

---

## Async

### Tokio Always

Use `tokio` as the async runtime. No exceptions.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ...
}
```

### Background Tasks

```rust
tokio::spawn(async move {
    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
    }
});
```

### Concurrency Control

Use `Arc<Semaphore>` to limit concurrent operations:

```rust
pub struct AIService {
    client: Arc<Client>,
    semaphore: Arc<Semaphore>,
}

pub async fn query(&self, input: &str) -> Result<String, Error> {
    let _permit = self.semaphore.acquire().await?;
    // permit drops automatically when scope ends
}
```

### Shared State

Wrap services in `Arc` for sharing across async tasks:

```rust
let postgres = Arc::new(PostgresService::new(&config.db_url).await?);
let redis = Arc::new(RedisService::new(&config.redis_url).await?);
```

---

## Logging

Use the `tracing` crate exclusively. Never use `println!` or `eprintln!` in library or server code.

**Info-heavy approach** — most operational logs at `info`, internals at `debug`:

| Level | What goes here |
|-------|---------------|
| `error!` | Something is broken and needs human attention |
| `warn!` | Degraded state, non-critical failures, fallbacks |
| `info!` | Lifecycle events, request handling, state changes |
| `debug!` | Cache hits/misses, query details, internal flow |
| `trace!` | Rarely used — extremely verbose diagnostics |

```rust
info!("Connected to Redis.");
warn!("Failed to cache message: {}", e);
error!(error = %e, "Media processing failed");
debug!("CACHE HIT in {:?}", elapsed);
```

### Instrument Async Functions

```rust
#[instrument(skip(self, ctx), fields(post_id = ?id))]
async fn create_post(&self, ctx: &Context<'_>, id: Option<Uuid>) -> Result<Post>
```

---

## Type Annotations

**Only when the compiler needs them.** Let inference do its job:

```rust
// Needed — collect() can't infer the target type
let roles: Vec<u64> = raw.split(',').map(|s| s.parse()?).collect();

// Not needed — obvious from context
let config = EnvConfig::from_env();
let reminder = data.redis.claim_reminder(id).await?;

// Turbofish when cleaner than a let binding annotation
let port = env::var("PORT")?.parse::<u16>()?;
```

---

## Testing

### Tests Live in `/tests`

Put tests in the `/tests` directory at the project root, not in inline `#[cfg(test)]` modules:

```rust
// tests/recurring_reminders.rs
#[test]
fn interval_adds_seconds() {
    let sched = RecurringSchedule::Interval {
        interval_seconds: 3600,
    };
    assert_eq!(sched.next_remind_at(1_000_000), Some(1_003_600));
}
```

### Test Helpers

Extract repeated setup into helper functions:

```rust
fn ts(datetime: &str, tz_str: &str) -> i64 {
    let tz: Tz = tz_str.parse().unwrap();
    let naive = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S").unwrap();
    tz.from_local_datetime(&naive).single().unwrap().timestamp()
}
```

### Cover Edge Cases

```rust
#[test]
fn invalid_timezone_returns_none() {
    let sched = RecurringSchedule::FixedTime {
        hour: 9,
        minute: 0,
        timezone: "Not/A/Timezone".to_string(),
    };
    assert_eq!(sched.next_remind_at(1_000_000), None);
}
```

---

## Documentation

**Document non-obvious code.** If the function name tells the full story, skip the doc comment. If it doesn't, write one:

```rust
// Needs a doc comment — name alone doesn't explain behavior
/// Computes the symmetric difference between two vectors.
/// Returns elements present in either `a` or `b`, but not in both.
pub fn diff<T: Eq + Hash + Clone>(a: &[T], b: &[T]) -> Vec<T>

// Does NOT need a doc comment — name is self-evident
pub fn is_pending(&self) -> bool {
    self.status == ReminderStatus::Pending
}
```

**Module-level docs** for complex modules:

```rust
//! Application-wide constants for limits, timeouts, and configuration defaults.
//! Centralizing these values makes them easy to find and modify.
```

---

## Visibility

| Modifier | When to use |
|----------|------------|
| `pub` | Public API — command handlers, route handlers, library exports |
| `pub(crate)` | Shared within the crate but not exposed externally |
| Private (none) | Helper functions, internal state, implementation details |

```rust
pub async fn ping(ctx: Context<'_>) -> Result<(), Error>
pub(crate) const MAX_REMINDERS: usize = 25;
fn normalize_url(url: &str) -> (String, bool)
```

---

## Constants

### Group Related Constants

Use section comments to organize:

```rust
// ── Request Limits ────────────────────────────────────────
pub const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;
pub const RATE_LIMIT_MAX_REQUESTS: u32 = 100;

// ── Cache TTLs ────────────────────────────────────────────
pub const CACHE_TTL_POST_SECS: u64 = 60 * 15;
const COMPLETED_REMINDER_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

// ── Pagination ────────────────────────────────────────────
pub const DEFAULT_PAGE_SIZE: u64 = 20;
pub const MAX_PAGE_SIZE: u64 = 100;
```

### `OnceLock` for Lazy Statics

```rust
pub static CONFIG: OnceLock<EnvConfig> = OnceLock::new();

pub fn config() -> &'static EnvConfig {
    CONFIG.get().expect("Config not initialized")
}
```

---

## Configuration

Load config from environment variables into a typed struct at startup. Use **runtime config** (env vars, config files) over compile-time feature flags:

```rust
#[derive(Clone, Debug)]
pub struct EnvConfig {
    pub port: i32,
    pub db_url: String,
    pub redis_url: String,
}

impl EnvConfig {
    fn get_env(key: &str) -> String {
        env::var(key).unwrap_or_else(|_| panic!("{} not set", key))
    }

    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            port: Self::get_env("PORT").parse().unwrap_or(8081),
            db_url: Self::get_env("POSTGRES_URL"),
            redis_url: Self::get_env("REDIS_URL"),
        }
    }
}
```

---

## Derive Order

Follow a consistent order:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]           // data types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]              // value types
#[derive(Debug, Error)]                                     // error types
```

### Serde Attributes

```rust
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case")]
```

---

## Web Serving

Use `actix-web` directly. No nginx or reverse proxy unless absolutely necessary.

### Service Composition

```rust
HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(postgres.clone()))
        .app_data(web::Data::new(redis.clone()))
        .configure(routes::configure)
})
```

### Typed API Responses

```rust
pub enum ApiResponse<T: Serialize> {
    Ok(T),
    EmptyOk,
    Created(T),
    NoContent,
}

pub type ApiResult<T> = Result<ApiResponse<T>, AppError>;
```

---

## Hard Rules — Never Do These

- `unwrap()` or `expect()` in production code
- Wildcard `_` matches on enums you control
- Commented-out code (delete it — git remembers)
- `println!` in server code
- Getter/setter boilerplate on data types
- `impl Trait` in return position
- Function-scoped imports
- Traits with one implementation
- Generics for one type
- Builders for two-field structs
- String-typed state (use enums)
- Nested `if let` chains (flatten with `let-else` or early returns)

## Always Do These

- Type aliases for long `Result<T, E>` and `Context` types
- `From` implementations for layer-boundary conversions
- Section comments (`// ── Section ──`) for grouping in long files
- `Arc<Semaphore>` for concurrency control
- Fluent builders for complex construction
- `tracing::instrument` on async functions
