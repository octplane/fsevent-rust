#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate getopts;
extern crate libc;

use std::c_str::CString;
use libc::types::os::arch::c95::c_char;
use libc::types::common::c95::c_void;

use getopts::{optopt,getopts,OptGroup};

use std::os;

#[repr(C)]
type CFRunLoopRef = u64;

#[repr(C)]
type FSEventStreamEventId = u64;

#[repr(C)]
type size_t = u64;

#[repr(C)]
type CFTImeInterval = f64;

#[repr(C)]
type FSEventStreamCreateFlags = u32;

#[repr(C)]
struct FSStreamEvent;

#[repr(C)]
struct FSClientCallBackInfo;

#[repr(C)]
struct FSEventPaths;

#[repr(C)]
struct FSEventFlags;

#[repr(C)]
struct FSEventIds;

#[repr(C)]
type CFAllocatorRef = u64;


#[repr(C)]
type CFStringRef = &'static str;

#[repr(C)]
type CFArrayRef = u64;

static KFSEventStreamEventIdSinceNow: FSEventStreamEventId = 0xFFFFFFFFFFFFFFFF;
static KCFAllocatorDefault: CFAllocatorRef = 0;
static kCFRunLoopDefaultMode: CFStringRef = "kCFRunLoopDefaultMode";

// #[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "CoreServices", kind = "framework")]
extern {
    fn CFArrayCreate (
       allocator: CFAllocatorRef,
       values: &[*const c_char],
       numValues: uint,
       dummy: u64
    ) -> CFArrayRef;

    fn FSEventsGetCurrentEventId() -> FSEventStreamEventId;

    fn FSEventStreamCreate(
        allocator: CFAllocatorRef,
        callback: extern fn(
            stream_ref: *const FSStreamEvent,
            client_call_back_info: *const FSClientCallBackInfo,
            num_events: size_t,
            event_paths: *const FSEventPaths,
            event_flags: *const FSEventFlags,
            event_ids: *const FSEventIds
            ),
        context: u64,
        pathsToWatch: *const int,
        sinceWhen: FSEventStreamEventId,
        latency: CFTImeInterval,
        flags: FSEventStreamCreateFlags,
    );
    fn FSEventStreamScheduleWithRunLoop(
        stream: FSEventStreamEventId,
        runLoop: CFRunLoopRef,
        runLoopMode: CFStringRef
        );

    fn CFRunLoopGetCurrent() -> CFRunLoopRef;

}

fn fs_get_current_event_id() -> FSEventStreamEventId {
    unsafe {
        let id = FSEventsGetCurrentEventId();
        id
    }
}

fn cf_run_loop_get_current() -> CFRunLoopRef {
    unsafe {
        let re = CFRunLoopGetCurrent();
        re
    }
}

fn cf_array_create(ary:Vec<&str>) -> CFArrayRef {    
    let pointers_vector: Vec<*const c_char> =  ary.
        iter().
        map( |it| it.to_c_str()).
        map( |it| it.as_ptr()).
        collect();

    let values = pointers_vector.as_slice();
    

    unsafe {
        let cf_array =  CFArrayCreate (
            KCFAllocatorDefault,
            values,
            values.len(),
            0
        );
        cf_array
    }
}



fn print_usage(program: &str, _opts: &[OptGroup]) {
    println!("Usage: {} [options]", program);
    println!("-p path\t\tPath to observe (default: .)");
    println!("-c command\t\tCommand to run (default: cargo build)");
    println!("-h --help\tUsage");
}

fn main() {
    let args: Vec<String> = os::args();
    let program = args[0].clone();

    let opts = [
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

    let last_id = fs_get_current_event_id();
    //let run_loop = cf_run_loop_get_current();
    
    let cf_array = cf_array_create(vec!["hop", "hap"]);
    
    println!("{} {} {} {}", path, command, last_id, cf_array);

}

