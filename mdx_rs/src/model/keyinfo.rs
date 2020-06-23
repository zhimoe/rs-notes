struct KeyInfo {
    tail_key_text: Vec<u8>,
    header_key_text: Vec<u8>,
    key_block_compressed_size_accumulator: u128,
    key_block_compressed_size: u128,
    key_block_decompressed_size: u128,
    num_entries: u128,
    num_entries_accumulator: u128,
}