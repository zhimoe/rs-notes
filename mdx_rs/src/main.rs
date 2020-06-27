#[macro_use]
extern crate hex_literal;
extern crate ripemd128;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use adler32::RollingAdler32;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::write::ZlibDecoder;
use miniz_oxide::inflate::decompress_to_vec;
use rbtree::RBTree;
use regex::Regex;
use ripemd128::{Digest, Ripemd128};

use rusqlite::{params, Connection, Result};

use crate::model::header::HeaderTag;
use crate::util::{adler32_checksum, NumberBytes};


mod model;
mod util;


fn main() -> Result<()> {
    let mdxfile = String::from("/home/cod3fn/code/rs-notes/resources/LSC4.mdx");
    let mut htb = model::header::HeaderTagBuilder::default();
    htb.file(mdxfile.to_owned());
    let mut reader = BufReader::new(File::open(&mdxfile).unwrap());
    let mut first_4bytes = [0; 4];
    // read exactly 4 bytes
    reader.read_exact(&mut first_4bytes);
    let header_len = util::be_bytes_unpack(&first_4bytes);
    let _key_block_offset: i32 = (4 + header_len + 4) as i32;
    let mut header_bytes = vec![0; header_len as usize];

    // read the header tag bytes
    reader.read_exact(&mut header_bytes);
    // reade 4 bytes: adler32 checksum of header, in little endian
    let mut adler32_bytes = [0; 4];
    reader.read_exact(&mut adler32_bytes);

    if !util::adler32_checksum(&header_bytes, &adler32_bytes, false) {
        panic!("unrecognized format");
    } else {
        println!("header bytes adler32_checksum success")
    }

    let current_pos = reader.seek(SeekFrom::Current(0)).expect("Could not get current position!");
    htb.key_block_offset(current_pos);

    // header text in utf-16 encoding ending with '\x00\x00'
    let (header, _end) = header_bytes.split_at(header_bytes.len() - 2);
    let header_txt = util::le_u8_string(&header).unwrap();

    println!("head_tag:{}", header_txt);

    let mut _header_map = HashMap::new();
    let re = Regex::new(r#"(\w+)=["](.*?)["]"#).unwrap();
    let caps = re.captures_iter(header_txt.as_str());
    for cap in caps {
        _header_map.insert(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str());
    }

    if let Some(v) = _header_map.get(&"GeneratedByEngineVersion") {
        htb.genversion(v.parse::<f32>().unwrap());
    }
    let _version = htb.genversion;
    if let Some(f) = _header_map.get(&"Format") {
        htb.format(f.to_string());
    }
    if let Some(k) = _header_map.get(&"KeyCaseSensitive") {
        if k == &"Yes" {
            htb.keycasesensitive(true);
        } else {
            htb.keycasesensitive(false);
        }
    }
    if let Some(s) = _header_map.get(&"StripKey") {
        if s == &"Yes" {
            htb.stripkey(true);
        } else {
            htb.stripkey(false);
        }
    }

    if let Some(en) = _header_map.get(&"Encrypted") {
        htb.encrypted(en.to_string());
    }
    if let Some(r) = _header_map.get(&"RegisterBy") {
        htb.registerby(r.to_string());
    }
    if let Some(Encoding) = _header_map.get(&"Encoding") {
        htb.encoding(Encoding.to_string());
    }


    if let Some(d) = _header_map.get(&"DataSourceFormat") {
        htb.datasourceformat(d.to_string());
    }
    if let Some(s) = _header_map.get(&"StyleSheet") {
        htb.stylesheet(s.to_string());
    }
    if let Some(c) = _header_map.get(&"Compact") { //or Compat
        if c == &"Yes" {
            htb.compact(true);
        } else {
            htb.compact(false);
        }
    }
    if let Some(l) = _header_map.get(&"Left2Right") {
        if l == &"Yes" {
            htb.left2right(true);
        } else {
            htb.left2right(false);
        }
    }

    // key block info
    let _number_width = if _version >= 2.0 { 8 } else { 4 };
    let num_bytes = if _version >= 2.0 { 8 * 5 } else { 4 * 4 };
    let mut key_block_info_bytes = vec![0; num_bytes];
    reader.read_exact(&mut key_block_info_bytes);

    let mut nb = NumberBytes::new(&key_block_info_bytes);
    let _num_key_blocks = nb.read_number(_number_width);
    let _num_entries = nb.read_number(_number_width);

    if _version >= 2.0 {
        let _key_block_info_decompress_size = nb.read_number(_number_width);
    }
    let key_block_info_size = nb.read_number(_number_width);
    let key_block_size = nb.read_number(_number_width);


    // reade 4 bytes: adler32 checksum of key block info, in big endian
    let mut adler32_bytes = [0; 4];
    reader.read_exact(&mut adler32_bytes);

    if !util::adler32_checksum(&key_block_info_bytes, &adler32_bytes, true) {
        panic!("unrecognized format");
    } else {
        println!("key block info adler32_checksum success")
    }

    let mut key_block_info_bytes = vec![0; key_block_info_size.unwrap() as usize];
    reader.read_exact(&mut key_block_info_bytes);

    let current_pos = reader.seek(SeekFrom::Current(0)).expect("Could not get current position!");
    htb.record_block_offset(current_pos);

    let header_tag = htb.build();
    let mut key_block_info_list = decode_key_block_info(&key_block_info_bytes, &header_tag);

    let mut key_block_bytes = vec![0; key_block_size.unwrap() as usize];
    reader.read_exact(&mut key_block_bytes);

    let mut key_list = decode_key_block(&key_block_bytes, &key_block_info_list);

    let _record_block_offset = reader.seek(SeekFrom::Current(0)).expect("Could not get current position!");
    println!("_record_block_offset={}", _record_block_offset);

    //parse record block
    let num_record_blocks = read_number(&mut reader, _number_width);
    let num_entries = read_number(&mut reader, _number_width);
    let record_block_info_size = read_number(&mut reader, _number_width);
    let record_block_size = read_number(&mut reader, _number_width);

    let mut record_block_info_list: Vec<(usize, usize)> = vec![];
    let mut size_counter = 0;
    // read all record_block_info bytes
    for i in 0..num_record_blocks {
        let compressed_size = read_number(&mut reader, _number_width);
        let decompressed_size = read_number(&mut reader, _number_width);
        record_block_info_list.push((compressed_size, decompressed_size));
        size_counter += _number_width * 2
    }
    assert_eq!(size_counter, record_block_info_size);

    // start read record block, decompress it
    let mut record_list: Vec<model::dict_index::Record> = vec![]; // important!
    let mut record_block_bytes_size_counter = 0;
    let mut i: usize = 0;
    let mut offset: usize = 0;
    for (c_size, d_size) in record_block_info_list {
        let cur_pos = reader.seek(SeekFrom::Current(0)).expect("Could not get current position!");
        // println!("cur_pos={}", cur_pos);
        let mut record_block_compressed: Vec<u8> = vec![0; c_size];
        reader.read_exact(&mut record_block_compressed);

        let record_block_type = &record_block_compressed[0..4];
        let adler32_bytes = &record_block_compressed[4..8];
        let mut record_block_decompressed = Vec::new();
        let mut _type = 2;
        match record_block_type {
            b"\x02\x00\x00\x00" => {
                _type = 2;
                let mut z = ZlibDecoder::new(record_block_decompressed);
                z.write_all(&record_block_compressed[8..]).unwrap();
                record_block_decompressed = z.finish().unwrap();
                if !adler32_checksum(&record_block_decompressed, &adler32_bytes, true) {
                    panic!("record block adler32 checksum failed");
                }
            }
            b"\x01\x00\x00\x00" => { println!("\x01\x00\x00\x00") }
            b"\x00\x00\x00\x00" => { println!("\x00\x00\x00\x00") }
            _ => {}
        }

        // split record block into record according to the offset info from key block
        while i < key_list.len() {
            let (id, text) = &key_list[i];
            let mut start = *id as usize;
            if start - offset >= d_size {
                break;
            }
            let mut record_end: usize = 0;
            if i < key_list.len() - 1 {
                record_end = key_list[i + 1].0 as usize;
            } else {
                record_end = d_size + offset;
            }
            let idx = model::dict_index::Record {
                file_pos: cur_pos as usize,
                compressed_size: c_size,
                decompressed_size: d_size,
                record_block_type: _type,
                record_start: id.clone() as usize,
                text: text.to_string(),
                offset: offset as usize,
                record_end: record_end as usize,
            };
            i += 1;

            // println!("start={},record_end={},offset={}", &start, &record_end, &offset);
            let mut record = &record_block_decompressed[(start - offset)..(record_end - offset)];
            // println!("{:x? }", &record[..(&record.len() - 3)]);
            let record_text = String::from_utf8_lossy(&record[..(&record.len() - 3)]);
            // if self._substyle and self._stylesheet:
            //     record = self._substitute_stylesheet(record)

            record_list.push(idx)
        }
        offset += d_size;
        record_block_bytes_size_counter += c_size;
    }

    // write the record_list info to sqlite
    let mut db_file = htb.file.clone();
    db_file.push_str(".db");
    if std::path::PathBuf::from(&db_file).exists() {
        println!("Removing old database");
        std::fs::remove_file(&db_file);
    }


    let mut conn = Connection::open(&db_file)?;

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
    )?;

    if std::path::PathBuf::from(db_file).exists() {
        println!("db_file created");
    }

    let tx = conn.transaction()?;


    for r in &record_list {
        tx.execute(
            "INSERT INTO MDX_INDEX VALUES (?,?,?,?,?,?,?,?)",
            params![r.text, r.file_pos as i32,r.compressed_size as i32,r.decompressed_size as i32 ,r.record_block_type,r.record_start as i32,r.record_end as i32 ,r.offset as i32],
        )?;
    }
    tx.commit();

    let mut stmt = conn.prepare("SELECT count(1) FROM MDX_INDEX")?;
    let mut rows = stmt.query(params![])?;
    let count = rows.next().unwrap().unwrap();
    println!("the word key count={:?}", count.get(0).unwrap());

    println!("mdx version {}", _version);
    println!("mdx filename {}", &htb.file);
    Ok(())
}

