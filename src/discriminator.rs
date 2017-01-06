use std::vec;

pub trait Discriminator<'a, K: 'a> {
    // fn discriminate<V, I>(&'a self, pairs: I)
    //     -> Discriminate<'a, K, V>
    //     where I: IntoIterator,
    //           I::Item: Into<(K, V)>,
    //           I::IntoIter: DoubleEndedIterator + 'a;
    // fn discriminate_unstable<V, I>(&'a self, pairs: I)
    //     -> DiscriminateUnstable<'a, K, V>
    //     where I: IntoIterator,
    //           I::Item: Into<(K, V)>,
    //           I::IntoIter: DoubleEndedIterator + 'a;
    fn discriminate_sorted<V: 'a, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, K, V>
        where I: IntoIterator,
              I::Item: Into<(K, V)>,
              I::IntoIter: DoubleEndedIterator + 'a;

    fn by_ref(&'a self) -> &'a Self {
        self
    }

    fn invert(self) -> Invert<Self>
        where Self: Sized
    {
        Invert::new(self)
    }

    fn map_key<J: 'a, F>(self, f: F) -> Map<F, Self>
        where Self: Sized,
              F: Fn(J) -> K
    {
        Map::new(f, self)
    }
}

pub struct DiscriminateSorted<'a, K: 'a, V: 'a>(DiscriminateSortedImpl<'a, K, V>);

enum DiscriminateSortedImpl<'a, K: 'a, V: 'a> {
    One(Option<V>),
    Trivial(Option<Box<DoubleEndedIterator<Item = (K, V)> + 'a>>),
    Natural(vec::IntoIter<Vec<V>>),
    Invert(Box<DiscriminateSorted<'a, K, V>>),
    Opaque(Box<DoubleEndedIterator<Item = DiscriminateSortedGroup<'a, K, V>> + 'a>),
}

impl<'a, K, V> Iterator for DiscriminateSorted<'a, K, V> {
    type Item = DiscriminateSortedGroup<'a, K, V>;

    fn next(&mut self) -> Option<DiscriminateSortedGroup<'a, K, V>> {
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
                        Some(vs) => {
                            if !vs.is_empty() {
                                return Some(DiscriminateSortedGroup(
                                DiscriminateSortedGroupImpl::Natural(vs.into_iter())));
                            } else {
                                continue;
                            }
                        }
                        None => return None,
                    }
                }
            }
            DiscriminateSortedImpl::Invert(ref mut inner) => inner.next_back(),
            DiscriminateSortedImpl::Opaque(ref mut inner) => inner.next(),
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
            DiscriminateSortedImpl::Opaque(ref inner) => inner.size_hint(),
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
        where F: FnMut(B, DiscriminateSortedGroup<'a, K, V>) -> B
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
            DiscriminateSortedImpl::Invert(inner) => inner.rev().fold(init, f),
            DiscriminateSortedImpl::Opaque(inner) => inner.fold(init, f),
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for DiscriminateSorted<'a, K, V> {
    fn next_back(&mut self) -> Option<DiscriminateSortedGroup<'a, K, V>> {
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
                        Some(vs) => {
                            if !vs.is_empty() {
                                return Some(DiscriminateSortedGroup(
                                DiscriminateSortedGroupImpl::Natural(vs.into_iter())));
                            } else {
                                continue;
                            }
                        }
                        None => return None,
                    }
                }
            }
            DiscriminateSortedImpl::Invert(ref mut inner) => inner.next(),
            DiscriminateSortedImpl::Opaque(ref mut inner) => inner.next_back(),
        }
    }
}

pub struct DiscriminateSortedGroup<'a, K: 'a, V: 'a>(DiscriminateSortedGroupImpl<'a, K, V>);

