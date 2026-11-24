// Kitap: 29.1. bölüm, Thread'ler ve Zamanlayıcı

use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;

pub fn run() {
    println!("== A1) RwLock: çok okuyucu YA DA tek yazar ==");
    show_rwlock();
    println!("\n== A2) Condvar: sınırlı kuyruk (üretici/tüketici, geri basınç) ==");
    show_condvar();
    println!("\n== A3) Deadlock ve kilit sıralaması: banka transferi ==");
    show_lock_order();
    println!("\n== A4) Atomikler: compare_exchange ve durdurma bayrağı ==");
    show_atomics();
    println!("\n== A5) Send / Sync: hangi tür neden geçer, neden geçmez ==");
    show_send_sync_table();
}

pub struct SettingsStore {
    settings: RwLock<HashMap<String, String>>,
    version: AtomicU64,
}

impl SettingsStore {
    pub fn new() -> Self {
        SettingsStore {
            settings: RwLock::new(HashMap::new()),
            version: AtomicU64::new(0),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.settings.read().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &str) {
        self.settings
            .write()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        self.version.fetch_add(1, Ordering::Relaxed);
    }

    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }
}

fn show_rwlock() {
    let store = SettingsStore::new();
    store.set("tema", "açık");

    {
        let first_reader = store.settings.read().unwrap();
        let can_read_again = store.settings.try_read().is_ok();
        let can_write = store.settings.try_write().is_ok();
        println!(
            "   bir okuyucu varken ({} ayar): ikinci okuyucu = {can_read_again}, yazar = {can_write}",
            first_reader.len()
        );
    }

    let bad_reads = AtomicU32::new(0);
    thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for _ in 0..1_000 {
                    match store.get("tema").as_deref() {
                        Some("açık") | Some("koyu") => {}
                        _ => {
                            bad_reads.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
        }
        scope.spawn(|| {
            for i in 0..100 {
                store.set("tema", if i % 2 == 0 { "koyu" } else { "açık" });
            }
        });
    });
    println!(
        "   son değer = {:?}, sürüm = {}, geçersiz okuma = {}",
        store.get("tema"),
        store.version(),
        bad_reads.into_inner()
    );
}

pub struct LimitedQueue<T> {
    state: Mutex<QueueState<T>>,
    not_full: Condvar,
    not_empty: Condvar,
    capacity: usize,
}

struct QueueState<T> {
    items: VecDeque<T>,
    is_closed: bool,
    max_length: usize,
}

impl<T> LimitedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        LimitedQueue {
            state: Mutex::new(QueueState {
                items: VecDeque::new(),
                is_closed: false,
                max_length: 0,
            }),
            not_full: Condvar::new(),
            not_empty: Condvar::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn put(&self, item: T) -> Result<(), T> {
        let lock = self.state.lock().unwrap();
        let mut state = self
            .not_full
            .wait_while(lock, |state| {
                state.items.len() >= self.capacity && !state.is_closed
            })
            .unwrap();
        if state.is_closed {
            return Err(item);
        }
        state.items.push_back(item);
        state.max_length = state.max_length.max(state.items.len());
        drop(state);
        self.not_empty.notify_one();
        Ok(())
    }

    pub fn take(&self) -> Option<T> {
        let lock = self.state.lock().unwrap();
        let mut state = self
            .not_empty
            .wait_while(lock, |state| state.items.is_empty() && !state.is_closed)
            .unwrap();
        let item = state.items.pop_front();
        drop(state);
        if item.is_some() {
            self.not_full.notify_one();
        }
        item
    }

    pub fn close(&self) {
        self.state.lock().unwrap().is_closed = true;
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }

    pub fn max_length(&self) -> usize {
        self.state.lock().unwrap().max_length
    }
}

pub fn produce_and_consume(queue: &LimitedQueue<u64>, per_producer: u64) -> u64 {
    thread::scope(|scope| {
        let producers: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    for i in 1..=per_producer {
                        queue.put(i).expect("kuyruk üretim bitmeden kapatılmamalı");
                    }
                })
            })
            .collect();
        let consumers: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    let mut total = 0;
                    while let Some(x) = queue.take() {
                        total += x;
                    }
                    total
                })
            })
            .collect();
        for producer in producers {
            producer.join().unwrap();
        }
        queue.close();
        consumers
            .into_iter()
            .map(|consumer| consumer.join().unwrap())
            .sum()
    })
}

