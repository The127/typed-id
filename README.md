# typed-id

Proc-macros for generating typed UUID and slug newtype wrappers with zero runtime cost.

## Motivation

Raw `Uuid` fields look identical to the compiler. Nothing stops you from passing a `user_id`
where an `order_id` is expected — both are `Uuid`, so the call compiles and the bug ships.

```rust
// Without typed-id: compiles, but wrong
fn ship_order(user_id: Uuid, order_id: Uuid) { ... }
ship_order(order.id, user.id); // transposed — no error
```

`typed-id` wraps each ID in its own newtype. The compiler then enforces correct use at every
call site, with no overhead at runtime — the wrappers are `#[repr(transparent)]`-equivalent
newtypes that optimise away completely.

```rust
// With typed-id: transposed arguments are a compile error
fn ship_order(user_id: UserId, order_id: OrderId) { ... }
ship_order(order.id, user.id); // error[E0308]: mismatched types
```

Slug IDs add a second layer of safety: the slug grammar is enforced on construction, so an
invalid slug string can never be stored inside a `ProjectSlug`.

## Features

- `uuid_id!(TypeName)` — emits a `Uuid`-backed newtype with `new()`, `from_uuid()`, `as_uuid()`
- `slug_id!(TypeName)` — emits a validated `String`-backed newtype with `new()`, `as_str()`, `Display`, `FromStr`, `TryFrom`
- Serde support out of the box on both kinds (`Serialize` + `Deserialize`)
- `Deserialize` on slug types validates the grammar — invalid JSON strings are rejected at parse time
- Optional `utoipa::ToSchema` derivation via the `openapi` feature flag
- No runtime dependencies — `uuid` and `serde` stay on the consumer crate

## Installation

```toml
[dependencies]
typed-id = { path = "../typed-id" }           # or version = "0.1"
uuid = { version = "1", features = ["v4"] }   # required by uuid_id!
serde = { version = "1", features = ["derive"] }

# Optional: enable utoipa schema generation
# typed-id = { ..., features = ["openapi"] }
# utoipa = "5"
```

## Usage

### UUID IDs

```rust
use typed_id::uuid_id;

uuid_id!(UserId);
uuid_id!(OrderId);

// Construction
let user_id  = UserId::new();                          // random v4
let order_id = OrderId::from_uuid(uuid::Uuid::new_v4());

// Access the inner value
let raw: &uuid::Uuid = user_id.as_uuid();

// Type safety — this does not compile:
// fn process(user: UserId, order: OrderId) {}
// process(order_id, user_id); // error: expected UserId, found OrderId

// Serde round-trip (serialises as a plain UUID string)
let json = serde_json::to_string(&user_id).unwrap();  // "\"550e8400-...\""
let back: UserId = serde_json::from_str(&json).unwrap();
assert_eq!(user_id, back);

// Default delegates to new()
let id: UserId = Default::default();
```

### Slug IDs

```rust
use typed_id::slug_id;

slug_id!(ProjectSlug);

// Validated construction
let slug = ProjectSlug::new("my-project").unwrap();

// Parse via FromStr / TryFrom
let slug: ProjectSlug = "my-project".parse().unwrap();
let slug = ProjectSlug::try_from("my-project").unwrap();

// Display renders the inner string
println!("{slug}");          // my-project
println!("{}", slug.as_str()); // my-project

// Invalid slugs are rejected
assert!(ProjectSlug::new("").is_err());             // empty
assert!(ProjectSlug::new("My-Project").is_err());   // uppercase
assert!(ProjectSlug::new("-leading").is_err());     // leading hyphen
assert!(ProjectSlug::new("trailing-").is_err());    // trailing hyphen
assert!(ProjectSlug::new("has space").is_err());    // whitespace

// Deserialisation also validates
let bad: Result<ProjectSlug, _> = serde_json::from_str("\"Bad_Slug\"");
assert!(bad.is_err());
```

### Type safety in practice

```rust
uuid_id!(UserId);
uuid_id!(OrderId);

fn assign_order_to_user(user: UserId, order: OrderId) {
    // compiler guarantees the arguments cannot be transposed
}

let user  = UserId::new();
let order = OrderId::new();

assign_order_to_user(user, order);   // OK
// assign_order_to_user(order, user); // error[E0308]: mismatched types
```

### OpenAPI (`utoipa`) integration

Enable the `openapi` feature on the consuming crate:

```toml
[features]
openapi = ["typed-id/openapi", "utoipa"]
```

The macros emit `#[cfg_attr(feature = "openapi", derive(::utoipa::ToSchema))]` on every
generated struct, so your ID types appear as plain `string` / `uuid` schemas in the generated
OpenAPI document without any extra boilerplate:

```rust
uuid_id!(UserId);   // ToSchema emitted when feature = "openapi"
slug_id!(OrgSlug);  // ToSchema emitted when feature = "openapi"

#[derive(utoipa::ToSchema)]
pub struct UserDto {
    pub id:   UserId,   // renders as { type: string, format: uuid }
    pub slug: OrgSlug,  // renders as { type: string }
}
```

## Design notes

**Newtype emission.** Each macro call expands to a single `pub struct Name(Inner)` plus an
`impl` block. No traits are implemented by hand where a `derive` suffices; the macro simply
emits the `#[derive(...)]` attribute on the generated struct.

**Traits derived on `uuid_id!` types:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`,
`Serialize`, `Deserialize`, `Default`.

**Traits derived / implemented on `slug_id!` types:** `Debug`, `Clone`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, `Hash`, `Serialize`. `Deserialize` is hand-rolled so that deserialising
an invalid slug string is a hard error rather than a silent success.  `Display`, `FromStr`,
`TryFrom<String>`, and `TryFrom<&str>` are also emitted as `impl` blocks.

**Consumer-side dependencies.** `typed-id` itself only depends on `proc-macro2`, `quote`, and
`syn` — all build-time only. `uuid` and `serde` are referenced with fully-qualified paths
(`::uuid::Uuid`, `::serde::Serialize`) in the emitted code, so the consuming crate must add
them to its own `[dependencies]`. This keeps `typed-id` from forcing transitive runtime deps
on users who might already pin their own versions.

**`openapi` feature.** The feature is declared on the *consuming* crate, not on `typed-id`
itself. The macros emit `#[cfg_attr(feature = "openapi", ...)]` tokens so the derive is
activated only when the feature is enabled in the final compilation context.

## License

MIT
