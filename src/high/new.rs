use std::marker::PhantomData;

use high::{CType, Type};
use middle::{self, FfiAbi};

pub struct Cif<F> {
    untyped: middle::Cif,
    _marker: PhantomData<F>,
}

pub type Cif1<A, R> = Cif<fn(A) -> R>;
pub type Cif2<A, B, R> = Cif<fn(A, B) -> R>;

pub struct Closure<'a, F> {
    untyped: middle::Closure<'a>,
    _marker: PhantomData<F>,
}

pub type Closure1<'a, A, R> = Closure<'a, fn(A) -> R>;
pub type Closure2<'a, A, B, R> = Closure<'a, fn(A, B) -> R>;

impl<F> Cif<F> {
    pub fn set_abi(&mut self, abi: FfiAbi) {
        self.untyped.set_abi(abi);
    }
}

impl<A, R> Cif1<A, R> {
    pub fn new(a: Type<A>, r: Type<R>) -> Self {
        let cif = middle::Cif::new(
            vec![ a.into_middle() ].into_iter(),
            r.into_middle());
        Cif {
            untyped: cif,
            _marker: PhantomData,
        }
    }
}

impl<A: CType, R: CType> Cif1<A, R> {
    pub fn reify() -> Self {
        Self::new(A::reify(), R::reify())
    }
}

impl<A, B, R> Cif2<A, B, R> {
    pub fn new(a: Type<A>, b: Type<B>, r: Type<R>) -> Self {
        let cif = middle::Cif::new(
            vec![ a.into_middle(), b.into_middle() ].into_iter(),
            r.into_middle());
        Cif {
            untyped: cif,
            _marker: PhantomData,
        }
    }
}

impl<A: CType, B: CType, R: CType> Cif2<A, B, R> {
    pub fn reify() -> Self {
        Self::new(A::reify(), B::reify(), R::reify())
    }
}

#[test]
fn test_cif() {
    use super::*;

    let cif1: Cif1<u32, u64> = Cif1::reify();
}
