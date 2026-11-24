// Kitap: 3. bölüm, Ownership (Sahiplik)

mod deep_dive;

fn main() {
    show_ownership_and_move();
    show_copy_and_move();
    show_clone();
    show_scope_and_drop();
    show_shared_borrow();
    show_mutable_borrow();
    show_no_dangling();
    show_slices();
    deep_dive::run();
}

fn show_ownership_and_move() {
    let s1 = String::from("merhaba");
    let s2 = s1;
    println!("s2 = {s2}");
}

fn show_copy_and_move() {
    let x = 5;
    let y = x;
    println!("x = {x}, y = {y}");
}

fn show_clone() {
    let s1 = String::from("ada");
    let s2 = s1.clone();
    println!("s1 = {s1}, s2 = {s2}");
}

fn show_scope_and_drop() {
    {
        let s = String::from("geçici");
        println!("içeride: {s}");
    }
}

fn show_shared_borrow() {
    let s = String::from("kitap");
    let n = calculate_length(&s);
    println!("'{s}' uzunluğu {n}");
}
fn calculate_length(x: &String) -> usize {
    x.len()
}

fn show_mutable_borrow() {
    let mut s = String::from("merhaba");
    add_world(&mut s);
    println!("{s}");

    let r1 = &s;
    let r2 = &s;
    println!("{r1} {r2}");

    let r3 = &mut s;
    r3.push('!');
    println!("{r3}");
}
fn add_world(x: &mut String) {
    x.push_str(" dünya");
}

fn show_no_dangling() {
    let s = make_owned();
    println!("sarkmayan: {s}");
}
fn make_owned() -> String {
    String::from("x")
}

fn show_slices() {
    let s = String::from("merhaba dünya");
    let first = first_word(&s);
    println!("ilk kelime: {first}");

    let array = [1, 2, 3, 4, 5];
    let middle = &array[1..4];
    println!("orta = {middle:?}");
}
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}
