use encoding_rs::*;
use rbtree::RBTree;
use std::io::{BufReader, Read};
use std::fs::File;
use adler32::RollingAdler32;

use regex::Regex;
use byteorder::{LittleEndian, ReadBytesExt, BigEndian};
use std::char::decode_utf16;
use std::collections::HashMap;


struct HeaderInfo {
    generatedbyengineversion: f32,
    requiredengineversion: f32,
    format: String,
    keycasesensitive: bool,
    stripkey: bool,
    /**
     * encryption flag
     * 0x00 - no encryption
     * 0x01 - encrypt record block
     * 0x02 - encrypt key info block
     */
    encrypted: bool,
    registerby: String,
    description: String,
    title: String,
    encoding: String,
    creationdate: String,
    compact: bool,
    left2right: bool,
    datasourceformat: String,
    stylesheet: String,
}

impl Default for HeaderInfo {
    fn default() -> Self {
        HeaderInfo {
            generatedbyengineversion: 0.0,
            requiredengineversion: 0.0,
            format: "".to_string(),
            keycasesensitive: false,
            stripkey: false,
            encrypted: false,
            registerby: "".to_string(),
            description: "".to_string(),
            title: "".to_string(),
            encoding: "".to_string(),
            creationdate: "".to_string(),
            compact: false,
            left2right: false,
            datasourceformat: "".to_string(),
            stylesheet: "".to_string(),
        }
    }
}

struct KeyInfo {
    tail_key_text: Vec<u8>,
    header_key_text: Vec<u8>,

    key_block_compressed_size_accumulator: u128,
    key_block_compressed_size: u128,
    key_block_decompressed_size: u128,
    num_entries: u128,
    num_entries_accumulator: u128,

}

struct Mdx {
    filename: String,
    header_info: HeaderInfo,

    empty_str: String,
    _charset: String,
    _encoding: String,
    delimiter_width: i8,
    _passcode: String,
    _version: f32,
    _number_width: i32,
    _num_entries: u128,
    _num_key_blocks: u128,
    _num_record_blocks: u128,

    accumulation_block_id_tree: RBTree<i32, String>,
    _key_block_size: u128,
    _key_block_info_size: u128,
    _key_block_info_decom_size: u128,
    _record_block_size: u128,
    _record_block_offset: u128,
    _record_block_start: u128,
    _key_block_offset: i32,
    _key_block_info_list: Vec<KeyInfo>,
    _record_info_struct_list: Vec<u8>,

    max_com_rec_size: i32,
    max_decompressed_size: u128,
    rec_decompressed_size: i32,

    max_decom_key_block_size: u128,
    max_com_key_block_size: u128,

}

impl Default for Mdx {
    fn default() -> Mdx {
        Mdx {
            filename: "".to_string(),
            header_info: HeaderInfo {
                generatedbyengineversion: 0.0,
                requiredengineversion: 0.0,
                format: "".to_string(),
                keycasesensitive: false,
                stripkey: false,
                encrypted: false,
                registerby: "".to_string(),
                description: "".to_string(),
                title: "".to_string(),
                encoding: "".to_string(),
                creationdate: "".to_string(),
                compact: false,
                left2right: false,
                datasourceformat: "".to_string(),
                stylesheet: "".to_string(),
            },
            empty_str: "".to_string(),
            _charset: "".to_string(),
            _encoding: "".to_string(),
            delimiter_width: 0,
            _passcode: "".to_string(),
            _version: 0.0,
            _number_width: 0,
            _num_entries: 0,
            _num_key_blocks: 0,
            _num_record_blocks: 0,
            accumulation_block_id_tree: RBTree::new(),
            _key_block_size: 0,
            _key_block_info_size: 0,
            _key_block_info_decom_size: 0,
            _record_block_size: 0,
            _record_block_offset: 0,
            _record_block_start: 0,
            _key_block_offset: 0,
            _key_block_info_list: vec![],
            _record_info_struct_list: vec![],
            max_com_rec_size: 0,
            max_decompressed_size: 0,
            rec_decompressed_size: 0,
            max_decom_key_block_size: 0,
            max_com_key_block_size: 0,
        }
    }
}

