use std::marker::PhantomData;

/**
Indicate that fluent methods should be stacked on top of eachother.
*/
pub enum Stack {}

/**
Indicate that fluent methods should override eachother.
*/
pub enum Override {}

/**
A structure that can contain a value, or stack mutating methods over one supplied later.

The `FluentBuilder<T>` is effectively a `StatefulFluentBuilder<T, ()>`.
*/
pub struct FluentBuilder<TValue, TStack = Override, TFluent = BoxedFluent<TValue>> {
    inner: StatefulFluentBuilder<TValue, (), TStack, TFluent>,
}

/**
A stateful structure that can contain a value, or stack mutating methods over one supplied later. 
*/
pub struct StatefulFluentBuilder<TValue, TSeed, TStack = Override, TFluent = BoxedFluent<TValue>> {
    inner: StatefulFluentBuilderInner<TValue, TSeed, TFluent>,
    _marker: PhantomData<TStack>,
}

/**
A boxed fluent method.
*/
pub struct BoxedFluent<TValue>(Box<Fluent<TValue>>);

struct StatefulFluentBuilderInner<TValue, TSeed, TFluent>(State<TValue, TSeed>, Option<TFluent>);

enum State<TValue, TSeed> {
    Value(TValue),
    Seed(TSeed),
}

impl<TValue, TStack> Default for FluentBuilder<TValue, TStack, BoxedFluent<TValue>> {
    fn default() -> Self {
        FluentBuilder {
            inner: StatefulFluentBuilder::from_seed(())
        }
    }
}

impl<TValue, TStack> FluentBuilder<TValue, TStack, BoxedFluent<TValue>> {
    /**
    Create a default `FluentBuilder`.
    */
    pub fn new() -> Self {
        FluentBuilder::default()
    }
}

impl<TValue, TStack, TFluent> FluentBuilder<TValue, TStack, TFluent> {
    /**
    Set a value on the builder.

    This will override any contained state.
    That means if the builder currently contains fluent methods then those methods will be discarded.
    */
    pub fn value(self, value: TValue) -> Self {
        FluentBuilder {
            inner: self.inner.value(value)
        }
    }
}

impl<TValue, TFluent> FluentBuilder<TValue, Stack, TFluent> where TFluent: Fluent<TValue> {
    /**
    Stack a fluent method on the builder.
    
    This will have the following behaviour depending on the current state of the builder if there is:

    - no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - a previous value, add the fluent method and retain that previous value.
    - a previous fluent method, stack this method on top and retain any previous value.

    Each call to `fluent` will box the given closure.
    */
    pub fn fluent<TNextFluent>(self, fluent_method: TNextFluent) -> FluentBuilder<TValue, Stack, Apply<TValue, TFluent, ByValue<TNextFluent>>>
    where
        TNextFluent: Fn(TValue) -> TValue
    {
        FluentBuilder {
            inner: self.inner.stack(|previous_fluent_method| Apply::new(previous_fluent_method, ByValue(fluent_method)))
        }
    }

    /**
    Stack a fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNextFluent>(self, fluent_method: TNextFluent) -> FluentBuilder<TValue, Stack, Apply<TValue, TFluent, ByRefMut<TNextFluent>>>
    where
        TNextFluent: Fn(&mut TValue)
    {
        FluentBuilder {
            inner: self.inner.stack(|previous_fluent_method| Apply::new(previous_fluent_method, ByRefMut(fluent_method)))
        }
    }
}

impl<TValue, TFluent> FluentBuilder<TValue, Override, TFluent> where TFluent: Fluent<TValue> {
    /**
    Create a new `StatefulFluentBuilder` from the given value.
    */
    pub fn fluent<TNextFluent>(self, fluent_method: TNextFluent) -> FluentBuilder<TValue, Override, Apply<TValue, BoxedFluent<TValue>, ByValue<TNextFluent>>>
    where
        TNextFluent: Fn(TValue) -> TValue + 'static
    {
        FluentBuilder {
            inner: self.inner.fluent((), fluent_method)
        }
    }

