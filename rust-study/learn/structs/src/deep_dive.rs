// Kitap: 4.1. bölüm, Struct'lar: Derinleşme

pub fn run() {
    println!("\n== Derinleşme: Structs — kullanıcı yönetimi senaryosu ==");
    show_valid_state();
    show_builder();
    show_self_choice();
    show_update_trap();
}

mod account {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Email(String);

    impl Email {
        pub fn parse(raw: &str) -> Result<Email, String> {
            let raw = raw.trim();
            let is_valid = raw.split_once('@').is_some_and(|(user_part, domain)| {
                !user_part.is_empty()
                    && domain.contains('.')
                    && !domain.contains('@')
                    && !raw.contains(char::is_whitespace)
            });
            if is_valid {
                Ok(Email(raw.to_lowercase()))
            } else {
                Err(format!("geçersiz e-posta: '{raw}'"))
            }
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }

        pub fn domain(&self) -> &str {
            self.0.split_once('@').map_or("", |(_, domain)| domain)
        }
    }

    #[derive(Debug)]
    pub struct User {
        name: String,
        email: Email,
        is_admin: bool,
        login_count: u32,
    }

    #[derive(Debug, Default)]
    pub struct UserBuilder {
        name: Option<String>,
        email: Option<String>,
        is_admin: bool,
    }

    impl UserBuilder {
        pub fn name(mut self, name: &str) -> Self {
            self.name = Some(name.trim().to_string());
            self
        }
        pub fn email(mut self, email: &str) -> Self {
            self.email = Some(email.to_string());
            self
        }
        pub fn admin(mut self, is_admin: bool) -> Self {
            self.is_admin = is_admin;
            self
        }
        pub fn build(self) -> Result<User, String> {
            let name = self
                .name
                .filter(|name| !name.is_empty())
                .ok_or_else(|| String::from("ad zorunlu"))?;
            let raw = self.email.ok_or_else(|| String::from("e-posta zorunlu"))?;
            Ok(User {
                name,
                email: Email::parse(&raw)?,
                is_admin: self.is_admin,
                login_count: 0,
            })
        }
    }

    impl User {
        pub fn builder() -> UserBuilder {
            UserBuilder::default()
        }
        pub fn name(&self) -> &str {
            &self.name
        }
        pub fn email(&self) -> &Email {
            &self.email
        }
        pub fn login_count(&self) -> u32 {
            self.login_count
        }
        pub fn summary(&self) -> String {
            let role = if self.is_admin { "yönetici" } else { "üye" };
            let User {
                name,
                email,
                login_count,
                ..
            } = self;
            format!("{name} <{}> [{role}], {login_count} giriş", email.as_str())
        }
        pub fn add_login(&mut self) {
            self.login_count += 1;
        }
        pub fn into_inactive(self) -> InactiveUser {
            InactiveUser {
                name: self.name,
                email: self.email,
                last_login_count: self.login_count,
            }
        }
    }

    #[derive(Debug)]
    pub struct InactiveUser {
        name: String,
        email: Email,
        last_login_count: u32,
    }

    impl InactiveUser {
        pub fn summary(&self) -> String {
            format!(
                "[pasif] {} <{}>, son giriş sayısı {}",
                self.name,
                self.email.as_str(),
                self.last_login_count
            )
        }
        pub fn into_active(self) -> User {
            User {
                name: self.name,
                email: self.email,
                is_admin: false,
                login_count: 0,
            }
        }
    }
}

use account::{Email, User};

fn show_valid_state() {
    println!("-- 1. Alan gizliliği ile değişmez koruma");
    for raw in ["  Kaan@Ornek.COM ", "kaan.ornek.com", "@ornek.com"] {
        match Email::parse(raw) {
            Ok(email) => println!("kabul: {} (alan adı: {})", email.as_str(), email.domain()),
            Err(error) => println!("ret: {error}"),
        }
    }
}

fn show_builder() {
    println!("-- 2. Builder kalıbı");
    let kaan = User::builder()
        .name("Kaan")
        .email("Kaan@Ornek.com")
        .admin(true)
        .build();
    let without_email = User::builder().name("Ada").build();
    let broken = User::builder().name("Ada").email("ada(at)ornek").build();
    for result in [kaan, without_email, broken] {
        match result {
            Ok(user) => println!("kuruldu: {}", user.summary()),
            Err(error) => println!("kurulamadı: {error}"),
        }
    }
}

fn show_self_choice() {
    println!("-- 3. Receiver seçimi = sahiplik sözleşmesi");
    let Ok(mut ada) = User::builder().name("Ada").email("ada@ornek.com").build() else {
        return;
    };
    ada.add_login();
    ada.add_login();
    println!("{} | {} harf", ada.summary(), ada.name().chars().count());
    println!(
        "{} girişin alan adı: {}",
        ada.login_count(),
        ada.email().domain()
    );

    let inactive = ada.into_inactive();
    println!("{}", inactive.summary());

    let active_again = inactive.into_active();
    println!("{}", active_again.summary());
}

#[derive(Debug, Clone, PartialEq)]
struct Settings {
    theme: String,
    language: String,
    has_notifications: bool,
    page_size: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: String::from("açık"),
            language: String::from("tr"),
            has_notifications: true,
            page_size: 20,
        }
    }
}

