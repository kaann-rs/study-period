// Kitap: 28. bölüm, Future'ın İç Yüzü

use std::future::Future;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};

fn main() {
    println!("== 1) poll / Pending / wake / yeniden-poll döngüsü (izlenebilir) ==");
    let result = watch_block_on(DelayedValue { remaining: 3 });
    println!("block_on döndü: {result:?}\n");

    println!("== 2) Waker gerçekte nedir? ==");
    show_waker();

    println!("\n== 3) Pin ve self-referential (RfR Böl. 8) ==");
    show_pin_and_self_reference();
}

struct DelayedValue {
    remaining: u32,
}
impl Future for DelayedValue {
    type Output = &'static str;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        println!("   [future.poll] çağrıldı — kalan = {}", self.remaining);
        if self.remaining == 0 {
            println!("   [future.poll] hazır → Poll::Ready");
            Poll::Ready("İŞ BİTTİ")
        } else {
            self.remaining -= 1;
            cx.waker().wake_by_ref();
            println!("   [future.poll] hazır değil → waker kaydedildi, Poll::Pending");
            Poll::Pending
        }
    }
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
fn watch_block_on<F: Future>(fut: F) -> F::Output
where
    F::Output: std::fmt::Debug,
{
    let mut fut = std::pin::pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut round = 0;
    loop {
        round += 1;
        println!("  [executor] tur {round}: poll ediyorum");
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                println!("  [executor] Ready geldi → döngüden çık");
                return v;
            }
            Poll::Pending => {
                println!("  [executor] Pending → uyu (park), wake bekliyorum");
                thread::park();
            }
        }
    }
}

fn show_waker() {
    let waker: Waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let waker_copy = waker.clone();
    println!(
        "   Waker = (veri ptr, vtable{{clone,wake,wake_by_ref,drop}}); klonlanır: {}",
        waker.will_wake(&waker_copy)
    );
    waker.wake_by_ref();
    println!("   wake_by_ref() çağrıldı (bu executor'ı 'yeniden poll et' diye dürter)");
}

struct SelfRef {
    data: String,
    data_pointer: *const String,
    _pin: PhantomPinned,
}
impl SelfRef {
    fn new(text: &str) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfRef {
            data: String::from(text),
            data_pointer: ptr::null(),
            _pin: PhantomPinned,
        });
        let address: *const String = &boxed.data;
        unsafe {
            let mut_ref: &mut SelfRef = Pin::get_unchecked_mut(boxed.as_mut());
            mut_ref.data_pointer = address;
        }
        boxed
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn read_through_pointer(&self) -> &String {
        unsafe { &*self.data_pointer }
    }
}
fn show_pin_and_self_reference() {
    let pinned = SelfRef::new("merhaba");
    println!("   data()                 = {}", pinned.data());
    println!(
        "   read_through_pointer() = {}",
        pinned.read_through_pointer()
    );
    println!("   iki yol da aynı adresi görüyor (Pin değeri yerinden oynatmadığı için geçerli)");
}