    /**
    Set the fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNextFluent>(self, fluent_method: TNextFluent) -> FluentBuilder<TValue, Override, Apply<TValue, BoxedFluent<TValue>, ByRefMut<TNextFluent>>>
    where
        TNextFluent: Fn(&mut TValue) + 'static
    {
        FluentBuilder {
            inner: self.inner.fluent_mut((), fluent_method)
        }
    }
}

impl<TValue, TStack, TFluent> FluentBuilder<TValue, TStack, TFluent> where TFluent: Fluent<TValue> {
    /**
    Convert the fluent builder into a value.

    This method will consume the builder and return a constructed `T`.
    This will have the following behaviour if the builder contains:

    - no value or fluent methods, then the default value is constructed.
    - a value, then that value is returned.
    - no value but fluent methods, then the methods are applied over the default value.
    - a value and fluent methods, then the methods are applied over that value.
    */
    pub fn into_value<TDefault>(self, default_value: TDefault) -> TValue
    where
        TDefault: Fn() -> TValue + 'static,
    {
        self.inner.into_value(move |_| default_value())
    }
}

impl<TValue, TStack, TFluent> FluentBuilder<TValue, TStack, TFluent>
where
    TValue: 'static,
    TFluent: Fluent<TValue> + 'static,
{
    /**
    Box a fluent builder so it can be easily shared.
    */
    pub fn boxed(self) -> FluentBuilder<TValue, TStack, BoxedFluent<TValue>> {
        FluentBuilder {
            inner: self.inner.boxed()
        }
    }
}

impl<TValue, TSeed, TStack, TFluent> StatefulFluentBuilder<TValue, TSeed, TStack, TFluent> {
    fn new(inner: StatefulFluentBuilderInner<TValue, TSeed, TFluent>) -> Self {
        StatefulFluentBuilder {
            inner: inner,
            _marker: PhantomData,
        }
    }

    /**
    Create a new `StatefulFluentBuilder` from the given value.
    */
    pub fn from_value(value: TValue) -> Self {
        StatefulFluentBuilder::new(StatefulFluentBuilderInner(State::Value(value), None))
    }

    /**
    Create a new `StatefulFluentBuilder` from the given seed.
    */
    pub fn from_seed(seed: TSeed) -> Self {
        StatefulFluentBuilder::new(StatefulFluentBuilderInner(State::Seed(seed), None))
    }

    /**
    Set a value on the builder.
    
    This will override any contained state.
    That means if the builder currently contains fluent methods then those methods will be discarded.
    */
    pub fn value(self, value: TValue) -> Self {
        StatefulFluentBuilder::new(StatefulFluentBuilderInner(State::Value(value), None))
    }
}

impl<TValue, TSeed, TStack> StatefulFluentBuilder<TValue, TSeed, TStack, BoxedFluent<TValue>> {
    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.
    */
    pub fn from_fluent<TFluent>(seed: TSeed, fluent_method: TFluent) -> StatefulFluentBuilder<TValue, TSeed, TStack, Apply<TValue, BoxedFluent<TValue>, ByValue<TFluent>>>
    where
        TFluent: Fn(TValue) -> TValue
    {
        let fluent_method = Apply::new(None, ByValue(fluent_method));
        StatefulFluentBuilder::new(StatefulFluentBuilderInner(State::Seed(seed), Some(fluent_method)))
    }

    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.

    This method is the same as `from_fluent`, but mutates the value instead of replacing it.
    */
    pub fn from_fluent_mut<TFluent>(seed: TSeed, fluent_method: TFluent) -> StatefulFluentBuilder<TValue, TSeed, TStack, Apply<TValue, BoxedFluent<TValue>, ByRefMut<TFluent>>>
    where
        TFluent: Fn(&mut TValue)
    {
        let fluent_method = Apply::new(None, ByRefMut(fluent_method));
        StatefulFluentBuilder::new(StatefulFluentBuilderInner(State::Seed(seed), Some(fluent_method)))
    }
}

