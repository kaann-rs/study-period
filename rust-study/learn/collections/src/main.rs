// Kitap: 7. bölüm, Koleksiyonlar

use std::collections::HashMap;

fn main() {
    show_vectors();
    show_strings();
    show_hash_maps();
}

fn show_vectors() {
    let mut v: Vec<i32> = Vec::new();
    v.push(10);
    v.push(20);
    v.push(30);
    let v2 = vec![1, 2, 3];

    println!("v[0] = {}", v[0]);
    match v.get(99) {
        Some(x) => println!("{x}"),
        None => println!("99. eleman yok (get → Option, güvenli)"),
    }

    let mut total = 0;
    for x in &v {
        total += x;
    }
    println!("toplam = {total}");
    for x in &mut v {
        *x += 1;
    }
    println!("{v:?} {v2:?}");

    #[derive(Debug)]
    enum SheetCell {
        Number(i32),
        Text(String),
        Empty,
    }
    let row = vec![
        SheetCell::Number(3),
        SheetCell::Text(String::from("merhaba")),
        SheetCell::Empty,
    ];
    println!("satır = {row:?}");
    for cell in &row {
        match cell {
            SheetCell::Number(number) => println!("  sayı: {number}"),
            SheetCell::Text(text) => println!("  metin: {text}"),
            SheetCell::Empty => println!("  boş"),
        }
    }
}

fn show_strings() {
    let mut s = String::from("mer");
    s.push_str("haba");
    s.push('!');
    let world = String::from("dünya");

    let joined = s.clone() + " " + &world;
    let formatted = format!("{s} {world}");
    println!("{joined} / {formatted}");

    let word = "şema";
    println!(
        "'{word}': bytes={} chars={}",
        word.len(),
        word.chars().count()
    );

    let first_char: String = word.chars().take(1).collect();
    println!("ilk karakter: {first_char}");
    print!("'ab' byte'ları: ");
    for byte in "ab".bytes() {
        print!("{byte} ");
    }
    println!();
}

fn show_hash_maps() {
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert(String::from("mavi"), 10);
    scores.insert(String::from("kirmizi"), 50);

    match scores.get("mavi") {
        Some(v) => println!("mavi = {v}"),
        None => println!("yok"),
    }

    *scores.entry(String::from("mavi")).or_insert(0) += 1;
    scores.entry(String::from("sari")).or_insert(7);

    let text = "a b a c b a";
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    let mut pairs: Vec<_> = counts.iter().collect();
    pairs.sort();
    println!("kelime sayımı: {pairs:?}");
    println!("puan tablosu eleman sayısı: {}", scores.len());
}
