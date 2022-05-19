#![cfg(test)]
#![allow(dead_code)]

use std::any::Any;

#[derive(Clone)]
#[persian_rug::contextual(C)]
struct Foo<C: persian_rug::Context> {
    _marker: core::marker::PhantomData<C>,
    a: i32,
}

#[derive(Clone)]
#[persian_rug::contextual(C)]
struct Bar<C: persian_rug::Context> {
    a: i32,
    foo: persian_rug::Proxy<Foo<C>>,
}

#[derive(Clone)]
#[persian_rug::contextual(C)]
struct Baz<C: persian_rug::Context> {
    a: i32,
    bar: persian_rug::Proxy<Bar<C>>,
}

#[derive(Clone)]
#[persian_rug::persian_rug]
pub struct State {
    #[table]
    foo: Foo<State>,
    #[table]
    bar: Bar<State>,
    #[table]
    baz: Baz<State>,
}

#[persian_rug::contextual(State2)]
struct Foo2 {
    a: i32,
}

#[persian_rug::persian_rug]
pub struct State2(
    #[table] Foo<State2>,
    #[table] Foo2,
    #[table] Bar<State2>,
    #[table] Baz<State2>,
);

mod context_tests {
    use super::*;

    use persian_rug::Context;

    #[test]
    fn test_context() {
        let mut s = State {
            foo: persian_rug::Table::new(),
            bar: persian_rug::Table::new(),
            baz: persian_rug::Table::new(),
        };

        let f1 = s.add(Foo {
            _marker: Default::default(),
            a: 0,
        });
        let f2 = s.add(Foo {
            _marker: Default::default(),
            a: 1,
        });
        let f3 = s.add(Foo {
            _marker: Default::default(),
            a: 2,
        });

        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Foo<State>>>(),
            f1.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Foo<State>>>(),
            f2.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Foo<State>>>(),
            f3.type_id()
        );

