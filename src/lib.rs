//! Generic worst-case-linear-time sorting and partitioning algorithms based on
//! [discrim][1][inators][2].  Partially a port of the [Haskell library of the
//! same name][3].
//!
//! [1]:http://www.diku.dk/hjemmesider/ansatte/henglein/papers/henglein2011a.pdf
//! [2]:http://www.diku.dk/hjemmesider/ansatte/henglein/papers/henglein2011c.pdf
//! [3]:https://github.com/ekmett/discrimination

pub extern crate either;

pub mod is_pair {
    pub trait HasFst {
        type Fst;
        fn fst(self) -> Self::Fst where Self: Sized;
        fn fst_ref(&self) -> &Self::Fst;
        fn fst_mut(&mut self) -> &mut Self::Fst;
    }

    impl<A, B> HasFst for (A, B) {
        type Fst = A;

        fn fst(self) -> Self::Fst {
            self.0
        }

        fn fst_ref(&self) -> &Self::Fst {
            &self.0
        }

        fn fst_mut(&mut self) -> &mut Self::Fst {
            &mut self.0
        }
    }

    pub trait HasSnd {
        type Snd;
        fn snd(self) -> Self::Snd where Self: Sized;
        fn snd_ref(&self) -> &Self::Snd;
        fn snd_mut(&mut self) -> &mut Self::Snd;
    }

    impl<A, B> HasSnd for (A, B) {
        type Snd = B;

        fn snd(self) -> Self::Snd {
            self.1
        }

        fn snd_ref(&self) -> &Self::Snd {
            &self.1
        }

        fn snd_mut(&mut self) -> &mut Self::Snd {
            &mut self.1
        }
    }

    pub trait IsPair: HasFst + HasSnd {
        type Flipped: ?Sized + IsPair<Fst = Self::Snd, Snd = Self::Fst, Flipped = Self>;
        fn into_pair(self) -> (Self::Fst, Self::Snd) where Self: Sized;
        fn from_pair(pair: (Self::Fst, Self::Snd)) -> Self where Self: Sized;
        fn as_pair_ref(&self) -> &(Self::Fst, Self::Snd);
        fn as_pair_mut(&mut self) -> &mut (Self::Fst, Self::Snd);
        fn flip(self) -> Self::Flipped
            where Self: Sized,
                  Self::Flipped: Sized;
    }

    impl<A, B> IsPair for (A, B) {
        type Flipped = (B, A);

        fn into_pair(self) -> (Self::Fst, Self::Snd) {
            self
        }

        fn from_pair(pair: (Self::Fst, Self::Snd)) -> Self {
            pair
        }

        fn as_pair_ref(&self) -> &(Self::Fst, Self::Snd) {
            self
        }

        fn as_pair_mut(&mut self) -> &mut (Self::Fst, Self::Snd) {
            self
        }

        fn flip(self) -> Self::Flipped {
            (self.1, self.0)
        }
    }
}

pub mod discrimination;

pub mod split_either;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
