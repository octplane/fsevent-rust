#![feature(link_args)]

extern crate fsevent;

fn main() {
    let fsevent = fsevent::FsEvent::new();

    fsevent.append_path(".");

    fsevent.observe();
}