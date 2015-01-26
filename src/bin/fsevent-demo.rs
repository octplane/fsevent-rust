#![feature(link_args)]

extern crate fsevent;
use std::sync::mpsc::channel;
use std::thread::Thread;

#[allow(dead_code)]
#[allow(unstable)]
fn main() {
  let (sender, receiver) = channel::<fsevent::Event>();

  let _t = Thread::spawn(move || {
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
