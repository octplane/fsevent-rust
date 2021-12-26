#![cfg(target_os = "macos")]

use core_foundation::runloop::{CFRunLoopGetCurrent, CFRunLoopRef, CFRunLoopStop};
use fsevent::*;
use std::{
    fs,
    fs::{read_link, OpenOptions},
    io::Write,
    path::{Component, PathBuf},
    sync::mpsc::{channel, Receiver},
    thread,
    time::{Duration, SystemTime},
};

// Helper to send the runloop from an observer thread.
struct CFRunLoopSendWrapper(CFRunLoopRef);

// Safety: According to the Apple documentation, it is safe to send CFRef types across threads.
//
// https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/Multithreading/ThreadSafetySummary/ThreadSafetySummary.html
unsafe impl Send for CFRunLoopSendWrapper {}

fn validate_recv(rx: Receiver<Event>, evs: Vec<(String, StreamFlags)>) {
    let timeout: Duration = Duration::new(5, 0);
    let deadline = SystemTime::now() + timeout;
    let mut evs = evs.clone();

    while SystemTime::now() < deadline {
        if let Ok(actual) = rx.try_recv() {
            let mut found: Option<usize> = None;
            for i in 0..evs.len() {
                let expected = evs.get(i).unwrap();
                if actual.path == expected.0 && actual.flag == expected.1 {
                    found = Some(i);
                    break;
                }
            }
            if let Some(i) = found {
                evs.remove(i);
            } else {
                panic!("actual: {:?} not found in expected: {:?}", actual, evs);
            }
        }
        if evs.is_empty() {
            break;
        }
    }
    assert!(
        evs.is_empty(),
        "Some expected events did not occur before the test timedout:\n\t\t{:?}",
        evs
    );
}

// TODO: replace with std::fs::canonicalize rust-lang/rust#27706.
fn resolve_path(path: &str) -> PathBuf {
    let mut out = PathBuf::new();
    let buf = PathBuf::from(path);
    for p in buf.components() {
        match p {
            Component::RootDir => out.push("/"),
            Component::Normal(osstr) => {
                out.push(osstr);
                if let Ok(real) = read_link(&out) {
                    if real.is_relative() {
                        out.pop();
                        out.push(real);
                    } else {
                        out = real;
                    }
                }
            }
            _ => (),
        }
    }
    out
}

#[test]
fn observe_folder_sync() {
    internal_observe_folder(false);
}

#[test]
fn observe_folder_async() {
    internal_observe_folder(true);
}