impl<TValue, TSeed, TStack, TFluent> StatefulFluentBuilder<TValue, TSeed, TStack, TFluent>
where
    TFluent: Fluent<TValue>,
{
    /**
    Convert the fluent builder into a value.

    This method will consume the builder and return a constructed `T`.
    This will have the following behaviour if the builder contains:

    - no value or fluent methods, then the default value is constructed.
    - a value, then that value is returned.
    - no value but fluent methods, then the methods are applied over the default value.
    - a value and fluent methods, then the methods are applied over that value.
    */
    pub fn into_value<TDefault>(self, default_value: TDefault) -> TValue
    where
        TDefault: Fn(TSeed) -> TValue + 'static,
    {
        let StatefulFluentBuilderInner(state, mut fluent_method) = self.inner;

        let default = match state {
            State::Value(value) => value,
            State::Seed(seed) => default_value(seed)
        };

        let value = match fluent_method {
            Some(ref mut fluent_method) => fluent_method.apply(default),
            None => default,
        };

        value
    }
}

impl<TValue, TSeed, TFluent> StatefulFluentBuilder<TValue, TSeed, Stack, TFluent>
where
    TFluent: Fluent<TValue>,
{
    fn stack<TFluentStacker, TNewFluent>(self, fluent_stacker: TFluentStacker) -> StatefulFluentBuilder<TValue, TSeed, Stack, TNewFluent>
    where
        TFluentStacker: FnOnce(Option<TFluent>) -> TNewFluent,
        TNewFluent: Fluent<TValue>,
    {
        let StatefulFluentBuilderInner(state, previous_fluent_method) = self.inner;

        let fluent_method = fluent_stacker(previous_fluent_method);

        StatefulFluentBuilder::new(StatefulFluentBuilderInner(state, Some(fluent_method)))
    }

    /**
    Stack a fluent method on the builder.
    
    This will have the following behaviour depending on the current state of the builder if there is:

    - no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - a previous value, add the fluent method and retain that previous value.
    - a previous fluent method, stack this method on top and retain any previous value.

    Each call to `fluent` will box the given closure.
    */
    pub fn fluent<TNewFluent>(self, seed: TSeed, fluent_method: TNewFluent) -> StatefulFluentBuilder<TValue, TSeed, Stack, StatefulApply<TValue, TSeed, TFluent, ByValue<TNewFluent>>>
    where
        TNewFluent: Fn(TValue, TSeed) -> TValue
    {
        self.stack(move |previous_fluent_method| StatefulApply::new(seed, previous_fluent_method, ByValue(fluent_method)))
    }

    /**
    Stack a fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNewFluent>(self, seed: TSeed, fluent_method: TNewFluent) -> StatefulFluentBuilder<TValue, TSeed, Stack, StatefulApply<TValue, TSeed, TFluent, ByRefMut<TNewFluent>>>
    where
        TNewFluent: Fn(&mut TValue, TSeed)
    {
        self.stack(move |previous_fluent_method| StatefulApply::new(seed, previous_fluent_method, ByRefMut(fluent_method)))
    }
}

impl<TValue, TSeed, TFluent> StatefulFluentBuilder<TValue, TSeed, Override, TFluent>
where
    TFluent: Fluent<TValue>,
{
    /**
    Set the fluent method on the builder.
    
    This will have the following behaviour depending on the current state of the builder if there is:

    - no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - a previous value, add the fluent method and remove that previous value.
    - a previous fluent method, that method will be replaced with the given one.

    Each call to `fluent` will box the given closure.
    */
    pub fn fluent<TNewFluent>(self, seed: TSeed, fluent_method: TNewFluent) -> StatefulFluentBuilder<TValue, TSeed, Override, Apply<TValue, BoxedFluent<TValue>, ByValue<TNewFluent>>>
    where
        TNewFluent: Fn(TValue) -> TValue + 'static
    {
        StatefulFluentBuilder::from_fluent(seed, fluent_method)
    }

    /**
    Set the fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNewFluent>(self, seed: TSeed, fluent_method: TNewFluent) -> StatefulFluentBuilder<TValue, TSeed, Override, Apply<TValue, BoxedFluent<TValue>, ByRefMut<TNewFluent>>>
    where
        TNewFluent: Fn(&mut TValue) + 'static
    {
        StatefulFluentBuilder::from_fluent_mut(seed, fluent_method)
    }
}

impl<TValue, TSeed, TStack, TFluent> StatefulFluentBuilder<TValue, TSeed, TStack, TFluent>
where
    TFluent: 'static,
    TSeed: 'static,
    TFluent: Fluent<TValue> + 'static,
{
    /**
    Box a fluent builder so it can be easily shared.
    */
    pub fn boxed(self) -> StatefulFluentBuilder<TValue, TSeed, TStack, BoxedFluent<TValue>> {
        let StatefulFluentBuilderInner(state, fluent_method) = self.inner;

        let fluent_method = fluent_method.map(|f| BoxedFluent(Box::new(f)));

        StatefulFluentBuilder::new(StatefulFluentBuilderInner(state, fluent_method))
    }
}

