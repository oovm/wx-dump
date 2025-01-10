use std::{
    fs,
    io::{Read, Write},
};
use walkdir::WalkDir;

const CODER: u8 = 0x8;

use crate::WxResult;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    path::Path,
};

#[derive(Default, Debug)]
pub struct FileHeaderMarks {
    magic_heads: BTreeMap<String, Vec<u8>>,
}

impl FileHeaderMarks {
    pub fn new() -> Self {
        let mut fileheaders = BTreeMap::new();
        fileheaders.insert("jpg", vec![0xFF, 0xD8, 0xFF]);
        fileheaders.insert("png", vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
        fileheaders.insert("gif", vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61]);
        fileheaders.insert("bmp", vec![0x42, 0x4D]);
        fileheaders.insert("pcx", vec![0x0A, 0x05, 0x01, 0x08]);
        fileheaders.insert("tif", vec![0x49, 0x49, 0x2A, 0x00]);
        fileheaders.insert("tiff", vec![0x4D, 0x4D, 0x00, 0x2A]);
        fileheaders.insert("iff", vec![0x46, 0x4F, 0x52, 0x4D]);
        fileheaders.insert("ani", vec![0x52, 0x49, 0x46, 0x46]);
        fileheaders.insert("tga", vec![0x00, 0x00, 0x02, 0x00, 0x00]);
        fileheaders.insert("ico", vec![0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x20, 0x20]);
        fileheaders.insert("cur", vec![0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x20, 0x20]);
        Self { magic_heads: fileheaders }
    }

    pub unsafe fn decrypt(&self, path: &Path) -> WxResult<()> {
        let mut xor_secret = 0;
        let mut ext = String::new();
        let data = std::fs::read(path).unwrap();
        if data.len() < 2 {
            // skip
            return Ok(());
        }
        for (key, magic) in self.magic_heads.iter() {
            let p0 = magic.get(0).unwrap_or(&0) ^ data.get_unchecked(0);
            let p4 = magic.get(1).unwrap_or(&0) ^ data.get_unchecked(1);
            if p0 == p4 {
                ext = key.to_string();
                xor_secret = p0;
            }
        }
        let bytes: Vec<_> = data.iter().map(|v| v ^ xor_secret).collect();
        let output = path.with_extension(ext);
        std::fs::write(output, bytes).expect("Unable to write file");
        Ok(())
    }
}

fn find_files(dir_path: &str) {
    let decode = FileHeaderMarks::new();
    for entry in WalkDir::new(dir_path) {
        let entry = entry.unwrap();
        let file_path = entry.path();
        if file_path.extension().eq(&Some(OsStr::new("dat"))) {
            unsafe {
                decode.decrypt(file_path);
            }
        }
    }
}

