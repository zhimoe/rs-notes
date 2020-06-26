use rbtree::RBTree;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::fs::File;
use adler32::RollingAdler32;

use regex::Regex;
use byteorder::{LittleEndian, ReadBytesExt, BigEndian, WriteBytesExt};
use std::collections::HashMap;
use crate::util::{NumberBytes, adler32_checksum};
use crate::model::header::HeaderTag;

use miniz_oxide::inflate::decompress_to_vec;

#[macro_use]
extern crate hex_literal;
extern crate ripemd128;

use ripemd128::{Ripemd128, Digest};
use flate2::write::ZlibDecoder;


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
    let _key_block_size = nb.read_number(_number_width);


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
    let mut key_block_info_list = decode_key_block_info_list(&key_block_info_bytes, &header_tag);

    // let mut key_block_bytes = vec![0; key_block_size.unwrap() as usize];
    // reader.read_exact(&mut key_block_info_bytes);

    // key_block_info = f.read(key_block_info_size)
    // key_block_info_list = self._decode_key_block_info(key_block_info)
    // assert (num_key_blocks == len(key_block_info_list))
    println!("mdx version {}", _version);
    println!("mdx filename {}", &htb.file);
}

pub fn decode_key_block_info_list(key_block_info_compressed: &Vec<u8>, header: &HeaderTag) -> Vec<(u64, u64)> {
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
            println!("key_block_info={:x?}", &key_block_info);
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
    let num_width = 8;
    let mut i = 0;
    let mut key_block_info_list: Vec<(u64, u64)> = vec![];
    use byteorder::{LittleEndian, ReadBytesExt, BigEndian};
    while i < key_block_info.len() {
        num_enteries += unpack_u64(&key_block_info[i..(i + num_width)]);
        println!("num_enteries={}, it should be 543255", &num_enteries);
        i += num_width;
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

        let key_block_compressed_size = unpack_u64(&key_block_info[i..(i + num_width)]);
        i += num_width;
        let key_block_decompressed_size = unpack_u64(&key_block_info[i..(i + num_width)]);
        i += num_width;
        key_block_info_list.push((key_block_compressed_size, key_block_decompressed_size))
    }
    println!("the key_block_info_list len= {}", &key_block_info_list.len());
    return key_block_info_list;
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
