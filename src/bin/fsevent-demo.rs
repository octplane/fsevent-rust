#![feature(link_args)]

extern crate fsevent;

fn cb(events: Vec<fsevent::Event>) {
	for i in events.iter() {
		println!("{:?}", i);
	}
}


fn main() {
    let fsevent = fsevent::FsEvent::new(cb);

    fsevent.append_path("../../");

    fsevent.observe();
}