impl<TValue, TFluent> Fluent<TValue> for TFluent
where
    TFluent: FnMut(TValue) -> TValue
{
    fn apply(&mut self, value: TValue) -> TValue {
        self(value)
    }
}

impl<TValue> Fluent<TValue> for BoxedFluent<TValue> {
    fn apply(&mut self, value: TValue) -> TValue {
        self.0.apply(value)
    }
}

/* pub(crate) items */

pub trait Fluent<TValue> {
    fn apply(&mut self, value: TValue) -> TValue;
}

pub struct ByValue<TFluent>(TFluent);

pub struct ByRefMut<TFluent>(TFluent);

pub struct Apply<TValue, TPreviousFluent, TNextFluent> {
    inner: Option<StatefulApply<TValue, (), TPreviousFluent, TNextFluent>>,
}

impl<TValue, TPreviousFluent, TNextFluent> Apply<TValue, TPreviousFluent, TNextFluent> {
    fn new(previous: Option<TPreviousFluent>, next: TNextFluent) -> Self {
        Apply {
            inner: Some(StatefulApply::new((), previous, next))
        }
    }
}

impl<TValue, TPreviousFluent, TNextFluent> Fluent<TValue> for Apply<TValue, TPreviousFluent, ByValue<TNextFluent>>
where
    TPreviousFluent: Fluent<TValue>,
    TNextFluent: FnMut(TValue) -> TValue
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let inner = mem::replace(&mut self.inner, None).expect("attempted to re-use builder");

        let (mut next, inner) = inner.take_next();

        inner.set_next(ByValue(move |value: TValue, _| (next.0)(value))).apply(value)
    }
}

impl<TValue, TPreviousFluent, TNextFluent> Fluent<TValue> for Apply<TValue, TPreviousFluent, ByRefMut<TNextFluent>>
where
    TPreviousFluent: Fluent<TValue>,
    TNextFluent: FnMut(&mut TValue)
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let inner = mem::replace(&mut self.inner, None).expect("attempted to re-use builder");

        let (mut next, inner) = inner.take_next();

        inner.set_next(ByRefMut(move |mut value: &mut TValue, _| (next.0)(&mut value))).apply(value)
    }
}

pub struct StatefulApply<TValue, TSeed, TPreviousFluent, TNextFluent> {
    seed: Option<TSeed>,
    previous: Option<TPreviousFluent>,
    next: TNextFluent,
    _marker: PhantomData<TValue>,
}

impl<TValue, TSeed, TPreviousFluent, TNextFluent> StatefulApply<TValue, TSeed, TPreviousFluent, TNextFluent> {
    fn new(seed: TSeed, previous: Option<TPreviousFluent>, next: TNextFluent) -> Self {
        StatefulApply {
            seed: Some(seed),
            previous: previous,
            next: next,
            _marker: PhantomData,
        }
    }

    fn take_next(self) -> (TNextFluent, StatefulApply<TValue, TSeed, TPreviousFluent, ()>) {
        let next = self.next;
        let self_sans_next = StatefulApply {
            seed: self.seed,
            previous: self.previous,
            next: (),
            _marker: PhantomData,
        };

        (next, self_sans_next)
    }

    fn set_next<TNewNextFluent>(self, next: TNewNextFluent) -> StatefulApply<TValue, TSeed, TPreviousFluent, TNewNextFluent> {
        StatefulApply {
            seed: self.seed,
            previous: self.previous,
            next: next,
            _marker: PhantomData,
        }
    }
}