enum DiscriminateSortedGroupImpl<'a, K: 'a, V: 'a> {
    One(Option<V>),
    Trivial(Box<DoubleEndedIterator<Item = (K, V)> + 'a>),
    Natural(vec::IntoIter<V>),
    Opaque(Box<DoubleEndedIterator<Item = V> + 'a>),
}

impl<'a, K, V> Iterator for DiscriminateSortedGroup<'a, K, V> {
    type Item = V;

    fn next(&mut self) -> Option<V> {
        match self.0 {
            DiscriminateSortedGroupImpl::One(ref mut v_opt) => v_opt.take(),
            DiscriminateSortedGroupImpl::Trivial(ref mut pairs) => pairs.next().map(|kv| kv.1),
            DiscriminateSortedGroupImpl::Natural(ref mut inner) => inner.next(),
            DiscriminateSortedGroupImpl::Opaque(ref mut inner) => inner.next(),
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
            DiscriminateSortedGroupImpl::Opaque(ref inner) => inner.size_hint(),
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
        where F: FnMut(B, V) -> B
    {
        match self.0 {
            DiscriminateSortedGroupImpl::One(v_opt) => {
                match v_opt {
                    None => init,
                    Some(v) => f(init, v),
                }
            }
            DiscriminateSortedGroupImpl::Trivial(pairs) => pairs.map(|kv| kv.1).fold(init, f),
            DiscriminateSortedGroupImpl::Natural(inner) => inner.fold(init, f),
            DiscriminateSortedGroupImpl::Opaque(inner) => inner.fold(init, f),
        }
    }
}

impl<'a, K, V> DoubleEndedIterator for DiscriminateSortedGroup<'a, K, V> {
    fn next_back(&mut self) -> Option<V> {
        match self.0 {
            DiscriminateSortedGroupImpl::One(ref mut v_opt) => v_opt.take(),
            DiscriminateSortedGroupImpl::Trivial(ref mut pairs) => pairs.next_back().map(|kv| kv.1),
            DiscriminateSortedGroupImpl::Natural(ref mut inner) => inner.next_back(),
            DiscriminateSortedGroupImpl::Opaque(ref mut inner) => inner.next_back(),
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

impl<'a, K: 'a> Discriminator<'a, K> for Trivial {
    fn discriminate_sorted<V: 'a, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, K, V>
        where I: IntoIterator,
              I::Item: Into<(K, V)>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(pairs.next()
                                                                       .map(|kv| kv.into().1)));
        }

        DiscriminateSorted(DiscriminateSortedImpl::Trivial(
            Some(Box::new(pairs.map(|kv| kv.into())))))
    }
}

#[derive(Debug,Copy,Clone)]
pub struct Natural {
    limit: usize,
    is_unchecked: bool,
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
        Natural { limit: limit,
                  is_unchecked: false, }
    }

    pub unsafe fn new_unchecked<N>(limit: N) -> Natural
        where N: Into<usize>
    {
        let limit = limit.into();
        debug_assert!(limit >= 2);
        Natural { limit: limit,
                  is_unchecked: true, }
    }

    fn bdisc<V, F, I>(&self, mut update: F, pairs: I) -> Vec<Vec<V>>
        where F: FnMut(&mut Vec<V>, V),
              I: DoubleEndedIterator,
              I::Item: Into<(usize, V)>
    {
        // initialize buckets
        let mut buckets = Vec::with_capacity(self.limit);
        for _ in 0..self.limit {
            buckets.push(Vec::new());
        }

        // fill buckets
        for kv in pairs {
            let (k, v) = kv.into();
            update(&mut buckets[k], v);
        }

        // return results
        return buckets;
    }

    unsafe fn bdisc_unchecked<V, F, I>(&self, mut update: F, pairs: I) -> Vec<Vec<V>>
        where F: FnMut(&mut Vec<V>, V),
              I: DoubleEndedIterator,
              I::Item: Into<(usize, V)>
    {
        // initialize buckets
        let mut buckets = Vec::with_capacity(self.limit);
        for _ in 0..self.limit {
            buckets.push(Vec::new());
        }

        // fill buckets
        for kv in pairs {
            let (k, v) = kv.into();
            update(buckets.get_unchecked_mut(k), v);
        }

        // return results
        return buckets;
    }
}

impl<'a> Discriminator<'a, usize> for Natural {
    fn discriminate_sorted<V: 'a, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, usize, V>
        where I: IntoIterator,
              I::Item: Into<(usize, V)>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(pairs.next()
                                                                       .map(|kv| kv.into().1)));
        }

        let res = if self.is_unchecked {
            self.bdisc(Vec::push, pairs)
        } else {
            unsafe { self.bdisc_unchecked(Vec::push, pairs) }
        };
        DiscriminateSorted(DiscriminateSortedImpl::Natural(res.into_iter()))
    }
}

