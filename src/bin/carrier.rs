#![feature(phase)]
#![feature(globs)]
#![feature(link_args)]

#[phase(plugin, link)] extern crate log;

extern crate getopts;
extern crate libc;
extern crate fsevent;


use std::c_str::CString;
use getopts::{optopt,getopts,OptGroup};
use std::os;

fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [options]", program);
    println!("-p path\t\tPath to observe (default: .)");
    println!("-c command\t\tCommand to run (default: cargo build)");
    println!("-h --help\tUsage");
}

extern fn callback(arg1: libc::c_int, arg2: libc::c_int)-> libc::c_int {
    println!("I'm called from C with value {0}", arg1);
    return 0;
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


    println!("k:{}", fsevent::kCFTypeArrayCallBacks);
}

