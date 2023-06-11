use std::{
    process::id,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{unbounded, Receiver, Sender};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct BroadcastListener<T> {
    sender: Sender<T>,
    pub channel: Receiver<T>,
}

impl<T> BroadcastListener<T> {
    pub fn register_broadcast_listener(broadcaster: Sender<Sender<T>>) -> BroadcastListener<T> {
        let (s, r) = unbounded::<T>();
        broadcaster.send(s.clone()).unwrap();
        BroadcastListener {
            sender: s,
            channel: r,
        }
    }
}

pub struct Trial<T> {
    data: Arc<Mutex<Vec<Sender<T>>>>,
}

impl<T> Trial<T>
where
    T: std::marker::Send + Clone,
{
    pub fn broadcast(&self, data: T) {
        let mut map = self.data.lock().unwrap();
        println!("map: {:#?}", map);
        for x in map.iter_mut() {
            let new_data = data.clone();
            x.send(new_data).unwrap();
        }
    }
}

pub struct Broadcaster<T> {
    sender: Sender<Sender<T>>,
    reciver: Receiver<Sender<T>>,
    data: Arc<Mutex<Vec<Sender<T>>>>,
}

impl<T> Broadcaster<T>
where
    T: std::marker::Send + Clone + 'static,
{
    pub fn new() -> (Trial<T>, Sender<Sender<T>>) {
        let (s, r) = unbounded::<crossbeam_channel::Sender<T>>();
        let broadcaster = Broadcaster {
            sender: s.clone(),
            reciver: r,
            data: Arc::new(Mutex::new(vec![])),
        };
        let tc = Trial {
            data: Arc::clone(&broadcaster.data),
        };
        tokio::spawn(async move {
            broadcaster.registration_loop().await;
        });
        (tc, s)
    }

    pub fn broadcaster(self) -> Sender<Sender<T>> {
        self.sender
    }

    pub async fn registration_loop(&self) {
        println!("reg_loop started");
        let r = self.reciver.clone();
        // thread::spawn(move || {
        //     for msg in r.iter() {
        //         println!("loop");
        //         let mut map = self.data.lock().unwrap();
        //         println!("reg!");
        //         map.push(msg);
        //     }
        // });
        thread::scope(|s| {
            // let gg = s.spawn(move || loop {
            //     let data = r.recv().unwrap();
            //     let mut map = self.data.lock().unwrap();
            //     println!("registration: new data!");
            //     map.push(data);
            // });
            s.spawn(move || {
                println!("lmfao {:#?}", thread::current().id());
                for msg in r.iter() {
                    println!("loop");
                    let mut map = self.data.lock().unwrap();
                    println!("reg!");
                    map.push(msg);
                }
                // for msg in r.iter() {
                //     println!("loop");
                //     let mut map = self.data.lock().unwrap();
                //     println!("reg!");
                //     map.push(msg);
                // }
            });
        });
    }

    pub fn broadcast(&self, data: T) {
        let mut map = self.data.lock().unwrap();
        println!("map: {:#?}", map);
        for x in map.iter_mut() {
            let new_data = data.clone();
            x.send(new_data).unwrap();
        }
    }

    pub fn listener(&self) -> Receiver<T> {
        let (s, r) = unbounded::<T>();
        self.sender.send(s).unwrap();
        r
    }
}

#[cfg(test)]
mod tests {
    #[derive(Clone)]
    pub struct gg {
        id: String,
    }
    use super::*;

    #[test]
    fn it_works() {
        println!("gg1");
        let lol = Broadcaster::<gg>::new();
        println!("gg");
        assert!(true);
    }
}
