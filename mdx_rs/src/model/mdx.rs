// mod header;
// struct Mdx {
//     filename: String,
//     header_info: HeaderTag,
//
//     empty_str: String,
//     charset: String,
//     encoding: String,
//     delimiter_width: i8,
//     passcode: String,
//     version: f32,
//     number_width: i32,
//     num_entries: u128,
//     num_key_blocks: u128,
//     num_record_blocks: u128,
//
//     // #[]
//     // accumulation_block_id_tree: RBTree<i32, String>,
//     key_block_size: u128,
//     key_block_info_size: u128,
//     key_block_info_decom_size: u128,
//     record_block_size: u128,
//     record_block_offset: u128,
//     record_block_start: u128,
//     key_block_offset: i32,
//     key_block_info_list: Vec<KeyInfo>,
//     record_info_struct_list: Vec<u8>,
//
//     max_com_rec_size: i32,
//     max_decompressed_size: u128,
//     rec_decompressed_size: i32,
//
//     max_decom_key_block_size: u128,
//     max_com_key_block_size: u128,
// }