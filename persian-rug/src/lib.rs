//! This crate provides a framework for managing arbitrary mutable
//! graphs of objects that link to one another. This is a pattern
//! which is difficult to replicate in Rust, because of the ownership
//! model, but is common in the wider software ecosystem.
//!
//! # Overview and motivation
//!
//! In the case where you have truly arbitrary graphs, most of an object's
//! dependencies cannot be usefully represented as being owned by that object.
//! For example, consider two types, `Foo` and `Bar`:
//!
//! ```rust
//! struct Foo {
//!     bars: Vec<Bar>
//! }
//!
//! struct Bar {
//!     my_special_foo: Option<Foo>
//! }
//! ```
//!
//! If no one `Foo` is the sole owner of a `Bar` (each `Foo` holds a
//! subset of available `Bar`s), and no one `Bar` is the sole owner of
//! any given `Foo` (the same `Foo` could be special for multiple
//! `Bar`s), then we end up with multiple copies of every object in
//! this representation, and maintaining consistency is likely to be
//! difficult.
//!
//! We might next try to use reference counting smart pointers for the
//! links between objects, but neither [`Rc`](std::rc::Rc) nor
//! [`Arc`](std::sync::Arc) gives us mutability on its own. If we then
//! consider using [`Mutex`](std::sync::Mutex) to provide mutability, we
//! end up with something like this:
//!
//! ```rust
//! use std::sync::{Arc, Mutex};
//!
//! struct Foo {
//!     bars: Vec<Arc<Mutex<Bar>>>
//! }
//!
//! struct Bar {
//!     my_special_foo: Option<Arc<Mutex<Foo>>>
//! }
//! ```
//!
//! in which each object is individually lockable to permit
//! mutation. But there is no meaningful lock order here, and deadlock
//! is all but assured.
//!
//! The approach taken in this crate is to store everything inside one
//! container (a [`Context`]) which gives a single location for locking.
//! Only the [`Context`] has ownership of data, everything else is
//! granted a [`Proxy`] which can be resolved using the [`Context`] into a
//! reference to the real object. We use attribute macros to
//! remove most of the boilerplate: [`contextual`] matches a type to
//! its owner, and [`persian_rug`] builds a suitable owner.
//!
//! That means the example using this crate looks like this:
//!
//! ```rust
//! use persian_rug::{contextual, persian_rug, Proxy};
//!
//! #[contextual(MyRug)]
//! struct Foo {
//!   bars: Vec<Proxy<Bar>>
//! }
//!
//! #[contextual(MyRug)]
//! struct Bar {
//!   my_special_foo: Option<Proxy<Foo>>
//! }
//!
//! #[persian_rug]
//! struct MyRug(#[table] Foo, #[table] Bar);
//! ```
//!
//! We will need to have an instance of `MyRug` available whenever we
//! want to read the contents of a `Foo` or a `Bar`. If we have a
//! mutable reference to the context, we can change any `Foo` or `Bar`
//! we wish.
//!
//! # The Persian Rug
//!
//! A [`Context`] provides the ability to insert, retrieve and iterate
//! over items by type. It can only support one collection of items
//! per type.
//!
//! > Please note:
//! > **this crate does not support deletion of objects** at present.
//!
//! ```rust
//! use persian_rug::{contextual, persian_rug, Context, Proxy, Table};
//!
//! #[contextual(C)]
//! struct Foo<C: Context> {
//!   _marker: core::marker::PhantomData<C>,
//!   pub a: i32,
//!   pub friend: Option<Proxy<Foo<C>>>
//! }
//!
//! impl<C: Context> Foo<C> {
//!   pub fn new(a: i32, friend: Option<Proxy<Foo<C>>>) -> Self {
//!     Self { _marker: Default::default(), a, friend }
//!   }
//! }
//!
//! #[persian_rug]
//! struct Rug(#[table] Foo<Rug>);
//!
//! let mut r = Rug(Table::new());
//! let p1 = r.add( Foo::new(1, None) );
//! let p2 = r.add( Foo::new(2, Some(p1)) );
//! let p3 = r.add( Foo::new(3, Some(p2)) );
//! r.get_mut(&p1).friend = Some(p3);
//!
//! ```
//!
//! Context read access is provided to implementations of [`Accessor`]
//! whose context matches. Shared references to the context are
//! accessors, as are [`Arc`](std::sync::Arc)s,
//! [`MutexGuard`](std::sync::MutexGuard)s and
//! [`RwLockReadGuard`](std::sync::RwLockReadGuard)s.
//!
//! Write access is provided to implementations of [`Mutator`] whose
//! context matches.  Exclusive references to the context are
//! mutators, as are [`MutexGuard`](std::sync::MutexGuard)s and
//! [`RwLockWriteGuard`](std::sync::RwLockWriteGuard)s. If you enable
//! the `clone-replace` feature, you can also use
//! [`MutateGuard`](clone_replace::MutateGuard)s for this.
//!
//! To prevent accidental misuse, each participating type must declare
//! its context by implementing [`Contextual`], and can only belong to
//! one context. This apparent restriction is easily lifted by making
//! the context a generic parameter of the participating type. The
//! [`constraints`] attribute can help with the boilerplate needed to
//! use generic parameters in this way.

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// A holder for [`Contextual`] types.
///
/// This is the "rug" in persian-rug (and in the examples, the context
/// is often called `Rug`). The implementor is the sole owner for the
/// [`Contextual`] objects it contains, and is responsible for resolving
/// [`Proxy`] objects into references to its owned copies.
///
/// You will not generally implement this trait directly. The
/// [`persian_rug`] attribute macro sets up the necessary
/// implementations for a usable [`Context`], converting each marked
/// field into a [`Table`], and providing an implementation of
/// [`Owner`] for each included type.
///
/// A context should only have one field of a given type. Its purpose
/// is to conveniently resolve [`Proxy`] objects and hold data; actual
/// data organisation ought to be done in a different object, by
/// holding whatever objects are required, in whatever organisation is
/// required, as proxies.
pub trait Context {
    /// Insert the given value, returning a [`Proxy`] for it.
    fn add<T>(&mut self, value: T) -> Proxy<T>
    where
        Self: Owner<T>,
        T: Contextual<Context = Self>;

