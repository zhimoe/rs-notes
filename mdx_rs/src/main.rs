#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};


use flate2::write::ZlibDecoder;
use rocket::config::{Config, Environment};
use rocket::response::content::{Html};
use rocket_contrib::serve::StaticFiles;
use rusqlite::{Connection, named_params, params, Result};

use crate::checksum::adler32_checksum;
use crate::mdx::{Mdx, RecordIndex};
use crate::unpack::Endian;

mod checksum;
mod mdx;
mod number;
mod unpack;

fn main() -> Result<()> {
    let mdx = Mdx::new("/home/cod3fn/code/rs-notes/resources/LSC4.mdx");

    // write the record_list info to sqlite
    let mdx_file = &mdx.header.file;
    let mut db_file = mdx_file.clone();
    db_file.push_str(".db");

    if std::path::PathBuf::from(&db_file).exists() {
        println!("Removing old db file");
        std::fs::remove_file(&db_file);
    }

    let mut conn = Connection::open(&db_file).unwrap();
    // key_text: String,
    // file_pos: u32,
    // compressed_size: u32,
    // decompressed_size: u32,
    // record_block_type: u32,
    // record_start: u32,
    // record_end: u32,
    // offset: u32,

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
    ).unwrap();
    //
    if std::path::PathBuf::from(db_file).exists() {
        println!("new db file created");
    }

    let tx = conn.transaction()?;


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
        )?;
    }
    tx.commit().expect("tx commit error");
    println!("insert all mdx record index to sqlite");

    // #[catch(404)]
    // fn not_found(req: &rocket::Request) -> String {
    //     format!("{:?}", "")
    // }

    #[get("/q/<word>")]
    fn index(word: String) -> Html<String> {
        let w = word;
        let mut conn = Connection::open("/home/cod3fn/code/rs-notes/resources/LSC4.mdx.db").unwrap();
        let mut stmt = conn.prepare("select * from MDX_INDEX WHERE key_text= :word;").unwrap();
        println!("query params={}", &w);
        let mut rows = stmt.query_named(named_params! { ":word": w }).unwrap();
        let row = rows.next().unwrap().unwrap();
        let r = RecordIndex {
            key_text: row.get::<usize, String>(0).unwrap(),
            file_pos: row.get::<usize, u32>(1).unwrap() as u32,
            compressed_size: row.get::<usize, u32>(2).unwrap() as u32,
            decompressed_size: row.get::<usize, u32>(3).unwrap() as u32,
            record_block_type: row.get::<usize, u8>(4).unwrap() as u32,
            record_start: row.get::<usize, i32>(5).unwrap() as u32,
            record_end: row.get::<usize, i32>(6).unwrap() as u32,
            offset: row.get::<usize, i32>(7).unwrap() as u32,
        };

        let idx = r;
        println!("Found Dict Index {:?}", &idx);
        let mut reader = BufReader::new(File::open("/home/cod3fn/code/rs-notes/resources/LSC4.mdx").unwrap());
        reader.seek(SeekFrom::Start(idx.file_pos as u64)).expect("reader seek error");


        let mut record_block_compressed: Vec<u8> = vec![0; idx.compressed_size as usize];
        reader.read_exact(&mut record_block_compressed);

        let record_block_type = &record_block_compressed[0..4];
        let adler32_bytes = &record_block_compressed[4..8];
        let mut record_block_decompressed = Vec::new();
        assert_eq!(b"\x02\x00\x00\x00", record_block_type);

        let mut z = ZlibDecoder::new(record_block_decompressed);
        z.write_all(&record_block_compressed[8..]).unwrap();
        record_block_decompressed = z.finish().unwrap();
        if !adler32_checksum(&record_block_decompressed, &adler32_bytes, Endian::BE) {
            panic!("record block adler32 checksum failed");
        }
        let s = idx.record_start - idx.offset;
        let e = idx.record_end - idx.offset;

        let record = &record_block_decompressed[s as usize..e as usize];
        let def = String::from_utf8_lossy(record);
        println!("def = {}  ", &def);

        Html(def.to_string())
    }

    let config = Config::build(Environment::Staging)
        .address("127.0.0.1")
        .port(9234)
        .unwrap();

    rocket::custom(config)
        .mount("/q/LSC4.css", StaticFiles::from("resources/"))
        .mount("/", routes![index])
        .launch();


    Ok(())
}


// fn get_mdx_by_index(reader: &mut BufReader<File>, index: &idx::DictIndex) -> String {
//     reader.seek(SeekFrom::Start(index.file_pos as u64));
//     let mut stmt = conn.prepare("select * from MDX_INDEX WHERE key_text='sloppy1111';")?;
//     let r_iter = stmt.query_map(params![], |row| {
//         Ok(row.get::<usize, String>(0).unwrap())
//     })?;
//     //  cursor = conn.execute("SELECT * FROM MDX_INDEX WHERE key_text = " + "\"" + keyword + "\"")
//     //         lookup_result_list = []
//     //         mdd_file = open(self._mdd_file,'rb')
//     //         for result in cursor:
//     //             index = {}
//     //             index['file_pos'] = result[1]
//     //             index['compressed_size'] = result[2]
//     //             index['decompressed_size'] = result[3]
//     //             index['record_block_type'] = result[4]
//     //             index['record_start'] = result[5]
//     //             index['record_end'] = result[6]
//     //             index['offset'] = result[7]
//     //             lookup_result_list.append(self.get_mdd_by_index(mdd_file, index))
//     //         mdd_file.close()
//     //         conn.close()
//     //         return lookup_result_list
//     String::from("")
// }

