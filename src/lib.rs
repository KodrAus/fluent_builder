/*!
A simple builder for constructing or mutating values.

This crate provides a simple `FluentBuilder` structure.
It offers some standard behaviour for constructing values from a given source, or by mutating a default that's supplied later.
This crate is intended to be used within other builders rather than consumed by your users directly.

# Usage

Create a `FluentBuilder` and construct a default value:

```
use fluent_builder::FluentBuilder;

let builder = FluentBuilder::<String>::default();

let value = builder.into_value(|| "A default value".to_owned());

assert_eq!("A default value", value);
```

Values can be supplied to the builder directly.
In this case that value will be used instead of constructing the default:

```
use fluent_builder::FluentBuilder;

let builder = FluentBuilder::<String>::default().value("A value".to_owned());

let value = builder.into_value(|| "A default value".to_owned());

assert_eq!("A value", value);
```

Mutating methods can be stacked and will either be applied to a concrete value, or the constructed default:

```
use fluent_builder::FluentBuilder;

let builder = FluentBuilder::<String>::default()
    .fluent_mut(|s| s.push_str(" fluent1"))
    .fluent_mut(|s| s.push_str(" fluent2"));

let value = builder.into_value(|| "A default value".to_owned());

assert_eq!("A default value fluent1 fluent2", value);
```

Fluent builders can also be used to thread required state through construction:

```
use fluent_builder::StatefulFluentBuilder;

#[derive(Debug, PartialEq, Eq)]
struct Builder {
    required: String,
    optional: Option<String>,
}

let builder = StatefulFluentBuilder::<Builder, String>::from_seed("A required value".to_owned())
    .fluent_mut("A required value".to_owned(), |b, s| {
        b.required = s;
        if let Some(ref mut optional) = b.optional.as_mut() {
            optional.push_str(" fluent1");
        }
    });

let value = builder.into_value(|s| Builder {
    required: s,
    optional: Some("A default value".to_owned())
});

assert_eq!("A required value", value.required);
assert_eq!("A default value fluent1", value.optional.unwrap());
```
*/

/**
A structure that can contain a value, or stack mutating methods over one supplied later.

The `FluentBuilder<T>` is effectively a `StatefulFluentBuilder<T, ()>`.
*/
pub struct FluentBuilder<T> {
    inner: StatefulFluentBuilder<T, ()>,
}

/**
A stateful structure that can contain a value, or stack mutating methods over one supplied later. 
*/
pub struct StatefulFluentBuilder<T, S> {
    inner: StatefulFluentBuilderInner<T, S>,
}

struct StatefulFluentBuilderInner<T, S>(State<T, S>, Option<Box<FnBox<T, T>>>);

enum State<T, S> {
    Value(T),
    Seed(S),
}

trait FnBox<TIn, TOut> {
    fn call_box(self: Box<Self>, arg: TIn) -> TOut;
}

impl<TIn, TOut, F> FnBox<TIn, TOut> for F
where
    F: FnOnce(TIn) -> TOut
{
    fn call_box(self: Box<F>, arg: TIn) -> TOut {
        (*self)(arg)
    }
}

impl<T> Default for FluentBuilder<T> {
    fn default() -> Self {
        FluentBuilder {
            inner: StatefulFluentBuilder::from_seed(())
        }
    }
}

impl<T> FluentBuilder<T> {
    /**
    Create a default `FluentBuilder`.
    */
    pub fn new() -> Self {
        FluentBuilder::default()
    }

    /**
    Set a value on the builder.
    
    This will override any contained state.
    That means if the builder currently contains fluent methods then those methods will be discarded.
    */
    pub fn value(self, value: T) -> Self {
        FluentBuilder {
            inner: self.inner.value(value)
        }
    }

    /**
    Convert the fluent builder into a value.

    This method will consume the builder and return a constructed `T`.
    This will have the following behaviour:

    - If the builder contains no value or fluent methods, then the default value is constructed.
    - If the builder contains a value, then that value is returned.
    - If the builder contains no value but fluent methods, then the methods are applied over the default value.
    - If the builder contains a value and fluent methods, then the methods are applied over that value.
    */
    pub fn into_value<F>(self, default_value: F) -> T
    where
        F: Fn() -> T + 'static,
    {
        self.inner.into_value(move |_| default_value())
    }
}

