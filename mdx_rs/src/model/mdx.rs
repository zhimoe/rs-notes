#[derive(Debug)]
pub struct HeaderTag {
    pub file: String,
    pub genversion: f32,
    pub stripkey: bool,
    /**
     * encryption flag
     * 0x00 - no encryption
     * 0x01 - encrypt record block
     * 0x02 - encrypt key info block,default
     */
    pub encrypted: String,
    //EMail
    pub registerby: String,
    pub encoding: String,
    pub compact: bool,
    pub stylesheet: String,
    pub key_block_offset: u64,
    pub record_block_offset: u64,
}

#[derive(Debug, Default)]
pub struct HeaderTagBuilder {
    pub file: String,
    pub genversion: f32,
    pub stripkey: bool,
    pub encrypted: String,
    pub registerby: String,
    pub encoding: String,
    pub compact: bool,
    pub stylesheet: String,
    pub key_block_offset: u64,
    pub record_block_offset: u64,
}

impl HeaderTagBuilder {
    pub fn file(&mut self, file: String) -> &mut Self {
        self.file = file;
        self
    }
    pub fn genversion(&mut self, genversion: f32) -> &mut Self {
        self.genversion = genversion;
        self
    }

    pub fn stripkey(&mut self, stripkey: bool) -> &mut Self {
        self.stripkey = stripkey;
        self
    }
    pub fn encrypted(&mut self, encrypted: String) -> &mut Self {
        self.encrypted = encrypted;
        self
    }
    pub fn registerby(&mut self, registerby: String) -> &mut Self {
        self.registerby = registerby;
        self
    }
    pub fn encoding(&mut self, encoding: String) -> &mut Self {
        self.encoding = encoding;
        self
    }
    pub fn compact(&mut self, compact: bool) -> &mut Self {
        self.compact = compact;
        self
    }

    pub fn stylesheet(&mut self, stylesheet: String) -> &mut Self {
        self.stylesheet = stylesheet;
        self
    }
    pub fn key_block_offset(&mut self, _key_block_offset: u64) -> &mut Self {
        self.key_block_offset = _key_block_offset;
        self
    }
    pub fn record_block_offset(&mut self, _record_block_offset: u64) -> &mut Self {
        self.record_block_offset = _record_block_offset;
        self
    }
    pub fn build(&self) -> HeaderTag {
        HeaderTag {
            file: self.file.to_owned(),
            genversion: self.genversion,
            stripkey: self.stripkey,
            encrypted: self.encrypted.to_owned(),
            registerby: self.registerby.to_owned(),
            encoding: self.encoding.to_owned(),
            compact: self.compact,
            stylesheet: self.stylesheet.to_owned(),
            key_block_offset: self.key_block_offset,
            record_block_offset: self.record_block_offset,
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
    header_info: HeaderTag,

    empty_str: String,
    charset: String,
    encoding: String,
    delimiter_width: i8,
    passcode: String,
    version: f32,
    number_width: i32,
    num_entries: u128,
    num_key_blocks: u128,
    num_record_blocks: u128,

    // #[]
    // accumulation_block_id_tree: RBTree<i32, String>,
    key_block_size: u128,
    key_block_info_size: u128,
    key_block_info_decom_size: u128,
    record_block_size: u128,
    record_block_offset: u128,
    record_block_start: u128,
    key_block_offset: i32,
    key_block_info_list: Vec<KeyInfo>,
    record_info_struct_list: Vec<u8>,

    max_com_rec_size: i32,
    max_decompressed_size: u128,
    rec_decompressed_size: i32,

    max_decom_key_block_size: u128,
    max_com_key_block_size: u128,
}