    /// Retrieve a reference to a value from a [`Proxy`].
    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self: Owner<T>,
        T: Contextual<Context = Self>;

    /// Retrieve a mutable reference to a value from a [`Proxy`].
    fn get_mut<T>(&mut self, what: &Proxy<T>) -> &mut T
    where
        Self: Owner<T>,
        T: Contextual<Context = Self>;

    /// Iterate over the values currently stored.
    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self: Owner<T>,
        T: Contextual<Context = Self>;

    /// Mutably iterate over the values currently stored.
    fn get_iter_mut<T>(&mut self) -> TableMutIterator<'_, T>
    where
        Self: Owner<T>,
        T: Contextual<Context = Self>;

    /// Iterate over (owned) proxies for the values currently stored.
    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self: Owner<T>,
        T: Contextual<Context = Self>;
}

/// A convenient way to handle [`Context`] read access.
///
/// Rather than plumbing references to a context throughout your code,
/// especially when you are implementing derive macros that rely on
/// this crate, it can be more convenient to use this abstraction of
/// read-only access to a context.
///
/// In most hand-written code, it is simplest to use a shared
/// reference to a context as an Accessor.
pub trait Accessor: Clone {
    type Context: Context;

    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;
}

impl<'a, C> Accessor for &'a C
where
    C: Context,
{
    type Context = C;

    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get(self, what)
    }

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter(self)
    }

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_proxy_iter(self)
    }
}

impl<C> Accessor for std::sync::Arc<C>
where
    C: Context,
{
    type Context = C;
    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get(self, what)
    }

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter(self)
    }

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_proxy_iter(self)
    }
}

