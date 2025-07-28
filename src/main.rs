use std::{sync::Arc, thread};
use rand::Rng;
use miniredis::kv_store::KVStore;

fn main() {
    let store = Arc::new(KVStore::new());

    let mut handles = Vec::new();

    for i in 0..10 {
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            let thread_id = i;
            let actions = vec!["get", "set", "delete"];

            let mut rng = rand::rng();

            for _ in 0..10 {
                let key = format!("key_{}", rng.random_range(0..5));
                let action = actions[rng.random_range(0..3)];

                match action {
                    "get" => {
                        let value = store_clone.get(&key);
                        println!("Thread {:?} got value {:?} for key {:?}", thread_id, value, key);
                    }
                    "set" => {
                        let value = format!("value_{}", rng.random_range(0..5));
                        store_clone.set(&key, &value);
                        println!("Thread {:?} set value {:?} for key {:?}", thread_id, value, key);
                    }
                    "delete" => {
                        store_clone.delete(&key);
                        println!("Thread {:?} deleted key {:?}", thread_id, key);
                    }
                    _ => {
                        println!("Thread {:?} did an unknown action {:?}", thread_id, action);
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All threads done!");
}
