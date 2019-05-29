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
Fluent methods will be stored inline.
*/
pub enum Inline {}

/**
Fluent methods will be boxed.

Note this doesn't necessarily mean each individual method will live in its own box.
Each call to `FluentBuilder.boxed` will create a box containing all methods since
the last time it was boxed.
*/
pub enum Boxed {}

/**
Fluent methods will be boxed, but additionally require `Send`.

Note this doesn't necessarily mean each individual method will live in its own box.
Each call to `FluentBuilder.boxed` will create a box containing all methods since
the last time it was boxed.
*/
pub enum Shared {}

/**
The default way to stack fluent methods.
*/
pub type DefaultStack = Override;

/**
The default way to store fluent methods.
*/
pub type DefaultStorage = Inline;

/**
A boxed fluent builder.
*/
pub type BoxedFluentBuilder<TValue, TStack = DefaultStack> = FluentBuilder<TValue, TStack, Boxed>;

/**
A shared fluent builder.
*/
pub type SharedFluentBuilder<TValue, TStack = DefaultStack> = FluentBuilder<TValue, TStack, Shared>;

/**
A boxed stateful fluent builder.
*/
pub type BoxedStatefulFluentBuilder<TSeed, TValue, TStack = DefaultStack> =
    StatefulFluentBuilder<TSeed, TValue, TStack, Boxed>;

/**
A shared stateful fluent builder.
*/
pub type SharedStatefulFluentBuilder<TSeed, TValue, TStack = DefaultStack> =
    StatefulFluentBuilder<TSeed, TValue, TStack, Shared>;

/**
A structure that can contain a value, or stack mutating methods over one supplied later.

The `FluentBuilder<T>` is effectively a `StatefulFluentBuilder<T, ()>`.
*/
pub struct FluentBuilder<TValue, TStack = DefaultStack, TStorage = DefaultStorage>
where
    TStorage: Storage<TValue>,
{
    inner: StatefulFluentBuilder<(), TValue, TStack, TStorage>,
}

/**
A stateful structure that can contain a value, or stack mutating methods over one supplied later.
*/
pub struct StatefulFluentBuilder<TSeed, TValue, TStack = DefaultStack, TStorage = DefaultStorage>
where
    TStorage: Storage<TValue>,
{
    inner: StatefulFluentBuilderInner<TSeed, TValue, TStorage>,
    _marker: PhantomData<TStack>,
}

/**
A boxed fluent method.
*/
pub struct BoxedMethod<TValue>(Box<Method<TValue>>);

/**
A shared fluent method.
*/
pub struct SharedMethod<TValue>(Box<Method<TValue> + Send>);

/**
The result of attempting to pull a value out of a builder.

Calling `try_into_value` will return `TryIntoValue::Value` if the builder has been given an explicit value.
It will return `TryIntoValue::Builder` with the original builder if it hasn't been given an explicit value.
Calling `into_value` will never fail, because if a value is missing it's constructed from the default function.
*/
pub enum TryIntoValue<TValue, TBuilder> {
    Value(TValue),
    Builder(TBuilder),
}

struct StatefulFluentBuilderInner<TSeed, TValue, TStorage>
where
    TStorage: Storage<TValue>,
{
    state: State<TSeed, TValue>,
    fluent_method: Option<TStorage::Method>,
}

enum State<TSeed, TValue> {
    Value(TValue),
    Seed(TSeed),
}

impl<TValue, TStack, TStorage> Default for FluentBuilder<TValue, TStack, TStorage>
where
    TStorage: Storage<TValue>,
{
    fn default() -> Self {
        FluentBuilder {
            inner: StatefulFluentBuilder::from_seed(()),
        }
    }
}

impl<TValue, TStack, TStorage> FluentBuilder<TValue, TStack, TStorage>
where
    TStorage: Storage<TValue>,
{
    /**
    Create a default `FluentBuilder`.
    */
    pub fn new() -> Self {
        FluentBuilder::default()
    }
}

impl<TValue, TStack, TStorage> FluentBuilder<TValue, TStack, TStorage>
where
    TStorage: Storage<TValue>,
{
    /**
    Set a value on the builder.

    This will override any contained state.
    That means if the builder currently contains fluent methods then those methods will be discarded.
    */
    pub fn value(self, value: TValue) -> Self {
        FluentBuilder {
            inner: self.inner.value(value),
        }
    }
}

