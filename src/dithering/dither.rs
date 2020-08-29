//! MULTI THREADED
use crate::color::{Color, ColorDiff, Palette};
use std::borrow::BorrowMut;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

//// WARNING

pub struct UnsafeSliceCell<'a, T>(UnsafeCell<&'a mut [T]>);

impl<'a, T> UnsafeSliceCell<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        Self(UnsafeCell::new(slice))
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get(&self) -> &mut [T] {
        &mut (*self.0.get())
    }
}

unsafe impl<T> Sync for UnsafeSliceCell<'_, T> {}

pub unsafe fn make_static<'a, 'b: 'a>(
    colors: &'a mut [Color<'b>],
) -> &'static mut [Color<'static>] {
    std::mem::transmute(colors)
}

//// !!!

// +---+---+---+
// | A | B | C |
// +---+---+---+---+
//     | A | B | C |
//     +---+---+---+
pub enum State {
    A,
    B,
    C,
}

impl State {
    pub fn advance(&mut self) {
        *self = match self {
            State::A => State::B,
            State::B => State::C,
            State::C => State::A,
        };
    }
}

pub struct Worker {
    x: isize,
    y: isize,
    state: State,

    width: usize,
    height: usize,

    flag: Option<Arc<AtomicBool>>,
    next: Option<(Arc<AtomicBool>, thread::Thread)>,

    palette: Arc<Palette>,
    // palette: &'a Palette,
    data: Arc<UnsafeSliceCell<'static, Color<'static>>>,
    // data: &'a UnsafeSliceCell<'a, Color>,
}

impl Worker {
    pub const fn new(
        start_x: isize,
        start_y: isize,
        width: usize,
        height: usize,
        flag: Option<Arc<AtomicBool>>,
        next: Option<(Arc<AtomicBool>, thread::Thread)>,
        data: Arc<UnsafeSliceCell<'static, Color<'static>>>,
        palette: Arc<Palette>,
    ) -> Self {
        Self {
            x: start_x,
            y: start_y,
            width,
            height,
            data,
            flag,
            next,
            palette,
            state: State::A,
        }
    }

    pub fn park(&self) {
        if let Some(flag) = &self.flag {
            while !flag.compare_and_swap(true, false, Ordering::Acquire) {
                thread::park();
            }
        }
    }

    pub fn notify_next(&self) {
        if let Some((flag, handle)) = &self.next {
            while !flag.compare_and_swap(false, true, Ordering::Release) {}
            handle.unpark();
        }
    }

    pub fn start(&mut self, loops: usize, number: usize) {
        for _ in 0..loops {
            if self.x >= 0
                && self.x < self.width as isize
                && self.y >= 0
                && self.y < self.height as isize
            {
                let index = self.x as usize + self.y as usize * self.width;
                let old_color = unsafe { &mut self.data.get()[index] };

                let new_color = self.palette.closest(old_color.clone());
                let new_error = old_color.clone() - new_color.clone();
                *old_color = new_color.clone();

                self.diffuse_error(new_error);
            }
            self.advance();

            println!("Worker {}: notifying", number);
            self.notify_next();
            println!("Worker {}: Notified, parking", number);
            self.park();
            println!("Worker {}: Continuing", number);
        }
    }

    //        | ### | 7/
    //        | ### | /16
    //   -----+-----+----
    //    3/  | 5/  | 1/
    //    /16 | /16 | /16
    fn diffuse_error(&mut self, error: ColorDiff) {
        let r = error * 7 / 16; // right
        let bl = error * 3 / 16; // bottom-left
        let bc = error * 5 / 16; // bottom-center
        let br = error / 16; // bottom-right

        let pixels = unsafe { self.data.get() };

        let y = self.y as usize;
        let x = self.x as usize;

        // bottom center check
        if y < self.height - 1 {
            let index = x + (y + 1) * self.width;
            pixels[index] += bc;

            // + bottom-rigth check
            if x < self.width - 1 {
                let index = (x + 1) + (y + 1) * self.width;
                pixels[index] += br;
            }
            // + bottom-left check
            if x > 0 {
                let index = (x - 1) + (y + 1) * self.width;
                pixels[index] += bl;
            }
        }
        // right check
        if x < self.width - 1 {
            let index = (x + 1) + y * self.width;
            pixels[index] += r;
        }
    }

    fn advance(&mut self) {
        match self.state {
            State::A => self.x += 1,
            State::B => self.x += 1,
            State::C => {
                self.x -= 2;
                self.y += 1;
            }
        }
        self.state.advance();
    }
}

pub fn dither<'a, 'b: 'a, T: BorrowMut<[Color<'b>]>>(
    data: &'a mut T,
    width: usize,
    height: usize,
    palette: Palette,
) {
    assert!(width <= std::isize::MAX as usize);
    assert!(height <= std::isize::MAX as usize);
    let workers = 1 + (width + height - 1) / 3;
    let start_x = -(height as isize);
    let start_y = height as isize / 3;
    let loops = 3 * height + width / 3;

    let data = data.borrow_mut();
    let data = UnsafeSliceCell::new(unsafe { make_static(data) });
    // WARNING
    let cell = Arc::new(data);
    let palette = Arc::new(palette);

    let mut join_handles = Vec::with_capacity(workers);
    let mut prev;
    let mut next = None;
    for i in (0..workers).rev() {
        prev = if i > 0 {
            Some(Arc::new(AtomicBool::new(false)))
        } else {
            None
        };
        let prev_cloned = prev.clone();

        let cell = Arc::clone(&cell);
        let palette = Arc::clone(&palette);
        let join = thread::spawn(move || {
            let mut worker = Worker::new(
                start_x + 3 * i as isize,
                start_y - i as isize,
                width,
                height,
                prev,
                next,
                cell,
                palette,
            );
            worker.park();
            worker.start(loops, i);
        });
        next = prev_cloned.map(|prev| (prev, join.thread().clone()));
        join_handles.push(join);
    }

    join_handles
        .into_iter()
        .try_for_each(JoinHandle::join)
        .unwrap();
}
