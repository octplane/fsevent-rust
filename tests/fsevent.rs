extern crate fsevent;
extern crate tempdir;
extern crate time;

use fsevent::*;
use std::io::Write;
use std::fs::OpenOptions;
use std::fs::read_link;
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use tempdir::TempDir;
use std::env;

const TIMEOUT_S: f64 = 5.0;

fn validate_recv(rx: Receiver<Event>, evs: Vec<(String, StreamFlags)>) {
  let mut deadline = time::precise_time_s() + TIMEOUT_S;
  let mut evs = evs.clone();

  while (time::precise_time_s() < deadline) {
      if let Ok(actual) = rx.try_recv() {
          println!("actual: {:?}", actual);
          let mut found: Option<usize> = None;
          for i in (0..evs.len()) {
            let expected = evs.get(i).unwrap();
            if actual.path == expected.0 && actual.flag == expected.1 {
                found = Some(i);
                break;
            }
          }
          if let Some(i) = found {
              evs.remove(i);
          } else {
            assert!(false, format!("actual: {:?} not found in expected: {:?}", actual, evs));
          }
      }
      if evs.is_empty() { break; }
  }
  assert!(evs.is_empty(),
          "Some expected events did not occur before the test timedout:\n\t\t{:?}", evs);
}


// TODO: replace with std::fs::canonicalize rust-lang/rust#27706.
fn resolvePath(path: &str) -> PathBuf {
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
          _ => ()
        }
    }
    out
}

#[test]
fn validate_watch_single_file() {
  let dir = TempDir::new("dir").unwrap();
  // Resolve path so we don't have to worry about affect of symlinks on the test.
  let mut dst = resolvePath(dir.path().to_str().unwrap());
  dst.push("out.txt");
  let (sender, receiver) = channel();

  let mut file = OpenOptions::new().write(true).create(true).open(dst.clone().as_path()).unwrap();
  file.write_all(b"create").unwrap();
  file.flush().unwrap();
  drop(file);
  println!("Just created file");

  {
    let dst = dst.clone();
    let _t = thread::spawn(move || {
      thread::sleep_ms(1000); // Wait a while after create above before observing.
      println!("Will observe now");
      let fsevent = fsevent::FsEvent::new(sender);
      fsevent.append_path(dst.as_path().to_str().unwrap());
      fsevent.observe();
    });
  }

  {
    let dst = dst.clone();
    let t3 = thread::spawn(move || {
      thread::sleep_ms(1500); // Wait another 500ms after observe.
      println!("Will append to file now");
      let mut file = OpenOptions::new().write(true).append(true).open(dst.as_path()).unwrap();
      file.write_all(b"foo").unwrap();
      file.flush().unwrap();
    });
    t3.join().unwrap();
  }

  validate_recv(receiver, vec![
    (dst.to_str().unwrap().to_string(), ITEM_MODIFIED | IS_FILE),
  ]);
}
