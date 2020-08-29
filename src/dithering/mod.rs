#[macro_use]
mod color;
mod shared;
mod worker;
use shared::{split, BorrowedSlice};
use std::borrow::BorrowMut;
use worker::{ScopedWorkerPool, Worker};

pub use color::{Color, Palette};
pub use worker::WorkerPool;

#[allow(clippy::cast_ref_to_mut, clippy::mut_from_ref)]
unsafe fn ref_to_mut_ref<T: ?Sized>(val: &T) -> &mut T {
    &mut *(val as *const T as *mut T)
}

pub fn dither<'a, 'b: 'a, T: BorrowMut<[Color<'b>]> + 'a>(
    data: &'a mut T,
    width: usize,
    height: usize,
    palette: &'a Palette,
    pool: ScopedWorkerPool<'a, '_>,
) {
    let data: &'a mut [Color<'b>] = data.borrow_mut();
    assert!(data.len() / width == height);

    let mut chunks = data.chunks_exact_mut(width);
    let first = chunks.next().unwrap();
    let mut own_row: BorrowedSlice<_> = first.into();
    for slice in chunks {
        let (owned, borrowed) = split(slice);
        let worker = Worker::new(own_row, Some(owned), palette, width);
        own_row = borrowed.into();
        pool.execute(worker);
    }
    pool.execute(Worker::new(own_row, None, palette, width));
}
