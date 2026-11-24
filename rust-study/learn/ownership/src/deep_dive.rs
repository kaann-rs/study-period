// Kitap: 3.1. bölüm, Ownership: Derinleşme

pub fn run() {
    println!("\n== Derinleşme: ownership pratikte ==");
    show_signature_choice();
    show_partial_move();
    show_user_store();
    show_common_errors();
    show_take_in_place();
}

fn show_signature_choice() {
    fn count_words(text: &str) -> usize {
        text.split_whitespace().count()
    }
    fn trim_spaces(text: &mut String) {
        let trimmed_length = text.trim().len();
        let start = text.len() - text.trim_start().len();
        text.drain(..start);
        text.truncate(trimmed_length);
    }
    fn into_greeting(mut name: String) -> String {
        name.insert_str(0, "Merhaba, ");
        name.push('!');
        name
    }

    let mut input = String::from("  Ada Lovelace  ");
    println!("kelime sayısı = {}", count_words(&input));
    println!("literal de olur = {}", count_words("bir iki üç"));
    trim_spaces(&mut input);
    println!("kırpılmış = '{input}'");
    let greeting = into_greeting(input);
    println!("{greeting}");
}

struct Profile {
    name: String,
    email: String,
    age: u32,
}

fn show_partial_move() {
    let profile = Profile {
        name: String::from("Kaan"),
        email: String::from("kaan@ornek.dev"),
        age: 30,
    };
    let name = profile.name;
    println!(
        "ad = {name}, e-posta = {}, yaş = {}",
        profile.email, profile.age
    );

    let second_profile = Profile {
        name: String::from("Ada"),
        email: String::from("ada@ornek.dev"),
        age: 36,
    };
    let Profile { name, email, age } = second_profile;
    println!("çözüldü: {name} <{email}> ({age})");
}

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub is_active: bool,
}

#[derive(Default)]
pub struct UserStore {
    users: Vec<User>,
}

impl UserStore {
    pub fn add(&mut self, user: User) {
        self.users.push(user);
    }

    pub fn find(&self, id: u32) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }

    pub fn find_mut(&mut self, id: u32) -> Option<&mut User> {
        self.users.iter_mut().find(|user| user.id == id)
    }

    pub fn remove(&mut self, id: u32) -> Option<User> {
        let index = self.users.iter().position(|user| user.id == id)?;
        Some(self.users.remove(index))
    }

    pub fn active_names(&self) -> Vec<&str> {
        self.users
            .iter()
            .filter(|user| user.is_active)
            .map(|user| user.name.as_str())
            .collect()
    }
}

fn show_user_store() {
    let mut store = UserStore::default();
    let ada = User {
        id: 1,
        name: String::from("Ada"),
        is_active: true,
    };
    store.add(ada);
    store.add(User {
        id: 2,
        name: String::from("Linus"),
        is_active: false,
    });
    store.add(User {
        id: 3,
        name: String::from("Grace"),
        is_active: true,
    });

    if let Some(user) = store.find(1) {
        println!("bulundu: {} (aktif: {})", user.name, user.is_active);
    }

    if let Some(user) = store.find_mut(2) {
        user.is_active = true;
    }
    println!("aktifler: {:?}", store.active_names());

    let removed = store.remove(1);
    println!("çıkarılan: {:?}", removed.map(|user| user.name));
    println!("kalan aktifler: {:?}", store.active_names());
}

fn show_common_errors() {
    let names = vec![String::from("Ada"), String::from("Grace")];
    for name in &names {
        print!("{name} ");
    }
    println!("→ {} ad (Vec hâlâ bizim)", names.len());

    let mut numbers = vec![1, 2, 3];
    let to_add: Vec<i32> = numbers
        .iter()
        .filter(|x| **x % 2 == 0)
        .map(|x| x * 10)
        .collect();
    numbers.extend(to_add);
    numbers.retain(|x| *x != 1);
    println!("numbers = {numbers:?}");

    let mut scores = [10, 20, 30];
    let (left, right) = scores.split_at_mut(1);
    std::mem::swap(&mut left[0], &mut right[0]);
    println!("puanlar = {scores:?}");

    let title = String::from("rapor");
    let title_ref = &title;
    println!("önce referansı kullan: {title_ref}");
    let new_owner = title;
    println!("sonra taşı: {new_owner}");

    let mut list = vec![String::from("bir"), String::from("iki"), String::from("üç")];
    let view: &String = &list[0];
    let copy: String = list[1].clone();
    println!("bakış = {view}, kopya = {copy}");
    let taken: String = list.swap_remove(0);
    println!("alınan = {taken}, kalan = {list:?}");
}

#[derive(Default)]
pub struct Cart {
    products: Vec<String>,
    coupon: Option<String>,
}

impl Cart {
    pub fn add(&mut self, product: &str) {
        self.products.push(product.to_string());
    }

    pub fn set_coupon(&mut self, code: &str) {
        self.coupon = Some(code.to_string());
    }

