#![feature(phase)]
#![feature(globs)]
#![feature(link_args)]
#![allow(unused_variables)]


#[phase(plugin, link)] extern crate log;

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


    add("./src/pipo");
}

// https://github.com/thibaudgg/rb-fsevent/blob/master/ext/fsevent_watch/main.c
fn add(source: &str) {
    let cp = source.to_c_str();
    unsafe {
        let mut url = fsevent::CFURLCreateFromFileSystemRepresentation(fsevent::MNULL, cp.as_ptr(), cp.len() as i64, false);
        let mut placeholder = fsevent::CFURLCopyAbsoluteURL(url);
        fsevent::CFRelease(url);


        let mut imaginary: fsevent::CFMutableArrayRef = fsevent::MNULL;

        while !fsevent::CFURLResourceIsReachable(placeholder, fsevent::MNULL) {

            if imaginary == fsevent::MNULL {
                imaginary = fsevent::CFArrayCreateMutable(fsevent::MNULL, 0, fsevent::kCFTypeArrayCallBacks);
            }

            let child = fsevent::CFURLCopyLastPathComponent(placeholder);
            println!("Appending to array");
            fsevent::CFShow(child);
            fsevent::CFArrayInsertValueAtIndex(imaginary, 0, child);
            fsevent::CFRelease(child);

            url = fsevent::CFURLCreateCopyDeletingLastPathComponent(fsevent::MNULL, placeholder);
            fsevent::CFRelease(placeholder);
            placeholder = url;
        }
        url = fsevent::CFURLCreateFileReferenceURL(fsevent::MNULL, placeholder, fsevent::MNULL);
        fsevent::CFRelease(placeholder);
        placeholder = fsevent::CFURLCreateFilePathURL(fsevent::MNULL, url, fsevent::MNULL);
        fsevent::CFRelease(url);

        if imaginary != fsevent::MNULL {
            let mut count = fsevent::CFArrayGetCount(imaginary);
            while { count > 0 }
            {
                let component = fsevent::CFArrayGetValueAtIndex(imaginary, count);
                fsevent::CFShow(component);
                url = fsevent::CFURLCreateCopyAppendingPathComponent(fsevent::MNULL, placeholder, component, false);
                fsevent::CFRelease(placeholder);
                placeholder = url;
                count = count - 1;
            }
            fsevent::CFRelease(imaginary);
        }

        fsevent::CFShow(placeholder);
    }
}
