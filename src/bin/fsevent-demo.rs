#![feature(link_args)]

extern crate fsevent;

#[allow(dead_code)]
fn cb(events: Vec<fsevent::Event>) {
	for i in events.iter() {
		println!("{:?}", i);
	}
}

#[allow(dead_code)]
fn main() {
    let fsevent = fsevent::FsEvent::new(cb);

    fsevent.append_path("../../");

    fsevent.observe();
}