/// A convenient way to handle [`Context`] write access.
///
/// Rather than plumbing references to a context throughout your code,
/// especially when you are implementing derive macros that rely on
/// this crate, it can be more convenient to use this abstraction of
/// read-write access to a context.
///
/// In most hand-written code, it is simplest to use an exclusive
/// reference to a context as a Mutator.
pub trait Mutator {
    type Context: Context;

    fn add<T>(&mut self, proxy: T) -> Proxy<T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;

    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;

    fn get_mut<T>(&mut self, what: &Proxy<T>) -> &mut T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;

    fn get_iter_mut<T>(&mut self) -> TableMutIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>;
}

impl<'a, C> Mutator for &'a mut C
where
    C: Context,
{
    type Context = C;

    fn add<T>(&mut self, value: T) -> Proxy<T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::add(self, value)
    }

    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get(self, what)
    }

    fn get_mut<T>(&mut self, what: &Proxy<T>) -> &mut T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_mut(self, what)
    }

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter(self)
    }

    fn get_iter_mut<T>(&mut self) -> TableMutIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter_mut(self)
    }

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_proxy_iter(self)
    }
}

impl<'a, C> Mutator for std::sync::MutexGuard<'a, C>
where
    C: Context,
{
    type Context = C;

    fn add<T>(&mut self, value: T) -> Proxy<T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::add(self, value)
    }

    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get(self, what)
    }

    fn get_mut<T>(&mut self, what: &Proxy<T>) -> &mut T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_mut(self, what)
    }

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter(self)
    }

    fn get_iter_mut<T>(&mut self) -> TableMutIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter_mut(self)
    }

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_proxy_iter(self)
    }
}

impl<'a, C> Mutator for std::sync::RwLockWriteGuard<'a, C>
where
    C: Context,
{
    type Context = C;

    fn add<T>(&mut self, value: T) -> Proxy<T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::add(self, value)
    }

    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get(self, what)
    }

    fn get_mut<T>(&mut self, what: &Proxy<T>) -> &mut T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_mut(self, what)
    }

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter(self)
    }

    fn get_iter_mut<T>(&mut self) -> TableMutIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter_mut(self)
    }

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_proxy_iter(self)
    }
}

#[cfg(feature = "clone-replace")]
impl<C> Mutator for clone_replace::MutateGuard<C>
where
    C: Context,
{
    type Context = C;

    fn add<T>(&mut self, value: T) -> Proxy<T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::add(self, value)
    }

    fn get<T>(&self, what: &Proxy<T>) -> &T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get(self, what)
    }

    fn get_mut<T>(&mut self, what: &Proxy<T>) -> &mut T
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_mut(self, what)
    }

    fn get_iter<T>(&self) -> TableIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter(self)
    }

    fn get_iter_mut<T>(&mut self) -> TableMutIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_iter_mut(self)
    }

    fn get_proxy_iter<T>(&self) -> TableProxyIterator<'_, T>
    where
        Self::Context: Owner<T>,
        T: Contextual<Context = Self::Context>,
    {
        <C as Context>::get_proxy_iter(self)
    }
}