        let b1 = s.add(Bar { foo: f1, a: 3 });
        let b2 = s.add(Bar { foo: f1, a: 4 });
        let b3 = s.add(Bar { foo: f2, a: 5 });

        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Bar<State>>>(),
            b1.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Bar<State>>>(),
            b2.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Bar<State>>>(),
            b3.type_id()
        );

        let z1 = s.add(Baz { bar: b1, a: 6 });
        let z2 = s.add(Baz { bar: b2, a: 7 });
        let z3 = s.add(Baz { bar: b2, a: 8 });

        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Baz<State>>>(),
            z1.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Baz<State>>>(),
            z2.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Baz<State>>>(),
            z3.type_id()
        );

        assert_eq!(std::any::TypeId::of::<Foo<State>>(), s.get(&f1).type_id());
        assert_eq!(std::any::TypeId::of::<Foo<State>>(), s.get(&f2).type_id());
        assert_eq!(std::any::TypeId::of::<Foo<State>>(), s.get(&f3).type_id());

        assert_eq!(std::any::TypeId::of::<Bar<State>>(), s.get(&b1).type_id());
        assert_eq!(std::any::TypeId::of::<Bar<State>>(), s.get(&b2).type_id());
        assert_eq!(std::any::TypeId::of::<Bar<State>>(), s.get(&b3).type_id());

        assert_eq!(std::any::TypeId::of::<Baz<State>>(), s.get(&z1).type_id());
        assert_eq!(std::any::TypeId::of::<Baz<State>>(), s.get(&z2).type_id());
        assert_eq!(std::any::TypeId::of::<Baz<State>>(), s.get(&z3).type_id());

        assert_eq!(s.get(&f1).a, 0);
        assert_eq!(s.get(&f2).a, 1);
        assert_eq!(s.get(&f3).a, 2);

        assert_eq!(s.get(&b1).a, 3);
        assert_eq!(s.get(&b2).a, 4);
        assert_eq!(s.get(&b3).a, 5);
        assert_eq!(s.get(&s.get(&b1).foo).a, 0);
        assert_eq!(s.get(&s.get(&b2).foo).a, 0);
        assert_eq!(s.get(&s.get(&b3).foo).a, 1);

        assert_eq!(s.get(&z1).a, 6);
        assert_eq!(s.get(&z2).a, 7);
        assert_eq!(s.get(&z3).a, 8);
        assert_eq!(s.get(&s.get(&z1).bar).a, 3);
        assert_eq!(s.get(&s.get(&z2).bar).a, 4);
        assert_eq!(s.get(&s.get(&z3).bar).a, 4);
        assert_eq!(s.get(&s.get(&s.get(&z1).bar).foo).a, 0);
        assert_eq!(s.get(&s.get(&s.get(&z2).bar).foo).a, 0);
        assert_eq!(s.get(&s.get(&s.get(&z3).bar).foo).a, 0);

        s.get_mut(&b2).a = 9;
        s.get_mut(&b2).foo = f3;
        s.get_mut(&z3).a = 10;
        s.get_mut(&z3).bar = b3;
        s.get_mut(&s.get(&b3).foo.clone()).a = 11;

        assert_eq!(s.get(&f1).a, 0);
        assert_eq!(s.get(&f2).a, 11);
        assert_eq!(s.get(&f3).a, 2);

        assert_eq!(s.get(&b1).a, 3);
        assert_eq!(s.get(&b2).a, 9);
        assert_eq!(s.get(&b3).a, 5);
        assert_eq!(s.get(&s.get(&b1).foo).a, 0);
        assert_eq!(s.get(&s.get(&b2).foo).a, 2);
        assert_eq!(s.get(&s.get(&b3).foo).a, 11);

        assert_eq!(s.get(&z1).a, 6);
        assert_eq!(s.get(&z2).a, 7);
        assert_eq!(s.get(&z3).a, 10);
        assert_eq!(s.get(&s.get(&z1).bar).a, 3);
        assert_eq!(s.get(&s.get(&z2).bar).a, 9);
        assert_eq!(s.get(&s.get(&z3).bar).a, 5);
        assert_eq!(s.get(&s.get(&s.get(&z1).bar).foo).a, 0);
        assert_eq!(s.get(&s.get(&s.get(&z2).bar).foo).a, 2);
        assert_eq!(s.get(&s.get(&s.get(&z3).bar).foo).a, 11);

        let foos = s.get_iter().collect::<Vec<&Foo<State>>>();
        assert_eq!(foos[0].a, 0);
        assert_eq!(foos[1].a, 11);
        assert_eq!(foos[2].a, 2);

        let bars = s.get_iter().collect::<Vec<&Bar<State>>>();
        assert_eq!(bars[0].a, 3);
        assert_eq!(bars[1].a, 9);
        assert_eq!(bars[2].a, 5);
        assert_eq!(bars[0].foo, f1);
        assert_eq!(bars[1].foo, f3);
        assert_eq!(bars[2].foo, f2);

        let bazs = s.get_iter().collect::<Vec<&Baz<State>>>();
        assert_eq!(bazs[0].a, 6);
        assert_eq!(bazs[1].a, 7);
        assert_eq!(bazs[2].a, 10);
        assert_eq!(bazs[0].bar, b1);
        assert_eq!(bazs[1].bar, b2);
        assert_eq!(bazs[2].bar, b3);

        let mut foos = s.get_iter_mut().collect::<Vec<&mut Foo<State>>>();
        foos[0].a = 12;

        assert_eq!(s.get(&f1).a, 12);
        assert_eq!(s.get(&f2).a, 11);
        assert_eq!(s.get(&f3).a, 2);
        assert_eq!(s.get(&s.get(&b1).foo).a, 12);
        assert_eq!(s.get(&s.get(&b2).foo).a, 2);
        assert_eq!(s.get(&s.get(&b3).foo).a, 11);

        let foos = s
            .get_proxy_iter()
            .collect::<Vec<&persian_rug::Proxy<Foo<State>>>>();
        assert_eq!(*foos[0], f1);
        assert_eq!(*foos[1], f2);
        assert_eq!(*foos[2], f3);

        let bars = s
            .get_proxy_iter()
            .collect::<Vec<&persian_rug::Proxy<Bar<State>>>>();
        assert_eq!(*bars[0], b1);
        assert_eq!(*bars[1], b2);
        assert_eq!(*bars[2], b3);

        let bazs = s
            .get_proxy_iter()
            .collect::<Vec<&persian_rug::Proxy<Baz<State>>>>();
        assert_eq!(*bazs[0], z1);
        assert_eq!(*bazs[1], z2);
        assert_eq!(*bazs[2], z3);
    }
}

mod table_tests {
    use super::*;
    use persian_rug::Table;

    #[test]
    fn test_table() {
        let mut t = Table::<Foo<State2>>::new();

        let f1 = t.push(Foo {
            _marker: Default::default(),
            a: 0,
        });
        let f2 = t.push(Foo {
            _marker: Default::default(),
            a: 1,
        });
        let f3 = t.push(Foo {
            _marker: Default::default(),
            a: 2,
        });

        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Foo<State2>>>(),
            f1.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Foo<State2>>>(),
            f2.type_id()
        );
        assert_eq!(
            std::any::TypeId::of::<persian_rug::Proxy<Foo<State2>>>(),
            f3.type_id()
        );

        assert_eq!(t.get(&f1).map(|f| f.a), Some(0));
        assert_eq!(t.get(&f2).map(|f| f.a), Some(1));
        assert_eq!(t.get(&f3).map(|f| f.a), Some(2));

        t.get_mut(&f1).map(|f| f.a = 3);
        t.get_mut(&f2).map(|f| f.a = 4);
        t.get_mut(&f3).map(|f| f.a = 5);

