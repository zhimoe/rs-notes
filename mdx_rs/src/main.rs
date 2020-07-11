use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use rusqlite::{Connection, named_params, params};
use warp::{Filter};
use warp::http::{Response};

use crate::mdx::{Mdx, RecordIndex};


mod checksum;
mod mdx;
mod number;
mod unpack;

const MDX_PATH: &str = "/home/cod3fn/code/rs-notes/resources/LSC4.mdx";

fn query(word: String) -> String {
    let w = word;

    let mut db_file = MDX_PATH.to_string();
    db_file.push_str(".db");
    let conn = Connection::open(&db_file).unwrap();
    let mut stmt = conn.prepare("select * from MDX_INDEX WHERE key_text= :word;").unwrap();
    println!("query params={}", &w);
    let mut rows = stmt.query_named(named_params! { ":word": w }).unwrap();
    let row = rows.next().unwrap();
    if let Some(row) = row {
        let idx = RecordIndex {
            key_text: row.get::<usize, String>(0).unwrap(),
            file_pos: row.get::<usize, u32>(1).unwrap() as u32,
            compressed_size: row.get::<usize, u32>(2).unwrap() as u32,
            decompressed_size: row.get::<usize, u32>(3).unwrap() as u32,
            record_block_type: row.get::<usize, u8>(4).unwrap() as u32,
            record_start: row.get::<usize, i32>(5).unwrap() as u32,
            record_end: row.get::<usize, i32>(6).unwrap() as u32,
            offset: row.get::<usize, i32>(7).unwrap() as u32,
        };

        let mut reader = BufReader::new(File::open(&MDX_PATH).unwrap());
        reader.seek(SeekFrom::Start(idx.file_pos as u64)).expect("reader seek error");

        let mut record_block_compressed: Vec<u8> = vec![0; idx.compressed_size as usize];
        reader.read_exact(&mut record_block_compressed).expect("read record_block_compressed error ");
        return Mdx::extract_definition(&mut record_block_compressed,
                                       idx.record_start as usize,
                                       idx.record_end as usize,
                                       idx.offset as usize);
    }
    return "not found".to_string();
}


fn indexing(db_file: &str, conn: &mut Connection, mdx: &Mdx) {
    conn.execute(
        "create table if not exists MDX_INDEX (
                key_text text not null,
                file_pos integer,
                compressed_size integer,
                decompressed_size integer,
                record_block_type integer,
                record_start integer,
                record_end integer,
                offset integer
         )",
        params![],
    ).expect("create db error");

    if std::path::PathBuf::from(db_file).exists() {
        println!("new db created");
    }
    let tx = conn.transaction().unwrap();
    for r in &mdx.records {
        tx.execute(
            "INSERT INTO MDX_INDEX VALUES (?,?,?,?,?,?,?,?)",
            params![
            r.key_text,
            r.file_pos as i32,
            r.compressed_size as i32,
            r.decompressed_size as i32 ,
            r.record_block_type as u32,
            r.record_start as i32,
            r.record_end as i32 ,
            r.offset as i32],
        ).expect("indexing mdx record info error");
    }
    tx.commit().expect("tx commit error");
    println!("indexing record info done");
}

#[tokio::main]
async fn main() {
    let mdx = Mdx::new(MDX_PATH);

    let mdx_file = &mdx.header.file;
    let mut db_file = mdx_file.clone();
    db_file.push_str(".db");

    if true {
        if std::path::PathBuf::from(&db_file).exists() {
            std::fs::remove_file(&db_file).expect("remove old db error");
            println!("Removing old db file:{}", &db_file);
        }
        let mut conn = Connection::open(&db_file).unwrap();
        indexing(&db_file, &mut conn, &mdx);
    }


    // get /q?key=value
    let query = warp::get()
        .and(warp::path("q"))
        .and(warp::query::<HashMap<String, String>>())
        .map(|p: HashMap<String, String>| match p.get("key") {
            Some(key) => Response::builder()
                .header("content-type", "text/html; charset=UTF-8")
                .body(format!("{}", query(key.clone()))),
            None => Response::builder().body(String::from("No \"key\" param in query.")),
        });


    let css = warp::path("LSC4.css").and(warp::fs::file("static/LSC4.css"));

    let routes = query.or(css);
    println!("server listening on localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}