pub fn read_number(reader: &mut BufReader<File>, width: usize) -> usize {
    let mut buf: Vec<u8> = vec![0; width];
    reader.read_exact(&mut buf);
    let mut slice = &buf[..];
    return match width {
        8 => slice.read_u64::<BigEndian>().unwrap() as usize,
        4 => slice.read_u32::<BigEndian>().unwrap() as usize,
        2 => slice.read_u16::<BigEndian>().unwrap() as usize,
        _ => 0,
    };
}


pub fn decode_key_block_info(key_block_info_compressed: &Vec<u8>, header: &HeaderTag) -> Vec<(usize, usize)> {
    let mut first4 = &key_block_info_compressed[0..4];
    let mut adler32_bytes = &key_block_info_compressed[4..8];
    println!("adler32_bytes={:x?}", &adler32_bytes);
    let mut data = &key_block_info_compressed[8..];
    let mut dataf = vec![0; data.len()];
    let mut key_block_info = Vec::new();
    if header.genversion >= 2.0 {
        assert!(b"\x02\x00\x00\x00" == first4);
        let encrypted = header.encrypted.parse::<u32>().unwrap();
        if encrypted & 0x02 == 0x02 {
            // create a RIPEMD-128 hasher instance
            let mut hasher = Ripemd128::new();
            let mut key_postfix = vec![];
            key_postfix.write_u32::<LittleEndian>(0x3695).unwrap();

            // process input message
            hasher.input([&adler32_bytes, &key_postfix[..]].concat());
            // acquire hash digest in the form of GenericArray,
            // which in this case is equivalent to [u8; 16]
            let ga = hasher.result();
            let key = ga.as_slice();
            println!("{:x?}", &key);
            let mut previous: u8 = 0x36;
            for i in 0..data.len() {
                let mut t = (data[i] >> 4 | data[i] << 4) & 0xff;
                t = t ^ previous ^ (i & 0xff) as u8 ^ key[i % key.len()];
                previous = data[i].clone();
                dataf[i] = t;
            }

            let mut z = ZlibDecoder::new(key_block_info);
            z.write_all(dataf.as_ref()).unwrap();
            key_block_info = z.finish().unwrap();

            //data now is decrypted, then decompress
            // println!("key_block_info={:x?}", &key_block_info);
            if !adler32_checksum(&key_block_info, &adler32_bytes, true) {
                panic!("key_block_info adler32 checksum failed!")
            }
        }
    } else {
        key_block_info = key_block_info_compressed.clone();
    }

    //start decode
    // let mut key_block_info_list = vec![];
    let mut num_enteries = 0 as u64;
    let mut big_endian = true;
    let mut byte_width = 1;
    let mut text_term = 0;
    if header.genversion >= 2.0 {
        big_endian = false;
        byte_width = 2;
        text_term = 1;
    }
    let NUM_WIDTH = 8;
    let mut i = 0;
    let mut key_block_info_list: Vec<(usize, usize)> = vec![];
    use byteorder::{LittleEndian, ReadBytesExt, BigEndian};
    while i < key_block_info.len() {
        num_enteries += unpack_u64(&key_block_info[i..(i + NUM_WIDTH)]);
        println!("num_enteries={}, it should be 543255", &num_enteries);
        i += NUM_WIDTH;
        println!("current i={}", &i);
        let text_head_size = unpack_u16(&key_block_info[i..(i + byte_width)]);
        i += byte_width;
        println!("text_head_size={}", &text_head_size);
        i += (text_head_size + text_term) as usize;
        println!("current i={}", &i);
        let text_tail_size = unpack_u16(&key_block_info[i..(i + byte_width)]);
        println!("text_tail_size={}", &text_tail_size);
        i += byte_width; //todo:
        i += (text_tail_size + text_term) as usize;

        let key_block_compressed_size = unpack_u64(&key_block_info[i..(i + NUM_WIDTH)]);
        i += NUM_WIDTH;
        let key_block_decompressed_size = unpack_u64(&key_block_info[i..(i + NUM_WIDTH)]);
        i += NUM_WIDTH;
        key_block_info_list.push((key_block_compressed_size as usize, key_block_decompressed_size as usize))
    }
    println!("the key_block_info_list len= {}", &key_block_info_list.len());
    return key_block_info_list;
}

