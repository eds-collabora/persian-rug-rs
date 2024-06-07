#![cfg(test)]
#![allow(dead_code)]

use std::collections::{BTreeSet, HashSet};

use persian_rug::{contextual, persian_rug, Context, Proxy, ProxySet};
use rand::Rng;

#[contextual(Bar)]
struct Foo {
    ix: u64,
}

#[persian_rug]
struct Bar(#[table] Foo);

#[test]
fn test_basic() {
    let mut bar = Bar(Default::default());

    let f = (0..16).map(|ix| bar.add(Foo { ix })).collect::<Vec<_>>();

    for i in 0..(2 << 16) {
        let mut ps = ProxySet::new();
        for j in 0..16 {
            if (i & (1 << j)) != 0 {
                ps.insert(f[j]);
            }
        }

        for j in 0..16 {
            assert_eq!(i & (1 << j) != 0, ps.contains(&f[j]));
        }
    }
}

#[test]
fn test_large() {
    let mut bar = Bar(Default::default());

    let f = (0..512).map(|ix| bar.add(Foo { ix })).collect::<Vec<_>>();
    let g = (0..512).step_by(32).map(|ix| f[ix]).collect::<Vec<_>>();

    for i in 0..(2 << 16) {
        let mut ps = ProxySet::new();
        for j in 0..16 {
            if (i & (1 << j)) != 0 {
                ps.insert(g[j]);
            }
        }

        for j in 0..16 {
            assert_eq!(i & (1 << j) != 0, ps.contains(&g[j]));
        }
    }
}

#[test]
fn test_random() {
    let mut bar = Bar(Default::default());

    let f = (0..65536).map(|ix| bar.add(Foo { ix })).collect::<Vec<_>>();

    let mut rng = rand::thread_rng();
    for _ in 0..250 {
        let mut hs = HashSet::new();
        let mut ps = ProxySet::new();

        let n = rng.gen_range(0..30000);
        for _ in 0..n {
            let item = f[rng.gen_range(0..f.len())];
            hs.insert(item);
            ps.insert(item);
        }

        for item in f.iter() {
            assert_eq!(hs.contains(&item), ps.contains(&item));
        }
    }
}

#[test]
fn test_iterator() {
    let mut bar = Bar(Default::default());

    let f = (0..65536).map(|ix| bar.add(Foo { ix })).collect::<Vec<_>>();

    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let mut hs = BTreeSet::new();
        let mut ps = ProxySet::new();

        let n = rng.gen_range(0..30000);
        for _ in 0..n {
            let item = f[rng.gen_range(0..f.len())];
            hs.insert(item);
            ps.insert(item);
        }

        for item in hs.iter() {
            assert!(ps.contains(&item));
        }

        for item in ps.iter() {
            assert!(hs.contains(&item));
            hs.remove(&item);
        }
        assert!(hs.is_empty());
    }
}