    pub fn take_order(&mut self) -> Vec<String> {
        std::mem::take(&mut self.products)
    }

    pub fn use_coupon(&mut self) -> Option<String> {
        self.coupon.take()
    }

    pub fn product_count(&self) -> usize {
        self.products.len()
    }
}

fn show_take_in_place() {
    let mut cart = Cart::default();
    cart.add("klavye");
    cart.add("fare");
    cart.set_coupon("HOSGELDIN");

    let order = cart.take_order();
    println!(
        "sipariş = {order:?}, sepette kalan = {}",
        cart.product_count()
    );
    println!("kupon 1. kullanım = {:?}", cart.use_coupon());
    println!("kupon 2. kullanım = {:?}", cart.use_coupon());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_store() -> UserStore {
        let mut store = UserStore::default();
        for (id, name, is_active) in [(1, "Ada", true), (2, "Linus", false), (3, "Grace", true)] {
            store.add(User {
                id,
                name: name.to_string(),
                is_active,
            });
        }
        store
    }

    #[test]
    fn find_lends_and_store_stays_owner() {
        let store = sample_store();
        assert_eq!(store.find(3).map(|user| user.name.as_str()), Some("Grace"));
        assert!(store.find(99).is_none());
    }

    #[test]
    fn find_mut_updates_in_place() {
        let mut store = sample_store();
        store.find_mut(2).unwrap().is_active = true;
        assert_eq!(store.active_names(), ["Ada", "Linus", "Grace"]);
    }

    #[test]
    fn remove_gives_ownership_back() {
        let mut store = sample_store();
        let removed = store.remove(1).unwrap();
        assert_eq!(removed.name, "Ada");
        assert!(store.find(1).is_none());
        assert!(store.remove(1).is_none());
    }

    #[test]
    fn mem_take_leaves_empty_value() {
        let mut cart = Cart::default();
        cart.add("klavye");
        assert_eq!(cart.take_order(), ["klavye"]);
        assert_eq!(cart.product_count(), 0);
        assert!(cart.take_order().is_empty());
    }

    #[test]
    fn coupon_is_used_only_once() {
        let mut cart = Cart::default();
        cart.set_coupon("X");
        assert_eq!(cart.use_coupon().as_deref(), Some("X"));
        assert_eq!(cart.use_coupon(), None);
    }
}

// Learn more
//
// You can learn how to choose function signatures (T, &T or &mut T) from these sources:
//   - Brown University Rust Book, "Fixing Ownership Errors": https://rust-book.cs.brown.edu/ch04-03-fixing-ownership-errors.html
//   - Rust Design Patterns, "Use borrowed types for arguments": https://rust-unofficial.github.io/patterns/idioms/coercion-arguments.html
//   - Rust API Guidelines, "Flexibility": https://rust-lang.github.io/api-guidelines/flexibility.html
//   - Easy: Easy Rust, "Giving references to functions": https://dhghomon.github.io/easy_rust/Chapter_18.html
//
// You can learn partial moves from these sources:
//   - Rust by Example, "Partial moves": https://doc.rust-lang.org/rust-by-example/scope/move/partial_move.html
//   - Rust Reference, "Moved and copied types": https://doc.rust-lang.org/reference/expressions.html#moved-and-copied-types
//   - Rust error codes, "E0382": https://doc.rust-lang.org/error_codes/E0382.html
//
// You can learn how to lend values from a store from these sources:
//   - Effective Rust, "Understand the borrow checker": https://effective-rust.com/borrows.html
//   - Standard library docs, "std::vec::Vec::swap_remove": https://doc.rust-lang.org/std/vec/struct.Vec.html#method.swap_remove
//   - Brown University Rust Book, "Fixing Ownership Errors": https://rust-book.cs.brown.edu/ch04-03-fixing-ownership-errors.html
//   - Easy: Tour of Rust, "Passing Around Borrowed Data": https://tourofrust.com/51_en.html
//
// You can learn how to fix ownership errors from these sources:
//   - Brown University Rust Book, "Fixing Ownership Errors": https://rust-book.cs.brown.edu/ch04-03-fixing-ownership-errors.html
//   - Effective Rust, "Understand the borrow checker": https://effective-rust.com/borrows.html
//   - Rust error codes, "E0505": https://doc.rust-lang.org/error_codes/E0505.html
//   - Easy: Let's Get Rusty (video), "Understanding Ownership in Rust": https://www.youtube.com/watch?v=VFIOSWy93H0
//
// You can learn mem::take and Option::take from these sources:
//   - Rust Design Patterns, "mem::{take(_), replace(_)}": https://rust-unofficial.github.io/patterns/idioms/mem-replace.html
//   - Standard library docs, "std::mem::take": https://doc.rust-lang.org/std/mem/fn.take.html
//   - Too Many Lists, "Option": https://rust-unofficial.github.io/too-many-lists/second-option.html