impl<TValue, TStorage> FluentBuilder<TValue, Stack, TStorage>
where
    TStorage: Storage<TValue>,
{
    /**
    Stack a fluent method on the builder.

    This will have the following behaviour depending on the current state of the builder if there is:

    - no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - a previous value, add the fluent method and retain that previous value.
    - a previous fluent method, stack this method on top and retain any previous value.
    */
    pub fn fluent<TNextMethod>(
        self,
        fluent_method: TNextMethod,
    ) -> FluentBuilder<TValue, Stack, Apply<TValue, TStorage::Method, ByValue<TNextMethod>>>
    where
        TNextMethod: FnOnce(TValue) -> TValue,
    {
        FluentBuilder {
            inner: self.inner.stack(|previous_fluent_method| {
                Apply::new(previous_fluent_method, ByValue(fluent_method))
            }),
        }
    }

    /**
    Stack a fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNextMethod>(
        self,
        fluent_method: TNextMethod,
    ) -> FluentBuilder<TValue, Stack, Apply<TValue, TStorage::Method, ByRefMut<TNextMethod>>>
    where
        TNextMethod: FnOnce(&mut TValue),
    {
        FluentBuilder {
            inner: self.inner.stack(|previous_fluent_method| {
                Apply::new(previous_fluent_method, ByRefMut(fluent_method))
            }),
        }
    }
}

impl<TValue, TStorage> FluentBuilder<TValue, Override, TStorage>
where
    TStorage: Storage<TValue>,
{
    /**
    Create a new `StatefulFluentBuilder` from the given value.
    */
    pub fn fluent<TNextMethod>(
        self,
        fluent_method: TNextMethod,
    ) -> FluentBuilder<TValue, Override, Apply<TValue, DefaultStorage, ByValue<TNextMethod>>>
    where
        TNextMethod: FnOnce(TValue) -> TValue + 'static,
    {
        FluentBuilder {
            inner: self.inner.fluent((), fluent_method),
        }
    }

    /**
    Set the fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNextMethod>(
        self,
        fluent_method: TNextMethod,
    ) -> FluentBuilder<TValue, Override, Apply<TValue, DefaultStorage, ByRefMut<TNextMethod>>>
    where
        TNextMethod: FnOnce(&mut TValue) + 'static,
    {
        FluentBuilder {
            inner: self.inner.fluent_mut((), fluent_method),
        }
    }
}

impl<TValue, TStack, TStorage> FluentBuilder<TValue, TStack, TStorage>
where
    TStorage: Storage<TValue>,
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
        TDefault: FnOnce() -> TValue + 'static,
    {
        self.inner.into_value(move |_| default_value())
    }

    /**
    Attempt to take a value from the builder.

    If the builder doesn't contain a concrete value then it is returned in the `Builder` variant.

    # Examples

    ```
    # use fluent_builder::{Stack, FluentBuilder, TryIntoValue};
    let builder = FluentBuilder::<String, Stack>::new()
        .value("A value".to_owned())
        .fluent_mut(|mut s| s.push_str(" and more"));

    match builder.try_into_value() {
        TryIntoValue::Value(value) => {
            // The builder has a value that we can use
            assert_eq!("A value and more", value);
        },
        TryIntoValue::Builder(builder) => {
            // The builder doesn't have a value but we can still use it
            let value = builder.into_value(|| String::new());
    #       panic!("expected value");
        }
    }
    ```
    */
    pub fn try_into_value(self) -> TryIntoValue<TValue, Self> {
        match self.inner.try_into_value() {
            TryIntoValue::Builder(inner) => TryIntoValue::Builder(FluentBuilder { inner }),
            TryIntoValue::Value(value) => TryIntoValue::Value(value),
        }
    }
}

impl<TValue, TStack, TStorage> FluentBuilder<TValue, TStack, TStorage>
where
    TValue: 'static,
    TStorage: Storage<TValue> + 'static,
{
    /**
    Box a fluent builder so it can be easily captured as a field without generics.
    */
    pub fn boxed(self) -> BoxedFluentBuilder<TValue, TStack> {
        FluentBuilder {
            inner: self.inner.boxed(),
        }
    }
}

