use super::*;
use crate::{WxError, WxResult};

use byteorder::{ LittleEndian, ReadBytesExt};
use std::{ collections::HashMap, ffi::c_void, fs::File, io::Read};
use tracing::{error, info};
use windows::{
    Win32::{
        Foundation::{GetLastError, HANDLE},
        Storage::FileSystem::{VS_FIXEDFILEINFO, VerQueryValueA},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, MODULEENTRY32, Module32Next, PROCESSENTRY32, Process32Next, TH32CS_SNAPMODULE,
                TH32CS_SNAPPROCESS, Toolhelp32ReadProcessMemory,
            },
            LibraryLoader::{FindResourceA, LOAD_LIBRARY_AS_DATAFILE, LoadLibraryExA, LoadResource},
            Memory::{MEMORY_BASIC_INFORMATION, VirtualQueryEx},
            Threading::{OpenProcess, PROCESS_ALL_ACCESS},
        },
        UI::WindowsAndMessaging::RT_VERSION,
    },
    core::PCSTR,
};

impl WxScanner {
    fn read_version(&mut self) -> WxResult<String> {
        get_version(&self.module)
    }

    fn read_string(&mut self, offset: Option<&usize>, field: &str) -> WxResult<String> {
        let offset = match offset {
            Some(0) => {
                return Err(WxError::unsupported_offset(&self.profile.version, field));
            }
            Some(s) => *s,
            None => {
                return Err(WxError::unsupported_offset(&self.profile.version, field));
            }
        };
        let buffer = read_memory_data(self.process.th32ProcessID, self.module.modBaseAddr as usize + offset, 128)?;
        Ok(String::from_utf8_lossy(buffer.split(|n| *n == 0).next().unwrap()).to_string())
    }
    /// 读取内存
    pub fn read_memory(&self, index: usize, len: usize, real_addr: bool) -> WxResult<Vec<u8>> {
        let process = self.process;
        let vec = if real_addr {
            read_memory_data(process.th32ProcessID, index, len)?
        }
        else {
            read_memory_data(process.th32ProcessID, self.module.modBaseAddr as usize + index, len)?
        };
        Ok(vec)
    }
    /// 内存搜索
    pub fn memory_search(&self, bytes: &[u8], real: bool) -> WxResult<Vec<usize>> {
        let process = self.process;
        let module = self.module;
        let vec = read_memory_data(process.th32ProcessID, module.modBaseAddr as usize, module.modBaseSize as usize)?;
        let r = (0..vec.len() - bytes.len())
            .filter(|&i| &vec[i..i + bytes.len()] == bytes)
            .map(|i| if real { module.modBaseAddr as usize + i } else { i })
            .collect();
        Ok(r)
    }
    /// 搜索所有微信进程的内存
    pub fn search_in_all_wechat_modules(
        &self,
        data: &[u8],
        absolute_address: bool,
        show_no_found_info: bool,
        show_error_info: bool,
    ) -> WxResult<()> {
        let process = self.process;
        for module in get_modules(&process)? {
            let vec = read_memory_data(process.th32ProcessID, module.modBaseAddr as usize, module.modBaseSize as usize);
            match vec {
                Ok(vec) => {
                    let r: Vec<usize> = (0..vec.len() - data.len())
                        .filter(|&i| &vec[i..i + data.len()] == data)
                        .map(|i| if absolute_address { module.modBaseAddr as usize + i } else { i })
                        .collect();
                    if r.len() > 0 {
                        println!(
                            "module: {}",
                            String::from_utf8(
                                module.szModule.split(|n| *n == 0).next().unwrap().iter().map(|i| *i as u8).collect()
                            )?
                        );
                        println!("{:?}", r);
                    }
                    else {
                        if show_no_found_info {
                            println!(
                                "在 {} 中未找到想要搜索的数据。开始位置: {},结束位置: {}, 长度: {}, vec 长度: {}",
                                String::from_utf8(
                                    module.szModule.split(|n| *n == 0).next().unwrap().iter().map(|i| *i as u8).collect()
                                )?,
                                module.modBaseAddr as usize,
                                module.modBaseAddr as usize + module.modBaseSize as usize,
                                module.modBaseSize,
                                vec.len()
                            );
                        }
                    }
                }
                Err(err) => {
                    if show_error_info {
                        println!(
                            "获取内存失败。module: {}。err: {err:?}",
                            String::from_utf8(
                                module.szModule.split(|n| *n == 0).next().unwrap().iter().map(|i| *i as u8).collect()
                            )?
                        );
                        println!(
                            "addr start: {:?},size: {:?},end: {:?}",
                            module.modBaseAddr as usize,
                            module.modBaseSize as usize,
                            module.modBaseAddr as usize + module.modBaseSize as usize
                        );
                    }
                    continue;
                }
            }
        }
        Ok(())
    }
    /// 搜索所有微信进程的内存
    pub fn search_in_all_wechat_data(
        &self,
        data: &[u8],
        real_addr: bool,
        show_no_found_info: bool,
        show_error_info: bool,
    ) -> WxResult<()> {
        for (base_addr, size) in get_all_memory_by_handle(&self.handle)? {
            let vec = read_memory_data(self.process.th32ProcessID, base_addr, size);
            match vec {
                Ok(vec) => {
                    let r: Vec<usize> = (0..vec.len() - data.len())
                        .filter(|&i| &vec[i..i + data.len()] == data)
                        .map(|i| if real_addr { base_addr + i } else { i })
                        .collect();
                    if r.len() > 0 {
                        println!("base_addr: {}", base_addr);
                        println!("{:?}", r);
                    }
                    else {
                        if show_no_found_info {
                            println!(
                                "未找到想要搜索的数据。开始位置: {},结束位置: {}, 长度: {}, vec 长度: {}",
                                base_addr,
                                base_addr + size,
                                size,
                                vec.len()
                            );
                        }
                    }
                }
                Err(err) => {
                    if show_error_info {
                        println!("获取内存失败。base_addr: {base_addr}。 size: {size}, err: {err:?}");
                    }
                    continue;
                }
            }
        }
        Ok(())
    }
    /// 打开微信进程
    pub fn open_wechat_process(
        &mut self,
        offset_map: &Option<String>,
        process_id: &Option<u32>,
        process_name: &String,
        module_name: &String,
    ) -> WxResult<()> {
        self.process = match process_id {
            Some(id) => get_process_by_id(*id)?,
            _ => get_process_by_name(&process_name)?,
        };
        self.handle = get_process_handle(self.process.th32ProcessID)?;
        self.module = get_module_by_name(&self.process, &module_name)?;
        self.profile.version = self.read_version()?;
        let mut buf = String::new();
        match offset_map {
            Some(s) => {
                match File::open(s) {
                    Ok(mut o) => {
                        o.read_to_string(&mut buf)?;
                    }
                    Err(_) => {
                        println!("无法找到 `on_windows.json` 配置, 使用内置映射");
                        buf.push_str(include_str!("on_windows.json"))
                    }
                };
            }
            None => {
                info!("未配置 `on_windows.json` ,使用内置映射")
            }
        }

        let offset_map_map: HashMap<String, Vec<usize>> = serde_json::de::from_str(&buf)?;
        let offsets = offset_map_map
            .get(&self.profile.version)
            .ok_or(WxError::custom(format!("微信版本为: {}，未找到该版本的偏移量", self.profile.version)))?;
        match self.read_string(offsets.get(0), "nick_name") {
            Ok(s) => self.profile.nick_name = s,
            Err(e) => error!("{}", e),
        };
        match self.read_string(offsets.get(1), "account") {
            Ok(s) => self.profile.user_name = s,
            Err(e) => error!("{}", e),
        };
        match self.read_string(offsets.get(2), "phone") {
            Ok(s) => self.profile.mobile = s,
            Err(e) => error!("{}", e),
        };
        match self.read_string(offsets.get(3), "email") {
            Ok(s) => self.profile.email = s,
            Err(e) => error!("{}", e),
        };
        self.profile.aes256 = read_wechat_key(self, offsets[4])?;
        Ok(())
    }

