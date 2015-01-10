#![feature(link_args)]
#![allow(unused_variables)]

extern crate fsevent;

fn main() {
    let fsevent = fsevent::default_fsevent();

    fsevent.append_path("./src/temp/build/pipo");

    fsevent.observe();
}