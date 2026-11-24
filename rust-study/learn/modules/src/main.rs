// Kitap: 6. bölüm, Modüller

mod restaurant {
    pub mod dining_room {
        pub fn set_table() {
            println!("salon: masa ayarlandı");
            super::kitchen::prepare_food();
        }
    }

    mod kitchen {
        pub fn prepare_food() {
            println!("mutfak: hazır");
        }
    }

    pub fn open() {
        self::dining_room::set_table();
        kitchen::prepare_food();
    }

    pub(crate) fn print_inner_report() {
        println!("lokanta: iç rapor (yalnız crate içi)");
    }
}

mod order {
    #[derive(Debug)]
    pub struct Receipt {
        pub product: String,
        price: u32,
    }

    impl Receipt {
        pub fn new(product: &str, price: u32) -> Receipt {
            Receipt {
                product: product.to_string(),
                price,
            }
        }
        pub fn price(&self) -> u32 {
            self.price
        }
    }

    pub fn demo() {
        let receipt = Receipt::new("çay", 15);
        println!("fiş: {} = {}", receipt.product, receipt.price());
    }
}

mod deep {
    pub mod deeper {
        pub mod inner {
            pub fn do_work() {
                println!("deep::deeper::inner::do_work çalıştı");
            }
        }
    }
}
mod front {
    pub use crate::deep::deeper::inner::do_work;
    pub fn demo() {
        do_work();
    }
}

use restaurant::dining_room;
use restaurant::dining_room::set_table as prepare_table;

fn main() {
    crate::restaurant::open();

    dining_room::set_table();
    prepare_table();

    restaurant::print_inner_report();
    order::demo();
    front::demo();
    front::do_work();
}

// Learn more
//
// You can learn modules from these sources:
//   - The Rust Book, "Control Scope and Privacy with Modules": https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html
//   - Rust by Example, "Modules": https://doc.rust-lang.org/rust-by-example/mod.html
//   - Comprehensive Rust, "Modules": https://google.github.io/comprehensive-rust/modules/modules.html
//   - Easy: Programiz, "Rust Module": https://www.programiz.com/rust/module
//   - Easy: Let's Get Rusty (video), "Rust's Module System Explained!": https://www.youtube.com/watch?v=5RPXgDQrjio
//
// You can learn paths, self and super from these sources:
//   - The Rust Book, "Paths for Referring to an Item in the Module Tree": https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html
//   - Rust by Example, "super and self": https://doc.rust-lang.org/rust-by-example/mod/super.html
//   - Rust Reference, "Paths": https://doc.rust-lang.org/reference/paths.html
//   - Easy: Tour of Rust, "Internal Module Referencing": https://tourofrust.com/114_en.html
//
// You can learn use and re-exports from these sources:
//   - The Rust Book, "Bringing Paths Into Scope with the use Keyword": https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html
//   - Rust by Example, "The use declaration": https://doc.rust-lang.org/rust-by-example/mod/use.html
//   - Rust Reference, "Use declarations": https://doc.rust-lang.org/reference/items/use-declarations.html
//   - Easy: Easy Rust, "Crates and modules": https://dhghomon.github.io/easy_rust/Chapter_57.html
//
// You can learn visibility (pub, pub(crate), private fields) from these sources:
//   - Rust Reference, "Visibility and privacy": https://doc.rust-lang.org/reference/visibility-and-privacy.html
//   - Rust by Example, "Struct visibility": https://doc.rust-lang.org/rust-by-example/mod/struct_visibility.html
//   - Effective Rust, "Minimize visibility": https://effective-rust.com/visibility.html
//   - Easy: Tour of Rust, "Structure Visibility": https://tourofrust.com/116_en.html
