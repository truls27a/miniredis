mod helpers;
use helpers::{send_command, start_test_server};

use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn multiple_clients_can_connect_simultaneously() {
    let address = start_test_server();

    // Spawn multiple threads that act as different clients
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let addr = address.clone();
            thread::spawn(move || {
                let key = format!("client_{}_key", i);
                let value = format!("client_{}_value", i);

                // Each client sets its own key-value pair
                let set_response = send_command(&addr, &format!("SET {} {}", key, value))
                    .expect("Failed to send SET command");
                assert_eq!(set_response, "OK");

                // Each client gets its own value back
                let get_response = send_command(&addr, &format!("GET {}", key))
                    .expect("Failed to send GET command");
                assert_eq!(get_response, value);
            })
        })
        .collect();

    // Wait for all clients to complete
    for handle in handles {
        handle.join().expect("Client thread panicked");
    }
}

#[test]
fn concurrent_operations_on_same_key() {
    let address = start_test_server();
    let num_threads = 10;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let addr = address.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                // Each thread tries to set the same key with its own value
                let value = format!("value_from_thread_{}", i);
                let set_response = send_command(&addr, &format!("SET shared_key {}", value))
                    .expect("Failed to send SET command");
                assert_eq!(set_response, "OK");

                // Try to get the value (might be from this thread or another)
                let get_response =
                    send_command(&addr, "GET shared_key").expect("Failed to send GET command");

                // The response should be one of the values set by any thread
                assert!(get_response.starts_with("value_from_thread_"));
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn concurrent_set_and_get_operations() {
    let address = start_test_server();
    let num_operations = 20;
    let barrier = Arc::new(Barrier::new(num_operations));

    let handles: Vec<_> = (0..num_operations)
        .map(|i| {
            let addr = address.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                if i % 2 == 0 {
                    // Even threads perform SET operations
                    let key = format!("key_{}", i / 2);
                    let value = format!("value_{}", i / 2);
                    let response = send_command(&addr, &format!("SET {} {}", key, value))
                        .expect("Failed to send SET command");
                    assert_eq!(response, "OK");
                } else {
                    // Odd threads perform GET operations
                    let key = format!("key_{}", (i - 1) / 2);
                    // Wait a bit to give SET operations a chance to complete
                    thread::sleep(Duration::from_millis(10));
                    let response = send_command(&addr, &format!("GET {}", key))
                        .expect("Failed to send GET command");
                    // Response can be either the value or "nil" depending on timing
                    assert!(response == "nil" || response.starts_with("value_"));
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn concurrent_delete_operations() {
    let address = start_test_server();

    // First, set up some initial data
    for i in 0..10 {
        let key = format!("delete_key_{}", i);
        let value = format!("delete_value_{}", i);
        send_command(&address, &format!("SET {} {}", key, value))
            .expect("Failed to set initial data");
    }

    let num_threads = 10;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            let addr = address.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                let key = format!("delete_key_{}", i);
                let response = send_command(&addr, &format!("DEL {}", key))
                    .expect("Failed to send DEL command");
                assert_eq!(response, "OK");
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all keys are deleted
    for i in 0..10 {
        let key = format!("delete_key_{}", i);
        let response =
            send_command(&address, &format!("GET {}", key)).expect("Failed to send GET command");
        assert_eq!(response, "nil");
    }
}

#[test]
fn stress_test_many_concurrent_operations() {
    let address = start_test_server();
    let num_threads = 50;
    let operations_per_thread = 20;
    let barrier = Arc::new(Barrier::new(num_threads));
    let results = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let addr = address.clone();
            let barrier = Arc::clone(&barrier);
            let results = Arc::clone(&results);

            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                let mut thread_results = Vec::new();

                for op_id in 0..operations_per_thread {
                    let key = format!("stress_key_{}_{}", thread_id, op_id);
                    let value = format!("stress_value_{}_{}", thread_id, op_id);

                    // Perform SET operation
                    let set_response = send_command(&addr, &format!("SET {} {}", key, value));
                    thread_results.push(("SET", set_response.is_ok()));

                    // Perform GET operation
                    let get_response = send_command(&addr, &format!("GET {}", key));
                    thread_results.push(("GET", get_response.is_ok()));

                    // Perform DEL operation
                    let del_response = send_command(&addr, &format!("DEL {}", key));
                    thread_results.push(("DEL", del_response.is_ok()));
                }

                // Store results
                let mut global_results = results.lock().unwrap();
                global_results.extend(thread_results);
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify all operations succeeded
    let global_results = results.lock().unwrap();
    let total_operations = num_threads * operations_per_thread * 3; // 3 operations per iteration
    assert_eq!(global_results.len(), total_operations);

    let successful_operations = global_results
        .iter()
        .filter(|(_, success)| *success)
        .count();
    assert_eq!(successful_operations, total_operations);
}

#[test]
fn concurrent_read_heavy_workload() {
    let address = start_test_server();

    // Set up initial data
    let num_keys = 10;
    for i in 0..num_keys {
        let key = format!("read_key_{}", i);
        let value = format!("read_value_{}", i);
        send_command(&address, &format!("SET {} {}", key, value))
            .expect("Failed to set initial data");
    }

    let num_readers = 20;
    let reads_per_reader = 50;
    let barrier = Arc::new(Barrier::new(num_readers));

    let handles: Vec<_> = (0..num_readers)
        .map(|reader_id| {
            let addr = address.clone();
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                // Wait for all readers to be ready
                barrier.wait();

                for _ in 0..reads_per_reader {
                    let key_index = reader_id % num_keys;
                    let key = format!("read_key_{}", key_index);
                    let expected_value = format!("read_value_{}", key_index);

                    let response = send_command(&addr, &format!("GET {}", key))
                        .expect("Failed to send GET command");
                    assert_eq!(response, expected_value);
                }
            })
        })
        .collect();

    // Wait for all readers to complete
    for handle in handles {
        handle.join().expect("Reader thread panicked");
    }
}

#[test]
fn concurrent_write_heavy_workload() {
    let address = start_test_server();
    let num_writers = 15;
    let writes_per_writer = 30;
    let barrier = Arc::new(Barrier::new(num_writers));

    let handles: Vec<_> = (0..num_writers)
        .map(|writer_id| {
            let addr = address.clone();
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                // Wait for all writers to be ready
                barrier.wait();

                for write_id in 0..writes_per_writer {
                    let key = format!("write_key_{}_{}", writer_id, write_id);
                    let value = format!("write_value_{}_{}", writer_id, write_id);

                    let response = send_command(&addr, &format!("SET {} {}", key, value))
                        .expect("Failed to send SET command");
                    assert_eq!(response, "OK");
                }
            })
        })
        .collect();

    // Wait for all writers to complete
    for handle in handles {
        handle.join().expect("Writer thread panicked");
    }

    // Verify all data was written correctly
    for writer_id in 0..num_writers {
        for write_id in 0..writes_per_writer {
            let key = format!("write_key_{}_{}", writer_id, write_id);
            let expected_value = format!("write_value_{}_{}", writer_id, write_id);

            let response = send_command(&address, &format!("GET {}", key))
                .expect("Failed to verify written data");
            assert_eq!(response, expected_value);
        }
    }
}

#[test]
fn mixed_concurrent_workload() {
    let address = start_test_server();
    let total_threads = 30;
    let operations_per_thread = 25;
    let barrier = Arc::new(Barrier::new(total_threads));

    let handles: Vec<_> = (0..total_threads)
        .map(|thread_id| {
            let addr = address.clone();
            let barrier = Arc::clone(&barrier);

            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier.wait();

                for op_id in 0..operations_per_thread {
                    let key = format!("mixed_key_{}_{}", thread_id, op_id);
                    let value = format!("mixed_value_{}_{}", thread_id, op_id);

                    match op_id % 4 {
                        0 => {
                            // SET operation
                            let response = send_command(&addr, &format!("SET {} {}", key, value))
                                .expect("Failed to send SET command");
                            assert_eq!(response, "OK");
                        }
                        1 => {
                            // GET operation (might return nil for new keys)
                            let response = send_command(&addr, &format!("GET {}", key))
                                .expect("Failed to send GET command");
                            assert!(response == "nil" || response.starts_with("mixed_value_"));
                        }
                        2 => {
                            // SET then GET
                            send_command(&addr, &format!("SET {} {}", key, value))
                                .expect("Failed to send SET command");
                            let response = send_command(&addr, &format!("GET {}", key))
                                .expect("Failed to send GET command");
                            assert_eq!(response, value);
                        }
                        3 => {
                            // SET then DEL
                            send_command(&addr, &format!("SET {} {}", key, value))
                                .expect("Failed to send SET command");
                            let response = send_command(&addr, &format!("DEL {}", key))
                                .expect("Failed to send DEL command");
                            assert_eq!(response, "OK");
                        }
                        _ => unreachable!(),
                    }
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
