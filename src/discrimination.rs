use is_pair::{HasSnd, IsPair};
use std::{iter, vec};

pub trait Discriminator<K> {
    // fn discriminate<'a, V, I>(&'a self, pairs: I)
    //     -> Discriminate<'a, V, I::IntoIter>
    //     where I: IntoIterator,
    //           I::Item: IsPair<Fst = K, Snd = V>,
    //           I::IntoIter: DoubleEndedIterator + 'a;
    // fn discriminate_unstable<'a, V, I>(&'a self, pairs: I)
    //     -> DiscriminateUnstable<'a, V, I::IntoIter>
    //     where I: IntoIterator,
    //           I::Item: IsPair<Fst = K, Snd = V>,
    //           I::IntoIter: DoubleEndedIterator + 'a;
    fn discriminate_sorted<'a, V: 'a, I>(&'a self,
                                         pairs: I)
                                         -> DiscriminateSorted<'a, V, I::IntoIter>
        where I: IntoIterator,
              I::Item: IsPair<Fst = K, Snd = V>,
              I::IntoIter: DoubleEndedIterator + 'a;
}

pub struct DiscriminateSorted<'a, V: 'a, I>(DiscriminateSortedImpl<'a, V, I>)
    where I: DoubleEndedIterator + 'a,
          I::Item: HasSnd<Snd = V>;

enum DiscriminateSortedImpl<'a, V: 'a, I>
    where I: DoubleEndedIterator + 'a,
          I::Item: HasSnd<Snd = V>
{
    One(Option<V>),
    Trivial(Option<I>),
    Natural(vec::IntoIter<Vec<V>>),
    Invert(Box<DiscriminateSorted<'a, V, I>>),
    Map(Box<DiscriminateSorted<'a, V, Box<DoubleEndedIterator<Item = ((), V)> + 'a>>>),
}

impl<'a, V: 'a, I> Iterator for DiscriminateSorted<'a, V, I>
    where I: DoubleEndedIterator + 'a,
          I::Item: HasSnd<Snd = V>
{
    type Item = DiscriminateSortedGroup<'a, V, I>;

    fn next(&mut self) -> Option<DiscriminateSortedGroup<'a, V, I>> {
        match self.0 {
            DiscriminateSortedImpl::One(ref mut v_opt) => {
                v_opt.take()
                     .map(|v| DiscriminateSortedGroup(DiscriminateSortedGroupImpl::One(Some(v))))
            }
            DiscriminateSortedImpl::Trivial(ref mut pairs_opt) => {
                pairs_opt.take().map(|pairs| {
                    DiscriminateSortedGroup(DiscriminateSortedGroupImpl::Trivial(pairs))
                })
            }
            DiscriminateSortedImpl::Natural(ref mut inner) => {
                loop {
                    match inner.next() {
                        Some(vs) if !vs.is_empty() => {
                            return Some(DiscriminateSortedGroup(
                                DiscriminateSortedGroupImpl::Natural(vs.into_iter())));
                        }
                        Some(_) => continue,
                        None => return None,
                    }
                }
            }
            DiscriminateSortedImpl::Invert(ref mut inner) => inner.next_back(),
            DiscriminateSortedImpl::Map(ref mut inner) => {
                inner.next().map(|group| {
                    DiscriminateSortedGroup(DiscriminateSortedGroupImpl::Map(Box::new(group)))
                })
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            DiscriminateSortedImpl::One(ref v_opt) => {
                let n = v_opt.is_some() as usize;
                (n, Some(n))
            }
            DiscriminateSortedImpl::Trivial(ref pairs_opt) => {
                let n = pairs_opt.is_some() as usize;
                (n, Some(n))
            }
            DiscriminateSortedImpl::Natural(ref inner) => (0, inner.size_hint().1),
            DiscriminateSortedImpl::Invert(ref inner) => inner.size_hint(),
            DiscriminateSortedImpl::Map(ref inner) => inner.size_hint(),
        }
    }

    fn fold<B, F>(self, init: B, f: F) -> B
        where F: FnMut(B, DiscriminateSortedGroup<'a, V, I>) -> B
    {
        match self.0 {
            DiscriminateSortedImpl::One(v_opt) => {
                match v_opt {
                    None => init,
                    Some(v) => {
                        f(init,
                          DiscriminateSortedGroup(DiscriminateSortedGroupImpl::One(Some(v))))
                    }
                }
            }
            DiscriminateSortedImpl::Trivial(pairs_opt) => {
                match pairs_opt {
                    None => init,
                    Some(pairs) => {
                        f(init,
                          DiscriminateSortedGroup(DiscriminateSortedGroupImpl::Trivial(pairs)))
                    }
                }
            }
            DiscriminateSortedImpl::Natural(inner) => {
                inner.filter(|vs| !vs.is_empty())
                     .map(|vs| {
                         DiscriminateSortedGroup(
                    DiscriminateSortedGroupImpl::Natural(vs.into_iter()))
                     })
                     .fold(init, f)
            }
            DiscriminateSortedImpl::Invert(ref mut inner) => inner.rev().fold(init, f),
            DiscriminateSortedImpl::Map(ref mut inner) => {
                inner.map(|group| {
                         DiscriminateSortedGroup(DiscriminateSortedGroupImpl::Map(Box::new(group)))
                     })
                     .fold(init, f)
            }
        }
    }
}

impl<'a, V: 'a, I> DoubleEndedIterator for DiscriminateSorted<'a, V, I>
    where I: DoubleEndedIterator + 'a,
          I::Item: HasSnd<Snd = V>
{
    fn next_back(&mut self) -> Option<DiscriminateSortedGroup<'a, V, I>> {
        match self.0 {
            DiscriminateSortedImpl::One(ref mut v_opt) => {
                v_opt.take()
                     .map(|v| DiscriminateSortedGroup(DiscriminateSortedGroupImpl::One(Some(v))))
            }
            DiscriminateSortedImpl::Trivial(ref mut pairs_opt) => {
                pairs_opt.take().map(|pairs| {
                    DiscriminateSortedGroup(DiscriminateSortedGroupImpl::Trivial(pairs))
                })
            }
            DiscriminateSortedImpl::Natural(ref mut inner) => {
                loop {
                    match inner.next_back() {
                        Some(vs) if !vs.is_empty() => {
                            return Some(DiscriminateSortedGroup(
                                DiscriminateSortedGroupImpl::Natural(vs.into_iter())));
                        }
                        Some(_) => continue,
                        None => return None,
                    }
                }
            }
            DiscriminateSortedImpl::Invert(ref mut inner) => inner.next(),
            DiscriminateSortedImpl::Map(ref mut inner) => {
                inner.next_back().map(|group| {
                    DiscriminateSortedGroup(DiscriminateSortedGroupImpl::Map(Box::new(group)))
                })
            }
        }
    }
}

pub struct DiscriminateSortedGroup<'a, V: 'a, I>(DiscriminateSortedGroupImpl<'a, V, I>)
    where I: DoubleEndedIterator + 'a,
          I::Item: HasSnd<Snd = V>;

enum DiscriminateSortedGroupImpl<'a, V: 'a, I>
    where I: DoubleEndedIterator + 'a,
          I::Item: HasSnd<Snd = V>
{
    One(Option<V>),
    Trivial(I),
    Natural(vec::IntoIter<V>),
    Map(Box<DiscriminateSortedGroup<'a, V, Box<DoubleEndedIterator<Item = ((), V)> + 'a>>>),
}

impl<'a, V, I> Iterator for DiscriminateSortedGroup<'a, V, I>
    where I: DoubleEndedIterator,
          I::Item: HasSnd<Snd = V> + 'a
{
    type Item = V;

    fn next(&mut self) -> Option<V> {
        match self.0 {
            DiscriminateSortedGroupImpl::One(ref mut v_opt) => v_opt.take(),
            DiscriminateSortedGroupImpl::Trivial(ref mut pairs) => pairs.next().map(|kv| kv.snd()),
            DiscriminateSortedGroupImpl::Natural(ref mut inner) => inner.next(),
            DiscriminateSortedGroupImpl::Map(ref mut inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            DiscriminateSortedGroupImpl::One(ref v_opt) => {
                let n = v_opt.is_some() as usize;
                (n, Some(n))
            }
            DiscriminateSortedGroupImpl::Trivial(ref pairs) => pairs.size_hint(),
            DiscriminateSortedGroupImpl::Natural(ref inner) => inner.size_hint(),
            DiscriminateSortedGroupImpl::Map(ref inner) => inner.size_hint(),
        }
    }

    fn fold<B, F>(self, init: B, f: F) -> B
        where F: FnMut(B, V) -> B
    {
        match self.0 {
            DiscriminateSortedGroupImpl::One(v_opt) => {
                match v_opt {
                    None => init,
                    Some(v) => f(init, v),
                }
            }
            DiscriminateSortedGroupImpl::Trivial(pairs) => pairs.map(|kv| kv.snd()).fold(init, f),
            DiscriminateSortedGroupImpl::Natural(inner) => inner.fold(init, f),
            DiscriminateSortedGroupImpl::Map(inner) => inner.fold(init, f),
        }
    }
}

impl<'a, V, I> DoubleEndedIterator for DiscriminateSortedGroup<'a, V, I>
    where I: DoubleEndedIterator,
          I::Item: HasSnd<Snd = V> + 'a
{
    fn next_back(&mut self) -> Option<V> {
        match self.0 {
            DiscriminateSortedGroupImpl::One(ref mut v_opt) => v_opt.take(),
            DiscriminateSortedGroupImpl::Trivial(ref mut pairs) => {
                pairs.next_back().map(|kv| kv.snd())
            }
            DiscriminateSortedGroupImpl::Natural(ref mut inner) => inner.next_back(),
            DiscriminateSortedGroupImpl::Map(ref mut inner) => inner.next_back(),
        }
    }
}

#[derive(Debug,Copy,Clone,Default)]
pub struct Trivial;

impl Trivial {
    pub fn new() -> Trivial {
        Trivial
    }
}

impl<K> Discriminator<K> for Trivial {
    fn discriminate_sorted<'a, V, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, V, I::IntoIter>
        where I: IntoIterator,
              I::Item: IsPair<Fst = K, Snd = V>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(
                pairs.next().map(|kv| kv.into_pair().1)));
        }

        DiscriminateSorted(DiscriminateSortedImpl::Trivial(Some(pairs)))
    }
}

#[derive(Debug,Copy,Clone)]
pub struct Natural {
    limit: usize,
}

impl From<usize> for Natural {
    fn from(n: usize) -> Natural {
        Natural::new(n)
    }
}

impl From<Natural> for usize {
    fn from(desc: Natural) -> usize {
        desc.limit
    }
}

impl AsRef<usize> for Natural {
    fn as_ref(&self) -> &usize {
        &self.limit
    }
}

impl AsMut<usize> for Natural {
    fn as_mut(&mut self) -> &mut usize {
        &mut self.limit
    }
}

impl Natural {
    pub fn new<N>(limit: N) -> Natural
        where N: Into<usize>
    {
        let limit = limit.into();
        debug_assert!(limit >= 2);
        Natural { limit: limit }
    }

    fn bdisc<V, F, I>(&self, update: F, pairs: I) -> Vec<Vec<V>>
        where F: FnMut(&mut Vec<V>, V),
              I: DoubleEndedIterator,
              I::Item: IsPair<Fst = usize, Snd = V>
    {
        // initialize buckets
        let mut buckets = Vec::with_capacity(self.limit);
        for k in 0..self.limit {
            buckets.push(Vec::new());
        }

        // fill buckets
        for kv in pairs {
            let (k, v) = kv.into_pair();
            update(&mut buckets[k], v);
        }

        // return results
        return buckets;
    }

    unsafe fn bdisc_unchecked<V, F, I>(&self, update: F, pairs: I) -> Vec<Vec<V>>
        where F: FnMut(&mut Vec<V>, V),
              I: DoubleEndedIterator,
              I::Item: IsPair<Fst = usize, Snd = V>
    {
        // initialize buckets
        let mut buckets = Vec::with_capacity(self.limit);
        for k in 0..self.limit {
            buckets.push(Vec::new());
        }

        // fill buckets
        for kv in pairs {
            let (k, v) = kv.into_pair();
            update(buckets.get_unchecked_mut(k), v);
        }

        // return results
        return buckets;
    }
}

impl Discriminator<usize> for Natural {
    fn discriminate_sorted<'a, V, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, V, I::IntoIter>
        where I: IntoIterator,
              I::Item: IsPair<Fst = usize, Snd = V>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(
                pairs.next().map(|kv| kv.into_pair().1)));
        }

        DiscriminateSorted(DiscriminateSortedImpl::Natural(self.bdisc(Vec::push, pairs)
                                                               .into_iter()))
    }
}