impl<TValue, TStack, TStorage> FluentBuilder<TValue, TStack, TStorage>
where
    TValue: 'static,
    TStorage: Storage<TValue>,
    TStorage::Method: Send + 'static,
{
    /**
    Box a fluent builder so it can be easily shared.
    */
    pub fn shared(self) -> SharedFluentBuilder<TValue, TStack> {
        FluentBuilder {
            inner: self.inner.shared(),
        }
    }
}

impl<TSeed, TValue, TStack, TStorage> StatefulFluentBuilder<TSeed, TValue, TStack, TStorage>
where
    TStorage: Storage<TValue>,
{
    fn new(inner: StatefulFluentBuilderInner<TSeed, TValue, TStorage>) -> Self {
        StatefulFluentBuilder {
            inner: inner,
            _marker: PhantomData,
        }
    }

    /**
    Create a new `StatefulFluentBuilder` from the given value.
    */
    pub fn from_value(value: TValue) -> Self {
        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state: State::Value(value),
            fluent_method: None,
        })
    }

    /**
    Create a new `StatefulFluentBuilder` from the given seed.
    */
    pub fn from_seed(seed: TSeed) -> Self {
        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state: State::Seed(seed),
            fluent_method: None,
        })
    }

    /**
    Set a value on the builder.

    This will override any contained state.
    That means if the builder currently contains fluent methods then those methods will be discarded.
    */
    pub fn value(self, value: TValue) -> Self {
        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state: State::Value(value),
            fluent_method: None,
        })
    }
}

impl<TSeed, TValue, TStack> StatefulFluentBuilder<TSeed, TValue, TStack, DefaultStorage> {
    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.
    */
    pub fn from_fluent<TNextStorage>(
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<
        TSeed,
        TValue,
        TStack,
        Apply<TValue, DefaultStorage, ByValue<TNextStorage>>,
    >
    where
        TNextStorage: FnOnce(TValue) -> TValue,
    {
        let fluent_method = Apply::new(None, ByValue(fluent_method));
        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state: State::Seed(seed),
            fluent_method: Some(fluent_method),
        })
    }

    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.

    This method is the same as `from_fluent`, but mutates the value instead of replacing it.
    */
    pub fn from_fluent_mut<TNextStorage>(
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<
        TSeed,
        TValue,
        TStack,
        Apply<TValue, DefaultStorage, ByRefMut<TNextStorage>>,
    >
    where
        TNextStorage: FnOnce(&mut TValue),
    {
        let fluent_method = Apply::new(None, ByRefMut(fluent_method));
        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state: State::Seed(seed),
            fluent_method: Some(fluent_method),
        })
    }
}

