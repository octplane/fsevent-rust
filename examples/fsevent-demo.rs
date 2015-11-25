extern crate fsevent;
use std::sync::mpsc::channel;
use std::thread;

fn main() {
  let (sender, receiver) = channel();

  let _t = thread::spawn(move || {
    let fsevent = fsevent::FsEvent::new(sender);
    fsevent.append_path(".");
    fsevent.observe();
  });

  loop {
    let val = receiver.recv();
    println!("{:?}", val.unwrap());
  }
}
