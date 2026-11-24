// Kitap: 15. bölüm, Eşzamanlılık

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

mod deep_dive;

fn main() {
    show_threads();
    show_channels();
    show_shared_state();
    deep_dive::run();
}

fn show_threads() {
    let data = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        let total: i32 = data.iter().sum();
        total
    });
    let result = handle.join().unwrap();
    println!("thread topladı = {result}");
}

fn show_channels() {
    let (tx, rx) = mpsc::channel();

    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(format!("thread {id} selam")).unwrap();
        });
    }
    drop(tx);

    let mut messages: Vec<String> = rx.iter().collect();
    messages.sort();
    println!("kanaldan gelenler (sıralı): {messages:?}");
}

fn show_shared_state() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut n = counter.lock().unwrap();
            *n += 1;
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("Arc<Mutex> sayaç = {}", *counter.lock().unwrap());
}

// Learn more
//
// You can learn threads from these sources:
//   - The Rust Book, "Using Threads to Run Code Simultaneously": https://doc.rust-lang.org/book/ch16-01-threads.html
//   - Rust by Example, "Threads": https://doc.rust-lang.org/rust-by-example/std_misc/threads.html
//   - Comprehensive Rust, "Plain Threads": https://google.github.io/comprehensive-rust/concurrency/threads/plain.html
//   - Easy: Easy Rust, "Multiple threads": https://dhghomon.github.io/easy_rust/Chapter_46.html
//   - Easy: Let's Get Rusty (video), "Concurrency in Rust - Creating Threads": https://www.youtube.com/watch?v=06WcsNPUNC8
//
// You can learn channels from these sources:
//   - The Rust Book, "Transfer Data Between Threads with Message Passing": https://doc.rust-lang.org/book/ch16-02-message-passing.html
//   - Rust by Example, "Channels": https://doc.rust-lang.org/rust-by-example/std_misc/channels.html
//   - Comprehensive Rust, "Senders and Receivers": https://google.github.io/comprehensive-rust/concurrency/channels/senders-receivers.html
//   - Easy: Easy Rust, "Channels": https://dhghomon.github.io/easy_rust/Chapter_50.html
//   - Easy: Let's Get Rusty (video), "Concurrency in Rust - Message Passing": https://www.youtube.com/watch?v=FE1BkKqYCGU
//
// You can learn shared state (Arc and Mutex) from these sources:
//   - The Rust Book, "Shared-State Concurrency": https://doc.rust-lang.org/book/ch16-03-shared-state.html
//   - Comprehensive Rust, "Mutex": https://google.github.io/comprehensive-rust/concurrency/shared-state/mutex.html
//   - Rust by Example, "Arc": https://doc.rust-lang.org/rust-by-example/std/arc.html
//   - Easy: Easy Rust, "Arc": https://dhghomon.github.io/easy_rust/Chapter_49.html
//   - Easy: Let's Get Rusty (video), "Concurrency in Rust - Sharing State": https://www.youtube.com/watch?v=mupwF9jbVZ4
