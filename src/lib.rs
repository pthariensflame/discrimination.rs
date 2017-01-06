//! Generic worst-case-linear-time sorting and partitioning algorithms based on
//! [discrim][1][inators][2].  Partially a port of the [Haskell library of the
//! same name][3].
//!
//! [1]:http://www.diku.dk/hjemmesider/ansatte/henglein/papers/henglein2011a.pdf
//! [2]:http://www.diku.dk/hjemmesider/ansatte/henglein/papers/henglein2011c.pdf
//! [3]:https://github.com/ekmett/discrimination

pub extern crate either;

pub mod discriminator;

pub mod split_either;

pub mod prelude {
    #[doc(no_inline)]
    pub use discriminator::{Discriminator, Natural, Trivial, U16, U8};
    #[doc(no_inline)]
    pub use either::Either;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
