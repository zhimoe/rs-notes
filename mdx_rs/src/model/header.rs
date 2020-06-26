#[derive(Debug)]
pub struct HeaderTag {
    pub file: String,
    pub genversion: f32,
    pub format: String,
    pub keycasesensitive: bool,
    pub stripkey: bool,
    /**
     * encryption flag
     * 0x00 - no encryption
     * 0x01 - encrypt record block
     * 0x02 - encrypt key info block
     */
    pub encrypted: String,
    pub registerby: String,
    pub encoding: String,
    pub creationdate: String,
    pub compact: bool,
    pub left2right: bool,
    pub datasourceformat: String,
    pub stylesheet: String,
    pub _key_block_offset: u64,
    pub _record_block_offset: u64,
}

#[derive(Debug, Default)]
pub struct HeaderTagBuilder {
    pub file: String,
    pub genversion: f32,
    pub format: String,
    pub keycasesensitive: bool,
    pub stripkey: bool,
    pub encrypted: String,
    pub registerby: String,
    pub encoding: String,
    pub compact: bool,
    pub left2right: bool,
    pub datasourceformat: String,
    pub stylesheet: String,
    pub _key_block_offset: u64,
    pub _record_block_offset: u64,
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
    pub fn format(&mut self, format: String) -> &mut Self {
        self.format = format;
        self
    }
    pub fn keycasesensitive(&mut self, keycasesensitive: bool) -> &mut Self {
        self.keycasesensitive = keycasesensitive;
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
    pub fn left2right(&mut self, left2right: bool) -> &mut Self {
        self.left2right = left2right;
        self
    }
    pub fn datasourceformat(&mut self, datasourceformat: String) -> &mut Self {
        self.datasourceformat = datasourceformat;
        self
    }
    pub fn stylesheet(&mut self, stylesheet: String) -> &mut Self {
        self.stylesheet = stylesheet;
        self
    }
    pub fn key_block_offset(&mut self, _key_block_offset: u64) -> &mut Self {
        self._key_block_offset = _key_block_offset;
        self
    }
    pub fn record_block_offset(&mut self, _record_block_offset: u64) -> &mut Self {
        self._record_block_offset = _record_block_offset;
        self
    }
    pub fn build(&self) -> HeaderTag {
        HeaderTag {
            file: self.file.to_owned(),
            genversion: self.genversion,
            format: self.format.to_owned(),
            keycasesensitive: self.keycasesensitive,
            stripkey: self.stripkey,
            encrypted: self.encrypted.to_owned(),
            registerby: self.registerby.to_owned(),
            encoding: self.encoding.to_owned(),
            creationdate: "".to_string(),
            compact: self.compact,
            left2right: self.left2right,
            datasourceformat: self.datasourceformat.to_owned(),
            stylesheet: self.stylesheet.to_owned(),
            _key_block_offset: self._key_block_offset,
            _record_block_offset: self._record_block_offset,
        }
    }
}