/// A type that owns (is the exclusive holder of) a [`Contextual`] type.
///
/// Implementations of this trait are normally provided for you by the
/// [`persian_rug`] attribute macro. You should rarely, if ever, need to
/// implement it yourself.
///
/// Each [`Contextual`] type has a single [`Context`] that owns it. Only
/// that context may implement this trait for the type, which provides
/// the standard API for [`Context`] objects, specialised to a single
/// type. The polymorphic interface for contexts calls through to the
/// functions defined in this trait, and you should never need to
/// call them directly; it is preferable to use the [`Context`] interface.
///
/// The main place in which [`Owner`] shows up in code written using
/// this crate is when specifying the constraints on what contexts are
/// permitted to call a given generic function. In general, those uses
/// of [`Owner`] can also be generated for you, using the [`constraints`]
/// attribute macro, but there are cases where you may need to refer
/// to it yourself: essentially whenever you need to assert that you
/// will be able to interact with a type `T` or a proxy for it, via
/// some context, then you can assert that the context implements
/// `Owner<T>`.
pub trait Owner<T>: Context
where
    T: Contextual<Context = Self>,
{
    /// Insert the given value, obtaining a [`Proxy`] for it.
    fn add(&mut self, value: T) -> Proxy<T>;
    /// Get a shared reference to a value from a [`Proxy`] for it.
    fn get(&self, proxy: &Proxy<T>) -> &T;
    /// Get an exclusive reference to a value from a [`Proxy`] for it.    
    fn get_mut(&mut self, proxy: &Proxy<T>) -> &mut T;
    /// Iterate over shared references to the stored values.
    fn get_iter(&self) -> TableIterator<'_, T>;
    /// Iterate over exclusive references to the stored values.
    fn get_iter_mut(&mut self) -> TableMutIterator<'_, T>;
    /// Iterate over shared references to [`Proxy`] objects for the
    /// stored values.
    fn get_proxy_iter(&self) -> TableProxyIterator<'_, T>;
}

/// Something that is associated to a context
///
/// An implementor of Contextual expects to be stored in a [`Table`]
/// in some other [`Context`] type. Generally, you want to do this if
/// there is some arbitrary object graph in which this participates,
/// where you want to link between objects with [`Proxy`] handles
/// provided by a [`Context`].  Even if your class itself contains no
/// links, and so could be adequately modeled by holding
/// [`Rc`](std::rc::Rc) or [`Arc`](std::sync::Arc) references to it,
/// you might opt to include it within a context for consistency, if
/// it's logically part of a larger whole which is mostly modeled
/// using this crate.
///
/// You will not generally implement this trait directly yourself,
/// instead you can use the attribute macro [`contextual`] to create
/// the necessary impl for you.
///
/// For types that don't necessarily always belong to the same [`Context`],
/// which is to say any types that could be re-used in a different scenario,
/// the recommended pattern is to make the context a generic parameter. For example:
///
/// ```rust
/// use persian_rug::{contextual, Context, Contextual};
///
/// #[contextual(C)]
/// struct Foo<C: Context> {
///   _marker: core::marker::PhantomData<C>
/// }
/// ```
///
/// The [`PhantomData`](core::marker::PhantomData) is only needed if your type
/// does not contain anything else to establish the context from. If you
/// contain another contextual type you can use that instead, for example:
/// ```rust
/// use persian_rug::{contextual, Context, Contextual, Proxy};
///
/// #[contextual(<T as Contextual>::Context)]
/// struct Bar<T: Contextual> {
///   my_proxy: Proxy<T>
/// }
/// ```
///
/// If you have collections of objects for which not all are required in every use case,
/// you might end up with a pattern like this:
///
/// ```rust
/// use persian_rug::{contextual, Context, Contextual, Proxy};
///
/// #[contextual(C)]
/// struct Foo<C: Context> {
///   _marker: core::marker::PhantomData<C>
/// }
///
/// #[contextual(C)]
/// struct Bar<C: Context> {
///   foo: Proxy<C>
/// }
///
/// ```
///
/// where it's possible to create a working [`Context`] that contains
/// only `Foo`s, or one that contains both `Foo`s and `Bar`s.
///
/// Having multiple different contexts for a single type is not
/// supported, and similarly holding [`Proxy`] objects for two
/// different contexts is likely to result in some difficulty. In
/// general, it is better to design your types to be usable in
/// different contexts if needed, as discussed above, but always
/// include everything needed in a given scenario in the same context.
pub trait Contextual {
    /// The [`Context`] type which owns values of this type.
    type Context: Context;
}

