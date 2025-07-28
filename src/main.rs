use std::{sync::Arc, thread};
use miniredis::kv_store::KVStore;

fn main() {
    let store = Arc::new(KVStore::new());

    let mut handles = Vec::new();

    for _ in 0..10 {
        let store = store.clone();
        let handle = thread::spawn(move || {
            store.set("key", "value");
            println!("set key for thread {:?}", thread::current().id());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads done!");
}
