use super::ref_to_mut_ref;
use std::ops;
use std::slice::SliceIndex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub fn split<T>(value: &mut [T]) -> (OwnedSplit<T>, BorrowedSplit<T>) {
    let split = Arc::new(AtomicUsize::new(0));
    let o = OwnedSplit {
        slice: unsafe { ref_to_mut_ref(value) },
        split: Arc::clone(&split),
    };
    let b = BorrowedSplit {
        slice: unsafe { ref_to_mut_ref(value) },
        split,
    };
    (o, b)
}

// Owned Slice Split
pub struct OwnedSplit<'a, T> {
    slice: &'a mut [T],
    split: Arc<AtomicUsize>,
}

impl<'a, T> OwnedSplit<'a, T> {
    pub fn lend(&mut self, amount: usize) {
        let split = self.split.fetch_add(amount, Ordering::AcqRel);
        assert!(split + amount < self.slice.len());
    }

    pub fn lend_all(&mut self) {
        self.split.store(self.slice.len(), Ordering::Release);
    }

    // pub fn lent(&self) -> usize {
    //     self.split.load(Ordering::SeqCst)
    // }
}

impl<'a, T> Drop for OwnedSplit<'a, T> {
    fn drop(&mut self) {
        self.lend_all()
    }
}

// Borrowed Slice Split
pub struct BorrowedSplit<'a, T> {
    slice: &'a mut [T],
    split: Arc<AtomicUsize>,
}

pub enum BorrowedSlice<'a, T> {
    Owned(&'a mut [T]),
    Shared(BorrowedSplit<'a, T>),
}

impl<'a, T, I> ops::Index<I> for OwnedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.slice[self.split.load(Ordering::Acquire)..][index]
    }
}

impl<'a, T, I> ops::IndexMut<I> for OwnedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.slice[self.split.load(Ordering::Acquire)..][index]
    }
}

impl<'a, T, I> ops::Index<I> for BorrowedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.slice[..self.split.load(Ordering::Acquire)][index]
    }
}

impl<'a, T, I> ops::IndexMut<I> for BorrowedSplit<'a, T>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.slice[..self.split.load(Ordering::Acquire)][index]
    }
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