fn show_update_trap() {
    println!("-- 4. Struct update: kısmi move");
    let base = Settings::default();
    let dark = Settings {
        theme: String::from("koyu"),
        ..base
    };
    println!(
        "{} / {} / base.theme hâlâ: {}",
        dark.theme, dark.language, base.theme
    );

    let base = Settings::default();
    let english = Settings {
        theme: base.theme.clone(),
        language: String::from("en"),
        ..base
    };
    println!(
        "{} / taban bütün: {}",
        english.language,
        base == Settings::default()
    );

    let wide = Settings {
        page_size: 50,
        ..base.clone()
    };
    let quiet = Settings {
        has_notifications: false,
        ..Default::default()
    };
    println!(
        "{} / {} / taban: {}",
        wide.page_size, quiet.has_notifications, base.page_size
    );
}

#[cfg(test)]
mod tests {
    use super::Settings;
    use super::account::{Email, User};

    #[test]
    fn email_is_cleaned_and_bad_one_is_rejected() {
        let email = Email::parse(" Kaan@Ornek.COM ").unwrap();
        assert_eq!(email.as_str(), "kaan@ornek.com");
        assert_eq!(email.domain(), "ornek.com");
        assert!(Email::parse("kaan.ornek.com").is_err());
        assert!(Email::parse("@ornek.com").is_err());
        assert!(Email::parse("kaan@ornek").is_err());
        assert!(Email::parse("ka an@ornek.com").is_err());
        assert!(Email::parse("kaan@or@nek.com").is_err());
    }

    #[test]
    fn builder_checks_required_fields() {
        let without_email = User::builder().name("Ada").build();
        assert_eq!(without_email.unwrap_err(), "e-posta zorunlu");
        let without_name = User::builder().name("   ").email("a@b.c").build();
        assert_eq!(without_name.unwrap_err(), "ad zorunlu");
    }

    #[test]
    fn builder_fills_defaults() {
        let user = User::builder()
            .name(" Ada ")
            .email("ada@ornek.com")
            .build()
            .unwrap();
        assert_eq!(user.name(), "Ada");
        assert_eq!(user.login_count(), 0);
        assert_eq!(user.summary(), "Ada <ada@ornek.com> [üye], 0 giriş");
    }

    #[test]
    fn inactive_user_keeps_logins_but_loses_admin() {
        let mut user = User::builder()
            .name("Kaan")
            .email("kaan@ornek.com")
            .admin(true)
            .build()
            .unwrap();
        user.add_login();
        let inactive = user.into_inactive();
        assert_eq!(
            inactive.summary(),
            "[pasif] Kaan <kaan@ornek.com>, son giriş sayısı 1"
        );
        let active_again = inactive.into_active();
        assert_eq!(
            active_again.summary(),
            "Kaan <kaan@ornek.com> [üye], 0 giriş"
        );
    }

    #[test]
    fn update_syntax_keeps_source_if_it_only_copies() {
        let base = Settings::default();
        let english = Settings {
            theme: base.theme.clone(),
            language: String::from("en"),
            ..base
        };
        assert_eq!(base, Settings::default());
        assert_eq!(english.language, "en");
        assert_eq!(english.page_size, base.page_size);
    }
}

// Learn more
//
// You can learn how to keep a value valid (newtype with a private field) from these sources:
//   - Comprehensive Rust, "Enforce Invariants": https://google.github.io/comprehensive-rust/idiomatic/leveraging-the-type-system/newtype-pattern/parse-don-t-validate.html
//   - How To Code It, "The Ultimate Guide to Rust Newtypes": https://www.howtocodeit.com/guides/ultimate-guide-rust-newtypes
//   - Effective Rust, "Embrace the newtype pattern": https://effective-rust.com/newtype.html
//   - Easy: Rustfinity, "Parse, Don't validate: An Effective Error Handling Strategy": https://www.rustfinity.com/blog/parse-dont-validate
//
// You can learn the builder pattern from these sources:
//   - Rust Design Patterns, "Builder": https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
//   - Effective Rust, "Use builders for complex types": https://effective-rust.com/builders.html
//   - Rust API Guidelines, "Type safety": https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder
//   - Easy: Easy Rust, "Default and the builder pattern": https://dhghomon.github.io/easy_rust/Chapter_55.html
//
// You can learn how to choose self, &self or &mut self from these sources:
//   - The Rust Book, "Methods": https://doc.rust-lang.org/book/ch05-03-method-syntax.html
//   - Comprehensive Rust, "Methods": https://google.github.io/comprehensive-rust/methods-and-traits/methods.html
//   - Rust by Example, "Methods": https://doc.rust-lang.org/rust-by-example/fn/methods.html
//   - Easy: Easy Rust, "Implementing structs and enums": https://dhghomon.github.io/easy_rust/Chapter_27.html
//
// You can learn the struct update trap (moved fields) from these sources:
//   - The Rust Book, "Creating Instances with Struct Update Syntax": https://doc.rust-lang.org/book/ch05-01-defining-structs.html#creating-instances-with-struct-update-syntax
//   - Rust Reference, "Functional update syntax": https://doc.rust-lang.org/reference/expressions/struct-expr.html#functional-update-syntax
//   - Rust by Example, "Partial moves": https://doc.rust-lang.org/rust-by-example/scope/move/partial_move.html
//   - Easy: Let's Get Rusty (video), "Structs in Rust": https://www.youtube.com/watch?v=n3bPhdiJm9I
