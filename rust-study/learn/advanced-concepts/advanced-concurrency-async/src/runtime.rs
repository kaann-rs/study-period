// Kitap: 29.2. bölüm, Çalışma Zamanı: Görev Kuyruğu ve Geri Basınç

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

type BoxedTask = Pin<Box<dyn Future<Output = ()> + Send>>;

struct Task {
    future: Mutex<Option<BoxedTask>>,
    queue: Sender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let queue = self.queue.clone();
        let _ = queue.send(self);
    }
}

pub struct Runner {
    ready_tasks: Receiver<Arc<Task>>,
    spawner: Spawner,
    poll_count: usize,
}

#[derive(Clone)]
pub struct Spawner {
    queue: Sender<Arc<Task>>,
    live_count: Arc<AtomicUsize>,
}

impl Spawner {
    pub fn spawn<F>(&self, future: F) -> TaskHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let shared = Arc::new(Mutex::new(HandleState {
            result: None,
            waker: None,
        }));
        let task_side = Arc::clone(&shared);
        let wrapped = async move {
            let value = future.await;
            let waker = {
                let mut state = task_side.lock().unwrap();
                state.result = Some(value);
                state.waker.take()
            };
            if let Some(waker) = waker {
                waker.wake();
            }
        };
        self.live_count.fetch_add(1, Ordering::SeqCst);
        let task = Arc::new(Task {
            future: Mutex::new(Some(Box::pin(wrapped))),
            queue: self.queue.clone(),
        });
        let _ = self.queue.send(task);
        TaskHandle { shared }
    }
}

impl Runner {
    pub fn new() -> Self {
        let (queue, ready_tasks) = mpsc::channel();
        Runner {
            ready_tasks,
            spawner: Spawner {
                queue,
                live_count: Arc::new(AtomicUsize::new(0)),
            },
            poll_count: 0,
        }
    }

    pub fn spawner(&self) -> Spawner {
        self.spawner.clone()
    }

    pub fn run(&mut self) -> usize {
        while self.spawner.live_count.load(Ordering::SeqCst) > 0 {
            let Ok(task) = self.ready_tasks.recv() else {
                break;
            };
            let mut slot = task.future.lock().unwrap();
            let Some(mut future) = slot.take() else {
                continue;
            };
            let waker = Waker::from(Arc::clone(&task));
            let mut cx = Context::from_waker(&waker);
            self.poll_count += 1;
            match future.as_mut().poll(&mut cx) {
                Poll::Pending => *slot = Some(future),
                Poll::Ready(()) => {
                    self.spawner.live_count.fetch_sub(1, Ordering::SeqCst);
                }
            }
        }
        self.poll_count
    }
}

pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let mut runner = Runner::new();
    let handle = runner.spawner().spawn(future);
    runner.run();
    handle.take_result().expect("görev bitti ama sonuç yok")
}

struct HandleState<T> {
    result: Option<T>,
    waker: Option<Waker>,
}

pub struct TaskHandle<T> {
    shared: Arc<Mutex<HandleState<T>>>,
}

impl<T> TaskHandle<T> {
    pub fn take_result(&self) -> Option<T> {
        self.shared.lock().unwrap().result.take()
    }
}

impl<T> Future for TaskHandle<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut state = self.shared.lock().unwrap();
        match state.result.take() {
            Some(value) => Poll::Ready(value),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

pub struct Sleep {
    duration: Duration,
    has_started: bool,
    state: Arc<Mutex<SleepState>>,
}

struct SleepState {
    is_over: bool,
    waker: Option<Waker>,
}

impl Sleep {
    pub fn new(duration: Duration) -> Self {
        Sleep {
            duration,
            has_started: false,
            state: Arc::new(Mutex::new(SleepState {
                is_over: false,
                waker: None,
            })),
        }
    }
}

pub fn sleep_ms(ms: u64) -> Sleep {
    Sleep::new(Duration::from_millis(ms))
}

impl Future for Sleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        let mut state = this.state.lock().unwrap();
        if state.is_over {
            return Poll::Ready(());
        }
        state.waker = Some(cx.waker().clone());
        drop(state);
        if !this.has_started {
            this.has_started = true;
            let shared = Arc::clone(&this.state);
            let duration = this.duration;
            thread::spawn(move || {
                thread::sleep(duration);
                let waker = {
                    let mut state = shared.lock().unwrap();
                    state.is_over = true;
                    state.waker.take()
                };
                if let Some(waker) = waker {
                    waker.wake();
                }
            });
        }
        Poll::Pending
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TimeoutError;

pub struct Timeout<T> {
    inner: Pin<Box<dyn Future<Output = T> + Send>>,
    timer: Sleep,
}

pub fn timeout<F>(duration: Duration, future: F) -> Timeout<F::Output>
where
    F: Future + Send + 'static,
{
    Timeout {
        inner: Box::pin(future),
        timer: Sleep::new(duration),
    }
}

impl<T> Future for Timeout<T> {
    type Output = Result<T, TimeoutError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Poll::Ready(value) = this.inner.as_mut().poll(cx) {
            return Poll::Ready(Ok(value));
        }
        if Pin::new(&mut this.timer).poll(cx).is_ready() {
            return Poll::Ready(Err(TimeoutError));
        }
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn block_on_returns_result() {
        assert_eq!(block_on(async { 6 * 7 }), 42);
    }

    #[test]
    fn sleep_takes_at_least_the_asked_time() {
        let start = Instant::now();
        block_on(sleep_ms(20));
        assert!(start.elapsed() >= Duration::from_millis(20));
    }

    #[test]
    fn task_spawns_task_and_waits_for_result() {
        let mut runner = Runner::new();
        let spawner = runner.spawner();
        let inner_spawner = spawner.clone();
        let handle = spawner.spawn(async move {
            let a = inner_spawner.spawn(async {
                sleep_ms(10).await;
                1
            });
            let b = inner_spawner.spawn(async { 2 });
            a.await + b.await
        });
        runner.run();
        assert_eq!(handle.take_result(), Some(3));
    }

    #[test]
    fn runner_polls_only_woken_tasks() {
        let mut runner = Runner::new();
        let spawner = runner.spawner();
        for _ in 0..10 {
            spawner.spawn(sleep_ms(15));
        }
        assert_eq!(runner.run(), 20);
    }

    #[test]
    fn timeout_passes_fast_job_and_cuts_slow_one() {
        let fast = block_on(timeout(Duration::from_millis(200), async {
            sleep_ms(5).await;
            "bitti"
        }));
        assert_eq!(fast, Ok("bitti"));
        let slow = block_on(timeout(Duration::from_millis(10), async {
            sleep_ms(300).await;
            "bitti"
        }));
        assert_eq!(slow, Err(TimeoutError));
    }
}
