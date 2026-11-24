// Kitap: 12.1. bölüm, Closure ve Iterator'lar: Derinleşme

use std::collections::{BTreeSet, HashMap};

pub fn run() {
    println!("\n== Derinleşme: Closures & Iterators ==");
    show_body_sets_trait();
    show_return_and_store();
    show_data_pipeline();
    show_slice_adapters();
    show_endless_iterator();
    show_lazy_and_collect();
    show_common_errors();
}

fn call_twice(f: impl Fn() -> usize) -> usize {
    f() + f()
}
fn call_three_times(mut f: impl FnMut() -> u32) -> u32 {
    f();
    f();
    f()
}
fn call_once(f: impl FnOnce() -> String) -> String {
    f()
}

fn show_body_sets_trait() {
    println!("-- 1. gövde trait'i belirler --");
    let name = String::from("Ayşe");
    let length = move || name.chars().count();
    println!("move + okuma  → Fn     : {}", call_twice(length));

    let mut counter = 0;
    let increase = || {
        counter += 1;
        counter
    };
    println!("değiştirme    → FnMut  : {}", call_three_times(increase));

    let note = String::from("teslim edildi");
    let give_note = || note;
    println!("dışarı taşıma → FnOnce : {}", call_once(give_note));
}

#[derive(Debug, Clone, PartialEq)]
struct Order {
    id: u32,
    customer: &'static str,
    city: &'static str,
    price: u32,
    is_delivered: bool,
}

fn at_least(limit: u32) -> impl Fn(&Order) -> bool {
    move |order| order.price >= limit
}

fn choose_discount(code: &str) -> Box<dyn Fn(u32) -> u32> {
    match code.split_once(':') {
        Some(("yuzde", n)) => {
            let rate: u32 = n.parse().unwrap_or(0).min(100);
            Box::new(move |price| {
                let discount = u64::from(price) * u64::from(rate) / 100;
                price.saturating_sub(u32::try_from(discount).unwrap_or(price))
            })
        }
        Some(("sabit", n)) => {
            let amount: u32 = n.parse().unwrap_or(0);
            Box::new(move |price| price.saturating_sub(amount))
        }
        _ => Box::new(|price| price),
    }
}

struct PriceCalculator<F: Fn(u32) -> u32> {
    rule: F,
}
impl<F: Fn(u32) -> u32> PriceCalculator<F> {
    fn apply(&self, price: u32) -> u32 {
        (self.rule)(price)
    }
}

