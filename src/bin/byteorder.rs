use encoding_rs::*;
use regex::Regex;
use std::collections::HashMap;

fn main() {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
    let mut big_endian: &[u8] = b"\x00\x00\x06V";
    let num = big_endian.read_u32::<BigEndian>().unwrap();
    println!("{}", num);

    let mut litter_endian: &[u8] = b"\x01*\xd2\x8b";
    let num1 = litter_endian.read_u32::<LittleEndian>().unwrap();
    println!("{}", num1);//2345806337


    let s = r##"<Dictionary GeneratedByEngineVersion="2.0" RequiredEngineVersion="2.0" Format="Html" KeyCaseSensitive="No" StripKey="Yes" Encrypted="2"
    RegisterBy="EMail"
    Description="&lt;table width=&quot;100%&quot; border=&quot;0&quot; bgcolor=&quot;#fef4f4&quot;&gt;&lt;tr&gt;&lt;td align=&quot;center&quot;&gt;&lt;h1 style=&quot;color:deeppink; font-family: Microsoft Yahei, sans-serif; margin-top: 5pt&quot;&gt;朗文当代高级英语辞典（第四版）&lt;/h1&gt;&lt;/td&gt;&lt;/tr&gt;&lt;tr&gt;&lt;td align=&quot;center&quot;&gt;&lt;h3 style=&quot;color:#e95295; font-family: Microsoft Yahei, sans-serif&quot;&gt;以Oeasy提供的APK为底稿，水湲2014年6月25日制作，2015年7月19日更新&lt;/h3&gt;&lt;/td&gt;&lt;/tr&gt;&lt;/table&gt;"
    Title="朗文当代双解"
    Encoding="UTF-8"
    CreationDate="2015-7-18"
    Compact="Yes"
    Compat="Yes"
    Left2Right="Yes"
    DataSourceFormat="106"
    StyleSheet=""/>"##;
    let mut _header_map = HashMap::new();
    let re = Regex::new(r#"(\w+)=["](.*?)["]"#).unwrap();
    let caps = re.captures_iter(s);
    let m1 = re.find_iter(s);
    for cap in caps {
        _header_map.insert(cap.get(1).unwrap().as_str(), cap.get(2).unwrap().as_str());
        // println!("{}", cap.get(1).unwrap().as_str());
        // println!("{}", cap.get(2).unwrap().as_str());
    }
    for kv in _header_map{
        println!("{}={}", kv.0, kv.1);
    }
}
