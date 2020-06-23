use rbtree::RBTree;
use std::io::{BufReader, Read};
use std::fs::File;
use adler32::RollingAdler32;

use regex::Regex;
use byteorder::{LittleEndian, ReadBytesExt, BigEndian};
use std::collections::HashMap;

mod model;
mod util;

fn main() {
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
    let adler32 = util::le_bytes_unpack(&adler32_bytes);


    // checksum
    let mut rolling_adler32 = RollingAdler32::new();
    rolling_adler32.update_buffer(&header_bytes);
    let header_hash = rolling_adler32.hash();

    if header_hash & 0xffffffff != adler32 as u32 {
        panic!("unrecognized format");
    } else {
        println!("sum & 0xffffffff={},adler32={}", header_hash & 0xffffffff, adler32);
    }


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

    if let Some(ver) = _header_map.get(&"GeneratedByEngineVersion") {
        htb.genversion(ver.parse::<f32>().unwrap());
    }
    let _version = htb.genversion;
    if let Some(Format) = _header_map.get(&"Format") {
        htb.format(Format.to_string());
    }
    if let Some(KeyCaseSensitive) = _header_map.get(&"KeyCaseSensitive") {
        if KeyCaseSensitive == &"Yes" {
            htb.keycasesensitive(true);
        } else {
            htb.keycasesensitive(false);
        }
    }
    if let Some(StripKey) = _header_map.get(&"StripKey") {
        if StripKey == &"Yes" {
            htb.stripkey(true);
        } else {
            htb.stripkey(false);
        }
    }

    if let Some(Encrypted) = _header_map.get(&"Encrypted") {
        htb.encrypted(Encrypted.to_string());
    }
    if let Some(RegisterBy) = _header_map.get(&"RegisterBy") {
        htb.registerby(RegisterBy.to_string());
    }
    if let Some(Encoding) = _header_map.get(&"Encoding") {
        htb.encoding(Encoding.to_string());
    }
    if let Some(Encoding) = _header_map.get(&"Encoding") {
        htb.encoding(Encoding.to_string());
    }

    if let Some(DataSourceFormat) = _header_map.get(&"DataSourceFormat") {
        htb.datasourceformat(DataSourceFormat.to_string());
    }
    if let Some(StyleSheet) = _header_map.get(&"StyleSheet") {
        htb.stylesheet(StyleSheet.to_string());
    }
    if let Some(Compact) = _header_map.get(&"Compact") { //or Compat
        if Compact == &"Yes" {
            htb.compact(true);
        } else {
            htb.compact(false);
        }
    }
    if let Some(Left2Right) = _header_map.get(&"Left2Right") {
        if Left2Right == &"Yes" {
            htb.left2right(true);
        } else {
            htb.left2right(false);
        }
    }

    let header_tag = htb.build();
    println!("header tag = {:?}", header_tag);

    // key block info
    let _number_width = if _version < 2.0 { 4 } else { 8 };
    let num_bytes = if _version < 2.0 { 4 * 4 } else { 8 * 5 + 4 };
    let mut num_bytes_buf = vec![0; num_bytes];
    reader.read_exact(&mut num_bytes_buf);

    let (mut key_block, _tail1) = num_bytes_buf.split_at(_number_width);
    let _num_key_blocks = key_block.read_u64::<BigEndian>().unwrap();
    println!("_num_key_blocks={}", _num_key_blocks);

    let (mut entries, _tail2) = _tail1.split_at(_number_width);
    let _num_entries = entries.read_u64::<BigEndian>().unwrap();
    println!("_num_entries={}", _num_entries);
    if _version >= 2.0 {
        let (mut key_block_info_decomp_size, _tail3) = _tail2.split_at(_number_width);
        let _key_block_info_decomp_size = key_block_info_decomp_size.read_u64::<BigEndian>().unwrap();
        println!("_key_block_info_decomp_size={}", _key_block_info_decomp_size);
    }

    // _num_entries = _read_number(sf);                                          // 2
    // if (_version >= 2.0) {
    //     _key_block_info_decomp_size = _read_number(sf);
    // }      //[3]
    // _key_block_info_size = _read_number(sf);                                  // 4
    // _key_block_size = _read_number(sf);                                       // 5

    println!("mdx version {}", _version);
    println!("mdx filename {}", header_tag.file);
}