fn decode_key_block(key_block_bytes: &Vec<u8>, key_block_info_list: &Vec<(usize, usize)>) -> Vec<(i32, String)> {
    let mut key_list: Vec<(i32, String)> = vec![];
    let mut i = 0;
    let mut start = 0;
    let mut end = i;
    for (key_block_compressed_size, key_block_decompressed_size) in key_block_info_list {
        start = i;
        end += *key_block_compressed_size;
        let key_block_type = &key_block_bytes[start as usize..(start + 4) as usize];
        let adler32_bytes = &key_block_bytes[(start + 4) as usize..(start + 8) as usize];
        let mut key_block = Vec::new();

        match key_block_type {
            b"\x02\x00\x00\x00" => {
                let mut z = ZlibDecoder::new(key_block);
                z.write_all(&key_block_bytes[(start + 8)..end]).unwrap();
                key_block = z.finish().unwrap();
                let one_key_list = split_key_block(&key_block);
                for t in one_key_list {
                    key_list.push(t);
                }
            }
            b"\x01\x00\x00\x00" => { println!("\x01\x00\x00\x00") }
            b"\x00\x00\x00\x00" => { println!("\x00\x00\x00\x00") }
            _ => {}
        }

        i += *key_block_compressed_size as usize;
    }

    return key_list;
}


