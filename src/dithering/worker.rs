use super::color::{Color, ColorDiff, Palette};
use super::shared::{BorrowedSlice, OwnedSplit};
use std::marker::PhantomData;
use std::mem::{replace, transmute, MaybeUninit};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

type StaticWorker = Worker<'static, 'static>;

pub struct Worker<'a, 'b: 'a> {
    palette: &'a Palette,
    own_row: BorrowedSlice<'a, Color<'b>>,
    next_row: Option<OwnedSplit<'a, Color<'b>>>,
    position: usize,
    width: usize,
}

impl<'a, 'b: 'a> Worker<'a, 'b> {
    pub fn new(
        own_row: BorrowedSlice<'a, Color<'b>>,
        next_row: Option<OwnedSplit<'a, Color<'b>>>,
        palette: &'a Palette,
        width: usize,
    ) -> Self {
        Self {
            palette,
            own_row,
            next_row,
            width,
            position: 0,
        }
    }

    pub fn run(&mut self) {
        let mut next = None;
        while self.position < self.width {
            /*
                ISSUE:
                After some testing I discovered that this loop spins much more times than
                I expected and it might be consuming a lot of CPU time.
                A soulution would be to park the thread but I might want to implement that
                in the Borrowed/OwnedSplit (or maybe not, idk)
            */
            let row = &mut self.own_row[self.position..];
            if row.len() > 1 || self.position > 0 {
                for old_color in row.iter_mut() {
                    if let Some(error) = next {
                        *old_color += error;
                    }
                    let colors = self.palette.colors();
                    let new_color = colors[self.palette.closest(old_color)].clone();
                    let new_error = old_color.clone() - new_color.clone();
                    old_color.set(new_color);

                    // let r = ((new_error.r + 255) / 2) as u8;
                    // let g = ((new_error.g + 255) / 2) as u8;
                    // let b = ((new_error.b + 255) / 2) as u8;
                    // old_color.set([r, g, b].into());

                    if let Some(ref mut next_row) = self.next_row {
                        Self::diffuse_error(next_row, new_error, self.position, self.width);
                        if self.position > 0 {
                            next_row.lend(1);
                        }
                    }
                    self.position += 1;
                    next = if self.position < self.width {
                        Some(new_error * 7 / 16)
                    } else {
                        None
                    };
                }
            }
        }
        if let Some(ref mut next_row) = self.next_row {
            next_row.lend_all();
        }
    }

    //        | ### | 7/
    //        | ### | /16
    //   -----+-----+----
    //    3/  | 5/  | 1/
    //    /16 | /16 | /16
    fn diffuse_error(
        next_row: &mut OwnedSplit<'a, Color<'b>>,
        error: ColorDiff,
        position: usize,
        width: usize,
    ) {
        let bl = error * 3 / 16; // bottom-left
        let bc = error * 5 / 16; // bottom-center
        let br = error / 16; // bottom-right

        if position == 0 {
            next_row[0] += bc;
            if width > 1 {
                next_row[1] += br;
            }
        } else {
            next_row[0] += bl;
            next_row[1] += bc;
            if width > position + 1 {
                next_row[2] += br;
            }
        }
    }
}

pub struct WorkerThread<'a> {
    handle: MaybeUninit<thread::JoinHandle<()>>,
    arc: Arc<(Mutex<Option<StaticWorker>>, AtomicBool)>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> WorkerThread<'a> {
    pub fn spawn(arc: Arc<(Mutex<Option<StaticWorker>>, AtomicBool)>) -> Self {
        let arc_cloned = Arc::clone(&arc);
        let handle = thread::spawn(move || {
            let (mutex, flag) = arc_cloned.as_ref();
            while let Ok(mut worker) = {
                while !flag.load(Ordering::Acquire) {
                    thread::park();
                }
                mutex.lock()
            } {
                match worker.as_mut() {
                    Some(worker) => worker.run(),
                    None => return,
                }
                // Note that the lock gets dropped after the store operation
                flag.store(false, Ordering::Release);
            }
        });
        Self {
            handle: MaybeUninit::new(handle),
            arc,
            _marker: PhantomData,
        }
    }

    pub fn unpark(&self) {
        self.arc.1.store(true, Ordering::Release);
        unsafe { &*self.handle.as_ptr() }.thread().unpark();
    }

    // pub fn try_execute(&self, worker: Worker<'a, '_>) -> bool {
    //     if !self.is_running() {
    //         unsafe { self.execute_unchecked(worker) };
    //         true
    //     } else {
    //         false
    //     }
    // }

    pub unsafe fn execute_unchecked(&self, worker: Worker<'a, '_>) {
        /*
            1. Update the mutex value
            2. Set the flag
            3. Unpark the thread
        */
        // differt scope so the lock gets dropped before unpark
        {
            let mut lock = self.arc.0.lock().unwrap();
            *lock = Some(transmute(worker));
        }
        self.unpark();
    }

    pub fn wait(&self) {
        while self.is_running() {
            let _lock = self.arc.0.lock().unwrap();
        }
        // assert!(!self.is_running()); // Maybe redundant
    }

    pub fn is_running(&self) -> bool {
        self.arc.1.load(Ordering::Acquire)
    }

    // pub fn join(self) {} // mem::drop(self)
}

impl Drop for WorkerThread<'_> {
    fn drop(&mut self) {
        /*
            1. Update the mutex value
            2. Set the flag
            3. Unpark the thread
        */
        // differt scope so the lock gets dropped before unpark
        {
            let mut lock = self.arc.0.lock().unwrap();
            *lock = None;
        }
        self.unpark();

        let handle = replace(&mut self.handle, MaybeUninit::uninit());
        unsafe { handle.assume_init() }.join().unwrap();
    }
}

pub struct WorkerPool<'a> {
    handles: Vec<WorkerThread<'a>>,
}

impl<'a> WorkerPool<'a> {
    pub fn new(workers: usize) -> Self {
        let mut handles = Vec::with_capacity(workers);
        let mut mutexes = Vec::with_capacity(workers);
        for _ in 0..workers {
            let arc = Arc::new((Mutex::new(None), AtomicBool::new(false)));
            mutexes.push(Arc::clone(&arc));
            handles.push(WorkerThread::spawn(arc));
        }
        Self { handles }
    }

    pub fn scope<'b>(&'b mut self) -> ScopedWorkerPool<'b, 'a> {
        // if a thread is running panic as it belongs to an higher level scope
        assert!(!self.handles.iter().any(WorkerThread::is_running));
        ScopedWorkerPool { pool: self }
    }

    pub fn wait_all(&self) {
        self.handles.iter().for_each(WorkerThread::wait);
    }

    pub fn execute<'b: 'a>(&self, worker: Worker<'b, '_>) {
        let thread = self
            .handles
            .iter()
            .cycle()
            .find(|w| !w.is_running())
            .unwrap();
        unsafe { thread.execute_unchecked(worker) };
    }
}

pub struct ScopedWorkerPool<'r, 'p: 'r> {
    pool: &'r mut WorkerPool<'p>,
}

impl<'r, 'p: 'r> ScopedWorkerPool<'r, 'p> {
    pub fn execute(&self, worker: Worker<'r, '_>) {
        // Extend the reference knowing that [TODO]
        self.pool.execute(unsafe { transmute(worker) });
    }
}

impl Drop for ScopedWorkerPool<'_, '_> {
    fn drop(&mut self) {
        self.pool.wait_all();
    }
}