/// A handle to an item stored in some context.
///
/// A proxy is a link between objects, like [`Arc`](std::sync::Arc) or
/// [`Rc`](std::rc::Rc). It allows multiple objects to reference a
/// single other object, without any one of them being declared the
/// owner. Unlike reference counted smart pointers, you need to
/// provide its [`Context`] to traverse the link represented by a [`Proxy`].
///
/// When writing code that makes use of proxies, things are made more
/// complicated by the need to ensure the right kind of access for the
/// context is available (i.e. every function is likely to receive
/// either a [`Mutator`] or an [`Accessor`] parameter, depending on
/// whether mutable access is needed. However, things are made simpler
/// in terms of establishing that access is safe, since there is only
/// one object to which you need a mutable reference: the context
/// object.
///
/// The following example traverses an arbitrary graph of `Foo`s
/// (checking for cycles by never revisiting any node). There are of
/// course other ways of doing this (both in Rust, and using this
/// crate), but holding and working with such graphs is generally
/// considered challenging in Rust. As the example shows, it can be
/// relatively convenient to do so using this crate:
/// ```rust
/// use persian_rug::{contextual, persian_rug, Accessor, Context, Contextual, Proxy};
/// use std::collections::BTreeSet;
///
/// #[contextual(Rug)]
/// struct Foo {
///    id: String,
///    links: Vec<Proxy<Foo>>
/// }
///
/// impl Foo {
///   pub fn print_graph<A: Accessor<Context=Rug>>(&self, access: A) {
///     let mut b = BTreeSet::new();
///     let mut work = Vec::new();
///     work.push(self);
///     while let Some(item) = work.pop() {
///       println!("{}", item.id);
///       b.insert(item.id.clone());
///       for link in &item.links {
///         let link = access.get(link);
///         if !b.contains(&link.id) {
///           work.push(link);
///         }
///       }
///     }
///   }
/// }
///
/// #[persian_rug]
/// struct Rug(#[table] Foo);
///
/// ```
/// Had we used [`Proxy<Foo>`](Proxy) as our start value here (and given up having
/// a self parameter), we could've used the proxies themselves to check
/// for uniqueness, avoiding the need to compare and clone [`String`] fields,
/// and removing any chance of a name collision.
/// ```rust
/// use persian_rug::{contextual, persian_rug, Accessor, Context, Contextual, Proxy};
/// use std::collections::BTreeSet;
///
/// #[contextual(Rug)]
/// struct Foo {
///    id: String,
///    links: Vec<Proxy<Foo>>
/// }
///
/// impl Foo {
///   pub fn print_graph<A: Accessor<Context=Rug>>(start: Proxy<Foo>, access: A) {
///     let mut b = BTreeSet::new();
///     let mut work = Vec::new();
///     work.push(start);
///     while let Some(item_proxy) = work.pop() {
///       let item = access.get(&item_proxy);
///       println!("{}", item.id);
///       b.insert(item_proxy);
///       for link in &item.links {
///         if !b.contains(link) {
///           work.push(*link);
///         }
///       }
///     }
///   }
/// }
///
/// #[persian_rug]
/// struct Rug(#[table] Foo);
///
/// ```
///
/// Note that a [`Proxy`] implements [`Copy`] as well as [`Eq`]. The
/// implementation of [`Ord`] is guaranteed to be consistent on a given
/// run of the program, but no other guarantees are made.
pub struct Proxy<T> {
    _marker: core::marker::PhantomData<T>,
    index: u64,
}

impl<T> Clone for Proxy<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Proxy<T> {}

impl<T> PartialOrd for Proxy<T> {
    fn partial_cmp(&self, other: &Proxy<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Proxy<T> {
    fn cmp(&self, other: &Proxy<T>) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> PartialEq for Proxy<T> {
    fn eq(&self, other: &Proxy<T>) -> bool {
        self.index.eq(&other.index)
    }
}

impl<T> Eq for Proxy<T> {}

impl<T> Hash for Proxy<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> std::fmt::Debug for Proxy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "persian_rug::Proxy<{}> {{ handle: {} }}",
            std::any::type_name::<T>(),
            self.index
        )
    }
}

