#![cfg(test)]
#![allow(dead_code)]

use std::any::Any;

#[persian_rug::contextual(C)]
struct Foo<C: persian_rug::Context> {
    _marker: core::marker::PhantomData<C>,
    a: i32,
}

#[persian_rug::contextual(C)]
struct Bar<C: persian_rug::Context> {
    a: i32,
    foo: persian_rug::Proxy<Foo<C>>,
}

#[persian_rug::contextual(C)]
struct Baz<C: persian_rug::Context> {
    a: i32,
    bar: persian_rug::Proxy<Bar<C>>,
}

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

        let f1 = t.push(Foo { _marker: Default::default(), a: 0 });
        let f2 = t.push(Foo { _marker: Default::default(), a: 1 });
        let f3 = t.push(Foo { _marker: Default::default(), a: 2 });

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

        let foos = t.iter_proxies().collect::<Vec<&persian_rug::Proxy<Foo<State2>>>>();
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
            persian_rug::Table::new()
        );

        let f1 = s.add( Foo2 { a: 0 } );
        let f2 = s.add( Foo2 { a: 0 } );
        let f3 = s.add( Foo2 { a: 0 } );
        let f4 = s.add( Foo2 { a: 0 } );

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
            persian_rug::Table::new()
        );

        let f1 = s.add( Foo2 { a: 0 } );
        let f2 = s.add( Foo2 { a: 0 } );
        let f3 = s.add( Foo2 { a: 0 } );
        let f4 = s.add( Foo2 { a: 0 } );

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
            persian_rug::Table::new()
        );

        let f1 = s.add( Foo2 { a: 0 } );
        let f2 = s.add( Foo2 { a: 0 } );
        let f3 = s.add( Foo2 { a: 0 } );
        let f4 = s.add( Foo2 { a: 0 } );

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
            persian_rug::Table::new()
        );

        let f1 = s.add( Foo2 { a: 0 } );
        let f2 = s.add( Foo2 { a: 0 } );
        let f3 = s.add( Foo2 { a: 0 } );
        let f4 = s.add( Foo2 { a: 0 } );

        assert_eq!(
            &format!("{:?}", f1),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 0 }");
        assert_eq!(
            &format!("{:?}", f2),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 1 }");
        assert_eq!(
            &format!("{:?}", f3),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 2 }");
        assert_eq!(
            &format!("{:?}", f4),
            "persian_rug::Proxy<test_suite::Foo2> { handle: 3 }");
    }
}
