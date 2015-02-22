#![feature(link_args)]
#![feature(std_misc)]

extern crate fsevent;
use std::sync::mpsc::channel;
use std::thread;

#[allow(dead_code)]
fn main() {
  let (sender, receiver) = channel::<fsevent::Event>();

  let _t = thread::spawn(move || {
    let fsevent = fsevent::FsEvent::new(sender);
    fsevent.append_path("../../");
    fsevent.observe();
  });

  loop {
    select! (
      val = receiver.recv() => {
        println!("{:?}", val);
      }
    )
  }
}
