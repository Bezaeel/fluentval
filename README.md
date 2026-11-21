# FluentVal <img src="https://github.com/Bezaeel/fluentval/actions/workflows/publish.yml/badge.svg" alt="actions status" />

A fluent validation library for Rust. FluentVal provides a chainable, type-safe API for validating data structures with rich error reporting.

## Features

- Fluent API — chain validation rules in a readable, expressive way
- Built-in rules for strings, numbers, emails, and `Option`-wrapped values
- Conditional rules via `.when(...)`
- Custom rules via `.rule(...)` and cross-property `.must(...)`
- Rich error reporting with per-property grouping
- Two equivalent usage styles: explicit (no macro) or `rule_for!` macro sugar
- Thread-safe — `FluentValidator<T>` is `Send + Sync`, so build it once and share it (e.g. `Arc` app state or a `static`)

## Installation

```toml
[dependencies]
fluentval = "2.0.0"
```

## Quick Start

```rust
use fluentval::{FluentValidator, rule_for, Validator};

struct User {
    name: String,
    email: String,
    age: i32,
}

let v = FluentValidator::<User>::new();
rule_for!(v, |u| &u.name)
    .not_empty(None::<String>)
    .min_length(2, None::<String>)
    .max_length(50, None::<String>);
rule_for!(v, |u| &u.email)
    .not_empty(None::<String>)
    .email(None::<String>);
rule_for!(v, |u| &u.age)
    .greater_than_or_equal(18, None::<String>);

let user = User { name: "John Doe".into(), email: "john@example.com".into(), age: 25 };

let result = v.validate(&user);
if result.is_valid() {
    println!("User is valid!");
} else {
    for error in result.errors() {
        println!("{}: {}", error.property, error.message);
    }
}
```

## Two Usage Styles

### Explicit (no macro)

```rust
let v = FluentValidator::<User>::new();
v.rule_for("name", |u| &u.name).not_empty(None::<String>).min_length(2, None::<String>);
v.rule_for("email", |u| &u.email).not_empty(None::<String>).email(None::<String>);
let result = v.validate(&user);
```

### `rule_for!` macro

The macro takes a closure `|c| &c.field` and supplies the property name automatically via `stringify!`:

```rust
rule_for!(v, |u| &u.name).not_empty(None::<String>);
// expands to: v.rule_for("name", |u: &_| &u.name).not_empty(None::<String>);
```

Both styles produce identical validators — pick whichever you prefer.

## Cross-Property Validation

The `must` method on the rule chain receives the full object and the field value:

```rust
use fluentval::{FluentValidator, rule_for, Validator};

struct Command {
    country_iso_code: String,
    phone_number: String,
    alt_phone_number: String,
}

fn is_valid_phone(phone: &str, country: &str) -> bool {
    match country {
        "US" => phone.len() == 10 && phone.chars().all(|c| c.is_ascii_digit()),
        _    => phone.len() >= 8 && phone.len() <= 15,
    }
}

let v = FluentValidator::<Command>::new();
rule_for!(v, |c| &c.phone_number)
    .not_empty(None::<String>)
    .must(|cmd, phone| is_valid_phone(phone, &cmd.country_iso_code),
          "Phone number is not valid for the specified country");
rule_for!(v, |c| &c.alt_phone_number)
    .not_empty(None::<String>)
    .must(|cmd, alt| alt != &cmd.phone_number,
          "Alternative phone must be different from primary");
```

## Conditional Rules with `.when()`

`.when(predicate)` gates the entire rule chain on a struct-level condition. When the predicate returns `false`, the property is treated as valid — no rules fire. Useful for "this field is only required when X" scenarios.

```rust
use fluentval::{FluentValidator, rule_for, Validator};

struct Checkout {
    payment_method: String,
    billing_address: String,
}

let v = FluentValidator::<Checkout>::new();
rule_for!(v, |c| &c.billing_address)
    .not_empty(Some("billing address is required for card payments"))
    .when(|c| c.payment_method == "CARD");

// CARD + empty billing  → fails
// CASH + empty billing  → passes (condition false, rule skipped)
// CARD + valid billing  → passes
```

**Semantics:**

- The condition is evaluated against `&T` (the full struct), not the field value.
- Position in the chain doesn't matter — `.when(...)` gates every rule on the builder regardless of order.
- Multiple `.when(...)` calls **AND** together:

