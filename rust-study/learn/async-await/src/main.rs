// Kitap: 16. bölüm, Async ve Await

use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

mod deep_dive;

fn main() {
    let result = block_on(add_waited_values());
    println!("block_on döndü: {result}");

    println!("-- iki görev eşzamanlı --");
    run_tasks(vec![Box::pin(run_task(1)), Box::pin(run_task(2))]);
    deep_dive::run();
}

struct ThreadWaker(Thread);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => thread::park(),
        }
    }
}

fn run_tasks(mut tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>) {
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    while !tasks.is_empty() {
        tasks.retain_mut(|task| task.as_mut().poll(&mut cx).is_pending());
        if !tasks.is_empty() {
            thread::park();
        }
    }
}

struct WaitOnce {
    is_ready: bool,
}

impl Future for WaitOnce {
    type Output = i32;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        if self.is_ready {
            Poll::Ready(42)
        } else {
            self.is_ready = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct HandOver(bool);
impl Future for HandOver {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

async fn greet(name: &str) -> String {
    format!("Selam {name}")
}

async fn add_waited_values() -> i32 {
    let n = WaitOnce { is_ready: false }.await;
    let greeting = greet("Kaan").await;
    println!("  add_waited_values içinden: {greeting}");
    n + 1
}

async fn run_task(id: u32) {
    println!("  görev {id}: başladı");

    HandOver(false).await;
    println!("  görev {id}: bitti");
}

// Learn more
//
// You can learn how to write a small executor (block_on) from these sources:
//   - Async Book, "Applied: Build an Executor": https://rust-lang.github.io/async-book/02_execution/04_executor.html
//   - Philipp Oppermann, "Async/Await": https://os.phil-opp.com/async-await/
//   - Crust of Rust (video), "async/await": https://www.youtube.com/watch?v=ThjvMReOXYM
//   - Easy: Learn by Building, "A Minimal Executor": https://its-saeed.github.io/learn-by-building/async/05-executor.html
//
// You can learn wakers from these sources:
//   - Async Book, "Task Wakeups with Waker": https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
//   - Standard library docs, "std::task::Waker": https://doc.rust-lang.org/std/task/struct.Waker.html
//   - Tokio tutorial, "Wakers": https://tokio.rs/tokio/tutorial/async#wakers
//   - Easy: Learn by Building, "Wakers & Waking": https://its-saeed.github.io/learn-by-building/async/03-wakers.html
//
// You can learn how to write a Future by hand from these sources:
//   - Async Book, "The Future Trait": https://rust-lang.github.io/async-book/02_execution/02_future.html
//   - Standard library docs, "std::future::Future": https://doc.rust-lang.org/std/future/trait.Future.html
//   - natkr, "Async from scratch 1: What's in a Future, anyway?": https://natkr.com/2025-04-10-async-from-scratch-1/
//   - Easy: Learn by Building, "Futures by Hand": https://its-saeed.github.io/learn-by-building/async/01-futures.html
//
// You can learn async and await from these sources:
//   - The Rust Book, "Futures and the Async Syntax": https://doc.rust-lang.org/book/ch17-01-futures-and-syntax.html
//   - Comprehensive Rust, "async/await": https://google.github.io/comprehensive-rust/concurrency/async/async-await.html
//   - Rust Reference, "Await expressions": https://doc.rust-lang.org/reference/expressions/await-expr.html
//   - Easy: DEV Community, "Async Rust, but less intimidating": https://dev.to/dotxlem/async-rust-but-less-intimidating-2c13
//
// You can learn how to run many tasks from these sources:
//   - The Rust Book, "Applying Concurrency with Async": https://doc.rust-lang.org/book/ch17-02-concurrency-with-async.html
//   - Comprehensive Rust, "Tasks": https://google.github.io/comprehensive-rust/concurrency/async/tasks.html
//   - Tokio tutorial, "Spawning": https://tokio.rs/tokio/tutorial/spawning
//   - Easy: DEV Community, "Async/Await in Rust: A Beginner's Guide": https://dev.to/leapcell/asyncawait-in-rust-a-beginners-guide-237h
