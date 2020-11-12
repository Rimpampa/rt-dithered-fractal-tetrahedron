#![allow(unused)]

use std::marker::PhantomData;
use std::ops;
use std::slice::SliceIndex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub fn split<T>(value: &mut [T]) -> (OwnedSplit<T>, BorrowedSplit<T>) {
    let split = Arc::new(AtomicUsize::new(0));
    let ptr = value.as_mut_ptr();
    let o = OwnedSplit {
        ptr,
        len: value.len(),
        split: Arc::clone(&split),
        _marker: PhantomData,
    };
    let b = BorrowedSplit {
        ptr,
        split,
        _marker: PhantomData,
    };
    (o, b)
}

// Owned Slice Split
pub struct OwnedSplit<'a, T> {
    ptr: *mut T,
    len: usize,
    split: Arc<AtomicUsize>,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> OwnedSplit<'a, T> {
    fn slice(&self) -> &'a [T] {
        let offset = self.split.load(Ordering::Acquire);
        unsafe {
            let ptr = self.ptr.add(offset);
            std::slice::from_raw_parts(ptr, self.len - offset)
        }
    }

    fn slice_mut(&mut self) -> &'a mut [T] {
        let offset = self.split.load(Ordering::Acquire);
        unsafe {
            let ptr = self.ptr.add(offset);
            std::slice::from_raw_parts_mut(ptr, self.len - offset)
        }
    }

    pub fn lend(&mut self, amount: usize) {
        let split = self.split.fetch_add(amount, Ordering::AcqRel);
        assert!(split + amount < self.len);
    }

    pub fn lend_all(&mut self) {
        self.split.store(self.len, Ordering::Release);
    }

    // pub fn lent(&self) -> usize {
    //     self.split.load(Ordering::SeqCst)
    // }
}

impl<'a, T, I> ops::Index<I> for OwnedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.slice()[index]
    }
}

impl<'a, T, I> ops::IndexMut<I> for OwnedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.slice_mut()[index]
    }
}

impl<'a, T> Drop for OwnedSplit<'a, T> {
    fn drop(&mut self) {
        self.lend_all()
    }
}

// Borrowed Slice Split
pub struct BorrowedSplit<'a, T> {
    ptr: *mut T,
    split: Arc<AtomicUsize>,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> BorrowedSplit<'a, T> {
    fn slice(&self) -> &'a [T] {
        let len = self.split.load(Ordering::Acquire);
        unsafe { std::slice::from_raw_parts(self.ptr, len) }
    }

    fn slice_mut(&mut self) -> &'a mut [T] {
        let len = self.split.load(Ordering::Acquire);
        unsafe { std::slice::from_raw_parts_mut(self.ptr, len) }
    }
}

impl<'a, T, I> ops::Index<I> for BorrowedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.slice()[index]
    }
}

impl<'a, T, I> ops::IndexMut<I> for BorrowedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.slice_mut()[index]
    }
}

pub enum BorrowedSlice<'a, T> {
    Owned(&'a mut [T]),
    Shared(BorrowedSplit<'a, T>),
}

impl<'a, T, I> ops::Index<I> for BorrowedSlice<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        match self {
            Self::Owned(v) => &v[index],
            Self::Shared(v) => &v[index],
        }
    }
}

impl<'a, T, I> ops::IndexMut<I> for BorrowedSlice<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        match self {
            Self::Owned(v) => &mut v[index],
            Self::Shared(v) => &mut v[index],
        }
    }
}

impl<'a, T> From<&'a mut [T]> for BorrowedSlice<'a, T> {
    fn from(slice: &'a mut [T]) -> Self {
        Self::Owned(slice)
    }
}

impl<'a, T> From<BorrowedSplit<'a, T>> for BorrowedSlice<'a, T> {
    fn from(slice: BorrowedSplit<'a, T>) -> Self {
        Self::Shared(slice)
    }
}

unsafe impl<'a, T> Send for BorrowedSplit<'a, T> {}
unsafe impl<'a, T> Send for OwnedSplit<'a, T> {}
