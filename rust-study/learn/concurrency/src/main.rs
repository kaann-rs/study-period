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
