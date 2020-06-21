extern crate threadpool;

use std::{fs, io};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;

use threadpool::ThreadPool;

fn main() -> io::Result<()> {
    let n_workers = 24;
    let pool = ThreadPool::new(n_workers);
    let start = SystemTime::now();
    let paths = fs::read_dir("/home/cod3fn/test/txt")?;
    let mut count = 0;
    for path in paths {
        if let Ok(file) = path {
            let p = file.path();
            pool.execute(move || {
                search_file(p);
            });
        }
        count += 1;
    }
    println!("total count={}", count);
    pool.join();
    println!("took {} ms", start.elapsed().unwrap().as_millis());
    Ok(())
}

fn search_file(p: PathBuf) {
    let mut reader = BufReader::new(File::open(p).unwrap());
    let mut bv: Vec<u8> = Vec::new();
    loop {
        let size = reader.read_until(b'\n', &mut bv);
        match size {
            Ok(0) => break,
            _ => {}
        }
        let line = String::from_utf8_lossy(&bv);
        if line.contains("hello") {
            println!("{}", line);
        }
        bv.clear();
    }
}