impl<T> FluentBuilder<T>
where
    T: 'static,
{
    /**
    Stack a fluent method on the builder.
    
    This will have the following behaviour depending on the current state of the builder:

    - If there is no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - If there is a previous value, add the fluent method and retain that previous value.
    - If there is a previous fluent method, stack this method on top and retain any previous value.
    */
    pub fn fluent<F>(self, fluent_method: F) -> Self
    where
        F: FnOnce(T) -> T + 'static
    {
        FluentBuilder {
            inner: self.inner.fluent((), |v, _| fluent_method(v))
        }
    }

    /**
    Stack a fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<F>(self, fluent_method: F) -> Self
    where
        F: FnOnce(&mut T) + 'static
    {
        FluentBuilder {
            inner: self.inner.fluent_mut((), |v, _| fluent_method(v))
        }
    }
}

impl<T, S> StatefulFluentBuilder<T, S> {
    /**
    Create a new `StatefulFlientBuilder` from the given value.
    */
    pub fn from_value(value: T) -> Self {
        StatefulFluentBuilder {
            inner: StatefulFluentBuilderInner(State::Value(value), None),
        }
    }

    /**
    Create a new `StatefulFlientBuilder` from the given seed.
    */
    pub fn from_seed(seed: S) -> Self {
        StatefulFluentBuilder {
            inner: StatefulFluentBuilderInner(State::Seed(seed), None),
        }
    }

    /**
    Set a value on the builder.
    
    This will override any contained state.
    That means if the builder currently contains fluent methods then those methods will be discarded.
    */
    pub fn value(self, value: T) -> Self {
        StatefulFluentBuilder {
            inner: StatefulFluentBuilderInner(State::Value(value), None)
        }
    }

    /**
    Convert the fluent builder into a value.

    This method will consume the builder and return a constructed `T`.
    This will have the following behaviour:

    - If the builder contains no value or fluent methods, then the default value is constructed.
    - If the builder contains a value, then that value is returned.
    - If the builder contains no value but fluent methods, then the methods are applied over the default value.
    - If the builder contains a value and fluent methods, then the methods are applied over that value.
    */
    pub fn into_value<F>(self, default_value: F) -> T
    where
        F: Fn(S) -> T + 'static,
    {
        let StatefulFluentBuilderInner(state, fluent_method) = self.inner;

        let default = match state {
            State::Value(value) => value,
            State::Seed(seed) => default_value(seed)
        };

        let value = match fluent_method {
            Some(fluent_method) => fluent_method.call_box(default),
            None => default,
        };

        value
    }
}