        assert_eq!(t.get(&f1).map(|f| f.a), Some(3));
        assert_eq!(t.get(&f2).map(|f| f.a), Some(4));
        assert_eq!(t.get(&f3).map(|f| f.a), Some(5));

        let foos = t.iter().collect::<Vec<&Foo<State2>>>();
        assert_eq!(foos[0].a, 3);
        assert_eq!(foos[1].a, 4);
        assert_eq!(foos[2].a, 5);

        let mut foos = t.iter_mut().collect::<Vec<&mut Foo<State2>>>();
        foos[0].a = 6;
        foos[1].a = 7;
        foos[2].a = 8;

        assert_eq!(t.get(&f1).map(|f| f.a), Some(6));
        assert_eq!(t.get(&f2).map(|f| f.a), Some(7));
        assert_eq!(t.get(&f3).map(|f| f.a), Some(8));

        let foos = t
            .iter_proxies()
            .collect::<Vec<&persian_rug::Proxy<Foo<State2>>>>();
        assert_eq!(foos[0], &f1);
        assert_eq!(foos[1], &f2);
        assert_eq!(foos[2], &f3);
    }
}

mod proxy_tests {
    use super::*;
    use persian_rug::Context;
    use std::cmp::{Ord, Ordering};

    #[test]
    fn test_ord() {
        let mut s = State2(
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
        );

        let f1 = s.add(Foo2 { a: 0 });
        let f2 = s.add(Foo2 { a: 0 });
        let f3 = s.add(Foo2 { a: 0 });
        let f4 = s.add(Foo2 { a: 0 });

        assert_eq!(f1.cmp(&f1), Ordering::Equal);
        assert_eq!(f1.cmp(&f2), Ordering::Less);
        assert_eq!(f1.cmp(&f3), Ordering::Less);
        assert_eq!(f1.cmp(&f4), Ordering::Less);
        assert_eq!(f2.cmp(&f1), Ordering::Greater);
        assert_eq!(f2.cmp(&f2), Ordering::Equal);
        assert_eq!(f2.cmp(&f3), Ordering::Less);
        assert_eq!(f2.cmp(&f4), Ordering::Less);
        assert_eq!(f3.cmp(&f1), Ordering::Greater);
        assert_eq!(f3.cmp(&f2), Ordering::Greater);
        assert_eq!(f3.cmp(&f3), Ordering::Equal);
        assert_eq!(f3.cmp(&f4), Ordering::Less);
        assert_eq!(f4.cmp(&f1), Ordering::Greater);
        assert_eq!(f4.cmp(&f2), Ordering::Greater);
        assert_eq!(f4.cmp(&f3), Ordering::Greater);
        assert_eq!(f4.cmp(&f4), Ordering::Equal);
    }

    #[test]
    fn test_partial_ord() {
        let mut s = State2(
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
        );

        let f1 = s.add(Foo2 { a: 0 });
        let f2 = s.add(Foo2 { a: 0 });
        let f3 = s.add(Foo2 { a: 0 });
        let f4 = s.add(Foo2 { a: 0 });

        assert_eq!(f1.partial_cmp(&f1), Some(Ordering::Equal));
        assert_eq!(f1.partial_cmp(&f2), Some(Ordering::Less));
        assert_eq!(f1.partial_cmp(&f3), Some(Ordering::Less));
        assert_eq!(f1.partial_cmp(&f4), Some(Ordering::Less));
        assert_eq!(f2.partial_cmp(&f1), Some(Ordering::Greater));
        assert_eq!(f2.partial_cmp(&f2), Some(Ordering::Equal));
        assert_eq!(f2.partial_cmp(&f3), Some(Ordering::Less));
        assert_eq!(f2.partial_cmp(&f4), Some(Ordering::Less));
        assert_eq!(f3.partial_cmp(&f1), Some(Ordering::Greater));
        assert_eq!(f3.partial_cmp(&f2), Some(Ordering::Greater));
        assert_eq!(f3.partial_cmp(&f3), Some(Ordering::Equal));
        assert_eq!(f3.partial_cmp(&f4), Some(Ordering::Less));
        assert_eq!(f4.partial_cmp(&f1), Some(Ordering::Greater));
        assert_eq!(f4.partial_cmp(&f2), Some(Ordering::Greater));
        assert_eq!(f4.partial_cmp(&f3), Some(Ordering::Greater));
        assert_eq!(f4.partial_cmp(&f4), Some(Ordering::Equal));
    }

