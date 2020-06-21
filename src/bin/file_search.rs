use std::{fs, io};
use std::time::SystemTime;
use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let start = SystemTime::now();
    let paths = fs::read_dir("/home/cod3fn/test/txt");

    for path in paths? {
        let p = &path?.path();
        search_file(p);
    }
    println!("took {} ms", start.elapsed().unwrap().as_millis());
    Ok(())
}

fn search_file(p: &PathBuf) {
    let mut reader = BufReader::new(File::open(p).unwrap());
    let mut buf = String::new();
    loop {
        match reader.read_line(&mut buf) {
            Err(_) | Ok(0) => break,
            Ok(_) => {
                if buf.contains("hello") {
                    println!("{}", buf);
                }
                buf.clear();
            }
        }
    }
}