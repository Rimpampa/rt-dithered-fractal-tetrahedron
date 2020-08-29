# Real Time Dithered Fractal Tetrahedron

This 3D graphics demo shows the
[Sierpinski tetrahedron](https://en.wikipedia.org/wiki/Sierpi%C5%84ski_triangle#Analogues_in_higher_dimensions)
being drawn using the
[Floyd–Steinberg dithering algorithm](https://en.wikipedia.org/wiki/Floyd%E2%80%93Steinberg_dithering)
to limit the colors to a fixed palette.

The palette is made of red, green, blue, white and black. To modify those
colors the source code has to be modified.

The fractal is made by subdividing the base tetrahedron in three and
repeating this process for all the subsequent tetrahedrons. The number
of iterations can be modified at the start of the program by passing a
number has argument, otherwise it will be 4.

> **Note** that what follows are my own suppositions and they might not be correct, so
> if someone notice something wrong please let me know

# Dithering Implementation

The Floyd–Steinberg dithering algorithm is very expensive because not only
it has to process each pixel of the image but the resulting color of each
pixel depends on the results of its upper left neighbours. This makes it
really difficult to parallelize the work and thus to make it work so fast
that it reaches a good frame rate, the resolution has to be halved.

To make the dithering fast enough I have implemented a thread pool
that processes multiple pixel rows at the same time. Using threads in this type
of problems is very difficult has if you use the wrong approach you might get no
benefits or even it might get slower.

There are two problems with multithreading:
- references lifetime has to be static;
- data races when working on the same array.

## Static References

First of all, *Why is it needed to share a reference between threads in the first place?*

The problem is that to makes things go fast you have to use the resources you already
have and avoid copying big chunks of data (like images), thus the threads must
work on the same memory location to be as efficient as possible.

Now, *Why do they have to be static?*

That's because the compiler doesn't know when the
thread will finish it's execution and thus it might live longer that the provided
lifetime. Solving this problem is not that hard:

the only thing to do is to make sure that the thread doesn't live longer than the
lifetime and to that you just need to call join before the lifetime of the reference ends.
The compiler, though, still can't assure that thus we need to use some unsafe code to make
him happy, knowing that what we are doing is safe, by expanding the reference scope to
the static lifetime.

## Syncronization and Data Races

Now it's time to talk about how the threads interact with the data:

_I'm going to assume that you know how the dithering algorith works and, also,
I'm going to use the word `Worker` instead of threads from now on, the reason will be
explained later._

Each worker has its own pixel row on which he makes computations, but it has, also, the next row
(the row right under its one) that it uses to propagate the error. This means that every worker
will share its own row with the previous one, apart from the first that has no previus worker.
Who has control of the row, though, is the previuos worker as if it doesn't go far enough it's next
worker will read pixels on which the error has not been added. The relation between the workers is
therefore hierarchical, and follows this sheme:
```
Worker 1  >  # # # # # # # # X - - -
Worker 2  >  # # # # # # X | - - - -
Worker 3  >  # # # # X | - | - - - -
Worker 4  >  # # X | - | - | - - - -
Worker 5  >  X | - | - | - | - - - -
```
Each worker must stay behind the column on the left to the pixel the previous worker is on.

To achieve this a special vector-type container should be used that can be splitted at some point
which then could be moved forewards by the worker who onws it and also it has to be thread-safe.
This structs I called them `OwnedSplit` and `BorroedSplit`, the first one being the one who can
move the split point. Both of them share the same mutable reference to the slice that they come
from and while it's unsafe to use two mutable reference at the same time, in this case it isn't
as the data they can see is not the same: the owned one sees all the data after the split point
and the other the one before. This is done by not providing direct access to the slice, instead
the `Index` trait is implemented for them which makes them usable like slices while restricting
the slice range to the one the can see.

So, to recap, each worker has a `BorrowedSplit` for their own row and an `OwnedSplit` for the
next one.

## The pool

The `WorkerPool` is filled with a fixed amout of `WorkerThread`s when it's created.
After that it can be used to execute `Workers` but whithout knowing the exact thread on which
it will be executed and the selection of the thread is a simple spin-loop around all the threads
the lasts until one that is not running is found.

This means that each worker is not executed on a diferrent thread but the all use the same threads,
thus a worker is not a thread (and viceversa).

Each `WorkerThread` has a `Mutex` that holds a worker (the one it's currently running or none if
it's idle) and an `AtomicBool` flag that signals when it has finished working and it's ready to accept
another worker and while waiting the thread gets parked in order to not consume CPU. Every time a
worker has to be executed, the mutex gets locked, the worker is moved inside it, the lock is freed,
the flag is set and the actual threads gets unparked.

The refernce problem discussed previusly here is solved by implementing the `Drop` trait on the
`WorkerThread` and joining the thread there. This then extnds to the `WorkerPool` has when it
gets dropped all the worker threads get dropped too.

The problem now is that with just this I'd have to create a new thread pool every time I dither
an image, so to solve this problem I implemented the `ScopedThreadPool` that wraps around
the `WorkerPool` and downgrades its lifetime. After the scope ends the `Drop` implementation just
waits for each thread to finish executing (not to return/exit) so that the lifetime is preserved.

# The End

I tried to address every single part of the implementation and there is a bit more that is just not that
important, mostly regarding Rust, but there might be some comments in the code explaining those things.

I actually don't know why I felt like I needed to write this down, but here it is, I hope someone finds
it useful/intresting.