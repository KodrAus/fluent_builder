/*!
A simple builder for constructing or mutating values.

This crate provides a simple `FluentBuilder` structure.
It offers some standard behaviour for constructing values from a given source, or by mutating a default that's supplied later.
This crate is intended to be used within other builders rather than consumed by your users directly.
It's especially useful for managing the complexity of keeping builders ergonomic when they're nested within other builders.

This crate is currently designed around builders that take self by-value instead of by-reference.

# Usage

Create a `FluentBuilder` and construct a default value:

```
use fluent_builder::FluentBuilder;

let value = FluentBuilder::<String>::default()
    .into_value(|| "A default value".to_owned());

assert_eq!("A default value", value);
```

Values can be supplied to the builder directly.
In this case that value will be used instead of constructing the default:

```
# use fluent_builder::FluentBuilder;
let value = FluentBuilder::<String>::default()
    .value("A value".to_owned())
    .into_value(|| "A default value".to_owned());

assert_eq!("A value", value);
```

Mutating methods will either be applied to a concrete value, or the constructed default:

```
# use fluent_builder::FluentBuilder;
let value = FluentBuilder::<String>::default()
    .fluent_mut(|s| s.push_str(" fluent2"))
    .into_value(|| "A default value".to_owned());

assert_eq!("A default value fluent2", value);
```

## Stacking fluent methods

Fluent methods are overriden by default each time `.fluent` is called, but can be configured to maintain state across calls using a generic parameter:

```
use fluent_builder::{FluentBuilder, Stack};

let value = FluentBuilder::<String, Stack>::default()
    .fluent_mut(|s| s.push_str(" fluent1"))
    .fluent_mut(|s| s.push_str(" fluent2"))
    .into_value(|| "A default value".to_owned());

assert_eq!("A default value fluent1 fluent2", value);
```

Which option is best depends on the use-case.
For collection-like values it might make more sense to use stacking builders.
For other kinds of values it probably makes more sense to use overriding builders, so they're the default choice.
Using a generic parameter instead of some value to control whether or not fluent methods are stacked means you can enforce a particular style through Rust's type system.

## Stateful builders

Fluent builders can also be used to thread required state through construction:

```
use fluent_builder::StatefulFluentBuilder;

#[derive(Debug, PartialEq, Eq)]
struct Builder {
    required: String,
    optional: Option<String>,
}

let value = StatefulFluentBuilder::<String, Builder>::from_seed("A required value".to_owned())
    .fluent_mut("A required value".to_owned(), |b| {
        if let Some(ref mut optional) = b.optional.as_mut() {
            optional.push_str(" fluent1");
        }
    })
    .into_value(|s| Builder {
        required: s,
        optional: Some("A default value".to_owned())
    });

assert_eq!("A required value", value.required);
assert_eq!("A default value fluent1", value.optional.unwrap());
```

Stateful builders can also stack fluent methods instead of overriding them.
The API requires each invocation of `fluent` deals with the required state:

```
# #[derive(Debug, PartialEq, Eq)]
# struct Builder {
#     required: String,
#     optional: Option<String>,
# }
use fluent_builder::{StatefulFluentBuilder, Stack};

let value = StatefulFluentBuilder::<String, Builder, Stack>::from_seed("A required value".to_owned())
    .fluent_mut("A required value".to_owned(), |s, b| {
        b.required = s;
        if let Some(ref mut optional) = b.optional.as_mut() {
            optional.push_str(" fluent1");
        }
    })
    .fluent_mut("A required value".to_owned(), |s, b| {
        b.required = s;
        if let Some(ref mut optional) = b.optional.as_mut() {
            optional.push_str(" fluent2");
        }
    })
    .into_value(|s| Builder {
        required: s,
        optional: Some("A default value".to_owned())
    });

assert_eq!("A required value", value.required);
assert_eq!("A default value fluent1 fluent2", value.optional.unwrap());
```

## Within other builders

The `FluentBuilder` and `StatefulFluentBuilder` types are designed to be used within other builders rather than directly.
They just provide some consistent underlying behaviour with respect to assigning and mutating inner builders:

```rust
use fluent_builder::{BoxedFluentBuilder, Stack};

#[derive(Default)]
struct RequestBuilder {
    // Use a `FluentBuilder` to manage the inner `BodyBuilder`
    body: BoxedFluentBuilder<BodyBuilder, Stack>,
}

#[derive(Default)]
struct BodyBuilder {
    bytes: Vec<u8>,
}

impl<B> From<B> for BodyBuilder
where
    B: AsRef<[u8]>
{
    fn from(bytes: B) -> Self {
        BodyBuilder {
            bytes: bytes.as_ref().to_vec()
        }
    }
}

struct Request {
    body: Body
}

struct Body(Vec<u8>);

impl RequestBuilder {
    fn new() -> Self {
        RequestBuilder {
            body: BoxedFluentBuilder::default(),
        }
    }

    // Accept any type that can be converted into a `BodyBuilder`
    // This will override any previously stored state or default
    fn body<B>(mut self, body: B) -> Self
    where
        B: Into<BodyBuilder>
    {
        self.body = self.body.value(body.into());
        self
    }

    // Mutate some `BodyBuilder` without having to name its type
    // If there's no previously supplied concrete value then some
    // default will be given on `build`
    fn body_fluent<F>(mut self, body: F) -> Self
    where
        F: Fn(BodyBuilder) -> BodyBuilder + 'static
    {
        self.body = self.body.fluent(body).boxed();
        self
    }

    fn build(self) -> Request {
        // Get a `Body` by converting the `FluentBuilder` into a `BodyBuilder`
        let body = self.body.into_value(|| BodyBuilder::default()).build();

        Request {
            body: body
        }
    }
}

impl BodyBuilder {
    fn append(mut self, bytes: &[u8]) -> Self {
        self.bytes.extend(bytes);
        self
    }

    fn build(self) -> Body {
        Body(self.bytes)
    }
}

// Use a builder to construct a request using fluent methods
let request1 = RequestBuilder::new()
    .body_fluent(|b| b.append(b"some"))
    .body_fluent(|b| b.append(b" bytes"))
    .build();

// Use a builder to construct a request using a given value
let request2 = RequestBuilder::new()
    .body(b"some bytes")
    .build();

assert_eq!(request1.body.0, request2.body.0);
```

This seems like a lot of boilerplate, but comes in handy when you have a lot of potentially nested builders and need to keep them consistent.
There's nothing really special about the above builders besides the use of `FluentBuilder`.
*/

mod imp;

pub use self::imp::{
    Boxed, BoxedFluentBuilder, BoxedStatefulFluentBuilder, DefaultStack, DefaultStorage,
    FluentBuilder, Inline, Override, Shared, SharedFluentBuilder, SharedStatefulFluentBuilder,
    Stack, StatefulFluentBuilder, TryIntoValue,
};