    /// 获取微信进程信息
    pub fn open_wechat_process_with_out_info(
        &mut self,
        process_id: &Option<u32>,
        process_name: &String,
        module_name: &String,
    ) -> WxResult<()> {
        self.process = match process_id {
            Some(id) => get_process_by_id(*id)?,
            _ => get_process_by_name(&process_name)?,
        };
        self.handle = get_process_handle(self.process.th32ProcessID)?;
        self.module = get_module_by_name(&self.process, &module_name)?;
        Ok(())
    }
}

fn read_wechat_key(wechat_info: &mut WxScanner, offset: usize) -> WxResult<[u8; 32]> {
    let process = wechat_info.process;
    let buffer = read_memory_data(process.th32ProcessID, wechat_info.module.modBaseAddr as usize + offset, 8)?;
    let mut cur = std::io::Cursor::new(&buffer);
    let offset = cur.read_u64::<LittleEndian>()?;
    let key_buffer = read_memory_data(process.th32ProcessID, offset as usize, 32)?;
    Ok(*&key_buffer[..].try_into()?)
}

pub fn get_version(module: &MODULEENTRY32) -> WxResult<String> {
    unsafe {
        let image = LoadLibraryExA(
            PCSTR::from_raw(module.szExePath.as_ptr() as *const u8),
            Some(HANDLE::default()),
            LOAD_LIBRARY_AS_DATAFILE,
        )?;
        let res_info = FindResourceA(Some(image), PCSTR(1u8 as _), PCSTR(RT_VERSION.as_ptr() as _))?;
        let res_data = LoadResource(Some(image), res_info)?;
        let mut info = VS_FIXEDFILEINFO::default();
        let mut info_ref = &mut info as *mut _ as *mut c_void;
        let mut size = std::mem::size_of::<VS_FIXEDFILEINFO>() as u32;
        let b =
            VerQueryValueA(res_data.0, PCSTR::from_raw("\0".as_bytes().as_ptr()), &mut info_ref as *mut *mut c_void, &mut size);
        let info_ref = info_ref as *mut VS_FIXEDFILEINFO;
        info = *info_ref;
        if b.0 == 0 {
            GetLastError().ok()?;
        }
        return Ok(format!(
            "{}.{}.{}.{}",
            info.dwFileVersionMS >> 16,
            info.dwFileVersionMS & 0xffff,
            info.dwFileVersionLS >> 16,
            info.dwFileVersionLS & 0xffff
        ));
    }
}