fn show_condvar() {
    let queue = LimitedQueue::new(4);
    let total = produce_and_consume(&queue, 500);
    println!(
        "   tüketilen toplam = {total}, doluluk hiç kapasiteyi (4) aşmadı = {}",
        queue.max_length() <= 4
    );
    println!("   kapalı kuyruğa koy → {:?}", queue.put(7));
}

pub struct Account {
    pub id: u32,
    balance: Mutex<i64>,
}

impl Account {
    pub fn new(id: u32, balance: i64) -> Self {
        Account {
            id,
            balance: Mutex::new(balance),
        }
    }

    pub fn balance(&self) -> i64 {
        *self.balance.lock().unwrap()
    }
}

pub fn transfer(from: &Account, to: &Account, amount: i64) -> bool {
    if from.id == to.id {
        return false;
    }
    if amount <= 0 {
        return false;
    }
    let is_source_first = from.id < to.id;
    let (first, second) = if is_source_first {
        (from, to)
    } else {
        (to, from)
    };
    let mut first_lock = first.balance.lock().unwrap();
    let mut second_lock = second.balance.lock().unwrap();
    let (source, target) = if is_source_first {
        (&mut *first_lock, &mut *second_lock)
    } else {
        (&mut *second_lock, &mut *first_lock)
    };
    if *source < amount {
        return false;
    }
    *source -= amount;
    *target += amount;
    true
}

fn show_lock_order() {
    let a = Account::new(1, 1_000);
    let b = Account::new(2, 1_000);
    thread::scope(|scope| {
        scope.spawn(|| (0..1_000).for_each(|_| assert!(transfer(&a, &b, 1))));
        scope.spawn(|| (0..500).for_each(|_| assert!(transfer(&b, &a, 1))));
    });
    println!(
        "   A = {}, B = {}, toplam = {} (para kaybolmadı, kilitlenme yok)",
        a.balance(),
        b.balance(),
        a.balance() + b.balance()
    );
    println!("   yetersiz bakiye → {}", transfer(&a, &b, 10_000));
}