impl<TSeed, TValue, TStack> StatefulFluentBuilder<TSeed, TValue, TStack, Shared> {
    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.
    */
    pub fn from_fluent<TNextStorage>(
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<TSeed, TValue, TStack, Shared>
    where
        TValue: Send + 'static,
        TSeed: Send + 'static,
        TNextStorage: FnOnce(TValue) -> TValue + Send + 'static,
    {
        StatefulFluentBuilder::<TSeed, TValue, TStack, Inline>::from_fluent(seed, fluent_method).shared()
    }

    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.

    This method is the same as `from_fluent`, but mutates the value instead of replacing it.
    */
    pub fn from_fluent_mut<TNextStorage>(
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<TSeed, TValue, TStack, Shared>
    where
        TValue: Send + 'static,
        TSeed: Send + 'static,
        TNextStorage: FnOnce(&mut TValue) + Send + 'static,
    {
        StatefulFluentBuilder::<TSeed, TValue, TStack, Inline>::from_fluent_mut(seed, fluent_method).shared()
    }
}

impl<TSeed, TValue, TStack> StatefulFluentBuilder<TSeed, TValue, TStack, Boxed> {
    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.
    */
    pub fn from_fluent<TNextStorage>(
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<TSeed, TValue, TStack, Boxed>
    where
        TValue: 'static,
        TSeed: 'static,
        TNextStorage: FnOnce(TValue) -> TValue + 'static,
    {
        StatefulFluentBuilder::<TSeed, TValue, TStack, Inline>::from_fluent(seed, fluent_method).boxed()
    }

    /**
    Create a new `StatefulFluentBuilder` from the given seed and fluent method.

    This method is the same as `from_fluent`, but mutates the value instead of replacing it.
    */
    pub fn from_fluent_mut<TNextStorage>(
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<TSeed, TValue, TStack, Boxed>
    where
        TValue: 'static,
        TSeed: 'static,
        TNextStorage: FnOnce(&mut TValue) + 'static,
    {
        StatefulFluentBuilder::<TSeed, TValue, TStack, Inline>::from_fluent_mut(seed, fluent_method).boxed()
    }
}

impl<TSeed, TValue, TStack, TStorage> StatefulFluentBuilder<TSeed, TValue, TStack, TStorage>
where
    TStorage: Storage<TValue>,
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
        TDefault: FnOnce(TSeed) -> TValue + 'static,
    {
        let StatefulFluentBuilderInner {
            state,
            mut fluent_method,
        } = self.inner;

        let default = match state {
            State::Value(value) => value,
            State::Seed(seed) => default_value(seed),
        };

        match fluent_method {
            Some(ref mut fluent_method) => fluent_method.apply(default),
            None => default,
        }
    }

    /**
    Attempt to take a value from the builder.

    If the builder doesn't contain a concrete value then it is returned in the `Builder` variant.

    # Examples

    ```
    # use fluent_builder::{Stack, StatefulFluentBuilder, TryIntoValue};
    let builder = StatefulFluentBuilder::<i32, String, Stack>::from_value("A value".to_owned())
        .fluent(1, |i, s| format!("{} and more {}", s, i));

    match builder.try_into_value() {
        TryIntoValue::Value(value) => {
            // The builder has a value that we can use
            assert_eq!("A value and more 1", value);
        },
        TryIntoValue::Builder(builder) => {
            // The builder doesn't have a value but we can still use it
            let value = builder.into_value(|i| i.to_string());
    #       panic!("expected value");
        }
    }
    ```
    */
    pub fn try_into_value(self) -> TryIntoValue<TValue, Self> {
        match self.inner {
            StatefulFluentBuilderInner {
                state: State::Value(value),
                mut fluent_method,
            } => TryIntoValue::Value(match fluent_method {
                Some(ref mut fluent_method) => fluent_method.apply(value),
                None => value,
            }),
            inner => TryIntoValue::Builder(StatefulFluentBuilder::new(inner)),
        }
    }
}

impl<TSeed, TValue, TStorage> StatefulFluentBuilder<TSeed, TValue, Stack, TStorage>
where
    TStorage: Storage<TValue>,
{
    fn stack<TMethodStacker, TNextStorage>(
        self,
        fluent_stacker: TMethodStacker,
    ) -> StatefulFluentBuilder<TSeed, TValue, Stack, TNextStorage>
    where
        TMethodStacker: FnOnce(Option<TStorage::Method>) -> TNextStorage::Method,
        TNextStorage: Storage<TValue>,
    {
        let StatefulFluentBuilderInner {
            state,
            fluent_method: previous_fluent_method,
        } = self.inner;

        let fluent_method = fluent_stacker(previous_fluent_method);

        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state,
            fluent_method: Some(fluent_method),
        })
    }

    /**
    Stack a fluent method on the builder.

    This will have the following behaviour depending on the current state of the builder if there is:

    - no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - a previous value, add the fluent method and retain that previous value.
    - a previous fluent method, stack this method on top and retain any previous value.
    */
    pub fn fluent<TNextStorage>(
        self,
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<
        TSeed,
        TValue,
        Stack,
        StatefulApply<TSeed, TValue, TStorage::Method, ByValue<TNextStorage>>,
    >
    where
        TNextStorage: FnOnce(TSeed, TValue) -> TValue,
    {
        self.stack(move |previous_fluent_method| {
            StatefulApply::new(seed, previous_fluent_method, ByValue(fluent_method))
        })
    }

    /**
    Stack a fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNextStorage>(
        self,
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<
        TSeed,
        TValue,
        Stack,
        StatefulApply<TSeed, TValue, TStorage::Method, ByRefMut<TNextStorage>>,
    >
    where
        TNextStorage: FnOnce(TSeed, &mut TValue),
    {
        self.stack(move |previous_fluent_method| {
            StatefulApply::new(seed, previous_fluent_method, ByRefMut(fluent_method))
        })
    }
}

impl<TSeed, TValue, TStorage> StatefulFluentBuilder<TSeed, TValue, Override, TStorage>
where
    TStorage: Storage<TValue>,
{
    /**
    Set the fluent method on the builder.

    This will have the following behaviour depending on the current state of the builder if there is:

    - no previous value, add the fluent method. This will be applied to a later-supplied default value.
    - a previous value, add the fluent method and remove that previous value.
    - a previous fluent method, that method will be replaced with the given one.
    */
    pub fn fluent<TNextStorage>(
        self,
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<
        TSeed,
        TValue,
        Override,
        Apply<TValue, DefaultStorage, ByValue<TNextStorage>>,
    >
    where
        TNextStorage: FnOnce(TValue) -> TValue + 'static,
    {
        StatefulFluentBuilder::<TSeed, TValue, Override, Inline>::from_fluent(seed, fluent_method)
    }

    /**
    Set the fluent method on the builder.

    This method behaves the same as `fluent`, but mutates the value instead of replacing it.
    */
    pub fn fluent_mut<TNextStorage>(
        self,
        seed: TSeed,
        fluent_method: TNextStorage,
    ) -> StatefulFluentBuilder<
        TSeed,
        TValue,
        Override,
        Apply<TValue, DefaultStorage, ByRefMut<TNextStorage>>,
    >
    where
        TNextStorage: FnOnce(&mut TValue) + 'static,
    {
        StatefulFluentBuilder::<TSeed, TValue, Override, Inline>::from_fluent_mut(seed, fluent_method)
    }
}

impl<TSeed, TValue, TStack, TStorage> StatefulFluentBuilder<TSeed, TValue, TStack, TStorage>
where
    TSeed: 'static,
    TStorage: Storage<TValue>,
    TStorage::Method: 'static,
{
    /**
    Box a fluent builder so it can be easily captured as a field without generics.
    */
    pub fn boxed(self) -> BoxedStatefulFluentBuilder<TSeed, TValue, TStack> {
        let StatefulFluentBuilderInner {
            state,
            fluent_method,
        } = self.inner;

        let fluent_method = fluent_method.map(|f| BoxedMethod(Box::new(f)));

        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state,
            fluent_method,
        })
    }
}

impl<TSeed, TValue, TStack, TStorage> StatefulFluentBuilder<TSeed, TValue, TStack, TStorage>
where
    TSeed: 'static,
    TStorage: Storage<TValue>,
    TStorage::Method: Send + 'static,
{
    /**
    Box a fluent builder so it can be easily shared.
    */
    pub fn shared(self) -> SharedStatefulFluentBuilder<TSeed, TValue, TStack> {
        let StatefulFluentBuilderInner {
            state,
            fluent_method,
        } = self.inner;

        let fluent_method = fluent_method.map(|f| SharedMethod(Box::new(f)));

        StatefulFluentBuilder::new(StatefulFluentBuilderInner {
            state,
            fluent_method,
        })
    }
}

impl<TValue, TFluent> Method<TValue> for TFluent
where
    TFluent: FnMut(TValue) -> TValue,
{
    fn apply(&mut self, value: TValue) -> TValue {
        self(value)
    }
}

impl<TValue> Method<TValue> for BoxedMethod<TValue> {
    fn apply(&mut self, value: TValue) -> TValue {
        self.0.apply(value)
    }
}

impl<TValue> Method<TValue> for SharedMethod<TValue> {
    fn apply(&mut self, value: TValue) -> TValue {
        self.0.apply(value)
    }
}

impl<TValue> Method<TValue> for Inline {
    fn apply(&mut self, value: TValue) -> TValue {
        value
    }
}

/* pub(crate) items */

pub trait Method<TValue> {
    fn apply(&mut self, value: TValue) -> TValue;
}

pub struct ByValue<TFluent>(TFluent);

pub struct ByRefMut<TFluent>(TFluent);

pub struct Apply<TValue, TPreviousMethod, TNextMethod> {
    inner: Option<StatefulApply<(), TValue, TPreviousMethod, TNextMethod>>,
}

impl<TValue, TPreviousMethod, TNextMethod> Apply<TValue, TPreviousMethod, TNextMethod> {
    fn new(previous: Option<TPreviousMethod>, next: TNextMethod) -> Self {
        Apply {
            inner: Some(StatefulApply::new((), previous, next)),
        }
    }
}

impl<TValue, TPreviousMethod, TNextMethod> Method<TValue>
    for Apply<TValue, TPreviousMethod, ByValue<TNextMethod>>
where
    TPreviousMethod: Method<TValue>,
    TNextMethod: FnOnce(TValue) -> TValue,
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let inner = mem::replace(&mut self.inner, None).expect("attempted to re-use builder");

