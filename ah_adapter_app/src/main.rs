mod consoleapp;
mod controller;
mod datastore;
mod serde_table;
mod settings;

use consoleapp::ConsoleApp;
use controller::Controller;
use datastore::DataStore;

fn main() {
    let data_store = DataStore::new().unwrap();
    let controller = Controller { data_store };
    ConsoleApp::run(&controller);
}
