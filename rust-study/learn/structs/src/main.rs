// Kitap: 4. bölüm, Struct'lar (Yapılar)

#[derive(Debug)]
struct User {
    name: String,
    email: String,
    is_active: bool,
    login_count: u64,
}

struct Quote<'a> {
    text: &'a str,
}

struct Color(u8, u8, u8);

struct Unit;

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    fn area(&self) -> u32 {
        self.width * self.height
    }
    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }
    fn into_sides(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

mod deep_dive;

fn main() {
    show_define_and_create();
    show_update_syntax();
    show_tuple_and_unit();
    show_owned_fields();
    show_methods();
    show_debug_output();
    deep_dive::run();
}

fn create_user(name: String, email: String) -> User {
    User {
        name,
        email,
        is_active: true,
        login_count: 1,
    }
}

fn show_define_and_create() {
    let mut user = create_user("Kaan".into(), "k@x.com".into());
    user.login_count += 1;
    println!("{user:?}");
    println!(
        "{} giriş: {} aktif: {}",
        user.name, user.login_count, user.is_active
    );
}

fn show_update_syntax() {
    let user1 = create_user("Ada".into(), "a@x.com".into());
    let user2 = User {
        name: "Ada II".into(),
        ..user1
    };
    println!("{} / {}", user2.name, user2.email);
}

fn show_tuple_and_unit() {
    let black = Color(0, 0, 0);
    println!("R={} G={} B={}", black.0, black.1, black.2);

    let _unit = Unit;
}

fn show_owned_fields() {
    let s = String::from("merhaba dünya");
    let quote = Quote { text: &s[0..7] };
    println!("alıntı: {}", quote.text);
}

fn show_methods() {
    let mut rectangle = Rectangle::new(3, 4);
    println!("alan = {}", rectangle.area());
    rectangle.scale(2);
    println!("ölçekli alan = {} ({:?})", rectangle.area(), rectangle);

    let (width, height) = rectangle.into_sides();
    println!("yok edildi: {width}x{height}");
}

fn show_debug_output() {
    let rectangle = Rectangle::new(6, 7);
    println!("{rectangle:?}");
    println!("{rectangle:#?}");
}
