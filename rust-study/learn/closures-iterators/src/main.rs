// Kitap: 12. bölüm, Closure ve Iterator'lar

mod deep_dive;

fn main() {
    show_closures();
    show_iterators();
    show_own_iterator();
    deep_dive::run();
}

fn show_closures() {
    let square = |x: i32| x * x;
    println!("square(5) = {}", square(5));

    let list = vec![1, 2, 3];
    let print_list = || println!("yakaladı (&): {list:?}");
    print_list();
    print_list();
    println!("liste hâlâ burada: {list:?}");

    let mut numbers = vec![1, 2, 3];
    let mut add = || numbers.push(4);
    add();
    println!("ekle sonrası: {numbers:?}");

    let owner = String::from("sahiplendim");
    let print_owner = move || println!("move ile: {owner}");
    print_owner();

    call_fn(|| println!("Fn: sadece okur"));
    let n = run_once(|| 7 * 6);
    println!("FnOnce döndürdü: {n}");
}

fn call_fn<F: Fn()>(f: F) {
    f();
    f();
}

fn run_once<F: FnOnce() -> i32>(f: F) -> i32 {
    f()
}

fn show_iterators() {
    let v = vec![1, 2, 3, 4, 5];

    let mut it = v.iter();
    println!("next: {:?} {:?}", it.next(), it.next());

    let result: Vec<i32> = v.iter().map(|x| x * 2).filter(|x| x % 3 != 0).collect();
    println!("map+filter+collect: {result:?}");

    let total: i32 = v.iter().sum();
    let sum_of_squares: i32 = v.iter().map(|x| x * x).sum();
    println!("toplam = {total}, kareler toplamı = {sum_of_squares}");

    let mut m = vec![10, 20, 30];
    for x in m.iter_mut() {
        *x += 1;
    }
    println!("iter_mut sonrası: {m:?}");
    let owned: Vec<String> = vec![String::from("a"), String::from("b")]
        .into_iter()
        .map(|s| s + "!")
        .collect();
    println!("into_iter: {owned:?}");
}

struct Counter {
    number: u32,
}
impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.number < 5 {
            self.number += 1;
            Some(self.number)
        } else {
            None
        }
    }
}
fn show_own_iterator() {
    let total: u32 = Counter { number: 0 }
        .zip(Counter { number: 0 }.skip(1))
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    println!("kendi iterator zinciri = {total}");
}
