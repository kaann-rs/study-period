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
