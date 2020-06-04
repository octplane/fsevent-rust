extern crate fsevent;
use std::sync::mpsc::channel;
use std::thread;

#[cfg(not(target_os="macos"))]
fn main() {}

#[cfg(target_os="macos")]
fn main() {
    let (sender, receiver) = channel();

    let _t = thread::spawn(move || {
        let fsevent = fsevent::FsEvent::new(vec![".".to_string()]);
        fsevent.observe(sender);
    });

    loop {
        let val = receiver.recv();
        println!("{:?}", val.unwrap());
    }
}
