// Kitap: 14.1. bölüm, Akıllı İşaretçiler: Derinleşme

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub fn run() {
    println!("\n== Derinleşme: akıllı işaretçiler birlikte ==");
    show_folder_tree();
    show_cycle_and_leak();
    show_interior_mutability();
    show_refcell_traps();
    show_copy_if_shared();
}

pub struct Folder {
    name: String,
    parent: RefCell<Weak<Folder>>,
    children: RefCell<Vec<Rc<Folder>>>,
}

impl Folder {
    pub fn new(name: &str) -> Rc<Folder> {
        Rc::new(Folder {
            name: name.to_string(),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    pub fn add_child(parent: &Rc<Folder>, child: Rc<Folder>) {
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    pub fn full_path(&self) -> String {
        match self.parent.borrow().upgrade() {
            Some(parent) => format!("{}/{}", parent.full_path(), self.name),
            None => self.name.clone(),
        }
    }

    pub fn child_count(&self) -> usize {
        self.children.borrow().len()
    }
}

fn show_folder_tree() {
    let root = Folder::new("");
    let home = Folder::new("home");
    let kaan = Folder::new("kaan");
    Folder::add_child(&root, Rc::clone(&home));
    Folder::add_child(&home, Rc::clone(&kaan));

    println!("tam yol = {}", kaan.full_path());
    println!(
        "home: güçlü = {}, zayıf = {}, çocuk = {}",
        Rc::strong_count(&home),
        Rc::weak_count(&home),
        home.child_count()
    );

    drop(root);
    drop(home);
    println!("ebeveyn düştükten sonra tam yol = {}", kaan.full_path());
}

struct Node {
    drop_counter: Rc<Cell<u32>>,
    strong_neighbor: RefCell<Option<Rc<Node>>>,
    weak_neighbor: RefCell<Weak<Node>>,
}

impl Node {
    fn new(counter: &Rc<Cell<u32>>) -> Rc<Node> {
        Rc::new(Node {
            drop_counter: Rc::clone(counter),
            strong_neighbor: RefCell::new(None),
            weak_neighbor: RefCell::new(Weak::new()),
        })
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.drop_counter.set(self.drop_counter.get() + 1);
    }
}

fn make_pair(is_back_link_weak: bool) -> u32 {
    let counter = Rc::new(Cell::new(0));
    {
        let a = Node::new(&counter);
        let b = Node::new(&counter);
        *a.strong_neighbor.borrow_mut() = Some(Rc::clone(&b));
        if is_back_link_weak {
            *b.weak_neighbor.borrow_mut() = Rc::downgrade(&a);
        } else {
            *b.strong_neighbor.borrow_mut() = Some(Rc::clone(&a));
        }
    }
    counter.get()
}

fn show_cycle_and_leak() {
    println!(
        "iki yön de Rc  → düşen düğüm = {} (sızıntı!)",
        make_pair(false)
    );
    println!("geri yön Weak  → düşen düğüm = {}", make_pair(true));
}

pub trait Notifier {
    fn send(&self, message: &str);
}

#[derive(Default)]
pub struct RecordingNotifier {
    messages: RefCell<Vec<String>>,
    calls: Cell<u32>,
}

impl Notifier for RecordingNotifier {
    fn send(&self, message: &str) {
        self.calls.set(self.calls.get() + 1);
        self.messages.borrow_mut().push(message.to_string());
    }
}

impl RecordingNotifier {
    pub fn call_count(&self) -> u32 {
        self.calls.get()
    }
    pub fn messages(&self) -> Vec<String> {
        self.messages.borrow().clone()
    }
}

pub struct LimitTracker<'a, N: Notifier> {
    notifier: &'a N,
    limit: u32,
    used: u32,
}

impl<'a, N: Notifier> LimitTracker<'a, N> {
    pub fn new(notifier: &'a N, limit: u32) -> Self {
        LimitTracker {
            notifier,
            limit: limit.max(1),

            used: 0,
        }
    }

    pub fn spend(&mut self, amount: u32) {
        self.used = self.used.saturating_add(amount);
        let percent = u64::from(self.used) * 100 / u64::from(self.limit);
        if percent >= 100 {
            self.notifier.send("Hata: kota aşıldı");
        } else if percent >= 75 {
            self.notifier.send("Uyarı: kotanın %75'i doldu");
        }
    }
}

fn show_interior_mutability() {
    let notifier = RecordingNotifier::default();
    let mut tracker = LimitTracker::new(&notifier, 100);
    tracker.spend(50);
    tracker.spend(30);
    tracker.spend(25);
    println!(
        "çağrı = {}, mesajlar = {:?}",
        notifier.call_count(),
        notifier.messages()
    );
}

fn show_refcell_traps() {
    let list = RefCell::new(vec![1, 2, 3]);

    {
        let reader = list.borrow();
        let attempt = list.try_borrow_mut();
        println!(
            "okuyucu açıkken (uzunluk {}) borrow_mut alınabildi mi? {}",
            reader.len(),
            attempt.is_ok()
        );
    }
    println!(
        "okuyucu kapandıktan sonra? {}",
        list.try_borrow_mut().is_ok()
    );

    let to_add: Vec<i32> = list.borrow().iter().map(|x| x * 10).collect();
    list.borrow_mut().extend(to_add);
    println!("liste = {:?}", list.borrow());

    let first = *list.borrow().first().unwrap();
    println!("ilk = {first}");
}

fn show_copy_if_shared() {
    let default_settings = Rc::new(vec![String::from("tema=açık"), String::from("dil=tr")]);
    let mut user_settings = Rc::clone(&default_settings);
    println!(
        "değişiklik öncesi aynı veri mi? {}",
        Rc::ptr_eq(&default_settings, &user_settings)
    );

    Rc::make_mut(&mut user_settings)[0] = String::from("tema=koyu");
    println!(
        "değişiklik sonrası aynı veri mi? {}",
        Rc::ptr_eq(&default_settings, &user_settings)
    );
    println!("varsayılan = {default_settings:?}");
    println!("kullanıcı  = {user_settings:?}");

    Rc::make_mut(&mut user_settings).push(String::from("yazı=büyük"));
    println!("kullanıcı  = {} ayar", user_settings.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_path_follows_parents() {
        let root = Folder::new("");
        let etc = Folder::new("etc");
        Folder::add_child(&root, Rc::clone(&etc));
        assert_eq!(etc.full_path(), "/etc");
        assert_eq!(root.child_count(), 1);
    }

    #[test]
    fn weak_does_not_keep_parent_alive() {
        let child = Folder::new("yetim");
        {
            let parent = Folder::new("gecici");
            Folder::add_child(&parent, Rc::clone(&child));
            assert_eq!(child.full_path(), "gecici/yetim");
            assert_eq!(Rc::strong_count(&parent), 1);
        }
        assert_eq!(child.full_path(), "yetim");
    }

    #[test]
    fn cycle_leaks_and_weak_fixes_it() {
        assert_eq!(make_pair(false), 0);
        assert_eq!(make_pair(true), 2);
    }

    #[test]
    fn mock_notifier_records_with_shared_reference() {
        let notifier = RecordingNotifier::default();
        let mut tracker = LimitTracker::new(&notifier, 10);
        tracker.spend(5);
        assert_eq!(notifier.call_count(), 0);
        tracker.spend(3);
        tracker.spend(2);
        assert_eq!(
            notifier.messages(),
            ["Uyarı: kotanın %75'i doldu", "Hata: kota aşıldı"]
        );
    }

    #[test]
    fn refcell_error_can_be_checked() {
        let cell = RefCell::new(0);
        let _reader = cell.borrow();
        assert!(cell.try_borrow().is_ok());
        assert!(cell.try_borrow_mut().is_err());
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn refcell_double_borrow_mut_panics() {
        let cell = RefCell::new(0);
        let _a = cell.borrow_mut();
        let _b = cell.borrow_mut();
    }

    #[test]
    fn make_mut_copies_only_if_shared() {
        let mut a = Rc::new(vec![1]);
        let old_address = Rc::as_ptr(&a);
        Rc::make_mut(&mut a).push(2);
        assert_eq!(Rc::as_ptr(&a), old_address);

        let b = Rc::clone(&a);
        Rc::make_mut(&mut a).push(3);
        assert_eq!(*a, [1, 2, 3]);
        assert_eq!(*b, [1, 2]);
    }
}