    #[test]
    fn test_eq() {
        let mut s = State2(
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
        );

        let f1 = s.add(Foo2 { a: 0 });
        let f2 = s.add(Foo2 { a: 0 });
        let f3 = s.add(Foo2 { a: 0 });
        let f4 = s.add(Foo2 { a: 0 });

        assert_eq!(f1.eq(&f1), true);
        assert_eq!(f1.eq(&f2), false);
        assert_eq!(f1.eq(&f3), false);
        assert_eq!(f1.eq(&f4), false);
        assert_eq!(f2.eq(&f1), false);
        assert_eq!(f2.eq(&f2), true);
        assert_eq!(f2.eq(&f3), false);
        assert_eq!(f2.eq(&f4), false);
        assert_eq!(f3.eq(&f1), false);
        assert_eq!(f3.eq(&f2), false);
        assert_eq!(f3.eq(&f3), true);
        assert_eq!(f3.eq(&f4), false);
        assert_eq!(f4.eq(&f1), false);
        assert_eq!(f4.eq(&f2), false);
        assert_eq!(f4.eq(&f3), false);
        assert_eq!(f4.eq(&f4), true);
    }

    #[test]
    fn test_debug() {
        let mut s = State2(
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
            persian_rug::Table::new(),
        );

        let f1 = s.add(Foo2 { a: 0 });
        let f2 = s.add(Foo2 { a: 0 });
        let f3 = s.add(Foo2 { a: 0 });
        let f4 = s.add(Foo2 { a: 0 });

        assert_eq!(
            &format!("{:?}", f1),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 0 }"
        );
        assert_eq!(
            &format!("{:?}", f2),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 1 }"
        );
        assert_eq!(
            &format!("{:?}", f3),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 2 }"
        );
        assert_eq!(
            &format!("{:?}", f4),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 3 }"
        );
    }
}

mod impl_constraints_tests {
    use super::*;

    #[persian_rug::constraints(context = C)]
    impl<C> Foo<C> {
        fn read_a(&self) -> i32 {
            self.a
        }
    }

    #[persian_rug::constraints(context=C, access(Foo<C>))]
    impl<C> Foo<C> {
        fn read_proxy_a<A: persian_rug::Accessor<Context = C>>(
            p: &persian_rug::Proxy<Foo<C>>,
            access: A,
        ) -> i32 {
            access.get(&p).a
        }
    }

    #[persian_rug::constraints(context = C)]
    impl<C> Bar<C> {
        fn read_a(&self) -> i32 {
            self.a
        }
    }

    #[persian_rug::constraints(context=C, access(Foo<C>))]
    impl<C> Bar<C> {
        fn read_foo_a<A: persian_rug::Accessor<Context = C>>(&self, access: A) -> i32 {
            access.get(&self.foo).a
        }
    }

    #[persian_rug::constraints(context=C, access(Bar<C>))]
    impl<C> Bar<C> {
        fn read_proxy_a<A: persian_rug::Accessor<Context = C>>(
            p: &persian_rug::Proxy<Bar<C>>,
            access: A,
        ) -> i32 {
            access.get(&p).a
        }
    }

    #[persian_rug::constraints(context=C, access(Foo<C>, Bar<C>))]
    impl<C: persian_rug::Context> Bar<C> {
        fn read_proxy_foo_a<A: persian_rug::Accessor<Context = C>>(
            p: &persian_rug::Proxy<Bar<C>>,
            access: A,
        ) -> i32 {
            access.get(&access.get(&p).foo).a
        }
    }

    #[persian_rug::constraints(context = C)]
    impl<C> Baz<C> {
        fn read_a(&self) -> i32 {
            self.a
        }
    }

    #[persian_rug::constraints(context=C, access(Bar<C>))]
    impl<C> Baz<C> {
        fn read_bar_a<A: persian_rug::Accessor<Context = C>>(&self, access: A) -> i32 {
            access.get(&self.bar).a
        }
    }

    #[persian_rug::constraints(context=C, access(Foo<C>, Bar<C>))]
    impl<C> Baz<C> {
        fn read_bar_foo_a<A: persian_rug::Accessor<Context = C>>(&self, access: A) -> i32 {
            access.get(&access.get(&self.bar).foo).a
        }
    }

    #[persian_rug::constraints(context=C, access(Baz<C>))]
    impl<C: persian_rug::Context> Baz<C> {
        fn read_proxy_a<A: persian_rug::Accessor<Context = C>>(
            p: &persian_rug::Proxy<Baz<C>>,
            access: A,
        ) -> i32 {
            access.get(&p).a
        }
    }

    #[persian_rug::constraints(context=C, access(Bar<C>, Baz<C>))]
    impl<C: persian_rug::Context> Baz<C> {
        fn read_proxy_bar_a<A: persian_rug::Accessor<Context = C>>(
            p: &persian_rug::Proxy<Baz<C>>,
            access: A,
        ) -> i32 {
            access.get(&access.get(&p).bar).a
        }
    }

    #[persian_rug::constraints(context=C, access(Foo<C>, Bar<C>, Baz<C>))]
    impl<C: persian_rug::Context> Baz<C> {
        fn read_proxy_bar_foo_a<A: persian_rug::Accessor<Context = C>>(
            p: &persian_rug::Proxy<Baz<C>>,
            access: A,
        ) -> i32 {
            access.get(&access.get(&access.get(&p).bar).foo).a
        }
    }

