#![feature(link_args)]
#![allow(unused_variables)]

extern crate fsevent;

fn main() {
    let fsevent = fsevent::FsEvent::new();

    fsevent.append_path("./src/temp/build/pipo");

    fsevent.observe();
}