        let (next, inner) = inner.take_next();

        inner
            .set_next(ByValue(move |_, value: TValue| (next.0)(value)))
            .apply(value)
    }
}

impl<TValue, TPreviousMethod, TNextMethod> Method<TValue>
    for Apply<TValue, TPreviousMethod, ByRefMut<TNextMethod>>
where
    TPreviousMethod: Method<TValue>,
    TNextMethod: FnOnce(&mut TValue),
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let inner = mem::replace(&mut self.inner, None).expect("attempted to re-use builder");

        let (next, inner) = inner.take_next();

        inner
            .set_next(ByRefMut(move |_, mut value: &mut TValue| {
                (next.0)(&mut value)
            }))
            .apply(value)
    }
}

pub struct StatefulApply<TSeed, TValue, TPreviousMethod, TNextMethod> {
    seed: Option<TSeed>,
    previous: Option<TPreviousMethod>,
    next: Option<TNextMethod>,
    _marker: PhantomData<TValue>,
}

impl<TSeed, TValue, TPreviousMethod, TNextMethod>
    StatefulApply<TSeed, TValue, TPreviousMethod, TNextMethod>
{
    fn new(seed: TSeed, previous: Option<TPreviousMethod>, next: TNextMethod) -> Self {
        StatefulApply {
            seed: Some(seed),
            previous: previous,
            next: Some(next),
            _marker: PhantomData,
        }
    }

    fn take_next(
        self,
    ) -> (
        TNextMethod,
        StatefulApply<TSeed, TValue, TPreviousMethod, ()>,
    ) {
        let next = self.next.expect("attempted to re-use builder");
        let self_sans_next = StatefulApply {
            seed: self.seed,
            previous: self.previous,
            next: Some(()),
            _marker: PhantomData,
        };

        (next, self_sans_next)
    }

    fn set_next<TNewNextMethod>(
        self,
        next: TNewNextMethod,
    ) -> StatefulApply<TSeed, TValue, TPreviousMethod, TNewNextMethod> {
        StatefulApply {
            seed: self.seed,
            previous: self.previous,
            next: Some(next),
            _marker: PhantomData,
        }
    }
}

