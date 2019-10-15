#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
extern crate rocket_config;
extern crate rocket_diesel;

use rocket::State;
use rocket_config::Factory as ConfigurationsFairing;
use rocket_diesel::Database as DieselDatabase;
use diesel::prelude::*;

mod models;
mod schema;
use models::User as UserModel;
use schema::users::dsl::{users as users_table};

#[get("/<_name>/<_age>")]
fn hello<'r>(database: State<'r, DieselDatabase>, _name: String, _age: u8) -> String {
    let database: &'r DieselDatabase = database.inner();

    println!("Interaction returned: {:?}", database.interact::<(), rocket_diesel::error::Error, _, _, _>(
        |mysql_conn| {
            let users = users_table
                .select(schema::users::all_columns)
                .limit(5)
                .load::<UserModel>(mysql_conn)
                .expect("Error loading users");

            println!("{:?}", users);
            Ok(())
        },
        |pg_conn| {
            let users = users_table
                .select(schema::users::all_columns)
                .limit(5)
                .load::<UserModel>(pg_conn)
                .expect("Error loading users");

            println!("{:?}", users);
            Ok(())
        },
        |sqlite_conn| {
            let users = users_table
                .select(schema::users::all_columns)
                .limit(5)
                .load::<UserModel>(sqlite_conn)
                .expect("Error loading users");

            println!("{:?}", users);
            Ok(())
        }
    ));

    format!("Hello!")
}

pub fn main() {
    let rocket = rocket::ignite()
        .attach(ConfigurationsFairing::new())
        .attach(DieselDatabase::new())
        .mount("/hello", routes![hello]);
    
    rocket.launch();
}