fn internal_observe_folder(run_async: bool) {
    let dir = tempfile::Builder::new().prefix("dur").tempdir().unwrap();
    // Resolve path so we don't have to worry about affect of symlinks on the test.
    let dst = resolve_path(dir.path().to_str().unwrap());

    let mut dst1 = dst.clone();
    dst1.push("dest1");

    let ddst1 = dst1.clone();
    fs::create_dir(dst1.as_path().to_str().unwrap()).unwrap();

    let mut dst2 = dst.clone();

    dst2.push("dest2");
    let ddst2 = dst2.clone();
    fs::create_dir(dst2.as_path().to_str().unwrap()).unwrap();

    let mut dst3 = dst.clone();

    dst3.push("dest3");
    let ddst3 = dst3.clone();
    fs::create_dir(dst3.as_path().to_str().unwrap()).unwrap();

    let (sender, receiver) = channel();

    let mut async_fsevent = fsevent::FsEvent::new(vec![]);
    let runloop_and_thread = if run_async {
        async_fsevent
            .append_path(dst1.as_path().to_str().unwrap())
            .unwrap();
        async_fsevent
            .append_path(dst2.as_path().to_str().unwrap())
            .unwrap();
        async_fsevent
            .append_path(dst3.as_path().to_str().unwrap())
            .unwrap();
        async_fsevent.observe_async(sender).unwrap();

        None
    } else {
        let (tx, rx) = std::sync::mpsc::channel();
        let observe_thread = thread::spawn(move || {
            let runloop = unsafe { CFRunLoopGetCurrent() };
            tx.send(CFRunLoopSendWrapper(runloop)).unwrap();

            let mut fsevent = fsevent::FsEvent::new(vec![]);
            fsevent
                .append_path(dst1.as_path().to_str().unwrap())
                .unwrap();
            fsevent
                .append_path(dst2.as_path().to_str().unwrap())
                .unwrap();
            fsevent
                .append_path(dst3.as_path().to_str().unwrap())
                .unwrap();
            fsevent.observe(sender);
        });

        let runloop = rx.recv().unwrap();

        Some((runloop.0, observe_thread))
    };

    validate_recv(
        receiver,
        vec![
            (
                ddst1.to_str().unwrap().to_string(),
                StreamFlags::ITEM_CREATED | StreamFlags::IS_DIR,
            ),
            (
                ddst2.to_str().unwrap().to_string(),
                StreamFlags::ITEM_CREATED | StreamFlags::IS_DIR,
            ),
            (
                ddst3.to_str().unwrap().to_string(),
                StreamFlags::ITEM_CREATED | StreamFlags::IS_DIR,
            ),
        ],
    );

    if let Some((runloop, thread)) = runloop_and_thread {
        unsafe {
            CFRunLoopStop(runloop);
        }

        thread.join().unwrap();
    } else {
        async_fsevent.shutdown_observe();
    }
}

#[test]
fn validate_watch_single_file_sync() {
    internal_validate_watch_single_file(false);
}

#[test]
fn validate_watch_single_file_async() {
    internal_validate_watch_single_file(true);
}

fn internal_validate_watch_single_file(run_async: bool) {
    let dir = tempfile::Builder::new().prefix("dur").tempdir().unwrap();
    // Resolve path so we don't have to worry about affect of symlinks on the test.
    let mut dst = resolve_path(dir.path().to_str().unwrap());
    dst.push("out.txt");
    let (sender, receiver) = channel();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dst.clone().as_path())
        .unwrap();
    file.write_all(b"create").unwrap();
    file.flush().unwrap();
    drop(file);

    let mut async_fsevent = fsevent::FsEvent::new(vec![]);
    let runloop_and_thread = if run_async {
        let dst = dst.clone();
        async_fsevent
            .append_path(dst.as_path().to_str().unwrap())
            .unwrap();
        async_fsevent.observe_async(sender).unwrap();

        None
    } else {
        let (tx, rx) = std::sync::mpsc::channel();
        let dst = dst.clone();

        // Leak the thread.
        let observe_thread = thread::spawn(move || {
            let runloop = unsafe { CFRunLoopGetCurrent() };
            tx.send(CFRunLoopSendWrapper(runloop)).unwrap();

            let mut fsevent = fsevent::FsEvent::new(vec![]);
            fsevent
                .append_path(dst.as_path().to_str().unwrap())
                .unwrap();
            fsevent.observe(sender);
        });

        let runloop = rx.recv().unwrap();

        Some((runloop.0, observe_thread))
    };

    {
        let dst = dst.clone();
        let t3 = thread::spawn(move || {
            thread::sleep(Duration::new(15, 0)); // Wait another 500ms after observe.
            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(dst.as_path())
                .unwrap();
            file.write_all(b"foo").unwrap();
            file.flush().unwrap();
        });
        t3.join().unwrap();
    }

    validate_recv(
        receiver,
        vec![(
            dst.to_str().unwrap().to_string(),
            StreamFlags::ITEM_MODIFIED | StreamFlags::ITEM_CREATED | StreamFlags::IS_FILE,
        )],
    );

    if let Some((runloop, observe_thread)) = runloop_and_thread {
        unsafe {
            CFRunLoopStop(runloop);
        }

        observe_thread.join().unwrap();
    } else {
        async_fsevent.shutdown_observe();
    }
}
