use rocket::{routes, get, post, launch, State};
use rocket_dyn_templates::Template;
use bdk::{SyncOptions, Wallet, FeeRate};
use bdk::wallet::{AddressIndex, signer::SignOptions};
use bdk::database::MemoryDatabase;
use bdk::blockchain::{Blockchain, EsploraBlockchain};
use std::{collections::HashMap, sync::Mutex, ops::Deref};
use bitcoin::{Address, Network};

macro_rules! wrap_error {
    ($e:expr) => ({
        match $e {
            Ok(r) => r,
            Err(e) => return Template::render("error", vec![("error", e.to_string())].into_iter().collect::<HashMap<_, _>>()),
        }
    });
}

struct UsefulBDKStuff {
    wallet: Wallet<MemoryDatabase>,
    blockchain: EsploraBlockchain,
}

#[get("/")]
fn main_page(stuff: &State<Mutex<UsefulBDKStuff>>) -> Template {
    let guard = stuff.lock().unwrap();
    let UsefulBDKStuff { wallet, blockchain } = guard.deref();
    wrap_error!(wallet.sync(blockchain, SyncOptions::default()));

    let address = wrap_error!(wallet.get_address(AddressIndex::New));
    let balance = wrap_error!(wallet.get_balance());
    let mut context = HashMap::new();
    context.insert("address", address.to_string());
    context.insert("balance", format!("{}", balance));
    if balance > 5_000 {
        context.insert("can_spend", "yes".to_string());
    }
    Template::render("index", &context)
}

#[post("/send", data = "<address>")]
fn send(stuff: &State<Mutex<UsefulBDKStuff>>, address: String) -> Template {
    let amount = 5000;
    let address = wrap_error!(address.replace("address=", "").trim().parse::<Address>());
    if address.network != Network::Testnet {
        let _ = wrap_error!(Result::Err("Invalid address"));
    }

    let guard = stuff.lock().unwrap();
    let UsefulBDKStuff { wallet, blockchain } = guard.deref();
    wrap_error!(wallet.sync(blockchain, SyncOptions::default()));

    let (mut psbt, _) = {
        let mut builder =  wallet.build_tx();
        builder
            .add_recipient(address.script_pubkey(), amount)
            .enable_rbf()
            .fee_rate(FeeRate::from_sat_per_vb(1.0));
        wrap_error!(builder.finish())
    };
    assert!(wrap_error!(wallet.sign(&mut psbt, SignOptions::default())));
    let tx = psbt.extract_tx();
    wrap_error!(blockchain.broadcast(&tx));

    let mut context = HashMap::new();
    context.insert("address", address.to_string());
    context.insert("amount", format!("{}", amount));
    context.insert("txid", tx.txid().to_string());
    Template::render("done", &context)
}

#[launch]
fn rocket() -> _ {
    let descriptor = "wsh(pk(cUXgHH7nBFZiWLdjj24nWunSAD6BLBpegdqPRbF1ZKgJoXEuZXrp))";
    let blockchain = EsploraBlockchain::new("https://blockstream.info/testnet/api/", 10);
    let wallet = Wallet::new(descriptor, None, Network::Testnet, MemoryDatabase::new()).unwrap();

    rocket::build().mount("/", routes![main_page, send]).manage(Mutex::new(UsefulBDKStuff { wallet, blockchain })).attach(Template::fairing())
}
