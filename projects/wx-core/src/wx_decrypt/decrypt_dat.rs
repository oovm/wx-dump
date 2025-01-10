use crate::WxResult;
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::create_dir_all,
    io::{Read, Write},
    path::Path,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct XorDecryptor {
    magic_heads: BTreeMap<Vec<u8>, String>,
}

impl Default for XorDecryptor {
    fn default() -> Self {
        let mut h = Self { magic_heads: BTreeMap::new() };
        h.add(&[0xFF, 0xD8, 0xFF], "jpg");
        h.add(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], "png");
        h.add(&[0x47, 0x49, 0x46, 0x38, 0x39, 0x61], "gif");
        h.add(&[0x42, 0x4D], "bmp");
        h.add(&[0x0A, 0x05, 0x01, 0x08], "pcx");
        h.add(&[0x49, 0x49, 0x2A, 0x00], "tif");
        h.add(&[0x4D, 0x4D, 0x00, 0x2A], "tiff");
        h.add(&[0x46, 0x4F, 0x52, 0x4D], "iff");
        h.add(&[0x52, 0x49, 0x46, 0x46], "ani");
        h.add(&[0x00, 0x00, 0x02, 0x00, 0x00], "tga");
        h.add(&[0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x20, 0x20], "ico");
        h.add(&[0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x20, 0x20], "cur");
        h
    }
}

impl XorDecryptor {
    pub fn new(head: &[u8], extension: &str) -> Self {
        let mut h = BTreeMap::new();
        h.insert(head.to_vec(), extension.to_string());
        Self { magic_heads: h }
    }
    pub fn add(&mut self, head: &[u8], extension: &str) {
        self.magic_heads.insert(head.to_vec(), extension.to_string());
    }
    pub fn decrypt_path(&self, in_path: &Path, out_path: &Path) -> WxResult<()> {
        for entry in WalkDir::new(in_path) {
            let path = match entry {
                Ok(o) => {
                    let path = o.path();
                    if path.extension().eq(&Some(OsStr::new("dat"))) {
                        let out = path.strip_prefix(in_path)?;
                        self.decrypt_file(path, &out_path.join(out))?;
                    }
                }
                Err(_) => continue,
            };
        }
        Ok(())
    }
    pub fn decrypt_file(&self, in_file: &Path, out_file: &Path) -> WxResult<()> {
        let input = std::fs::read(in_file)?;
        let (e, output) = unsafe { self.decrypt_bytes(&input) };
        if output.is_empty() {
            return Ok(());
        }
        match out_file.parent() {
            Some(s) => create_dir_all(s)?,
            None => {}
        }
        let out_path = out_file.with_extension(e);
        Ok(std::fs::write(out_path, output)?)
    }

    pub unsafe fn decrypt_bytes(&self, input: &[u8]) -> (&str, Vec<u8>) {
        if input.len() < 2 {
            return ("", vec![]);
        }
        let mut xor_secret = 0;
        let mut ext = "";
        for (magic, key) in self.magic_heads.iter() {
            let p0 = magic.get_unchecked(0) ^ input.get_unchecked(0);
            let p4 = magic.get_unchecked(1) ^ input.get_unchecked(1);
            if p0 == p4 {
                ext = key;
                xor_secret = p0;
            }
        }
        if ext.is_empty() {
            return ("", vec![]);
        }
        (ext, input.iter().map(|x| x ^ xor_secret).collect())
    }
}
