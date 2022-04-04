use rocket::{routes, get, post, launch, State};
use rocket_dyn_templates::Template;
use bdk::{SyncOptions, Wallet};
use bdk::wallet::AddressIndex;
use bdk::database::MemoryDatabase;
use bdk::blockchain::EsploraBlockchain;
use std::{collections::HashMap, sync::Mutex};
use bitcoin::{Address, Network};

#[get("/")]
fn main_page(stuff: &State<Mutex<UsefulBDKStuff>>) -> Template {
}

#[post("/send", data = "<address>")]
fn send(stuff: &State<Mutex<UsefulBDKStuff>>, address: String) -> Template {
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![main_page, send]).attach(Template::fairing())
}