impl<TValue, TSeed, TPreviousFluent, TNextFluent> Fluent<TValue> for StatefulApply<TValue, TSeed, TPreviousFluent, ByValue<TNextFluent>>
where
    TPreviousFluent: Fluent<TValue>,
    TNextFluent: FnMut(TValue, TSeed) -> TValue
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let seed = mem::replace(&mut self.seed, None).expect("attempted to re-use builder");

        let value = match self.previous {
            Some(ref mut previous) => previous.apply(value),
            None => value
        };

        (self.next.0)(value, seed)
    }
}

impl<TValue, TSeed, TPreviousFluent, TNextFluent> Fluent<TValue> for StatefulApply<TValue, TSeed, TPreviousFluent, ByRefMut<TNextFluent>>
where
    TPreviousFluent: Fluent<TValue>,
    TNextFluent: FnMut(&mut TValue, TSeed)
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let seed = mem::replace(&mut self.seed, None).expect("attempted to re-use builder");

        let mut value = match self.previous {
            Some(ref mut previous) => previous.apply(value),
            None => value
        };

        (self.next.0)(&mut value, seed);
        value
    }
}

#[cfg(test)]
mod tests {
    mod stateless {
        mod fluent_override {
            use imp::*;

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

                assert_eq!("default_f2", result);
            }

            #[test]
            fn default_value_fluent() {
                let builder = FluentBuilder::<String>::default()
                    .value("value".to_owned())
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"));

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default_f2", result);
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

        mod fluent_stack {
            use imp::*;

            #[test]
            fn default() {
                let builder = FluentBuilder::<String, Stack>::default();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default", result);
            }

            #[test]
            fn default_value() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .value("value".to_owned());

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("value", result);
            }

            #[test]
            fn default_fluent() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"));

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default_f1_f2", result);
            }

            #[test]
            fn default_value_fluent() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .value("value".to_owned())
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"));

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("value_f1_f2", result);
            }

            #[test]
            fn default_fluent_value() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"))
                    .value("value".to_owned());

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("value", result);
            }
        }
    }

    mod stateful {
        mod fluent_override {
            use imp::*;

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
                    .fluent_mut("f1".to_owned(), |v| {
                        v.optional = Some("f1".to_owned())
                    })
                    .fluent_mut("f2".to_owned(), |v| {
                        v.optional = Some("f2".to_owned())
                    });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None
                });

                let expected = Builder {
                    required: "f2".to_owned(),
                    optional: Some("f2".to_owned()),
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
                    .fluent_mut("f1".to_owned(), |v| {
                        if let Some(ref mut optional) = v.optional.as_mut() {
                            optional.push_str("_f1");
                        }
                    })
                    .fluent_mut("f2".to_owned(), |v| {
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
                    optional: None,
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_fluent_value() {
                let builder = StatefulFluentBuilder::<Builder, String>::from_seed("seed".to_owned())
                    .fluent_mut("f1".to_owned(), |v| {
                        v.optional = Some("f1".to_owned())
                    })
                    .fluent_mut("f2".to_owned(), |v| {
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

        mod fluent_stack {
            use imp::*;

            #[derive(Debug, PartialEq, Eq)]
            struct Builder {
                required: String,
                optional: Option<String>,
            }

            #[test]
            fn from_seed() {
                let builder = StatefulFluentBuilder::<Builder, String, Stack>::from_seed("seed".to_owned());

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
                let builder = StatefulFluentBuilder::<Builder, String, Stack>::from_fluent_mut("seed".to_owned(), |v| {
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
                let builder = StatefulFluentBuilder::<Builder, String, Stack>::from_value(Builder {
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
                let builder = StatefulFluentBuilder::<Builder, String, Stack>::from_seed("seed".to_owned())
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
                let builder = StatefulFluentBuilder::<Builder, String, Stack>::from_seed("seed".to_owned())
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
                let builder = StatefulFluentBuilder::<Builder, String, Stack>::from_seed("seed".to_owned())
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
                let builder = StatefulFluentBuilder::<Builder, String, Stack>::from_seed("seed".to_owned())
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
}