// big endian bytes unpack,等价下面的代码
// from struct import unpack
// unpack('>I', bytes_arr)
//  b"\x00\x00\x06V" => 1622
fn be_bytes_unpack(byte: &[u8]) -> u32 {
    let mut buf_out: &[u8] = byte.clone();
    let num = buf_out.read_u32::<BigEndian>().unwrap();
    num
}

// unpack('<I', bytes_arr)
// b"\x01*\xd2\x8b" => 2345806337
fn le_bytes_unpack(byte: &[u8]) -> u32 {
    let mut buf_out: &[u8] = byte.clone();
    let num = buf_out.read_u32::<LittleEndian>().unwrap();
    num
}

// // python bytes.decode("utf-16").encode("utf-8")
// fn u8_to_utf16_string(bytes: &[u8]) -> Option<String> {
//     let (front, slice, back) = unsafe {
//         bytes.align_to::<u16>()
//     };
//     if front.is_empty() && back.is_empty() {
//         String::from_utf16(slice).ok()
//     } else {
//         None
//     }
// }
// 等价上面的u8_to_utf16_string方法
pub fn le_u8_string(slice: &[u8]) -> Option<String> {
    let idx = slice.len() / 2;
    let iter = (0..idx)
        .map(|i| u16::from_le_bytes([slice[2 * i], slice[2 * i + 1]]));

    std::char::decode_utf16(iter).collect::<Result<String, _>>().ok()
}

impl Mdx {
    fn new(f: String) -> Self {
        let mut reader = BufReader::new(File::open(&f).unwrap());
        let mut bytes1 = [0; 4];
        // read exactly 4 bytes
        reader.read_exact(&mut bytes1);
        let header_len = be_bytes_unpack(&bytes1);
        let _key_block_offset: i32 = (4 + header_len + 4) as i32;
        let mut header_bytes = vec![0; header_len as usize];

        // read the header info
        reader.read_exact(&mut header_bytes);

        // reade 4 bytes: adler32 checksum of header, in little endian
        let mut adler32_bytes = [0; 4];
        reader.read_exact(&mut adler32_bytes);
        let adler32 = le_bytes_unpack(&adler32_bytes);


        let mut rolling_adler32 = RollingAdler32::new();
        rolling_adler32.update_buffer(&header_bytes);
        let header_hash = rolling_adler32.hash();
        println!("sum & 0xffffffff={},adler32={}", header_hash & 0xffffffff, adler32);
        if header_hash & 0xffffffff != adler32 as u32 {
            panic!("unrecognized format");
        }


        // header text in utf-16 encoding ending with '\x00\x00'
        let (header, end) = header_bytes.split_at(header_bytes.len() - 2);
        let header_txt = le_u8_string(&header).unwrap();

        println!("{}", header_txt);

        let mut _header_map = HashMap::new();
        let re = Regex::new(r#"(\w+)=["](.*?)["]"#).unwrap();
        let caps = re.captures_iter(header_txt.as_str());
        for cap in caps {
            _header_map.insert(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str());
        }
        // for kv in _header_map {
        //     println!("{}={}", kv.0, kv.1);
        // }

        let _version = _header_map.get("GeneratedByEngineVersion").unwrap().parse::<f32>().unwrap();
        let headerinfo = HeaderInfo::default();
        // for (k, v) in &_header_map.iter() {
        //     match *k {
        //         "GeneratedByEngineVersion" => headerinfo.generatedbyengineversion = *_version,
        //         _ => {}
        //     }
        // }


        //
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


        Mdx { filename: f, header_info: headerinfo, ..Default::default() }
    }
}

fn main() {
    let mdx = Mdx::new(String::from("/home/cod3fn/code/rs-notes/resources/LSC4.mdx"));
    println!("mdx version {}", mdx._version);
    println!("mdx filename {}", mdx.filename);
}