# typed-id

Proc-macros for generating typed UUID and slug newtype wrappers with zero runtime cost.

## Why

Raw `Uuid` fields look identical to the compiler. Nothing stops you from passing a `user_id`
where an `order_id` is expected:

```rust
fn ship_order(user_id: Uuid, order_id: Uuid) { ... }
ship_order(order.id, user.id); // transposed — compiles fine, bug ships
```

`typed-id` wraps each ID in its own newtype so the compiler catches this:

```rust
fn ship_order(user_id: UserId, order_id: OrderId) { ... }
ship_order(order.id, user.id); // error[E0308]: mismatched types
```

The wrappers are `#[repr(transparent)]` newtypes — they optimise away completely at runtime.

Slug IDs go further: the slug grammar is enforced on construction, so an invalid slug string
can never be stored inside a `ProjectSlug`.

## What you get

- `uuid_id!(TypeName)` — `Uuid`-backed newtype with `new()`, `from_uuid()`, `as_uuid()`
- `slug_id!(TypeName)` — validated `String`-backed newtype with `new()`, `as_str()`, `Display`, `FromStr`, `TryFrom`
- `Serialize` + `Deserialize` on both kinds; slug `Deserialize` validates the grammar
- Optional `utoipa::ToSchema` via the `openapi` feature

No runtime deps are added — `uuid` and `serde` stay on the consumer crate.

## Installation

```toml
[dependencies]
raccoon-typed-id = "0.1"
uuid = { version = "1", features = ["v4"] }   # required by uuid_id!
serde = { version = "1", features = ["derive"] }

# Optional: enable utoipa schema generation
# raccoon-typed-id = { version = "0.1", features = ["openapi"] }
# utoipa = "5"
```

## Usage

### UUID IDs

```rust
use typed_id::uuid_id;

uuid_id!(UserId);
uuid_id!(OrderId);

let user_id  = UserId::new();                          // random v4
let order_id = OrderId::from_uuid(uuid::Uuid::new_v4());

let raw: &uuid::Uuid = user_id.as_uuid();

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

let slug = ProjectSlug::new("my-project").unwrap();
let slug: ProjectSlug = "my-project".parse().unwrap();
let slug = ProjectSlug::try_from("my-project").unwrap();

println!("{slug}");            // my-project
println!("{}", slug.as_str()); // my-project

// Invalid slugs are rejected at construction time...
assert!(ProjectSlug::new("").is_err());             // empty
assert!(ProjectSlug::new("My-Project").is_err());   // uppercase
assert!(ProjectSlug::new("-leading").is_err());     // leading hyphen
assert!(ProjectSlug::new("trailing-").is_err());    // trailing hyphen
assert!(ProjectSlug::new("has space").is_err());    // whitespace

// ...and at deserialisation
let bad: Result<ProjectSlug, _> = serde_json::from_str("\"Bad_Slug\"");
assert!(bad.is_err());
```

### OpenAPI (`utoipa`) integration

Enable the `openapi` feature on the consuming crate:

```toml
[features]
openapi = ["typed-id/openapi", "utoipa"]
```

The macros emit `#[cfg_attr(feature = "openapi", derive(::utoipa::ToSchema))]` on every
generated struct, so your ID types appear as plain `string` / `uuid` schemas with no extra
boilerplate:

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
`impl` block. Where a `derive` suffices the macro just emits the `#[derive(...)]` attribute
rather than hand-rolling the impl.

**Traits on `uuid_id!` types:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`,
`Serialize`, `Deserialize`, `Default`.

**Traits on `slug_id!` types:** `Debug`, `Clone`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, `Hash`, `Serialize`. `Deserialize` is hand-rolled so that deserialising
an invalid slug is a hard error. `Display`, `FromStr`, `TryFrom<String>`, and `TryFrom<&str>`
are emitted as `impl` blocks.

**Consumer-side dependencies.** `typed-id` depends only on `proc-macro2`, `quote`, and `syn`
— all build-time. `uuid` and `serde` are referenced by fully-qualified path in the emitted
code (`::uuid::Uuid`, `::serde::Serialize`), so the consuming crate must declare them in its
own `[dependencies]`. This avoids forcing transitive runtime deps on consumers who pin their
own versions.

**`openapi` feature.** Declared on the *consuming* crate, not on `typed-id` itself. The macros
emit `#[cfg_attr(feature = "openapi", ...)]` tokens, so the derive activates only when the
feature is enabled in the final compilation context.

## License

MIT
