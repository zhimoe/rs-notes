use encoding_rs::*;

fn main() {
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
    let mut big_endian: &[u8] = b"\x00\x00\x06V";
    let num = big_endian.read_u32::<BigEndian>().unwrap();
    println!("{}", num);

    let mut litter_endian: &[u8] = b"\x01*\xd2\x8b";
    let num1 = litter_endian.read_u32::<LittleEndian>().unwrap();
    println!("{}", num1);//2345806337


    let header: &[u8] = b"<\x00D\x00i\x00c\x00t\x00i\x00o\x00n\x00a\x00r\x00y\x00 \x00G\x00e\x00n\x00e\x00r\x00a\x00t\x00e\x00d\x00B\x00y\x00E\x00n\x00g\x00i";
    let mut decoder = UTF_16LE.new_decoder();
    let &mut s;
    decoder.decode_to_str(&header, &mut s, false);
}
