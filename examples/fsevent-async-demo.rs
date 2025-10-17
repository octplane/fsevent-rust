use std::{sync::mpsc::channel, thread};

#[cfg(not(target_os = "macos"))]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {
    let (sender, receiver) = channel();

    let t = thread::spawn(move || {
        let mut fsevent = fsevent::FsEvent::new(vec![".".to_string()]);
        fsevent.observe_async(sender).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5)); // sleep five seconds
        fsevent.shutdown_observe();
    });

    loop {
        let duration = std::time::Duration::from_secs(1);
        match receiver.recv_timeout(duration) {
            Ok(val) => println!("{:?}", val),
            // This is the case where nothing entered the channel buffer (no file mods).
            Err(e) => {
                if e == std::sync::mpsc::RecvTimeoutError::Disconnected {
                    break;
                }
            }
        }
    }

    t.join().unwrap();
}