#[derive(Debug,Copy,Clone,Default)]
pub struct U8;

impl U8 {
    pub fn new() -> Self {
        U8
    }
}

impl<'a> Discriminator<'a, u8> for U8 {
    fn discriminate_sorted<V: 'a, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, u8, V>
        where I: IntoIterator,
              I::Item: Into<(u8, V)>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        // when `const fn` support goes stable, this will be nicer
        fn conv(k: u8) -> usize {
            k as usize
        }
        const DESC: &'static Map<fn(u8) -> usize, Natural> =
            &Map(conv,
                 Natural { limit: ::std::u8::MAX as usize,
                           is_unchecked: true, });
        DESC.discriminate_sorted(pairs)
    }
}

#[derive(Debug,Copy,Clone,Default)]
pub struct U16;

impl U16 {
    pub fn new() -> Self {
        U16
    }
}

impl<'a> Discriminator<'a, u16> for U16 {
    fn discriminate_sorted<V: 'a, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, u16, V>
        where I: IntoIterator,
              I::Item: Into<(u16, V)>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        if cfg!(target_pointer_width = "8") {
            unimplemented!(); // TODO: `Either`-based solution using `U8`
        } else {
            // when `const fn` support goes stable, this will be nicer
            fn conv(k: u16) -> usize {
                k as usize
            }
            const DESC: &'static Map<fn(u16) -> usize, Natural> =
                &Map(conv,
                     Natural { limit: ::std::u16::MAX as usize,
                               is_unchecked: true, });
            DESC.discriminate_sorted(pairs)
        }
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

impl<'a, K: 'a, D: ?Sized> Discriminator<'a, K> for Invert<D>
    where D: Discriminator<'a, K>
{
    fn discriminate_sorted<V: 'a, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, K, V>
        where I: IntoIterator,
              I::Item: Into<(K, V)>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(pairs.next_back()
                                                                       .map(|kv| kv.into().1)));
        }

        DiscriminateSorted(DiscriminateSortedImpl::Invert(Box::new(
            self.0.discriminate_sorted(pairs))))
    }
}



#[derive(Debug,Copy,Clone,Default)]
pub struct Map<F, D: ?Sized>(F, D);

impl<F, D> Map<F, D> {
    pub fn new<I, G>(f: G, inner: I) -> Map<F, D>
        where G: Into<F>,
              I: Into<D>
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

impl<'a, K: 'a, J: 'a, F, D: ?Sized> Discriminator<'a, K> for Map<F, D>
    where D: Discriminator<'a, J>,
          F: Fn(K) -> J
{
    fn discriminate_sorted<V: 'a, I>(&'a self, pairs: I) -> DiscriminateSorted<'a, K, V>
        where I: IntoIterator,
              I::Item: Into<(K, V)>,
              I::IntoIter: DoubleEndedIterator + 'a
    {
        let mut pairs = pairs.into_iter();

        if pairs.size_hint().1.map(|n| n <= 1) == Some(true) {
            return DiscriminateSorted(DiscriminateSortedImpl::One(pairs.next_back()
                                                                       .map(|kv| kv.into().1)));
        }

        DiscriminateSorted(DiscriminateSortedImpl::Opaque(Box::new(
            self.1.discriminate_sorted(pairs.map(move |kv| {
                let (k, v) = kv.into();
                ((self.0)(k), v)
            })).map(|group| {
                DiscriminateSortedGroup(DiscriminateSortedGroupImpl::Opaque(Box::new(group)))
            }))))
    }
}