pub fn get_all_memory_by_handle(handle: &HANDLE) -> WxResult<Vec<(usize, usize)>> {
    let mut lp_addr = None;
    let mut vec = vec![];

    loop {
        let mut memory_basic_info = MEMORY_BASIC_INFORMATION::default();
        unsafe {
            let r = VirtualQueryEx(*handle, lp_addr, &mut memory_basic_info, std::mem::size_of::<MEMORY_BASIC_INFORMATION>());
            if r == 0 {
                break;
            }
        }
        let base_addr = memory_basic_info.BaseAddress as usize;
        let size = memory_basic_info.RegionSize as usize;
        lp_addr = Some((base_addr + size) as *const c_void);
        vec.push((base_addr, size))
    }

    Ok(vec)
}

/// 读取实际内存中的数据
pub fn read_memory_data(process_id: u32, offset: usize, mut length: usize) -> WxResult<Vec<u8>> {
    let mut vec = Vec::with_capacity(length);
    unsafe {
        for i in 0..=length / 4096 {
            let mut buffer = [0u8; 4096];
            let mut numberofbytesread = 0;
            if length > 4096 {
                let b = Toolhelp32ReadProcessMemory(
                    process_id,
                    (offset + i * 4096) as *const c_void,
                    buffer.as_mut_ptr() as *mut c_void,
                    4096,
                    &mut numberofbytesread,
                );
                if b.0 == 0 {
                    GetLastError().ok()?;
                }
                vec.append(&mut buffer.to_vec());
                length = length - 4096;
            }
            else {
                let b = Toolhelp32ReadProcessMemory(
                    process_id,
                    (offset + i * 4096) as *const c_void,
                    buffer.as_mut_ptr() as *mut c_void,
                    length,
                    &mut numberofbytesread,
                );
                if b.0 == 0 {
                    GetLastError().ok()?;
                }
                vec.append(&mut buffer[0..length].to_vec());
                break;
            }
        }
    }
    Ok(vec)
}

/// 获取进程勾柄
pub fn get_process_handle(process_id: u32) -> WxResult<HANDLE> {
    Ok(unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process_id) }?)
}

pub fn get_process_by_name(process_name: &str) -> WxResult<PROCESSENTRY32> {
    unsafe {
        let mut process = PROCESSENTRY32::default();
        process.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
        let process_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        loop {
            Process32Next(process_snapshot, &mut process)?;
            if process.szExeFile.split(|n| *n == 0).next().unwrap().iter().map(|i| *i as u8).collect::<Vec<_>>()
                == process_name.as_bytes()
            {
                return Ok(process);
            }
        }
    }
}

pub fn get_process_by_id(process_id: u32) -> WxResult<PROCESSENTRY32> {
    unsafe {
        let mut process = PROCESSENTRY32::default();
        process.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
        let process_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        loop {
            Process32Next(process_snapshot, &mut process)?;
            if process.th32ProcessID == process_id {
                return Ok(process);
            }
        }
    }
}

pub fn get_module_by_name(process: &PROCESSENTRY32, module_name: &str) -> WxResult<MODULEENTRY32> {
    let mut module = MODULEENTRY32::default();
    module.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, process.th32ProcessID)?;
        loop {
            Module32Next(snapshot, &mut module)?;
            if module.szModule.split(|n| *n == 0).next().unwrap().iter().map(|i| *i as u8).collect::<Vec<_>>()
                == module_name.as_bytes()
            {
                return Ok(module);
            }
        }
    }
}

pub fn get_modules(process: &PROCESSENTRY32) -> WxResult<Vec<MODULEENTRY32>> {
    let mut vec = vec![];
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, process.th32ProcessID)?;
        loop {
            let mut module = MODULEENTRY32::default();
            module.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;
            let r = Module32Next(snapshot, &mut module);
            if r.is_ok() {
                vec.push(module);
            }
            else {
                break;
            }
        }
    }
    Ok(vec)
}