    #[test]
    fn test_impls() {
        use persian_rug::Context;

        let mut s = State {
            foo: persian_rug::Table::new(),
            bar: persian_rug::Table::new(),
            baz: persian_rug::Table::new(),
        };

        let f1 = s.add(Foo {
            a: 1,
            _marker: Default::default(),
        });
        let b1 = s.add(Bar { a: 2, foo: f1 });
        let z1 = s.add(Baz { a: 3, bar: b1 });

        assert_eq!(s.get(&f1).read_a(), 1);
        assert_eq!(Foo::read_proxy_a(&f1, &s), 1);

        assert_eq!(s.get(&b1).read_a(), 2);
        assert_eq!(s.get(&b1).read_foo_a(&s), 1);
        assert_eq!(Bar::read_proxy_a(&b1, &s), 2);
        assert_eq!(Bar::read_proxy_foo_a(&b1, &s), 1);

        assert_eq!(s.get(&z1).read_a(), 3);
        assert_eq!(s.get(&z1).read_bar_a(&s), 2);
        assert_eq!(s.get(&z1).read_bar_foo_a(&s), 1);
        assert_eq!(Baz::read_proxy_a(&z1, &s), 3);
        assert_eq!(Baz::read_proxy_bar_a(&z1, &s), 2);
        assert_eq!(Baz::read_proxy_bar_foo_a(&z1, &s), 1);
    }
}

mod fn_constraints_tests {
    use super::*;

    #[persian_rug::constraints(context = C)]
    fn foo_read_a<C>(foo: &Foo<C>) -> i32 {
        foo.a
    }

    #[persian_rug::constraints(context=C, access(Foo<C>))]
    fn foo_read_proxy_a<C, A: persian_rug::Accessor<Context = C>>(
        p: &persian_rug::Proxy<Foo<C>>,
        access: A,
    ) -> i32 {
        access.get(&p).a
    }

    #[persian_rug::constraints(context = C)]
    fn bar_read_a<C>(bar: &Bar<C>) -> i32 {
        bar.a
    }

    #[persian_rug::constraints(context=C, access(Foo<C>))]
    fn bar_read_foo_a<C, A: persian_rug::Accessor<Context = C>>(bar: &Bar<C>, access: A) -> i32 {
        access.get(&bar.foo).a
    }

    #[persian_rug::constraints(context=C, access(Bar<C>))]
    fn bar_read_proxy_a<C, A: persian_rug::Accessor<Context = C>>(
        p: &persian_rug::Proxy<Bar<C>>,
        access: A,
    ) -> i32 {
        access.get(&p).a
    }

    #[persian_rug::constraints(context=C, access(Foo<C>, Bar<C>))]
    fn bar_read_proxy_foo_a<C, A: persian_rug::Accessor<Context = C>>(
        p: &persian_rug::Proxy<Bar<C>>,
        access: A,
    ) -> i32 {
        access.get(&access.get(&p).foo).a
    }

    #[persian_rug::constraints(context = C)]
    fn baz_read_a<C>(baz: &Baz<C>) -> i32 {
        baz.a
    }

    #[persian_rug::constraints(context=C, access(Bar<C>))]
    fn baz_read_bar_a<C, A: persian_rug::Accessor<Context = C>>(baz: &Baz<C>, access: A) -> i32 {
        access.get(&baz.bar).a
    }

    #[persian_rug::constraints(context=C, access(Foo<C>, Bar<C>))]
    fn baz_read_bar_foo_a<C, A: persian_rug::Accessor<Context = C>>(
        baz: &Baz<C>,
        access: A,
    ) -> i32 {
        access.get(&access.get(&baz.bar).foo).a
    }

    #[persian_rug::constraints(context=C, access(Baz<C>))]
    fn baz_read_proxy_a<C, A: persian_rug::Accessor<Context = C>>(
        p: &persian_rug::Proxy<Baz<C>>,
        access: A,
    ) -> i32 {
        access.get(&p).a
    }

    #[persian_rug::constraints(context=C, access(Bar<C>, Baz<C>))]
    fn baz_read_proxy_bar_a<C, A: persian_rug::Accessor<Context = C>>(
        p: &persian_rug::Proxy<Baz<C>>,
        access: A,
    ) -> i32 {
        access.get(&access.get(&p).bar).a
    }

    #[persian_rug::constraints(context=C, access(Foo<C>, Bar<C>, Baz<C>))]
    fn baz_read_proxy_bar_foo_a<C, A: persian_rug::Accessor<Context = C>>(
        p: &persian_rug::Proxy<Baz<C>>,
        access: A,
    ) -> i32 {
        access.get(&access.get(&access.get(&p).bar).foo).a
    }

