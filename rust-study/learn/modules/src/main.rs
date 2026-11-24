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
