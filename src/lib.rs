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
*/

/**
A structure that can contain a value, or stack mutating methods over one supplied later. 
*/
pub struct FluentBuilder<T> {
    inner: Option<FluentBuilderInner<T>>,
}

impl<T> Default for FluentBuilder<T> {
    fn default() -> Self {
        FluentBuilder {
            inner: None
        }
    }
}

enum FluentBuilderInner<T> {
    Value(T),
    FluentBuilder(Option<T>, Box<Fn(T) -> T>),
}

impl<T> FluentBuilder<T> {
    /**
    Create a new fluent builder.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
    Set a value on the builder.
    
    This will override any contained state.
    That means if the builder currently contains fluent methods then those methods will be discarded.
    */
    pub fn value(self, value: T) -> Self {
        FluentBuilder {
            inner: Some(FluentBuilderInner::Value(value))
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
        F: Fn() -> T + 'static
    {
        match self.inner {
            Some(FluentBuilderInner::Value(value)) => value,
            Some(FluentBuilderInner::FluentBuilder(seed, fluent_method)) => fluent_method(seed.unwrap_or_else(default_value)),
            None => default_value(),
        }
    }
}

impl<T> FluentBuilder<T> where T: 'static {
    /**
    Stack a fluent method on the builder.
    
    This will have the following behaviour depending on the current state of the builder:

    - If there is no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - If there is a previous value, add the fluent method and retain that previous value.
    - If there is a previous fluent method, stack this method on top and retain any previous value.
    */
    pub fn fluent<F>(self, fluent_method: F) -> Self
    where
        F: Fn(T) -> T + 'static
    {
        let inner = match self.inner {
            Some(FluentBuilderInner::Value(seed)) => {
                FluentBuilderInner::FluentBuilder(Some(seed), Box::new(fluent_method))
            },
            Some(FluentBuilderInner::FluentBuilder(seed, previous_fluent_method)) => {
                let fluent_method = Box::new(move |value| fluent_method(previous_fluent_method(value)));

                FluentBuilderInner::FluentBuilder(seed, fluent_method)
            },
            None => {
                FluentBuilderInner::FluentBuilder(None, Box::new(fluent_method))
            }
        };

        FluentBuilder {
            inner: Some(inner),
        }
    }

    /**
    Stack a fluent method on the builder.
    
    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<F>(self, fluent_method: F) -> Self
    where
        F: Fn(&mut T) + 'static
    {
        self.fluent(move |mut value| {
            fluent_method(&mut value);
            value
        })
    }
}

impl<T> FluentBuilder<T> where T: Clone {
    /**
    Construct a value from the fluent builder.

    This method won't consume the builder and will return a clone of any inner value.
    Otherwise it behaves the same as `into_value`.
    */
    pub fn to_value<F>(&self, default_value: F) -> T
    where
        F: Fn() -> T + 'static
    {
        match self.inner {
            Some(FluentBuilderInner::Value(ref value)) => value.clone(),
            Some(FluentBuilderInner::FluentBuilder(ref seed, ref fluent_method)) => fluent_method(seed.clone().unwrap_or_else(default_value)),
            None => default_value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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