    #[test]
    fn test_fns() {
        use persian_rug::Context;

        let mut s = State {
            foo: persian_rug::Table::new(),
            bar: persian_rug::Table::new(),
            baz: persian_rug::Table::new(),
        };

        let f1 = s.add(Foo {
            a: 1,
            _marker: Default::default(),
        });
        let b1 = s.add(Bar { a: 2, foo: f1 });
        let z1 = s.add(Baz { a: 3, bar: b1 });

        assert_eq!(foo_read_a(s.get(&f1)), 1);
        assert_eq!(foo_read_proxy_a(&f1, &s), 1);

        assert_eq!(bar_read_a(s.get(&b1)), 2);
        assert_eq!(bar_read_foo_a(s.get(&b1), &s), 1);
        assert_eq!(bar_read_proxy_a(&b1, &s), 2);
        assert_eq!(bar_read_proxy_foo_a(&b1, &s), 1);

        assert_eq!(baz_read_a(s.get(&z1)), 3);
        assert_eq!(baz_read_bar_a(s.get(&z1), &s), 2);
        assert_eq!(baz_read_bar_foo_a(s.get(&z1), &s), 1);
        assert_eq!(baz_read_proxy_a(&z1, &s), 3);
        assert_eq!(baz_read_proxy_bar_a(&z1, &s), 2);
        assert_eq!(baz_read_proxy_bar_foo_a(&z1, &s), 1);
    }
}

mod struct_constraints_tests {
    #[persian_rug::constraints(context = C)]
    #[persian_rug::contextual(C)]
    struct Foo3<C> {
        _marker: core::marker::PhantomData<C>,
        a: i32,
    }

    #[persian_rug::constraints(context = C, access(Foo3<C>))]
    #[persian_rug::contextual(C)]
    struct Bar3<C> {
        a: i32,
        foo: persian_rug::Proxy<Foo3<C>>,
    }

    #[persian_rug::constraints(context = C, access(Foo3<C>, Bar3<C>))]
    #[persian_rug::contextual(C)]
    struct Baz3<C> {
        a: i32,
        bar: persian_rug::Proxy<Bar3<C>>,
    }

    #[persian_rug::persian_rug]
    pub struct State3a {
        #[table]
        foo: Foo3<State3a>,
    }

    #[persian_rug::persian_rug]
    pub struct State3b {
        #[table]
        foo: Foo3<State3b>,
        #[table]
        bar: Bar3<State3b>,
    }

    #[persian_rug::persian_rug]
    pub struct State3c {
        #[table]
        foo: Foo3<State3c>,
        #[table]
        bar: Bar3<State3c>,
        #[table]
        baz: Baz3<State3c>,
    }

    #[test]
    fn test_structs() {
        use persian_rug::Context;

        let mut s3a = State3a {
            foo: Default::default(),
        };

        let _f1 = s3a.add(Foo3 {
            a: 1,
            _marker: Default::default(),
        });

        let mut s3b = State3b {
            foo: Default::default(),
            bar: Default::default(),
        };

        let f1 = s3b.add(Foo3 {
            a: 1,
            _marker: Default::default(),
        });
        let _b1 = s3b.add(Bar3 { a: 2, foo: f1 });

        let mut s3c = State3c {
            foo: Default::default(),
            bar: Default::default(),
            baz: Default::default(),
        };

        let f1 = s3c.add(Foo3 {
            a: 1,
            _marker: Default::default(),
        });
        let b1 = s3c.add(Bar3 { a: 2, foo: f1 });
        let _z1 = s3c.add(Baz3 { a: 3, bar: b1 });
    }
}

mod enum_constraints_tests {
    #[persian_rug::constraints(context = C)]
    #[persian_rug::contextual(C)]
    enum Foo4<C> {
        Base {
            _marker: core::marker::PhantomData<C>,
            a: i32,
        },
    }

    #[persian_rug::constraints(context = C, access(Foo4<C>))]
    #[persian_rug::contextual(C)]
    enum Bar4<C> {
        Base {
            a: i32,
            foo: persian_rug::Proxy<Foo4<C>>,
        },
    }

    #[persian_rug::constraints(context = C, access(Foo4<C>, Bar4<C>))]
    #[persian_rug::contextual(C)]
    enum Baz4<C> {
        Base {
            a: i32,
            bar: persian_rug::Proxy<Bar4<C>>,
        },
    }

    #[persian_rug::persian_rug]
    pub struct State4a {
        #[table]
        foo: Foo4<State4a>,
    }

    #[persian_rug::persian_rug]
    pub struct State4b {
        #[table]
        foo: Foo4<State4b>,
        #[table]
        bar: Bar4<State4b>,
    }

    #[persian_rug::persian_rug]
    pub struct State4c {
        #[table]
        foo: Foo4<State4c>,
        #[table]
        bar: Bar4<State4c>,
        #[table]
        baz: Baz4<State4c>,
    }