type Rule = Box<dyn Fn(&Order) -> bool>;
struct RuleSet {
    rules: Vec<(&'static str, Rule)>,
}
impl RuleSet {
    fn add(mut self, name: &'static str, rule: impl Fn(&Order) -> bool + 'static) -> Self {
        self.rules.push((name, Box::new(rule)));
        self
    }
    fn broken_rules(&self, order: &Order) -> Vec<&'static str> {
        let broken_rule = |(name, rule): &(&'static str, Rule)| (!rule(order)).then_some(*name);
        self.rules.iter().filter_map(broken_rule).collect()
    }
}

fn show_return_and_store() {
    println!("-- 2. closure döndürme ve saklama --");
    let orders = read_orders(RAW_RECORDS);
    let big_orders: Vec<u32> = orders
        .iter()
        .filter(|order| at_least(200)(order))
        .map(|order| order.id)
        .collect();
    println!("at_least(200) → {big_orders:?}");
    let discounted: Vec<u32> = ["yuzde:10", "sabit:300", "yok"]
        .iter()
        .map(|code| choose_discount(code)(250))
        .collect();
    println!("choose_discount → {discounted:?}");

    let tax = PriceCalculator {
        rule: |price| price + price / 5,
    };
    let max_price = 300;
    let rules = RuleSet { rules: Vec::new() }
        .add("teslim edilmemiş", |order| order.is_delivered)
        .add("limit aşımı", move |order| order.price <= max_price);
    println!("tax(250) = {}", tax.apply(250));
    for order in orders
        .iter()
        .filter(|order| !rules.broken_rules(order).is_empty())
    {
        println!("  #{} {:?}", order.id, rules.broken_rules(order));
    }
}

const RAW_RECORDS: &str = "1;Ayşe;İzmir;250;evet\n2;Can;Ankara;90;hayır\nbozuk satır\n\
                            3;Ayşe;İzmir;40;evet\n4;Deniz;Bursa;310;evet\n\
                            5;Can;Ankara;120;evet\n6;Ece;İzmir;75;hayır";

fn parse_order(line: &'static str) -> Option<Order> {
    let mut fields = line.split(';');
    Some(Order {
        id: fields.next()?.parse().ok()?,
        customer: fields.next()?,
        city: fields.next()?,
        price: fields.next()?.parse().ok()?,
        is_delivered: fields.next()? == "evet",
    })
}
fn read_orders(raw: &'static str) -> Vec<Order> {
    raw.lines().filter_map(parse_order).collect()
}

fn sales_by_city(orders: &[Order]) -> Vec<(&'static str, u32)> {
    let mut groups: HashMap<&'static str, u32> = HashMap::new();
    for order in orders {
        *groups.entry(order.city).or_insert(0) += order.price;
    }
    let mut sorted: Vec<_> = groups.into_iter().collect();
    sorted.sort_by_key(|&(city, sales)| (std::cmp::Reverse(sales), city));
    sorted
}

fn show_data_pipeline() {
    println!("-- 3. veri hattı --");
    let orders = read_orders(RAW_RECORDS);
    println!(
        "okunan: {} (bozuk satır filter_map ile elendi)",
        orders.len()
    );

    let delivered_sales: u32 = orders
        .iter()
        .filter(|order| order.is_delivered)
        .map(|order| order.price)
        .sum();
    let (count, total) = orders.iter().fold((0, 0), |(count, total), order| {
        (count + 1, total + order.price)
    });
    println!(
        "teslim cirosu = {delivered_sales}, ortalama = {}",
        total.checked_div(count).unwrap_or(0)
    );

    println!("şehir ciroları = {:?}", sales_by_city(&orders));

    let smallest = orders
        .iter()
        .min_by_key(|order| order.price)
        .map(|order| order.id);
    let best_score = [4.5, 3.8, 4.9]
        .into_iter()
        .max_by(|a: &f64, b| a.total_cmp(b));
    println!("en küçük sipariş = {smallest:?}, en iyi puan = {best_score:?}");

    let (delivered, waiting): (Vec<&Order>, Vec<&Order>) =
        orders.iter().partition(|order| order.is_delivered);
    println!("teslim/bekleyen = {}/{}", delivered.len(), waiting.len());

    let letters: BTreeSet<char> = orders
        .iter()
        .flat_map(|order| order.customer.chars().take(1))
        .collect();
    println!("müşteri baş harfleri = {letters:?}");
}

fn group_neighbors<'a>(items: &[&'a str]) -> Vec<(&'a str, usize)> {
    let mut stream = items.iter().copied().peekable();
    let mut groups = Vec::new();
    while let Some(item) = stream.next() {
        let mut count = 1;
        while stream.next_if_eq(&item).is_some() {
            count += 1;
        }
        groups.push((item, count));
    }
    groups
}

fn show_slice_adapters() {
    println!("-- 4. konum ve komşu adaptörleri --");
    let daily_sales = [120, 90, 150, 150, 80, 200];
    let targets = [100, 100, 160];
    let good_days: Vec<usize> = daily_sales
        .iter()
        .zip(targets)
        .enumerate()
        .filter(|(_, (sales, target))| *sales >= target)
        .map(|(day, _)| day + 1)
        .collect();
    println!("hedefi tutan günler = {good_days:?}");
    let first_run: Vec<_> = daily_sales
        .iter()
        .take_while(|sales| **sales >= 90)
        .collect();
    println!("90 altına düşene kadar = {first_run:?}");
    let rising_days = daily_sales
        .windows(2)
        .filter(|window| window[1] > window[0])
        .count();
    let package_totals: Vec<i32> = daily_sales
        .chunks(4)
        .map(|package| package.iter().sum())
        .collect();
    println!("artış olan gün = {rising_days}, 4'lü paket toplamları = {package_totals:?}");
    let log_levels = ["BİLGİ", "BİLGİ", "HATA", "HATA", "HATA", "BİLGİ"];
    println!("ardışık gruplar = {:?}", group_neighbors(&log_levels));
}

struct Fibonacci {
    current: u64,
    next: u64,
}
impl Fibonacci {
    fn new() -> Self {
        Fibonacci {
            current: 0,
            next: 1,
        }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let new_next = self.current.checked_add(self.next)?;
        let result = std::mem::replace(&mut self.current, self.next);
        self.next = new_next;
        Some(result)
    }
}

