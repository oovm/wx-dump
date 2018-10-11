use crate::{WxResult, errors::WxError};
use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::NoPadding};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::{
    ffi::OsStr,
    fs::{File,  create_dir_all},
    io::{Read, Write},
    path::{ PathBuf},
};
use tracing::{debug, trace};
use url::Url;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct WxDecryptor {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub key: [u8; 32],
    pub need_check_hmac: bool,
}

impl WxDecryptor {
    pub async fn decrypt(&self) -> WxResult<()> {
        if self.output_path.exists() {
            if !self.output_path.is_dir() {
                return Err(WxError::custom(format!(
                    "指定的保存目录已经存在并且不是一个文件夹，请检查。{}",
                    self.source_path.display()
                )));
            }
        }
        else {
            // create_dir_all(&self.output_path)?;
            create_dir_all(&self.output_path.join("Multi"))?;
        }
        match Url::from_file_path(&self.source_path) {
            Ok(o) => {
                println!("原始路径: {}", o);
            }
            Err(_) => {}
        }
        match Url::from_file_path(&self.output_path) {
            Ok(o) => {
                println!("解密路径: {}", o);
            }
            Err(_) => {}
        }
        for entry in WalkDir::new(&self.source_path) {
            match entry {
                Ok(o) => {
                    if o.path().extension().eq(&Some(OsStr::new("db"))) {
                        let relative = o.path().strip_prefix(&self.source_path)?;
                        self.decrypt_file(relative.to_str().unwrap()).await?;
                    }
                }
                Err(_) => {}
            }
        }
        Ok(())
    }
    async fn decrypt_file(&self, source_file: &str) -> WxResult<()> {
        let file_db = self.source_path.join(source_file);
        let file_wal = file_db.with_extension("db-wal");
        let file_shm = file_db.with_extension("db-shm");
        let file_out = self.output_path.join(source_file);
        debug!("正在解密: {}", source_file);
        let mut saved_file = File::open(&file_db)?;
        let mut buffer = vec![];
        saved_file.read_to_end(&mut buffer)?;
        let salt = &buffer[0..16];
        let mut byte_key = [0; 32];
        pbkdf2::pbkdf2::<Hmac<Sha1>>(&self.key, salt, 64000, &mut byte_key)?;
        let first = &buffer[16..4096];
        let mac_salt = salt.iter().map(|i| i ^ 58).collect::<Vec<_>>();
        if check_hmac(first, &byte_key, &mac_salt, 1, 32)? {
            trace!("密码正确: {}", source_file);
            let pages: Vec<&[u8]> = buffer[..].chunks(4096).collect();

            let mut out_file = File::create(&file_out).unwrap();
            for (index, page) in pages.iter().enumerate() {
                let mut decrypted_page = vec![];
                if self.need_check_hmac {
                    decrypt_data_hmac(index as u32 + 1, page, &byte_key, &mut decrypted_page, 48, 48, &mac_salt, 32)?;
                }
                else {
                    decrypt_data_hmac_unchecked(index as u32 + 1, page, &byte_key, &mut decrypted_page, 48, 48)?;
                }
                out_file.write_all(&decrypted_page)?;
            }
            out_file.flush()?;
            trace!("解密成功: {}", source_file);

            if file_wal.exists() {
                let mut save_wal_file = File::open(&file_wal)?;
                let mut save_wal_buffer = vec![];
                save_wal_file.read_to_end(&mut save_wal_buffer)?;
                if save_wal_buffer.len() != 0 {
                    let order_byte = save_wal_buffer[3];
                    let (mut dis_decrypt_sum1, mut dis_decrypt_sum2) =
                        get_check_sum(0, 0, &save_wal_buffer[..24], &order_byte)?;
                    let (mut decrypted_sum1, mut decrypted_sum2) = get_check_sum(0, 0, &save_wal_buffer[..24], &order_byte)?;
                    let mut decrypted_wal_file = File::create(&file_out.with_extension("db-wal"))?;
                    decrypted_wal_file.write_all(&save_wal_buffer[0..32])?;
                    let wal_frames: Vec<&[u8]> = save_wal_buffer[32..].chunks(24 + 4096).collect();
                    for wal_frame in wal_frames {
                        let mut decrypt_buf = vec![];
                        let mut cur = std::io::Cursor::new(&wal_frame[0..24]);
                        let page_index = cur.read_u32::<BigEndian>()?;
                        cur.set_position(16);
                        let wal_frame_sum1 = cur.read_u32::<BigEndian>()?;
                        let wal_frame_sum2 = cur.read_u32::<BigEndian>()?;
                        (dis_decrypt_sum1, dis_decrypt_sum2) =
                            get_check_sum(dis_decrypt_sum1, dis_decrypt_sum2, &wal_frame[..8], &order_byte)?;
                        (dis_decrypt_sum1, dis_decrypt_sum2) =
                            get_check_sum(dis_decrypt_sum1, dis_decrypt_sum2, &wal_frame[24..], &order_byte)?;
                        if self.need_check_hmac {
                            decrypt_data_hmac(
                                page_index,
                                &wal_frame[24..],
                                &byte_key,
                                &mut decrypt_buf,
                                48,
                                48,
                                &mac_salt,
                                32,
                            )?;
                        }
                        else {
                            decrypt_data_hmac_unchecked(page_index, &wal_frame[24..], &byte_key, &mut decrypt_buf, 48, 48)?;
                        }
                        (decrypted_sum1, decrypted_sum2) =
                            get_check_sum(decrypted_sum1, decrypted_sum2, &wal_frame[..8], &order_byte)?;
                        (decrypted_sum1, decrypted_sum2) =
                            get_check_sum(decrypted_sum1, decrypted_sum2, &decrypt_buf, &order_byte)?;

                        if wal_frame_sum1 == dis_decrypt_sum1 && wal_frame_sum2 == dis_decrypt_sum2 {
                            decrypted_wal_file.write_all(&wal_frame[0..16])?;
                            decrypted_wal_file.write_all(&decrypted_sum1.to_be_bytes())?;
                            decrypted_wal_file.write_all(&decrypted_sum2.to_be_bytes())?;
                            decrypted_wal_file.write_all(&decrypt_buf)?;
                        }
                        else {
                            decrypted_wal_file.write_all(&wal_frame[0..24])?;
                            decrypted_wal_file.write_all(&decrypt_buf)?;
                        }
                    }
                    trace!("解密成功: {}-wal", source_file);
                }

                if file_shm.exists() {
                    tokio::fs::copy(&file_shm, &file_out.with_extension("db-shm")).await?;
                    trace!("解密成功: {}-shm", source_file);
                }
                Ok(())
            }
            else {
                Ok(())
            }
        }
        else {
            Err(WxError::invalid_key(self.key, &self.source_path))?
        }
    }
}