pub fn choose_leader(candidate_count: u32) -> (u32, u32) {
    let leader = AtomicU32::new(0);
    let winner_count = AtomicU32::new(0);
    thread::scope(|scope| {
        for candidate in 1..=candidate_count {
            let (leader, winner_count) = (&leader, &winner_count);
            scope.spawn(move || {
                if leader
                    .compare_exchange(0, candidate, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    winner_count.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    (leader.into_inner(), winner_count.into_inner())
}

fn show_atomics() {
    let (leader, winner_count) = choose_leader(8);
    println!(
        "   8 aday yarıştı: kazanan sayısı = {winner_count}, lider geçerli bir aday = {}",
        (1..=8).contains(&leader)
    );

    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let stop = Arc::clone(&stop);
        thread::spawn(move || {
            let mut round: u64 = 0;
            while !stop.load(Ordering::Relaxed) {
                round += 1;
                thread::yield_now();
            }
            round
        })
    };
    stop.store(true, Ordering::Relaxed);
    let round = worker.join().unwrap();
    println!(
        "   işçi bayrağı görüp durdu (sayaç geçerli: {})",
        round < u64::MAX
    );
}

fn needs_send<T: Send>() {}
fn needs_sync<T: Sync>() {}

fn show_send_sync_table() {
    needs_send::<Arc<Mutex<Vec<u8>>>>();
    needs_sync::<Arc<RwLock<String>>>();
    needs_send::<Cell<u8>>();
    needs_sync::<Mutex<Cell<u8>>>();
    println!("   derleme anı üyelik testleri geçti (kod çalıştırmadan, yalnız derlenerek)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rwlock_readers_share_but_writer_is_alone() {
        let store = SettingsStore::new();
        store.set("a", "1");
        let first_reader = store.settings.read().unwrap();
        assert!(store.settings.try_read().is_ok());
        assert!(store.settings.try_write().is_err());
        drop(first_reader);
        assert!(store.settings.try_write().is_ok());
        assert_eq!(store.version(), 1);
    }

    #[test]
    fn limited_queue_loses_no_item() {
        let queue = LimitedQueue::new(2);
        assert_eq!(produce_and_consume(&queue, 200), 2 * (200 * 201 / 2));
        assert!(queue.max_length() <= 2);
    }

    #[test]
    fn closed_queue_gives_the_rest_then_none() {
        let queue = LimitedQueue::new(3);
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.close();
        assert_eq!(queue.put(3), Err(3));
        assert_eq!(
            (queue.take(), queue.take(), queue.take()),
            (Some(1), Some(2), None)
        );
    }

    #[test]
    fn full_queue_makes_producer_wait() {
        let queue = LimitedQueue::new(1);
        queue.put(1).unwrap();
        thread::scope(|scope| {
            let producer = scope.spawn(|| queue.put(2));
            thread::sleep(std::time::Duration::from_millis(20));
            assert!(!producer.is_finished());
            assert_eq!(queue.take(), Some(1));
            assert_eq!(producer.join().unwrap(), Ok(()));
        });
        assert_eq!(queue.take(), Some(2));
        assert_eq!(queue.max_length(), 1);
    }

    #[test]
    fn opposite_transfers_do_not_deadlock_and_keep_total() {
        let a = Account::new(1, 500);
        let b = Account::new(2, 500);
        thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| (0..200).filter(|_| transfer(&a, &b, 1)).count());
                scope.spawn(|| (0..200).filter(|_| transfer(&b, &a, 1)).count());
            }
        });
        assert_eq!(a.balance() + b.balance(), 1_000);
        assert!(!transfer(&a, &a, 1));
        assert!(!transfer(&a, &b, -5));
        assert!(!transfer(&a, &b, 0));
    }

    #[test]
    fn compare_exchange_picks_one_winner() {
        for _ in 0..20 {
            let (leader, winner_count) = choose_leader(6);
            assert_eq!(winner_count, 1);
            assert!((1..=6).contains(&leader));
        }
    }
}

// Learn more
//
// You can learn RwLock from these sources:
//   - Standard library docs, "std::sync::RwLock": https://doc.rust-lang.org/std/sync/struct.RwLock.html
//   - Rust Atomics and Locks, "Basics of Rust Concurrency": https://mara.nl/atomics/basics.html#reader-writer-lock
//   - Easy: TutorialEdge, "Using RwLock and CondVars in Rust": https://tutorialedge.net/rust/using-rwlocks-and-condvars-rust/
//
// You can learn Condvar (a limited queue) from these sources:
//   - Standard library docs, "std::sync::Condvar": https://doc.rust-lang.org/std/sync/struct.Condvar.html
//   - Rust Atomics and Locks, "Basics of Rust Concurrency": https://mara.nl/atomics/basics.html#condvar
//   - Easy: TutorialEdge, "Using RwLock and CondVars in Rust": https://tutorialedge.net/rust/using-rwlocks-and-condvars-rust/
//
// You can learn lock order and deadlocks from these sources:
//   - Effective Rust, "Be wary of shared-state parallelism": https://effective-rust.com/deadlock.html
//   - Standard library docs, "std::sync::Mutex": https://doc.rust-lang.org/std/sync/struct.Mutex.html
//   - Rust users forum, "Lock order reversals: how to prevent them?": https://users.rust-lang.org/t/lock-order-reversals-how-to-prevent-them/65016
//
// You can learn atomics and memory ordering from these sources:
//   - Rust Atomics and Locks, "Memory Ordering": https://mara.nl/atomics/memory-ordering.html
//   - Rustonomicon, "Atomics": https://doc.rust-lang.org/nomicon/atomics.html
//   - Crust of Rust (video), "Atomics and Memory Ordering": https://www.youtube.com/watch?v=rMGWeSjctlY
//   - Vorner's blog, "Atomics": https://vorner.github.io/2018/03/25/Atomics.html
//   - Easy: DEV Community, "The Missing Guide to Rust's Atomic::Ordering": https://dev.to/aryan_getsrusty/the-missing-guide-to-rusts-atomicordering-1g9p
//
// You can learn which types are Send and Sync from these sources:
//   - Comprehensive Rust, "Examples": https://google.github.io/comprehensive-rust/concurrency/send-sync/examples.html
//   - Rustonomicon, "Send and Sync": https://doc.rust-lang.org/nomicon/send-and-sync.html
//   - Crust of Rust (video), "Send, Sync, and their implementors": https://www.youtube.com/watch?v=yOezcP-XaIw
//   - Easy: 0xpluie, "Making Sense of Send and Sync in Rust": https://pluiee.github.io/making-sense-of-send-and-sync-in-rust/
//   - Easy: Let's Get Rusty (video), "Concurrency in Rust - Sharing State": https://www.youtube.com/watch?v=mupwF9jbVZ4