    #[test]
    fn test_enums() {
        use persian_rug::Context;

        let mut s4a = State4a {
            foo: Default::default(),
        };

        let _f1 = s4a.add(Foo4::Base {
            a: 1,
            _marker: Default::default(),
        });

        let mut s4b = State4b {
            foo: Default::default(),
            bar: Default::default(),
        };

        let f1 = s4b.add(Foo4::Base {
            a: 1,
            _marker: Default::default(),
        });
        let _b1 = s4b.add(Bar4::Base { a: 2, foo: f1 });

        let mut s4c = State4c {
            foo: Default::default(),
            bar: Default::default(),
            baz: Default::default(),
        };

        let f1 = s4c.add(Foo4::Base {
            a: 1,
            _marker: Default::default(),
        });
        let b1 = s4c.add(Bar4::Base { a: 2, foo: f1 });
        let _z1 = s4c.add(Baz4::Base { a: 3, bar: b1 });
    }
}

mod trait_constraints_tests {
    #[persian_rug::constraints(context = C)]
    trait Foo5<C> {
        fn read_a(&self) -> i32;
    }

    #[persian_rug::constraints(context = C)]
    impl<C> persian_rug::Contextual for Box<dyn Foo5<C>> {
        type Context = C;
    }

    #[persian_rug::constraints(context = C)]
    #[persian_rug::contextual(C)]
    struct F5<C> {
        _marker: core::marker::PhantomData<C>,
    }

    #[persian_rug::constraints(context = C)]
    impl<C> Foo5<C> for F5<C> {
        fn read_a(&self) -> i32 {
            1
        }
    }

    #[persian_rug::constraints(context = C, access(Box<dyn Foo5<C>>))]
    trait Bar5<C> {
        fn read_a(&self) -> i32;
        fn read_foo(&self) -> persian_rug::Proxy<Box<dyn Foo5<C>>>;
    }

    #[persian_rug::constraints(context = C)]
    impl<C> persian_rug::Contextual for Box<dyn Bar5<C>> {
        type Context = C;
    }

    #[persian_rug::constraints(context = C, access(Box<dyn Foo5<C>>))]
    #[persian_rug::contextual(C)]
    struct B5<C> {
        foo: persian_rug::Proxy<Box<dyn Foo5<C>>>,
    }

    #[persian_rug::constraints(context = C, access(Box<dyn Foo5<C>>))]
    impl<C> Bar5<C> for B5<C> {
        fn read_a(&self) -> i32 {
            2
        }
        fn read_foo(&self) -> persian_rug::Proxy<Box<dyn Foo5<C>>> {
            self.foo
        }
    }

    #[persian_rug::constraints(context = C, access(Box<dyn Foo5<C>>, Box<dyn Bar5<C>>))]
    trait Baz5<C> {
        fn read_a(&self) -> i32;
        fn read_bar(&self) -> persian_rug::Proxy<Box<dyn Bar5<C>>>;
    }

    #[persian_rug::constraints(context = C)]
    impl<C> persian_rug::Contextual for Box<dyn Baz5<C>> {
        type Context = C;
    }

    #[persian_rug::constraints(context = C, access(Box<dyn Foo5<C>>, Box<dyn Bar5<C>>))]
    #[persian_rug::contextual(C)]
    struct Z5<C> {
        bar: persian_rug::Proxy<Box<dyn Bar5<C>>>,
    }

    #[persian_rug::constraints(context = C, access(Box<dyn Foo5<C>>, Box<dyn Bar5<C>>))]
    impl<C> Baz5<C> for Z5<C> {
        fn read_a(&self) -> i32 {
            3
        }
        fn read_bar(&self) -> persian_rug::Proxy<Box<dyn Bar5<C>>> {
            self.bar
        }
    }

    #[persian_rug::persian_rug]
    pub struct State5a {
        #[table]
        foo: Box<dyn Foo5<State5a>>,
    }

    #[persian_rug::persian_rug]
    pub struct State5b {
        #[table]
        foo: Box<dyn Foo5<State5b>>,
        #[table]
        bar: Box<dyn Bar5<State5b>>,
    }

    #[persian_rug::persian_rug]
    pub struct State5c {
        #[table]
        foo: Box<dyn Foo5<State5c>>,
        #[table]
        bar: Box<dyn Bar5<State5c>>,
        #[table]
        baz: Box<dyn Baz5<State5c>>,
    }