fn show_endless_iterator() {
    println!("-- 5. kendi iterator'un: Fibonacci --");
    let first_ten: Vec<u64> = Fibonacci::new().take(10).collect();
    let even_total: u64 = Fibonacci::new()
        .take_while(|&x| x < 100)
        .filter(|x| x % 2 == 0)
        .sum();
    let first_over_thousand = Fibonacci::new().enumerate().find(|&(_, x)| x > 1000);
    println!("ilk 10 = {first_ten:?}");
    println!(
        "100'den küçük çiftlerin toplamı = {even_total}, 1000'i aşan ilk = {first_over_thousand:?}"
    );
    println!(
        "taşmadan önce üretilen terim sayısı = {}",
        Fibonacci::new().count()
    );
}

fn show_lazy_and_collect() {
    println!("-- 6. sahiplik, tembellik, collect --");
    let names = vec![String::from("ece"), String::from("can")];
    let long_name_count = names
        .iter()
        .filter(|name: &&String| name.len() >= 3)
        .count();
    let big: Vec<String> = names.into_iter().map(|name| name.to_uppercase()).collect();
    println!("uzunlar = {long_name_count}, büyük = {big:?}");

    let lazy = [1, 2, 3, 4]
        .iter()
        .inspect(|x| println!("  işleniyor: {x}"))
        .map(|x| x * 10);
    println!("zincir kuruldu (henüz işlenen yok)");
    let first_two: Vec<i32> = lazy.take(2).collect();
    println!("first_two = {first_two:?}");

    let squares: Vec<u32> = (1..=4).map(|x| x * x).collect();
    let text = (1..=4).map(|x| x.to_string()).collect::<Vec<_>>().join("-");
    let letters: String = ["r", "u", "s", "t"].into_iter().collect();
    println!("kareler = {squares:?}, metin = {text}, harfler = {letters}");
}

