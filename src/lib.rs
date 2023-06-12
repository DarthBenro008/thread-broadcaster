/*
* Hemanth Krishna (DarthBenro008), Jun 2023
*/

/*!
# thread-broadcaster
Thread Broadcaster is a Single Channel Multi-Producer (SPMC) library that enables the sending of notifications between threads.
Unlike most Multi-Producer Multi-Consumer (MPMC) implementations, Thread Broadcaster ensures that all listeners receive the data, rather than just the first one.

## Example

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
*/

use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{unbounded, Receiver, Sender};

/// Responsible for registring new listeners to the broadcaster and to recieve data
pub struct BroadcastListener<T> {
    pub channel: Receiver<T>,
}

impl<T> BroadcastListener<T> {
    pub fn register_broadcast_listener(broadcaster: Sender<Sender<T>>) -> BroadcastListener<T> {
        let (s, r) = unbounded::<T>();
        broadcaster.send(s.clone()).unwrap();
        BroadcastListener { channel: r }
    }
}

/// Returned objet on creation of thread-broadcaster responsible to broadcast data to threads
pub struct Controller<T> {
    data: Arc<Mutex<Vec<Sender<T>>>>,
}

impl<T> Controller<T>
where
    T: std::marker::Send + Clone,
{
    pub fn broadcast(&self, data: T) {
        tracing::debug_span!("broadcasting data");
        let mut map = self.data.lock().unwrap();
        for x in map.iter_mut() {
            let new_data = data.clone();
            x.send(new_data).unwrap();
        }
    }
}

/// Allows to create a thread-broadcaster
pub struct Broadcaster<T> {
    sender: Sender<Sender<T>>,
    reciver: Receiver<Sender<T>>,
    data: Arc<Mutex<Vec<Sender<T>>>>,
}

impl<T> Broadcaster<T>
where
    T: std::marker::Send + Clone + 'static,
{
    pub fn new() -> (Controller<T>, Sender<Sender<T>>) {
        let (s, r) = unbounded::<crossbeam_channel::Sender<T>>();
        let broadcaster = Broadcaster {
            sender: s.clone(),
            reciver: r,
            data: Arc::new(Mutex::new(vec![])),
        };
        let tc = Controller {
            data: Arc::clone(&broadcaster.data),
        };
        thread::spawn(move || {
            tracing::debug_span!("starting registration loop");
            broadcaster.registration_loop();
        });
        (tc, s)
    }

    pub fn broadcaster(self) -> Sender<Sender<T>> {
        self.sender.clone()
    }

    fn registration_loop(&self) {
        let r = self.reciver.clone();
        thread::scope(|s| {
            s.spawn(move || {
                for msg in r.iter() {
                    tracing::debug_span!("got a registration from listener");
                    let mut map = self.data.lock().unwrap();
                    map.push(msg);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    #[derive(Clone)]
    pub struct Test {
        pub id: String,
    }

    use core::time;

    use super::*;

    #[test]
    fn single_listener() {
        let (b, s) = Broadcaster::<Test>::new();
        let listener = BroadcastListener::register_broadcast_listener(s);
        let obj = Test {
            id: "test broadcast".to_string(),
        };
        thread::spawn(move || {
            thread::sleep(time::Duration::from_secs(1));
            b.broadcast(obj);
        });
        assert_eq!(listener.channel.recv().unwrap().id, "test broadcast")
    }

    #[test]
    fn broadcast_two_listener() {
        let (b, s) = Broadcaster::<Test>::new();
        let ls2 = s.clone();
        let listener = BroadcastListener::register_broadcast_listener(s);
        let listener2 = BroadcastListener::register_broadcast_listener(ls2);
        let results = Arc::new(Mutex::new(Vec::<String>::new()));
        let comparator = Arc::new(Mutex::new(vec![
            "test broadcast".to_string(),
            "test broadcast".to_string(),
        ]));
        let ar1 = Arc::clone(&results);
        let ar2 = Arc::clone(&results);
        let obj = Test {
            id: "test broadcast".to_string(),
        };
        let t1 = thread::spawn(move || {
            let data = listener.channel.recv();
            ar1.lock().unwrap().push(data.unwrap().id);
        });
        let t2 = thread::spawn(move || {
            let data = listener2.channel.recv();
            ar2.lock().unwrap().push(data.unwrap().id);
        });
        thread::spawn(move || {
            thread::sleep(time::Duration::from_secs(1));
            b.broadcast(obj);
        });
        let _ = t1.join().unwrap();
        let _ = t2.join().unwrap();
        assert_eq!(*comparator.lock().unwrap(), *results.lock().unwrap());
    }
}
