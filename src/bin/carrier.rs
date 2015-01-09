#![feature(phase)]
#![feature(link_args)]
#![allow(unused_variables)]



extern crate getopts;
extern crate libc;
extern crate fsevent;


use getopts::{optopt,getopts,OptGroup};
use std::os;
use std::ffi::CString;

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
    let fsevent: FsEvent;
    unsafe {
        fsevent = FsEvent{
            paths: fsevent::CFArrayCreateMutable(fsevent::kCFAllocatorDefault, 0, &fsevent::kCFTypeArrayCallBacks),
            since_when: fsevent::kFSEventStreamEventIdSinceNow,
            latency: 0.1,
//            flags: fsevent::kFSEventStreamCreateFlagNone,
        };
    }

    fsevent.add("./src/temp/build/pipo");
    let stream_context = fsevent::FSEventStreamContext{
        version: 0,
        info: fsevent::NULL,
        retain: fsevent::NULL,
        copy_description: fsevent::NULL };

    // let stream = FSEventStreamCreate(kCFAllocatorDefault,
    //    (FSEventStreamCallback)&callback,
    //    &stream_context,
    //    config.paths,
    //    config.sinceWhen,
    //    config.latency,
    //    config.flags);
}

struct FsEvent {
    paths: fsevent::CFMutableArrayRef,
    since_when: fsevent::FSEventStreamEventId,
    latency: fsevent::CFTimeInterval,
//    flags: fsevent::FSEventStreamCreateFlags,
}

impl FsEvent {

    // https://github.com/thibaudgg/rb-fsevent/blob/master/ext/fsevent_watch/main.c
    fn add(&self,source: &str) {
        unsafe {
            let cp = CString::from_slice(source.as_bytes());
            let c_path = cp.as_slice_with_nul();
            let mut url = fsevent::CFURLCreateFromFileSystemRepresentation(fsevent::kCFAllocatorDefault, c_path.as_ptr(), c_path.len() as i64, false);
            let mut placeholder = fsevent::CFURLCopyAbsoluteURL(url);
            fsevent::CFRelease(url);

            let imaginary: fsevent::CFRef = fsevent::CFArrayCreateMutable(fsevent::kCFAllocatorDefault, 0, &fsevent::kCFTypeArrayCallBacks);

            while !fsevent::CFURLResourceIsReachable(placeholder, fsevent::kCFAllocatorDefault) {

                let child = fsevent::CFURLCopyLastPathComponent(placeholder);
                fsevent::CFArrayInsertValueAtIndex(imaginary, 0, child);
                fsevent::CFRelease(child);
                let component = fsevent::CFArrayGetValueAtIndex(imaginary, 0);

                url = fsevent::CFURLCreateCopyDeletingLastPathComponent(fsevent::kCFAllocatorDefault, placeholder);
                fsevent::CFRelease(placeholder);
                placeholder = url;
            }

            url = fsevent::CFURLCreateFileReferenceURL(fsevent::kCFAllocatorDefault, placeholder, fsevent::kCFAllocatorDefault);
            fsevent::CFRelease(placeholder);
            placeholder = fsevent::CFURLCreateFilePathURL(fsevent::kCFAllocatorDefault, url, fsevent::kCFAllocatorDefault);
            fsevent::CFRelease(url);

            fsevent::CFShow(imaginary);
            let component = fsevent::CFArrayGetValueAtIndex(imaginary, 1) as fsevent::CFStringRef;
            fsevent::CFShow(component);


            if imaginary != fsevent::kCFAllocatorDefault {
                let mut count =  0;
                while { count < fsevent::CFArrayGetCount(imaginary) }
                {
                    let component = fsevent::CFArrayGetValueAtIndex(imaginary, count);
                    fsevent::CFShow(component);
                    url = fsevent::CFURLCreateCopyAppendingPathComponent(fsevent::kCFAllocatorDefault, placeholder, component, false);
                    fsevent::CFRelease(placeholder);
                    placeholder = url;
                    count = count + 1;
                }
                fsevent::CFRelease(imaginary);
            }


            let cf_path = fsevent::CFURLCopyFileSystemPath(placeholder, fsevent::kCFURLPOSIXPathStyle);
            fsevent::CFArrayAppendValue(self.paths, cf_path);
            fsevent::CFRelease(cf_path);
            fsevent::CFRelease(placeholder);
        }
    }
}