fn get_check_sum(mut s1: u32, mut s2: u32, list: &[u8], order_byte: &u8) -> WxResult<(u32, u32)> {
    let get_i32 = if *order_byte == 0x82 {
        |cursor: &mut std::io::Cursor<&[u8]>| cursor.read_u32::<LittleEndian>().unwrap()
    }
    else if *order_byte == 0x83 {
        |cursor: &mut std::io::Cursor<&[u8]>| cursor.read_u32::<BigEndian>().unwrap()
    }
    else {
        return Err(WxError::custom("bad order_byte"));
    };
    let mut list = list.chunks(4).map(|data| {
        let mut cursor = std::io::Cursor::new(&data[..]);
        get_i32(&mut cursor)
    });
    loop {
        if let Some(first) = list.next() {
            s1 = u32::wrapping_add(u32::wrapping_add(s1, first), s2);
            s2 = u32::wrapping_add(u32::wrapping_add(s2, list.next().unwrap()), s1);
        }
        else {
            break;
        }
    }
    Ok((s1, s2))
}

fn decrypt_data<F>(
    index: u32,
    data: &[u8],
    key: &[u8],
    decrypted_data: &mut Vec<u8>,
    iv_offset: usize,
    dis_decrypted_offset: usize,
    check_fn: F,
) -> WxResult<()>
where
    F: Fn(&[u8], &[u8], u32) -> WxResult<bool>,
{
    let page = if index == 1 {
        decrypted_data.append(&mut "SQLite format 3\x00".as_bytes().to_vec());
        &data[16..]
    }
    else {
        &data[..]
    };
    if !check_fn(page, key, index)? {
        return Err(WxError::custom(format!("数据校验未通过, index: {}", index)));
    }
    let iv = &page[page.len() - iv_offset..page.len() - iv_offset + 16];
    let mut decrypt_buf = vec![0u8; page.len() - dis_decrypted_offset];
    let decryptor = cbc::Decryptor::<aes::Aes256>::new_from_slices(&key, iv)?;
    decryptor.decrypt_padded_b2b_mut::<NoPadding>(&page[..page.len() - dis_decrypted_offset], &mut decrypt_buf)?;
    decrypted_data.append(&mut decrypt_buf);
    decrypted_data.append(&mut data[data.len() - dis_decrypted_offset..].to_vec());
    Ok(())
}

fn decrypt_data_hmac(
    index: u32,
    data: &[u8],
    key: &[u8],
    decrypted_data: &mut Vec<u8>,
    iv_offset: usize,
    dis_decrypted_offset: usize,
    salt: &[u8],
    hmac_offset: usize,
) -> WxResult<()> {
    decrypt_data(index, data, key, decrypted_data, iv_offset, dis_decrypted_offset, |data, key, index| {
        check_hmac(data, key, salt, index, hmac_offset)
    })
}

fn check_hmac(data: &[u8], key: &[u8], salt: &[u8], index: u32, offset_hmac: usize) -> WxResult<bool> {
    let mut mac_key = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha1>>(&key, &salt, 2, &mut mac_key)?;
    let mut hash_mac = Hmac::<Sha1>::new_from_slice(&mac_key)?;
    hash_mac.update(&data[..data.len() - offset_hmac]);
    hash_mac.update(&index.to_le_bytes());
    let r = hash_mac.finalize().into_bytes();
    Ok(r[..] == data[data.len() - offset_hmac..data.len() - offset_hmac + 20])
}

fn decrypt_data_hmac_unchecked(
    index: u32,
    data: &[u8],
    key: &[u8],
    decrypted_data: &mut Vec<u8>,
    offset_iv: usize,
    dis_decrypted_offset: usize,
) -> WxResult<()> {
    decrypt_data(index, data, key, decrypted_data, offset_iv, dis_decrypted_offset, |_, _, _| Ok(true))
}
