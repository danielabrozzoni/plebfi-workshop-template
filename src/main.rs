use rocket::{routes, get, post, launch, State};
use rocket_dyn_templates::Template;
use bdk::{SyncOptions, SignOptions, Wallet};
use bdk::wallet::AddressIndex;
use bdk::database::MemoryDatabase;
use bdk::blockchain::{EsploraBlockchain, Blockchain};
use std::{collections::HashMap, sync::Mutex, ops::Deref};
use bitcoin::{Address, Network};

struct UsefulBDKStuff {
    wallet: Wallet<MemoryDatabase>,
    blockchain: EsploraBlockchain,
}

#[get("/")]
fn main_page(stuff: &State<Mutex<UsefulBDKStuff>>) -> Template {
    let guard = stuff.lock().unwrap(); 
    let UsefulBDKStuff { wallet, blockchain } = guard.deref();
    wallet.sync(blockchain, SyncOptions::default()).unwrap();
    let balance = wallet.get_balance().unwrap();
    let address = wallet.get_address(AddressIndex::New).unwrap();
    let mut map = HashMap::new();
    map.insert("balance", balance.to_string());
    map.insert("address", address.to_string());

    if balance > 5000 {
        map.insert("can_spend", "randomstring".to_string());
    }

    Template::render("index", &map)
}

#[post("/send", data = "<address>")]
fn send(stuff: &State<Mutex<UsefulBDKStuff>>, address: String) -> Template {
    let guard = stuff.lock().unwrap(); 
    let UsefulBDKStuff { wallet, blockchain } = guard.deref();
    wallet.sync(blockchain, SyncOptions::default()).unwrap();

    let address = address.replace("address=", "").trim().parse::<Address>().unwrap();
    if address.network != Network::Testnet {
        panic!("invalid network");
    }

    let amount = 5000;
    let (mut psbt, _) = {
        let mut builder = wallet.build_tx();
        builder
            .add_recipient(address.script_pubkey(), amount);

        builder.finish().unwrap()
    };
    assert!(wallet.sign(&mut psbt, SignOptions::default()).unwrap());
    let tx = psbt.extract_tx();
    blockchain.broadcast(&tx).unwrap();

    let txid = tx.txid();

    let mut map = HashMap::new();
    map.insert("amount", amount.to_string());
    map.insert("txid", txid.to_string());
    map.insert("address", address.to_string());

    Template::render("done", &map)
}

#[launch]
fn rocket() -> _ {
    let state = Mutex::new(UsefulBDKStuff {
        wallet: Wallet::new("wpkh(cUXgHH7nBFZiWLdjj24nWunSAD6BLBpegdqPRbF1ZKgJoXEuZXrp)", None, Network::Testnet, MemoryDatabase::new()).unwrap(),
        blockchain: EsploraBlockchain::new("https://blockstream.info/testnet/api", 20),
    });

    rocket::build().mount("/", routes![main_page, send]).manage(state).attach(Template::fairing())
}
