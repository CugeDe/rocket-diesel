# rocket-diesel

rocket-diesel is a [Fairing](https://api.rocket.rs/v0.4/rocket/fairing/trait.Fairing.html)
designed for Rocket, a web framework for Rust (nightly).

```rust
#![feature(proc_macro_hygiene)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_config;
extern crate rocket_diesel;

use rocket_config::Factory as ConfigurationsFairing;
use rocket_diesel::DieselDatabase;

#[get("/<name>/<age>")]
fn hello(_database_: DieselDatabase, name: String, age: u8)
-> String
{
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    rocket::ignite()
        .attach(ConfigurationsFairing::new())
        .attach(DieselDatabase::new())
        .mount("/hello", routes![hello]).launch();
}
```