/// A holder for [`Contextual`] objects.
///
/// It is unlikely that you will ever need to instantiate this class,
/// unless for some reason the [`persian_rug`] attribute macro which
/// creates [`Context`] implementations is not suitable for your use.
///
/// A context object will generally have one table per object type,
/// and that table does the work of storing, retrieving and iterating
/// over objects of that type, and the [`Proxy`] objects that refer to
/// them.
#[derive(Clone, Debug)]
pub struct Table<T> {
    members: BTreeMap<u64, T>,
    proxies: Vec<Proxy<T>>,
    next_index: u64,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self {
            members: Default::default(),
            proxies: Default::default(),
            next_index: Default::default(),
        }
    }
}

impl<T> Table<T> {
    /// Create a new table.
    ///
    /// Tables are created empty.
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert a new item.
    ///
    /// The return value is a [`Proxy`] that you can store, and later
    /// use to retrieve the stored object from the table.
    pub fn push(&mut self, value: T) -> Proxy<T> {
        let ix = self.next_index;
        self.next_index += 1;
        self.members.insert(ix, value);
        let p = Proxy {
            _marker: Default::default(),
            index: ix,
        };
        self.proxies.push(p);
        p
    }

    /// Retrieve a previously stored item.
    ///
    /// Note that the return value is an [`Option`], because not all
    /// [`Proxy`] objects of a given type can be necessarily retrieved
    /// from a given [`Table`]. This is clearly the ideal however, and
    /// [`Context`] implementations created with the [`persian_rug`]
    /// attribute macro unwrap this return value, causing a panic on
    /// failure.
    pub fn get(&self, p: &Proxy<T>) -> Option<&T> {
        self.members.get(&p.index)
    }

    /// Retrieve a previously stored item mutably.
    ///
    /// Note that the return value is an [`Option`], because not all
    /// [`Proxy`] objects of a given type can be necessarily retrieved
    /// from a given [`Table`]. This is clearly the ideal however, and
    /// [`Context`] implementations created with the [`persian_rug`]
    /// attribute macro unwrap this return value, causing a panic on
    /// failure.
    pub fn get_mut(&mut self, p: &Proxy<T>) -> Option<&mut T> {
        self.members.get_mut(&p.index)
    }

    /// Iterate over shared references to all stored items.
    pub fn iter(&self) -> TableIterator<T> {
        TableIterator {
            iter: self.members.values(),
        }
    }

    /// Iterate over mutable references to all stored items.
    pub fn iter_mut(&mut self) -> TableMutIterator<T> {
        TableMutIterator {
            iter: self.members.values_mut(),
        }
    }

    /// Iterate over proxies for all stored items.
    ///
    /// Note that [`Proxy`] implements [`Copy`] so that although this
    /// returns references, you can cheaply convert them to owned
    /// values as required with the [`copied`][Iterator::copied]
    /// method on [`Iterator`].
    pub fn iter_proxies(&self) -> TableProxyIterator<T> {
        TableProxyIterator {
            iter: self.proxies.iter(),
        }
    }
}

/// An [`Iterator`] over references to [`Contextual`] objects.
pub struct TableIterator<'a, T> {
    iter: std::collections::btree_map::Values<'a, u64, T>,
}

impl<'a, T> Iterator for TableIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// An [`Iterator`] over references to [`Proxy`] objects for [`Contextual`]
/// objects.
pub struct TableProxyIterator<'a, T> {
    iter: std::slice::Iter<'a, Proxy<T>>,
}

impl<'a, T> Iterator for TableProxyIterator<'a, T> {
    type Item = &'a Proxy<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// An [`Iterator`] over exclusive references to [`Contextual`] objects.
pub struct TableMutIterator<'a, T> {
    iter: std::collections::btree_map::ValuesMut<'a, u64, T>,
}

impl<'a, T> Iterator for TableMutIterator<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub use persian_rug_derive::{constraints, contextual, persian_rug};