fn show_common_errors() {
    println!("-- 7. sık hatalar: doğru kullanım --");
    let label = String::from("acil");
    let give_note = || label.clone();
    println!("(a) {} {}", give_note(), give_note());

    let mut n = 0;
    call_three_times(|| {
        n += 10;
        n
    });
    println!("(b) n = {n}");

    let mut v = vec![1, 2, 3];
    let mut add = |x| v.push(x);
    add(4);
    add(5);
    println!("(c) uzunluk = {}", v.len());

    let add_to_start = |target: &mut Vec<i32>| target.insert(0, 0);
    let add_to_end = |target: &mut Vec<i32>| target.push(6);
    add_to_start(&mut v);
    add_to_end(&mut v);
    println!("(d) v = {v:?}");

    println!(
        "(e) at_least(100)(price=250) = {}",
        at_least(100)(&read_orders(RAW_RECORDS)[0])
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discount_does_not_panic_on_edge_cases() {
        assert_eq!(choose_discount("yuzde:150")(250), 0);
        assert_eq!(choose_discount("yuzde:abc")(250), 250);
        assert_eq!(
            choose_discount("yuzde:50")(u32::MAX),
            u32::MAX - u32::MAX / 2
        );
    }

    #[test]
    fn broken_lines_are_skipped() {
        let orders = read_orders(RAW_RECORDS);
        assert_eq!(orders.len(), 6);
        assert_eq!(orders[0].customer, "Ayşe");
        assert_eq!(parse_order("7;Ali;Van;abc;evet"), None);
    }

    #[test]
    fn city_sales_are_sorted_high_to_low() {
        let city_sales = sales_by_city(&read_orders(RAW_RECORDS));
        assert_eq!(
            city_sales,
            vec![("İzmir", 365), ("Bursa", 310), ("Ankara", 210)]
        );
    }

    #[test]
    fn discount_codes_return_different_closures() {
        assert_eq!(choose_discount("yuzde:20")(200), 160);
        assert_eq!(choose_discount("sabit:50")(30), 0);
        assert_eq!(choose_discount("bilinmeyen")(30), 30);
    }

    #[test]
    fn rule_set_returns_broken_rule_names() {
        let rules = RuleSet { rules: Vec::new() }
            .add("teslim", |order| order.is_delivered)
            .add("limit", |order| order.price <= 100);
        let orders = read_orders(RAW_RECORDS);
        assert_eq!(rules.broken_rules(&orders[1]), vec!["teslim"]);
        assert_eq!(rules.broken_rules(&orders[2]), Vec::<&str>::new());
        assert_eq!(rules.broken_rules(&orders[3]), vec!["limit"]);
    }

    #[test]
    fn grouping_keeps_the_edge_item() {
        assert_eq!(
            group_neighbors(&["a", "a", "b", "a"]),
            vec![("a", 2), ("b", 1), ("a", 1)]
        );
        assert!(group_neighbors(&[]).is_empty());
    }

    #[test]
    fn fibonacci_ends_without_panic_on_overflow() {
        assert_eq!(Fibonacci::new().nth(10), Some(55));
        let mut stream = Fibonacci::new();
        assert_eq!(stream.by_ref().count(), 92);
        assert_eq!(stream.next(), None);
    }
}

// Learn more
//
// You can learn Fn, FnMut and FnOnce from these sources:
//   - Comprehensive Rust, "Closure Traits": https://google.github.io/comprehensive-rust/closures/traits.html
//   - Huon Wilson, "Finding Closure in Rust": https://huonw.github.io/blog/2015/05/finding-closure-in-rust/
//   - Rust by Example, "As input parameters": https://doc.rust-lang.org/rust-by-example/fn/closures/input_parameters.html
//   - Easy: DEV Community, "Rust Closures — Why Are There Three? Fn, FnMut, and FnOnce Finally Explained": https://dev.to/divyesh_kakadiya/rust-closures-why-are-there-three-fn-fnmut-and-fnonce-finally-explained-1mhp
//
// You can learn how to return and store closures from these sources:
//   - The Rust Book, "Returning Closures": https://doc.rust-lang.org/book/ch20-04-advanced-functions-and-closures.html#returning-closures
//   - Rust by Example, "As output parameters": https://doc.rust-lang.org/rust-by-example/fn/closures/output_parameters.html
//   - Crust of Rust (video), "Functions, Closures, and Their Traits": https://www.youtube.com/watch?v=dHkzSZnYXmk
//   - Easy: Let's Get Rusty (video), "Advanced Function and Closures in Rust": https://www.youtube.com/watch?v=rnxutl7t1hI
//
// You can learn data pipelines with iterators from these sources:
//   - Effective Rust, "Consider using iterator transforms instead of explicit loops": https://effective-rust.com/iterators.html
//   - The Rust Book, "Methods That Produce Other Iterators": https://doc.rust-lang.org/book/ch13-02-iterators.html#methods-that-produce-other-iterators
//   - Comprehensive Rust, "Iterator Helper Methods": https://google.github.io/comprehensive-rust/iterators/helpers.html
//   - Easy: Easy Rust, "Chaining methods": https://dhghomon.github.io/easy_rust/Chapter_35.html
//
// You can learn slice helpers (windows, chunks, chunk_by) from these sources:
//   - Standard library docs, "slice::chunk_by": https://doc.rust-lang.org/std/primitive.slice.html#method.chunk_by
//   - Standard library docs, "slice::windows": https://doc.rust-lang.org/std/primitive.slice.html#method.windows
//   - Rust Performance Book, "Iterators": https://nnethercote.github.io/perf-book/iterators.html
//   - Easy: Easy Rust, "Closures": https://dhghomon.github.io/easy_rust/Chapter_37.html
//
// You can learn endless iterators from these sources:
//   - Standard library docs, "Infinity": https://doc.rust-lang.org/std/iter/index.html#infinity
//   - Rust by Example, "Iterators": https://doc.rust-lang.org/rust-by-example/trait/iter.html
//   - Standard library docs, "std::iter::successors": https://doc.rust-lang.org/std/iter/fn.successors.html
//   - Easy: Easy Rust, "Iterators": https://dhghomon.github.io/easy_rust/Chapter_36.html
//
// You can learn lazy iterators and collect from these sources:
//   - Standard library docs, "Laziness": https://doc.rust-lang.org/std/iter/index.html#laziness
//   - The Rust Book, "Processing a Series of Items with Iterators": https://doc.rust-lang.org/book/ch13-02-iterators.html
//   - Comprehensive Rust, "collect": https://google.github.io/comprehensive-rust/iterators/collect.html
//   - Easy: Easy Rust, "Iterators": https://dhghomon.github.io/easy_rust/Chapter_36.html
//
// You can learn common closure errors from these sources:
//   - Rust error codes, "E0373": https://doc.rust-lang.org/error_codes/E0373.html
//   - Rust error codes, "E0525": https://doc.rust-lang.org/error_codes/E0525.html
//   - Comprehensive Rust, "Capturing": https://google.github.io/comprehensive-rust/closures/capturing.html
//   - Easy: DEV Community, "Rust Closures — Why Are There Three? Fn, FnMut, and FnOnce Finally Explained": https://dev.to/divyesh_kakadiya/rust-closures-why-are-there-three-fn-fnmut-and-fnonce-finally-explained-1mhp
