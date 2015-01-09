#![feature(phase)]
#![feature(link_args)]
#![allow(unused_variables)]



extern crate getopts;
extern crate libc;
extern crate fsevent;



use getopts::{optopt,getopts,OptGroup};
use std::os;

fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [options]", program);
    println!("-p path\t\tPath to observe (default: .)");
    println!("-c command\t\tCommand to run (default: cargo build)");
    println!("-h --help\tUsage");
}



fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    let opts = &[
        optopt("h", "help", "display help", "HELP"),
        optopt("p", "", "set path", "PATH"),
        optopt("c", "", "command", "COMMAND"),

    ];

    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(program.as_slice(), opts);
        return;
    }

    let path = match matches.opt_str("p") {
    	Some(p) => p,
    	None => ".".to_string()
    };

    let command = match matches.opt_str("c") {
    	Some(p) => p,
    	None => "cargo build".to_string()
    };


    let (ok, msg) = fsevent::is_api_available();
    if !ok {
        println!("Sorry: {}", msg);
        return;
    }

    let fsevent = fsevent::default_fsevent();

    fsevent.append_path("./src/temp/build/pipo");

    // let stream = FSEventStreamCreate(kCFAllocatorDefault,
    //    (FSEventStreamCallback)&callback,
    //    &stream_context,
    //    config.paths,
    //    config.sinceWhen,
    //    config.latency,
    //    config.flags);

}