impl<TSeed, TValue, TPreviousMethod, TNextMethod> Method<TValue>
    for StatefulApply<TSeed, TValue, TPreviousMethod, ByValue<TNextMethod>>
where
    TPreviousMethod: Method<TValue>,
    TNextMethod: FnOnce(TSeed, TValue) -> TValue,
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let seed = mem::replace(&mut self.seed, None).expect("attempted to re-use builder");
        let next = mem::replace(&mut self.next, None).expect("attempted to re-use builder");

        let value = match self.previous {
            Some(ref mut previous) => previous.apply(value),
            None => value,
        };

        (next.0)(seed, value)
    }
}

impl<TSeed, TValue, TPreviousMethod, TNextMethod> Method<TValue>
    for StatefulApply<TSeed, TValue, TPreviousMethod, ByRefMut<TNextMethod>>
where
    TPreviousMethod: Method<TValue>,
    TNextMethod: FnOnce(TSeed, &mut TValue),
{
    fn apply(&mut self, value: TValue) -> TValue {
        use std::mem;
        let seed = mem::replace(&mut self.seed, None).expect("attempted to re-use builder");
        let next = mem::replace(&mut self.next, None).expect("attempted to re-use builder");

        let mut value = match self.previous {
            Some(ref mut previous) => previous.apply(value),
            None => value,
        };

        (next.0)(seed, &mut value);
        value
    }
}

pub trait Storage<TValue> {
    type Method: Method<TValue>;
}

impl<TValue> Storage<TValue> for Boxed {
    type Method = BoxedMethod<TValue>;
}

impl<TValue> Storage<TValue> for Shared {
    type Method = SharedMethod<TValue>;
}

impl<TValue> Storage<TValue> for Inline {
    type Method = Self;
}

impl<TValue, TPreviousMethod, TNextMethod> Storage<TValue>
    for Apply<TValue, TPreviousMethod, TNextMethod>
where
    Apply<TValue, TPreviousMethod, TNextMethod>: Method<TValue>,
{
    type Method = Self;
}

