# thread-broadcaster

![release](https://img.shields.io/github/v/release/DarthBenro008/thread-broadcaster)
[![GitHub License](https://img.shields.io/github/license/darthbenro008/thread-broadcaster)](https://github.com/DarthBenro008/thread-broadcaster/blob/master/LICENSE)
[![Documentation](https://docs.rs/thread-broadcaster/badge.svg)](
https://docs.rs/thread-broadcaster)

Thread Broadcaster is a Single Channel Multi-Producer (SPMC) library that enables the sending of notifications between threads. Unlike most Multi-Producer Multi-Consumer (MPMC) implementations, Thread Broadcaster ensures that all listeners receive the data, rather than just the first one.

## 🤔 Features

- Single Channel Multi-Producer (SPMC) architecture
- Tad bit faster than MPMC (thanks to [crossbeam-channel](https://crates.io/crates/crossbeam-channel)!)
- Senders and Receivers can be cloned and shared among threads.
- Simple and intuitive API for sending and receiving notifications
- Asynchronous notification broadcasting between threads
- Support `tracing` for debugs

## ⚡️ Quickstart

```rust
use core::time;
use std::thread;

use thread_broadcaster::{BroadcastListener, Broadcaster};

fn main() {
    let (b, s) = Broadcaster::<String>::new();
    let s2 = s.clone();
    thread::spawn(move || {
        let ls1 = BroadcastListener::register_broadcast_listener(s);
        for msg in ls1.channel {
            println!(
                "got broadcast with data: {} on thread {:#?}",
                msg,
                thread::current().id()
            );
        }
    });
    thread::spawn(move || {
        let ls2 = BroadcastListener::register_broadcast_listener(s2);
        for msg in ls2.channel {
            println!(
                "got broadcast with data: {} on thread {:#?}",
                msg,
                thread::current().id()
            );
        }
    });
    thread::spawn(move || {
        // we wait for registration
        thread::sleep(time::Duration::from_secs(1));
        b.broadcast("something to broadcast".to_string());
        // we wait for listeners to pickup before being dropped
        thread::sleep(time::Duration::from_secs(2));
    })
    .join()
    .unwrap();
}
```

## ⬇️ Installation

```
cargo add thread-broadcaster
```
## 🤝 Contributions

- Feel Free to Open a PR/Issue for any feature or bug(s).
- Make sure you follow the [community guidelines](https://docs.github.com/en/github/site-policy/github-community-guidelines).
- Feel free to open an issue to ask a question/discuss anything about melonpan.
- Have a feature request? Open an Issue!


## ⚖ License

Copyright 2022 Hemanth Krishna

Licensed under MIT License : https://opensource.org/licenses/MIT

<p align="center">Made with ❤ and multiple cups of coffee </p>