fn split_key_block(key_block: &Vec<u8>) -> Vec<(i32, String)> {
    let NUM_WIDTH: usize = 8;
    let mut key_list: Vec<(i32, String)> = vec![];
    let mut key_start_index = 0;
    let mut key_end_index = 0;

    let delimiter = b"\x00";
    let width = 1;
    let mut i = 0;

    while key_start_index < key_block.len() {
        let slice = &key_block[key_start_index..(key_start_index + NUM_WIDTH)];
        let key_id = unpack_u64(slice);

        i = key_start_index + NUM_WIDTH;
        while i < key_block.len() {
            if &key_block[i..(i + width)] == delimiter {
                key_end_index = i;
                break;
            }
            i += width;
        }
        let key_text = std::str::from_utf8(&key_block[(key_start_index + NUM_WIDTH)..(key_end_index)]).unwrap().to_string();
        key_start_index = key_end_index + width;
        // println!("key_id={},key_text={}", &key_id, &key_text);
        key_list.push((key_id as i32, key_text));
    }
    return key_list;
}


pub fn unpack_u64(byte: &[u8]) -> u64 {
    let mut out: &[u8] = byte.clone();
    let num = out.read_u64::<BigEndian>().unwrap();
    num
}

pub fn unpack_u16(byte: &[u8]) -> u16 {
    let mut out: &[u8] = byte.clone();
    let num = out.read_u16::<BigEndian>().unwrap();
    num
}