impl<TSeed, TValue, TPreviousMethod, TNextMethod> Storage<TValue>
    for StatefulApply<TSeed, TValue, TPreviousMethod, TNextMethod>
where
    StatefulApply<TSeed, TValue, TPreviousMethod, TNextMethod>: Method<TValue>,
{
    type Method = Self;
}

#[cfg(test)]
mod tests {
    mod boxed {
        mod fluent_override {
            use imp::*;

            #[test]
            fn default() {
                let builder = FluentBuilder::<String>::default().boxed();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default", result);
            }

            #[test]
            fn default_value() {
                let builder = FluentBuilder::<String>::default()
                    .value("value".to_owned())
                    .boxed();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("value", result);
            }

            #[test]
            fn default_fluent() {
                let builder = FluentBuilder::<String>::default()
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"))
                    .boxed();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default_f2", result);
            }
        }

        mod fluent_stack {
            use imp::*;

            #[test]
            fn default() {
                let builder = FluentBuilder::<String, Stack>::default().boxed();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default", result);
            }

            #[test]
            fn default_value() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .value("value".to_owned())
                    .boxed();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("value", result);
            }

            #[test]
            fn default_fluent() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"))
                    .boxed();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default_f1_f2", result);
            }
        }
    }

    mod shared {
        mod fluent_override {
            use imp::*;

            #[test]
            fn default() {
                let builder = FluentBuilder::<String>::default().shared();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default", result);
            }

            #[test]
            fn default_value() {
                let builder = FluentBuilder::<String>::default()
                    .value("value".to_owned())
                    .shared();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("value", result);
            }

            #[test]
            fn default_fluent() {
                let builder = FluentBuilder::<String>::default()
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"))
                    .shared();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default_f2", result);
            }
        }

        mod fluent_stack {
            use imp::*;

            #[test]
            fn default() {
                let builder = FluentBuilder::<String, Stack, Inline>::default().shared();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default", result);
            }

            #[test]
            fn default_value() {
                let builder = FluentBuilder::<String, Stack, Inline>::default()
                    .value("value".to_owned())
                    .shared();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("value", result);
            }

            #[test]
            fn default_fluent() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"))
                    .shared();

                let result = builder.into_value(|| "default".to_owned());

                assert_eq!("default_f1_f2", result);
            }
        }
    }

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
                let builder = FluentBuilder::<String>::default().value("value".to_owned());

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
            fn value_take() {
                let builder = FluentBuilder::<String>::default().value("value".to_owned());

                let result = match builder.try_into_value() {
                    TryIntoValue::Value(value) => value,
                    _ => panic!("expected `TryIntoValue::Value`"),
                };

                assert_eq!("value", result);
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
                let builder = FluentBuilder::<String, Stack>::default().value("value".to_owned());

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

            #[test]
            fn value_fluent_take() {
                let builder = FluentBuilder::<String, Stack>::default()
                    .value("value".to_owned())
                    .fluent_mut(|v| v.push_str("_f1"))
                    .fluent_mut(|v| v.push_str("_f2"));

                let result = match builder.try_into_value() {
                    TryIntoValue::Value(value) => value,
                    _ => panic!("expected `TryIntoValue::Value`"),
                };

                assert_eq!("value_f1_f2", result);
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
                let builder =
                    StatefulFluentBuilder::<String, Builder>::from_seed("seed".to_owned());

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "seed".to_owned(),
                    optional: None,
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_fluent() {
                let builder = StatefulFluentBuilder::<String, Builder>::from_fluent_mut(
                    "seed".to_owned(),
                    |v| v.optional = Some("fluent".to_owned()),
                );

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "seed".to_owned(),
                    optional: Some("fluent".to_owned()),
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_value() {
                let builder = StatefulFluentBuilder::<String, Builder>::from_value(Builder {
                    required: "seed".to_owned(),
                    optional: None,
                });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "seed".to_owned(),
                    optional: None,
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_value() {
                let builder = StatefulFluentBuilder::<String, Builder>::from_seed(
                    "seed".to_owned(),
                )
                .value(Builder {
                    required: "value".to_owned(),
                    optional: Some("value".to_owned()),
                });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "value".to_owned(),
                    optional: Some("value".to_owned()),
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_fluent() {
                let builder =
                    StatefulFluentBuilder::<String, Builder>::from_seed("seed".to_owned())
                        .fluent_mut("f1".to_owned(), |v| v.optional = Some("f1".to_owned()))
                        .fluent_mut("f2".to_owned(), |v| v.optional = Some("f2".to_owned()));

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "f2".to_owned(),
                    optional: Some("f2".to_owned()),
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_value_fluent() {
                let builder =
                    StatefulFluentBuilder::<String, Builder>::from_seed("seed".to_owned())
                        .value(Builder {
                            required: "value".to_owned(),
                            optional: Some("value".to_owned()),
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
                    optional: None,
                });

                let expected = Builder {
                    required: "f2".to_owned(),
                    optional: None,
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_fluent_value() {
                let builder =
                    StatefulFluentBuilder::<String, Builder>::from_seed("seed".to_owned())
                        .fluent_mut("f1".to_owned(), |v| v.optional = Some("f1".to_owned()))
                        .fluent_mut("f2".to_owned(), |v| {
                            if let Some(ref mut optional) = v.optional.as_mut() {
                                optional.push_str("_f2");
                            }
                        })
                        .value(Builder {
                            required: "value".to_owned(),
                            optional: Some("value".to_owned()),
                        });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
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
                let builder =
                    StatefulFluentBuilder::<String, Builder, Stack>::from_seed("seed".to_owned());

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "seed".to_owned(),
                    optional: None,
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_fluent() {
                let builder = StatefulFluentBuilder::<String, Builder, Stack>::from_fluent_mut(
                    "seed".to_owned(),
                    |v| v.optional = Some("fluent".to_owned()),
                );

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "seed".to_owned(),
                    optional: Some("fluent".to_owned()),
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_value() {
                let builder =
                    StatefulFluentBuilder::<String, Builder, Stack>::from_value(Builder {
                        required: "seed".to_owned(),
                        optional: None,
                    });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "seed".to_owned(),
                    optional: None,
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_value() {
                let builder =
                    StatefulFluentBuilder::<String, Builder, Stack>::from_seed("seed".to_owned())
                        .value(Builder {
                            required: "value".to_owned(),
                            optional: Some("value".to_owned()),
                        });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "value".to_owned(),
                    optional: Some("value".to_owned()),
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_fluent() {
                let builder =
                    StatefulFluentBuilder::<String, Builder, Stack>::from_seed("seed".to_owned())
                        .fluent_mut("f1".to_owned(), |s, v| {
                            v.required = s;
                            v.optional = Some("f1".to_owned())
                        })
                        .fluent_mut("f2".to_owned(), |s, v| {
                            v.required = s;
                            if let Some(ref mut optional) = v.optional.as_mut() {
                                optional.push_str("_f2");
                            }
                        });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "f2".to_owned(),
                    optional: Some("f1_f2".to_owned()),
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_value_fluent() {
                let builder =
                    StatefulFluentBuilder::<String, Builder, Stack>::from_seed("seed".to_owned())
                        .value(Builder {
                            required: "value".to_owned(),
                            optional: Some("value".to_owned()),
                        })
                        .fluent_mut("f1".to_owned(), |s, v| {
                            v.required = s;
                            if let Some(ref mut optional) = v.optional.as_mut() {
                                optional.push_str("_f1");
                            }
                        })
                        .fluent_mut("f2".to_owned(), |s, v| {
                            v.required = s;
                            if let Some(ref mut optional) = v.optional.as_mut() {
                                optional.push_str("_f2");
                            }
                        });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
                });

                let expected = Builder {
                    required: "f2".to_owned(),
                    optional: Some("value_f1_f2".to_owned()),
                };

                assert_eq!(expected, result);
            }

            #[test]
            fn from_seed_fluent_value() {
                let builder =
                    StatefulFluentBuilder::<String, Builder, Stack>::from_seed("seed".to_owned())
                        .fluent_mut("f1".to_owned(), |s, v| {
                            v.required = s;
                            v.optional = Some("f1".to_owned())
                        })
                        .fluent_mut("f2".to_owned(), |s, v| {
                            v.required = s;
                            if let Some(ref mut optional) = v.optional.as_mut() {
                                optional.push_str("_f2");
                            }
                        })
                        .value(Builder {
                            required: "value".to_owned(),
                            optional: Some("value".to_owned()),
                        });

                let result = builder.into_value(|seed| Builder {
                    required: seed,
                    optional: None,
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
