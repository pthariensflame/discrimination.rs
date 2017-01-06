use either::Either::{self, Left, Right};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub enum NonAtomic {}

pub enum Atomic {}

pub trait Sharing<T: ?Sized> {
    type Shared: Clone;
    fn create(v: T) -> Self::Shared where T: Sized;
    fn modify<R, F>(this: &Self::Shared, f: F) -> R where F: FnMut(&mut T) -> R;
    fn inspect<R, F>(this: &Self::Shared, f: F) -> R where F: FnMut(&T) -> R;
}

impl<T: ?Sized> Sharing<T> for NonAtomic {
    type Shared = Rc<RefCell<T>>;

    fn create(v: T) -> Rc<RefCell<T>>
        where T: Sized
    {
        Rc::new(RefCell::new(v))
    }

    fn modify<R, F>(this: &Rc<RefCell<T>>, f: F) -> R
        where F: FnOnce(&mut T) -> R
    {
        f(&mut this.borrow_mut())
    }

    fn inspect<R, F>(this: &Rc<RefCell<T>>, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        f(&this.borrow())
    }
}

impl<T: ?Sized> Sharing<T> for Atomic {
    type Shared = Arc<RwLock<T>>;

    fn create(v: T) -> Arc<RwLock<T>>
        where T: Sized
    {
        Arc::new(RwLock::new(v))
    }

    fn modify<R, F>(this: &Arc<RwLock<T>>, f: F) -> R
        where F: FnOnce(&mut T) -> R
    {
        f(&mut this.write().expect("Modification under shared RwLock failed"))
    }

    fn inspect<R, F>(this: &Arc<RwLock<T>>, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        f(&this.read().expect("Inspection under shared RwLock failed"))
    }
}

pub struct SplitEitherImpl<A, B, I: ?Sized> {
    left: VecDeque<A>,
    right: VecDeque<B>,
    left_back: VecDeque<A>,
    right_back: VecDeque<B>,
    inner: I,
}

pub struct SplitEitherLeft<A, B, I: ?Sized, S: ?Sized = NonAtomic>(
    <S as Sharing<SplitEitherImpl<A, B, I>>>::Shared)
    where S: Sharing<SplitEitherImpl<A, B, I>>;

pub struct SplitEitherRight<A, B, I: ?Sized, S: ?Sized = NonAtomic>(
    <S as Sharing<SplitEitherImpl<A, B, I>>>::Shared)
    where S: Sharing<SplitEitherImpl<A, B, I>>;

pub fn split_either<A, B, I, S: ?Sized>
    (inner: I)
     -> (SplitEitherLeft<A, B, I::IntoIter, S>, SplitEitherRight<A, B, I::IntoIter, S>)
    where I: IntoIterator<Item = Either<A, B>>,
          S: Sharing<SplitEitherImpl<A, B, I::IntoIter>>
{
    let shared_left = <S as Sharing<SplitEitherImpl<A, B, I::IntoIter>>>::create(SplitEitherImpl {
        left: VecDeque::new(),
        right: VecDeque::new(),
        left_back: VecDeque::new(),
        right_back: VecDeque::new(),
        inner: inner.into_iter(),
    });
    let shared_right = shared_left.clone();
    (SplitEitherLeft(shared_left), SplitEitherRight(shared_right))
}

impl<A, B, I: ?Sized, S: ?Sized> Iterator for SplitEitherLeft<A, B, I, S>
    where I: Iterator<Item = Either<A, B>>,
          S: Sharing<SplitEitherImpl<A, B, I>>
{
    type Item = A;

    fn next(&mut self) -> Option<A> {
        <S as Sharing<SplitEitherImpl<A, B, I>>>::modify(&self.0, |this| {
            if let Some(val) = this.left.pop_front() {
                return Some(val);
            }
            while let Some(val_or_other) = this.inner.next() {
                match val_or_other {
                    Left(val) => return Some(val),
                    Right(other) => this.right.push_back(other),
                }
            }
            this.left_back.pop_back()
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        <S as Sharing<SplitEitherImpl<A, B, I>>>::inspect(&self.0, |this| {
            let mut low = 0usize;
            let (_, mut high_opt) = this.inner.size_hint();
            let front_extra = this.left.len();
            let back_extra = this.left.len();
            low = low.saturating_add(front_extra);
            high_opt = high_opt.map(|high| high.saturating_add(front_extra));
            low = low.saturating_add(back_extra);
            high_opt = high_opt.map(|high| high.saturating_add(back_extra));
            (low, high_opt)
        })
    }
}

impl<A, B, I: ?Sized, S: ?Sized> DoubleEndedIterator for SplitEitherLeft<A, B, I, S>
    where I: DoubleEndedIterator<Item = Either<A, B>>,
          S: Sharing<SplitEitherImpl<A, B, I>>
{
    fn next_back(&mut self) -> Option<A> {
        <S as Sharing<SplitEitherImpl<A, B, I>>>::modify(&self.0, |this| {
            if let Some(val) = this.left_back.pop_front() {
                return Some(val);
            }
            while let Some(val_or_other) = this.inner.next() {
                match val_or_other {
                    Left(val) => return Some(val),
                    Right(other) => this.right_back.push_back(other),
                }
            }
            this.left.pop_back()
        })
    }
}

impl<A, B, I: ?Sized, S: ?Sized> Iterator for SplitEitherRight<A, B, I, S>
    where I: Iterator<Item = Either<A, B>>,
          S: Sharing<SplitEitherImpl<A, B, I>>
{
    type Item = B;

    fn next(&mut self) -> Option<B> {
        <S as Sharing<SplitEitherImpl<A, B, I>>>::modify(&self.0, |this| {
            if let Some(val) = this.right.pop_front() {
                return Some(val);
            }
            while let Some(val_or_other) = this.inner.next() {
                match val_or_other {
                    Right(val) => return Some(val),
                    Left(other) => this.left.push_back(other),
                }
            }
            this.right_back.pop_back()
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        <S as Sharing<SplitEitherImpl<A, B, I>>>::inspect(&self.0, |this| {
            let mut low = 0usize;
            let (_, mut high_opt) = this.inner.size_hint();
            let front_extra = this.right.len();
            let back_extra = this.right.len();
            low = low.saturating_add(front_extra);
            high_opt = high_opt.map(|high| high.saturating_add(front_extra));
            low = low.saturating_add(back_extra);
            high_opt = high_opt.map(|high| high.saturating_add(back_extra));
            (low, high_opt)
        })
    }
}

impl<A, B, I: ?Sized, S: ?Sized> DoubleEndedIterator for SplitEitherRight<A, B, I, S>
    where I: DoubleEndedIterator<Item = Either<A, B>>,
          S: Sharing<SplitEitherImpl<A, B, I>>
{
    fn next_back(&mut self) -> Option<B> {
        <S as Sharing<SplitEitherImpl<A, B, I>>>::modify(&self.0, |this| {
            if let Some(val) = this.right_back.pop_front() {
                return Some(val);
            }
            while let Some(val_or_other) = this.inner.next() {
                match val_or_other {
                    Right(val) => return Some(val),
                    Left(other) => this.left_back.push_back(other),
                }
            }
            this.right.pop_back()
        })
    }
}