```rust
rule_for!(v, |o| &o.tracking_code)
    .not_empty(None::<String>)
    .when(|o| o.country == "US")
    .when(|o| o.priority == "EXPRESS");   // only required for US + EXPRESS orders
```

## Validating `Option<V>` Fields

Rules apply transparently through `Option<V>` — when the field is `None`, value rules pass silently; when it's `Some(_)`, the inner value is checked. Pair with `.not_null(...)` to require presence.

```rust
use fluentval::{FluentValidator, rule_for, Validator};

struct Person {
    marital_status: String,
    age: Option<usize>,
}

let v = FluentValidator::<Person>::new();
rule_for!(v, |p| &p.age)
    .not_null(Some("age is required for married persons"))   // None → error
    .greater_than_or_equal(18, Some("must be at least 18"))  // Some(<18) → error; None → skipped
    .when(|p| p.marital_status == "Married");

// Married + None      → "age is required..."
// Married + Some(15)  → "must be at least 18"
// Married + Some(25)  → passes
// Single  + anything  → passes (when-condition false)
```

This works for all string and numeric rules on `Option<V>`:

- String: `not_empty`, `min_length`, `max_length`, `length`, `email`
- Numeric: `greater_than`, `greater_than_or_equal`, `less_than`, `less_than_or_equal`, `inclusive_between`

## Custom Error Messages

Every rule accepts an optional custom message:

```rust
rule_for!(v, |u| &u.name)
    .not_empty(Some("Name is required"))
    .min_length(2, Some("Name must be at least 2 characters"));
rule_for!(v, |u| &u.age)
    .greater_than_or_equal(18, Some("You must be at least 18"));
```

Pass `None::<String>` to use the default message.

## Available Rules

### String (`V: AsRef<str>`) — and `Option<V>` thereof

- `not_empty(msg)` — non-empty after trimming
- `min_length(n, msg)` / `max_length(n, msg)` / `length(min, max, min_msg, max_msg)`
- `email(msg)`

### Numeric (`V: Numeric`) — and `Option<V>` thereof

- `greater_than(n, msg)` / `greater_than_or_equal(n, msg)`
- `less_than(n, msg)` / `less_than_or_equal(n, msg)`
- `inclusive_between(min, max, msg)`

`Numeric` is implemented for `i8…i64`, `isize`, `u8…u64`, `usize`, `f32`, `f64`.

### Option (`V: OptionLike`)

- `not_null(msg)` — value is `Some(_)`

### Conditional

- `when(|t| -> bool)` — gate the rule chain on a struct-level predicate

### Custom

- `rule(|v| -> Option<String>)` — custom value-level rule
- `must(|t, v| -> bool, msg)` — predicate with access to the whole struct

## Standalone `RuleBuilder`

`RuleBuilder<T, V>` is the underlying type. You can build a single rule fn without a validator — useful for unit-testing a rule chain or composing rules manually:

```rust
use fluentval::RuleBuilder;

// Cross-property rule against an enclosing struct:
let rule = RuleBuilder::for_property("alt_phone", |r: &Request| &r.alt_phone)
    .not_empty(None::<String>)
    .must(|r, v| v != &r.primary_phone, "alt phone cannot match primary")
    .build();
let errors = rule(&Request { primary_phone: "x".into(), alt_phone: "x".into() });

// Rule on a bare value (identity accessor — T = V = String):
let rule = RuleBuilder::for_property("name", |s: &String| s)
    .not_empty(None::<String>)
    .min_length(2, None::<String>)
    .build();
let errors = rule(&"".to_string());
```

`.build()` consumes the builder and returns `impl Fn(&T) -> Vec<ValidationError>`. When the builder comes from `FluentValidator::rule_for(...)` instead, you don't call `.build()` — it auto-registers when dropped at the end of the statement.

## Advanced

### Multiple rule statements for the same property

Rules accumulate across separate `rule_for!` calls on the same field:

```rust
let v = FluentValidator::<User>::new();
rule_for!(v, |u| &u.name).not_empty(None::<String>).min_length(2, None::<String>);
rule_for!(v, |u| &u.name).max_length(50, None::<String>);    // merged with the first
```

### Custom property label

If you want a property label that differs from the Rust field name (e.g., `taxId` for the field `tax_id`), use the explicit form:

```rust
v.rule_for("taxId", |u| &u.tax_id).not_empty(None::<String>);
```

### Nested field paths

```rust
struct Address { city: String }
struct Person { address: Address }

let v = FluentValidator::<Person>::new();
rule_for!(v, |p| &p.address.city).not_empty(None::<String>);
// Error property will be "address.city"
```

### Working with results

```rust
let result = v.validate(&user);

if result.is_valid() { /* ... */ }
for error in result.errors() {
    println!("{}: {}", error.property, error.message);
}

let by_prop = result.errors_by_property();
if let Some(msg) = result.first_error_for("email") {
    println!("Email error: {}", msg);
}
```

## Breaking Changes in v2.0

`FluentValidator<T>` and rules produced by `RuleBuilder::build()` are now `Send + Sync`. To make that possible, every closure you pass in — the accessor, `.rule(...)`, `.when(...)`, and `.must(...)` — must also be `Send + Sync`.

Most code is unaffected. You only need to change rules whose closures capture non-thread-safe values such as `Rc`, `Cell`, or `RefCell`; swap those for `Arc`, atomics, `Mutex`, or `RwLock`.

```rust
// v1.x — compiled, but the validator could not be shared across threads
let banned = Rc::new(vec!["admin".to_string()]);
v.rule_for("name", |u: &User| &u.name).must(move |_, n| !banned.contains(n), "reserved");

// v2.0 — use Arc instead of Rc
let banned = Arc::new(vec!["admin".to_string()]);
v.rule_for("name", |u: &User| &u.name).must(move |_, n| !banned.contains(n), "reserved");
```

Minimum supported Rust version is now **1.80**.

## Breaking Changes from v0.x

v1.0 is a clean break. The previous `ValidatorBuilder` / standalone `RuleBuilder<V>` API has been removed and replaced with a single unified surface.

| v0.x | v1.0 |
|---|---|
| `ValidatorBuilder<T>` | **Removed.** Use `FluentValidator<T>`. |
| `validate(&user, &validator)` free fn | **Removed.** Call `validator.validate(&user)` directly. |
| `RuleBuilder<V>` (single generic) | `RuleBuilder<T, V>` (struct type + field type). |
| `RuleBuilder::<V>::for_property("name")` | `RuleBuilder::for_property("name", \|v: &V\| v)` for value-only; `for_property("name", \|t: &T\| &t.field)` for struct-aware. |
| `.must(\|v\| -> bool, msg)` (1-arg closure) | `.must(\|t, v\| -> bool, msg)` (2-arg; use `\|_, v\| …` for value-only). |
| `let validator = v.build();` then `validator.validate(&u)` | Just `v.validate(&u)` — no separate build step. |
| `PropertyRuleBuilder<T, V>` (internal) | **Removed.** Merged into `RuleBuilder<T, V>`. |
| `ValidationError` had a `property: String` field — unchanged | (No change.) |

### New in v1.0

- **`.when(predicate)`** — conditional rule chain (predicate receives `&T`).
- **Option-transparent rules** — string and numeric rules apply through `Option<V>` automatically; `None` values pass, `Some(_)` is checked.
- **`NotOption` marker trait** (sealed) — used internally to disambiguate bare-value impls from `Option<V>` impls. You generally won't reference it directly.
- `Numeric` now covers `isize` and `usize` in addition to the other integer/float primitives.
- `RuleBuilder` from `FluentValidator::rule_for(...)` auto-registers on drop — no explicit registration call needed.

### Migration sketch

```rust
// v0.x
let validator = ValidatorBuilder::<User>::new()
    .rule_for(
        "name",
        |u| &u.name,
        RuleBuilder::for_property("name")
            .not_empty(None::<String>)
            .min_length(2, None::<String>),
    )
    .must("phone", |u| &u.phone,
          |u, phone| phone != &u.alt_phone, "phone must differ from alt_phone")
    .build();
let result = validate(&user, &validator);

// v1.0
let v = FluentValidator::<User>::new();
rule_for!(v, |u| &u.name).not_empty(None::<String>).min_length(2, None::<String>);
rule_for!(v, |u| &u.phone)
    .must(|u, phone| phone != &u.alt_phone, "phone must differ from alt_phone");
let result = v.validate(&user);
```

## License

MIT — see [LICENSE](LICENSE).

## Contributing

Contributions welcome — please open a PR or issue.