impl<T, S> StatefulFluentBuilder<T, S>
where
    T: 'static,
    S: 'static,
{
    /**
    Create a new `StatefulFlientBuilder` from the given seed and fluent method.
    */
    pub fn from_fluent<F>(seed: S, fluent_method: F) -> Self
    where
        F: FnOnce(T) -> T + 'static
    {
        StatefulFluentBuilder {
            inner: StatefulFluentBuilderInner(State::Seed(seed), Some(Box::new(fluent_method)))
        }
    }

    /**
    Create a new `StatefulFlientBuilder` from the given seed and fluent method.

    This method is the same as `from_fluent`, but mutates the value instead of replacing it.
    */
    pub fn from_fluent_mut<F>(seed: S, fluent_method: F) -> Self
    where
        F: FnOnce(&mut T) + 'static
    {
        Self::from_fluent(seed, |mut value| {
            fluent_method(&mut value);
            value
        })
    }

    /**
    Stack a fluent method on the builder.
    
    This will have the following behaviour depending on the current state of the builder:

    - If there is no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - If there is a previous value, add the fluent method and retain that previous value.
    - If there is a previous fluent method, stack this method on top and retain any previous value.
    */
    pub fn fluent<F>(self, seed: S, fluent_method: F) -> Self
    where
        F: FnOnce(T, S) -> T + 'static
    {
        let StatefulFluentBuilderInner(state, previous_fluent_method) = self.inner;

        let fluent_method = Box::new(move |value| {
            let value = match previous_fluent_method {
                Some(previous_fluent_method) => previous_fluent_method.call_box(value),
                None => value
            };

            fluent_method(value, seed)
        });

        StatefulFluentBuilder {
            inner: StatefulFluentBuilderInner(state, Some(fluent_method)),
        }
    }

    /**
    Stack a fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<F>(self, seed: S, fluent_method: F) -> Self
    where
        F: FnOnce(&mut T, S) + 'static
    {
        self.fluent(seed, move |mut value, seed| {
            fluent_method(&mut value, seed);
            value
        })
    }
}

#[cfg(test)]
mod tests {
    mod stateless {
        use ::*;

        #[test]
        fn default() {
            let builder = FluentBuilder::<String>::default();

            let result = builder.into_value(|| "default".to_owned());

            assert_eq!("default", result);
        }

        #[test]
        fn default_value() {
            let builder = FluentBuilder::<String>::default()
                .value("value".to_owned());

            let result = builder.into_value(|| "default".to_owned());

            assert_eq!("value", result);
        }

        #[test]
        fn default_fluent() {
            let builder = FluentBuilder::<String>::default()
                .fluent_mut(|v| v.push_str("_f1"))
                .fluent_mut(|v| v.push_str("_f2"));

            let result = builder.into_value(|| "default".to_owned());

            assert_eq!("default_f1_f2", result);
        }

        #[test]
        fn default_value_fluent() {
            let builder = FluentBuilder::<String>::default()
                .value("value".to_owned())
                .fluent_mut(|v| v.push_str("_f1"))
                .fluent_mut(|v| v.push_str("_f2"));

            let result = builder.into_value(|| "default".to_owned());

            assert_eq!("value_f1_f2", result);
        }

        #[test]
        fn default_fluent_value() {
            let builder = FluentBuilder::<String>::default()
                .fluent_mut(|v| v.push_str("_f1"))
                .fluent_mut(|v| v.push_str("_f2"))
                .value("value".to_owned());

            let result = builder.into_value(|| "default".to_owned());

            assert_eq!("value", result);
        }
    }

    mod stateful {
        use ::*;

        #[derive(Debug, PartialEq, Eq)]
        struct Builder {
            required: String,
            optional: Option<String>,
        }

        #[test]
        fn from_seed() {
            let builder = StatefulFluentBuilder::<Builder, String>::from_seed("seed".to_owned());

            let result = builder.into_value(|seed| Builder {
                required: seed,
                optional: None
            });

            let expected = Builder {
                required: "seed".to_owned(),
                optional: None,
            };

            assert_eq!(expected, result);
        }

        #[test]
        fn from_fluent() {
            let builder = StatefulFluentBuilder::<Builder, String>::from_fluent_mut("seed".to_owned(), |v| {
                v.optional = Some("fluent".to_owned())
            });

            let result = builder.into_value(|seed| Builder {
                required: seed,
                optional: None
            });

            let expected = Builder {
                required: "seed".to_owned(),
                optional: Some("fluent".to_owned()),
            };

            assert_eq!(expected, result);
        }

        #[test]
        fn from_value() {
            let builder = StatefulFluentBuilder::<Builder, String>::from_value(Builder {
                required: "seed".to_owned(),
                optional: None,
            });

            let result = builder.into_value(|seed| Builder {
                required: seed,
                optional: None
            });

            let expected = Builder {
                required: "seed".to_owned(),
                optional: None,
            };

            assert_eq!(expected, result);
        }

        #[test]
        fn from_seed_value() {
            let builder = StatefulFluentBuilder::<Builder, String>::from_seed("seed".to_owned())
                .value(Builder {
                    required: "value".to_owned(),
                    optional: Some("value".to_owned())
                });

            let result = builder.into_value(|seed| Builder {
                required: seed,
                optional: None
            });

            let expected = Builder {
                required: "value".to_owned(),
                optional: Some("value".to_owned()),
            };

            assert_eq!(expected, result);
        }

        #[test]
        fn from_seed_fluent() {
            let builder = StatefulFluentBuilder::<Builder, String>::from_seed("seed".to_owned())
                .fluent_mut("f1".to_owned(), |v, s| {
                    v.required = s;
                    v.optional = Some("f1".to_owned())
                })
                .fluent_mut("f2".to_owned(), |v, s| {
                    v.required = s;
                    if let Some(ref mut optional) = v.optional.as_mut() {
                        optional.push_str("_f2");
                    }
                });

            let result = builder.into_value(|seed| Builder {
                required: seed,
                optional: None
            });

            let expected = Builder {
                required: "f2".to_owned(),
                optional: Some("f1_f2".to_owned()),
            };

            assert_eq!(expected, result);
        }

        #[test]
        fn from_seed_value_fluent() {
            let builder = StatefulFluentBuilder::<Builder, String>::from_seed("seed".to_owned())
                .value(Builder {
                    required: "value".to_owned(),
                    optional: Some("value".to_owned())
                })
                .fluent_mut("f1".to_owned(), |v, s| {
                    v.required = s;
                    if let Some(ref mut optional) = v.optional.as_mut() {
                        optional.push_str("_f1");
                    }
                })
                .fluent_mut("f2".to_owned(), |v, s| {
                    v.required = s;
                    if let Some(ref mut optional) = v.optional.as_mut() {
                        optional.push_str("_f2");
                    }
                });

            let result = builder.into_value(|seed| Builder {
                required: seed,
                optional: None
            });

            let expected = Builder {
                required: "f2".to_owned(),
                optional: Some("value_f1_f2".to_owned()),
            };

            assert_eq!(expected, result);
        }

        #[test]
        fn from_seed_fluent_value() {
            let builder = StatefulFluentBuilder::<Builder, String>::from_seed("seed".to_owned())
                .fluent_mut("f1".to_owned(), |v, s| {
                    v.required = s;
                    v.optional = Some("f1".to_owned())
                })
                .fluent_mut("f2".to_owned(), |v, s| {
                    v.required = s;
                    if let Some(ref mut optional) = v.optional.as_mut() {
                        optional.push_str("_f2");
                    }
                })
                .value(Builder {
                    required: "value".to_owned(),
                    optional: Some("value".to_owned())
                });

            let result = builder.into_value(|seed| Builder {
                required: seed,
                optional: None
            });

            let expected = Builder {
                required: "value".to_owned(),
                optional: Some("value".to_owned()),
            };

            assert_eq!(expected, result);
        }
    }
}