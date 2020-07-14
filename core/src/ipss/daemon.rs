use std::fs::File;
use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher, op};
use std::sync::mpsc::channel;
use crate::replication::engine;
use dirs;
use signal_hook::{iterator::Signals, SIGINT};
use std::{error::Error, thread, process};

pub fn init() -> Result<(), Box<dyn Error>> {
  println!("Initializing the daemon...");
  // Create a channel to receive the events.
  let (tx, rx) = channel();

  // Create a watcher object, delivering raw events.
  // The notification back-end is selected based on the platform.
  let mut watcher = raw_watcher(tx).unwrap();

  // Add a path to be watched. All files and directories at that path and
  // below will be monitored for changes.
  if let Some(dir) = dirs::download_dir() {
    watcher.watch(dir, RecursiveMode::Recursive).unwrap();
  }
  if let Some(dir) = dirs::document_dir() {
    watcher.watch(dir, RecursiveMode::Recursive).unwrap();
  }
  if let Some(dir) = dirs::desktop_dir() {
    watcher.watch(dir, RecursiveMode::Recursive).unwrap();
  }

  let signals = Signals::new(&[SIGINT])?;
  thread::spawn(move || {
    for _ in signals.forever() {
      println!("\nExiting the daemon...");
      process::exit(1)
    }
  });


  println!("Waiting for changes...");
  loop {
    match rx.recv() {
      Ok(RawEvent { path: Some(path), op: Ok(op), cookie }) => {
        println!("{:?} {:?} ({:?})", op, path, cookie);
        match op {
          op::CREATE => {
            match File::open(path) {
              Ok(file) => engine::add(&file),
              _ => ()
            }
          },
          op::CLOSE_WRITE => {
            match File::open(path) {
              Ok(file) => engine::modify(&file),
              _ => ()
            }
          },
          op::REMOVE => {
            match File::open(path) {
              Ok(file) => engine::remove(&file),
              _ => ()
            }
          },
          op::RENAME => {
            match File::open(path) {
              Ok(file) => engine::rename(&file),
              _ => ()
            }
          },
          _ => {}
        }
      },
      Ok(event) => println!("broken event: {:?}", event),
      Err(e) => println!("watch error: {:?}", e),
    }
  }
}