#[derive(Debug,Copy,Clone,Default)]
pub struct Invert<D: ?Sized>(D);

impl<D> Invert<D> {
    pub fn new<I>(inner: I) -> Invert<D>
        where I: Into<D>
    {
        Invert(inner.into())
    }
}

impl<D> From<D> for Invert<D> {
    fn from(inner: D) -> Invert<D> {
        Invert::new(inner)
    }
}

impl<D: ?Sized> AsRef<D> for Invert<D> {
    fn as_ref(&self) -> &D {
        &self.0
    }
}

impl<D: ?Sized> AsMut<D> for Invert<D> {
    fn as_mut(&mut self) -> &mut D {
        &mut self.0
    }
}

impl<K, D: ?Sized> Discriminator<K> for Invert<D>
    where D: Discriminator<K>
{
    fn discriminate_sorted<'a, V, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, V, I::IntoIter>
        where I: IntoIterator,
              I::Item: IsPair<Fst = K, Snd = V>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(
                pairs.next_back().map(|kv| kv.into_pair().1)));
        }

        DiscriminateSorted(DiscriminateSortedImpl::Invert(
            Box::new(self.0.discriminate_sorted(pairs))))
    }
}



#[derive(Debug,Copy,Clone,Default)]
pub struct Map<F, D: ?Sized>(F, D);

impl<F, D> Map<F, D> {
    pub fn new<I, G>(inner: I, f: G) -> Map<F, D>
        where I: Into<D>,
              G: Into<F>
    {
        Map(f.into(), inner.into())
    }
}

impl<F, D: ?Sized> AsRef<D> for Map<F, D> {
    fn as_ref(&self) -> &D {
        &self.1
    }
}

impl<F, D: ?Sized> AsMut<D> for Map<F, D> {
    fn as_mut(&mut self) -> &mut D {
        &mut self.1
    }
}

impl<K, J, F, D: ?Sized> Discriminator<K> for Map<F, D>
    where D: Discriminator<J>,
          F: Fn(K) -> J
{
    fn discriminate_sorted<'a, V, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, V, I::IntoIter>
        where I: IntoIterator,
              I::Item: IsPair<Fst = K, Snd = V>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(
                pairs.next_back().map(|kv| kv.into_pair().1)));
        }

        let Map(ref f, ref inner) = *self;
        DiscriminateSorted(DiscriminateSortedImpl::Map(
            Box::new(inner.discriminate_sorted(pairs.map(|kv| {
                let (k, v) = kv.into_pair();
                (f(k), v)
            })))))
    }
}
