// Kitap: 5.1. bölüm, Enum'lar: Derinleşme

pub fn run() {
    println!("\n== Derinleşme: Enums — sipariş durum makinesi ve Option/Result ==");
    show_bad_states();
    show_state_machine();
    show_combinators();
    show_common_errors();
}

struct LooseOrder {
    is_paid: bool,
    is_shipped: bool,
    tracking_id: Option<String>,
}

impl LooseOrder {
    fn is_valid(&self) -> bool {
        let has_no_payment = self.is_shipped && !self.is_paid;
        let has_no_tracking = self.is_shipped != self.tracking_id.is_some();
        !has_no_payment && !has_no_tracking
    }
}

fn show_bad_states() {
    println!("-- 1. Geçersiz durumları temsil edilemez kıl");
    let strange_order = LooseOrder {
        is_paid: false,
        is_shipped: true,
        tracking_id: None,
    };
    println!(
        "bayraklı model derlendi, tutarlı mı? {}",
        strange_order.is_valid()
    );

    let shipped = OrderState::Shipped {
        payment_id: 4821,
        tracking_id: String::from("TR123"),
    };
    println!("enum modeli: {}", shipped.summary());
}

#[derive(Debug, Clone, PartialEq)]
enum OrderState {
    InCart,
    Paid {
        payment_id: u32,
    },
    Shipped {
        payment_id: u32,
        tracking_id: String,
    },
    Delivered {
        tracking_id: String,
    },
    Cancelled {
        reason: String,
        refund: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum Event {
    PaymentReceived { payment_id: u32 },
    PackageSent { tracking_id: String },
    Delivered,
    CancelRequested { reason: String },
}

#[derive(Debug, PartialEq)]
struct InvalidChange {
    state: OrderState,
    event: Event,
}

impl OrderState {
    fn apply(self, event: Event) -> Result<OrderState, InvalidChange> {
        use OrderState as State;
        match (self, event) {
            (State::InCart, Event::PaymentReceived { payment_id }) => {
                Ok(State::Paid { payment_id })
            }
            (State::Paid { payment_id }, Event::PackageSent { tracking_id }) => {
                Ok(State::Shipped {
                    payment_id,
                    tracking_id,
                })
            }
            (State::Shipped { tracking_id, .. }, Event::Delivered) => {
                Ok(State::Delivered { tracking_id })
            }
            (State::InCart, Event::CancelRequested { reason }) => Ok(State::Cancelled {
                reason,
                refund: None,
            }),
            (State::Paid { payment_id }, Event::CancelRequested { reason }) => {
                Ok(State::Cancelled {
                    reason,
                    refund: Some(payment_id),
                })
            }
            (state, event) => Err(InvalidChange { state, event }),
        }
    }

    fn summary(&self) -> String {
        match self {
            OrderState::InCart => String::from("sepette"),
            OrderState::Paid { payment_id } => format!("ödendi (ödeme #{payment_id})"),
            OrderState::Shipped {
                payment_id,
                tracking_id,
            } => {
                format!("kargoda (ödeme #{payment_id}, takip {tracking_id})")
            }
            OrderState::Delivered { tracking_id } => format!("teslim edildi (takip {tracking_id})"),
            OrderState::Cancelled { reason, refund } => {
                let refund =
                    refund.map_or(String::from("iade yok"), |id| format!("ödeme #{id} iade"));
                format!("iptal edildi ({reason}; {refund})")
            }
        }
    }

    fn is_done(&self) -> bool {
        matches!(
            self,
            OrderState::Delivered { .. } | OrderState::Cancelled { .. }
        )
    }
}

fn process_events(start: OrderState, events: Vec<Event>) -> OrderState {
    let mut state = start;
    for event in events {
        state = match state.apply(event) {
            Ok(new_state) => {
                println!("  → {}", new_state.summary());
                new_state
            }
            Err(InvalidChange { state, event }) => {
                println!("  ret: '{}' durumunda {event:?} işlenemez", state.summary());
                state
            }
        };
    }
    state
}

fn show_state_machine() {
    println!("-- 2. Sipariş durum makinesi");
    let last_state = process_events(
        OrderState::InCart,
        vec![
            Event::PaymentReceived { payment_id: 4821 },
            Event::PackageSent {
                tracking_id: String::from("TR123"),
            },
            Event::CancelRequested {
                reason: String::from("vazgeçtim"),
            },
            Event::Delivered,
        ],
    );
    println!("bitti mi? {}", last_state.is_done());

    let cancelled = process_events(
        OrderState::Paid { payment_id: 77 },
        vec![Event::CancelRequested {
            reason: String::from("yanlış ürün"),
        }],
    );
    println!("bitti mi? {}", cancelled.is_done());
}

#[derive(Debug, PartialEq)]
enum CouponError {
    Missing,
    BadFormat(String),
    RateTooHigh(u8),
}

struct Customer {
    name: &'static str,
    coupon: Option<&'static str>,
}

fn parse_coupon(coupon: Option<&str>) -> Result<u8, CouponError> {
    let code = coupon.ok_or(CouponError::Missing)?;
    let bad_format = || CouponError::BadFormat(code.to_string());
    let number = code.strip_prefix("IND-").ok_or_else(bad_format)?;
    let rate: u8 = number.parse().map_err(|_| bad_format())?;
    if rate > 50 {
        return Err(CouponError::RateTooHigh(rate));
    }
    Ok(rate)
}

fn find_coupon(customers: &[Customer], name: &str) -> Option<&'static str> {
    customers
        .iter()
        .find(|customer| customer.name == name)
        .and_then(|customer| customer.coupon)
}

fn calculate_price(price: u32, coupon: Option<&str>) -> u32 {
    let rate = parse_coupon(coupon).unwrap_or_else(|error| {
        if error != CouponError::Missing {
            println!("  (kupon yok sayıldı: {error:?})");
        }
        0
    });
    let discount = u64::from(price) * u64::from(rate) / 100;
    price.saturating_sub(u32::try_from(discount).unwrap_or(price))
}

fn show_combinators() {
    println!("-- 3. Option/Result kombinatörleri");
    let customers = [
        Customer {
            name: "Ada",
            coupon: Some("IND-20"),
        },
        Customer {
            name: "Can",
            coupon: Some("IND-90"),
        },
        Customer {
            name: "Ece",
            coupon: Some("YAZ"),
        },
        Customer {
            name: "Naz",
            coupon: None,
        },
    ];
    for name in ["Ada", "Can", "Ece", "Naz", "Yok"] {
        let coupon = find_coupon(&customers, name);
        println!("{name}: {coupon:?} → {} TL", calculate_price(1000, coupon));
    }

    let best_rate = customers
        .iter()
        .filter_map(|customer| parse_coupon(customer.coupon).ok())
        .max();
    let length = find_coupon(&customers, "Ada")
        .map(str::len)
        .filter(|n| *n > 3);
    println!("en iyi oran: {best_rate:?}, Ada'nın kupon uzunluğu: {length:?}");
}

struct Profile {
    nickname: Option<String>,
}

fn show_common_errors() {
    println!("-- 4. Sık yapılan hatalar");
    let some_number: Option<u32> = "5".parse().ok();

    println!(
        "(a) {:?} / {}",
        some_number.map(|n| n + 1),
        some_number.unwrap_or(0) + 1
    );

    let number: Option<u32> = Some("42").and_then(|s| s.parse().ok());
    println!("(b) {number:?}");

    println!("(c) {:?}", parse_coupon(None));

    let profiles = [
        Profile {
            nickname: Some(String::from("kbo")),
        },
        Profile { nickname: None },
    ];
    let names: Vec<&str> = profiles
        .iter()
        .map(|p| p.nickname.as_deref().unwrap_or("anonim"))
        .collect();
    println!("(d) {names:?}");

    let state = OrderState::Shipped {
        payment_id: 1,
        tracking_id: String::from("TR9"),
    };
    if let OrderState::Shipped { tracking_id, .. } = &state {
        println!(
            "(e) takip: {tracking_id}, durum hâlâ bizde: {}",
            state.summary()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discount_does_not_overflow_on_big_price() {
        assert_eq!(
            calculate_price(4_000_000_000, Some("IND-50")),
            2_000_000_000
        );
        assert_eq!(calculate_price(u32::MAX, None), u32::MAX);
    }

    #[test]
    fn valid_events_end_with_delivery() {
        let state = OrderState::InCart
            .apply(Event::PaymentReceived { payment_id: 7 })
            .and_then(|state| {
                state.apply(Event::PackageSent {
                    tracking_id: String::from("TR1"),
                })
            })
            .and_then(|state| state.apply(Event::Delivered))
            .unwrap();
        assert_eq!(
            state,
            OrderState::Delivered {
                tracking_id: String::from("TR1")
            }
        );
        assert!(state.is_done());
    }

    #[test]
    fn invalid_change_gives_state_back() {
        let error = OrderState::InCart.apply(Event::Delivered).unwrap_err();
        assert_eq!(error.state, OrderState::InCart);
        assert_eq!(error.event, Event::Delivered);
        assert!(!error.state.is_done());
    }

    #[test]
    fn cancelling_paid_order_creates_refund() {
        let cancel = |state: OrderState| {
            state.apply(Event::CancelRequested {
                reason: String::from("x"),
            })
        };
        let paid = cancel(OrderState::Paid { payment_id: 9 }).unwrap();
        assert_eq!(paid.summary(), "iptal edildi (x; ödeme #9 iade)");
        let in_cart = cancel(OrderState::InCart).unwrap();
        assert_eq!(in_cart.summary(), "iptal edildi (x; iade yok)");
    }

    #[test]
    fn coupon_parser_separates_each_error() {
        assert_eq!(parse_coupon(Some("IND-20")), Ok(20));
        assert_eq!(parse_coupon(None), Err(CouponError::Missing));
        assert_eq!(
            parse_coupon(Some("IND-90")),
            Err(CouponError::RateTooHigh(90))
        );
        assert_eq!(
            parse_coupon(Some("YAZ")),
            Err(CouponError::BadFormat(String::from("YAZ")))
        );
        assert_eq!(
            parse_coupon(Some("IND-yirmi")),
            Err(CouponError::BadFormat(String::from("IND-yirmi")))
        );
    }

    #[test]
    fn bad_coupon_does_not_stop_the_sale() {
        assert_eq!(calculate_price(1000, Some("IND-20")), 800);
        assert_eq!(calculate_price(1000, Some("IND-90")), 1000);
        assert_eq!(calculate_price(1000, None), 1000);
    }

    #[test]
    fn flag_model_allows_invalid_values() {
        let strange_order = LooseOrder {
            is_paid: false,
            is_shipped: true,
            tracking_id: None,
        };
        assert!(!strange_order.is_valid());
    }
}