    #[test]
    fn test_traits() {
        use persian_rug::Context;

        let mut s5a = State5a {
            foo: Default::default(),
        };

        let _f1 = s5a.add(Box::new(F5 {
            _marker: Default::default(),
        }));

        let mut s5b = State5b {
            foo: Default::default(),
            bar: Default::default(),
        };

        let f1 = s5b.add::<Box<dyn Foo5<State5b>>>(Box::new(F5 {
            _marker: Default::default(),
        }));
        let _b1 = s5b.add::<Box<dyn Bar5<State5b>>>(Box::new(B5 { foo: f1 }));

        let mut s5c = State5c {
            foo: Default::default(),
            bar: Default::default(),
            baz: Default::default(),
        };

        let f1 = s5c.add::<Box<dyn Foo5<State5c>>>(Box::new(F5 {
            _marker: Default::default(),
        }));
        let b1 = s5c.add::<Box<dyn Bar5<State5c>>>(Box::new(B5 { foo: f1 }));
        let _z1 = s5c.add::<Box<dyn Baz5<State5c>>>(Box::new(Z5 { bar: b1 }));
    }
}

mod mutator_tests {
    use super::*;

    use clone_replace::CloneReplace;
    use persian_rug::{Mutator, Proxy};
    use std::sync::{Mutex, RwLock};

    fn run_mutation_test<'b, B>(mut mutator: B) -> B
    where
        B: 'b + Mutator<Context = State>,
    {
        // add
        let f1 = mutator.add(Foo {
            _marker: Default::default(),
            a: 0,
        });
        let b1 = mutator.add(Bar { a: 1, foo: f1 });
        let z1 = mutator.add(Baz { a: 2, bar: b1 });

        // get
        assert_eq!(mutator.get(&f1).a, 0);
        assert_eq!(mutator.get(&b1).a, 1);
        assert_eq!(mutator.get(&b1).foo, f1);
        assert_eq!(mutator.get(&z1).a, 2);
        assert_eq!(mutator.get(&z1).bar, b1);

        // get_mut
        mutator.get_mut(&f1).a = 2;
        mutator.get_mut(&b1).a = 3;
        mutator.get_mut(&z1).a = 4;
        assert_eq!(mutator.get(&f1).a, 2);
        assert_eq!(mutator.get(&b1).a, 3);
        assert_eq!(mutator.get(&b1).foo, f1);
        assert_eq!(mutator.get(&z1).a, 4);
        assert_eq!(mutator.get(&z1).bar, b1);

        // get_iter
        let foos = mutator.get_iter().collect::<Vec<&Foo<State>>>();
        assert_eq!(foos.len(), 1);
        assert_eq!(foos[0] as *const _, mutator.get(&f1) as *const _);
        let bars = mutator.get_iter().collect::<Vec<&Bar<State>>>();
        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0] as *const _, mutator.get(&b1) as *const _);
        let bazs = mutator.get_iter().collect::<Vec<&Baz<State>>>();
        assert_eq!(bazs.len(), 1);
        assert_eq!(bazs[0] as *const _, mutator.get(&z1) as *const _);

        // get_iter_mut
        for foo in mutator.get_iter_mut::<Foo<State>>() {
            foo.a = 5;
        }
        for bar in mutator.get_iter_mut::<Bar<State>>() {
            bar.a = 6;
        }
        for baz in mutator.get_iter_mut::<Baz<State>>() {
            baz.a = 7;
        }
        assert_eq!(mutator.get(&f1).a, 5);
        assert_eq!(mutator.get(&b1).a, 6);
        assert_eq!(mutator.get(&b1).foo, f1);
        assert_eq!(mutator.get(&z1).a, 7);
        assert_eq!(mutator.get(&z1).bar, b1);

        // get_proxy_iter
        let foos = mutator
            .get_proxy_iter()
            .copied()
            .collect::<Vec<Proxy<Foo<State>>>>();
        assert_eq!(foos.len(), 1);
        assert_eq!(foos[0], f1);
        let bars = mutator
            .get_proxy_iter()
            .copied()
            .collect::<Vec<Proxy<Bar<State>>>>();
        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0], b1);
        let bazs = mutator
            .get_proxy_iter()
            .copied()
            .collect::<Vec<Proxy<Baz<State>>>>();
        assert_eq!(bazs.len(), 1);
        assert_eq!(bazs[0], z1);

        mutator
    }

    #[test]
    fn test_mut_ref() {
        let mut s = State {
            foo: Default::default(),
            bar: Default::default(),
            baz: Default::default(),
        };

        run_mutation_test(&mut s);
    }

    #[test]
    fn test_mutex_guard() {
        let s = Mutex::new(State {
            foo: Default::default(),
            bar: Default::default(),
            baz: Default::default(),
        });

        let _ = run_mutation_test(s.lock().unwrap());
    }

    #[test]
    fn test_rw_lock_write_guard() {
        let s = RwLock::new(State {
            foo: Default::default(),
            bar: Default::default(),
            baz: Default::default(),
        });

        let _ = run_mutation_test(s.write().unwrap());
    }

    #[test]
    fn test_mutate_guard() {
        let s = CloneReplace::new(State {
            foo: Default::default(),
            bar: Default::default(),
            baz: Default::default(),
        });

        let _ = run_mutation_test(s.mutate());
    }
}
