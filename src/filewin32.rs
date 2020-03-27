// FileWin32 struct

//use std::io;
use std::cmp;
use std::ptr::*;
use std::mem;

use winapi::shared::minwindef::*;
use winapi::shared::ntdef::{ BOOLEAN, FALSE, HANDLE, TRUE };
use winapi::um::winnt::*;
use winapi::um::fileapi::*;
use winapi::um::debugapi::*;
use winapi::um::winbase::*;
use winapi::um::minwinbase::{OVERLAPPED,LPOVERLAPPED,LPOVERLAPPED_COMPLETION_ROUTINE};
use winapi::shared::winerror::*;
//use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::errhandlingapi::*;
use winapi::um::ioapiset::*;
use winapi::um::synchapi::*;

use crate::common::*;

pub type OvlComplFn = fn(*const OvlReader);

pub(self) struct TOVL {
  pub ovl_: OVERLAPPED,
  pub host_: *mut OvlReader
}

pub struct OvlReader {
  pub(self) hndl_: HANDLE,
  pub(self) ovls_: Vec<TOVL>,
  pub(self) callback_: OvlComplFn,
  pub buffer : Vec<u8>,
  pub active_readers : u32,
  pub ok_read : usize
}

pub struct FileWin32 {
  pub(self) hndl_: HANDLE
}

impl FileWin32 {
  pub fn default() -> Self { FileWin32{ hndl_: std::ptr::null_mut() } }
  pub fn new() -> Self { FileWin32{ hndl_: std::ptr::null_mut() } }
  pub fn raw(&self) -> HANDLE { self.hndl_ }
  pub fn not_file(&self) -> bool { std::ptr::null_mut()==self.hndl_ }
  pub fn close(&mut self) {
    if(!self.not_file()) {
      unsafe {
        CloseHandle(self.raw());
      }
      self.hndl_=std::ptr::null_mut();
    }
  }
  pub fn open(&mut self, fname: &String)->Result<(), DWORD> {
    unsafe{
      let ufn = from_str_unchecked(fname);
      //OutputDebugStringW(ufn.as_ptr());
      let hf = CreateFileW(ufn.as_ptr(), GENERIC_READ, FILE_SHARE_READ, std::ptr::null_mut(), 
        OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL|FILE_FLAG_OVERLAPPED, std::ptr::null_mut());
      if hf==INVALID_HANDLE_VALUE {
        return Err(GetLastError());
      }
      self.set(hf);
    }
    Ok(())
  }
  pub(self) fn set(&mut self, newhdl: winapi::um::winnt::HANDLE) {
    self.close();
    self.hndl_ = newhdl;
  }
  pub fn getFullPath(rname: &String) -> Result<(String, isize), DWORD> {
    let mut tbuffer: Vec<u16> = Vec::with_capacity(MAX_PATH);
    tbuffer.resize(MAX_PATH, 0);
    let filePath: LPWSTR = tbuffer.as_mut_ptr();
    let mut filePart: LPWSTR = null_mut();
    unsafe {
      let ufn = from_str_unchecked(rname);
      let mut retVal: DWORD = GetFullPathNameW(ufn.as_ptr(), tbuffer.len() as u32, 
                        filePath, &mut filePart as *mut LPWSTR);
      if(0==retVal) {
        return Err(0)
      }
      if(retVal>tbuffer.len() as u32) {
        tbuffer.resize(retVal as usize, 0);
        retVal = GetFullPathNameW(ufn.as_ptr(), tbuffer.len() as u32, 
                    tbuffer.as_mut_ptr(), &mut filePart as *mut LPWSTR);
        if(0==retVal) {
          return Err(0)
        }
      }
      let mut offset = -1isize;
      if(null_mut()!=filePart) {
        offset = (((filePart as *const _ as usize)-(filePath as *const _ as usize)) 
                    / mem::size_of::<u16>()) as isize
      }
      //println!("{:#?}, {:#?}, {:#?}", filePath, filePart, offset);
      Ok((U16String::from_ptr_str(tbuffer.as_ptr()).to_string_lossy(), offset))
    }
  }
  pub fn getSize(&self) -> isize {
    if(self.not_file()) {
      return -1isize
    }
    unsafe {
      let mut FileSize : LARGE_INTEGER = mem::zeroed();
      if(0==GetFileSizeEx(self.raw(), &mut FileSize)) {
        return -1isize
      }
      *FileSize.QuadPart() as isize
    }
  }
  //////////
  pub fn read(&self, offset: u64, size: u64, chunk_size: u64, compl_proc: OvlComplFn) -> 
        Result<Box<OvlReader>, DWORD> {
    let mut chnk_sz = chunk_size as u64;
    if (chunk_size > (MAXDWORD as u64)) {
      println!("Reading by 4G.");
      chnk_sz = (MAXDWORD as u64);
    }
    if (0 == size || 0==chunk_size || self.not_file()) {
      return Err(ERROR_INVALID_PARAMETER)
    }
    let chunk_quantity : usize = ((size+chnk_sz-1)/chnk_sz) as usize;
    let mut newrdr = Box::new(OvlReader::new(self.raw(), size as usize, chunk_quantity, compl_proc));
    println!("Buffer allocated, start reading");
    let mut left = size as usize;
    let mut off = offset;
    let mut mem_off = 0usize;
    let mut chunk_idx = 0usize;
    while(0!=left) {
      let toread = std::cmp::min(left as u64, chnk_sz) as usize;
      let res = newrdr.start_chunk(off, toread as u32, mem_off, chunk_idx);
      if(ERROR_SUCCESS!=res) {
        return Err(res)
      }
      left-=toread; off+=toread as u64; mem_off+=toread; chunk_idx+=1
    }
    println!("{}, {} readers started", time_mark(), newrdr.active_readers);
    Ok(newrdr)
  }
}
impl Drop for FileWin32 {
  fn drop(&mut self) {
    self.close()
  }
}


///////////////////////////////////
impl OvlReader {
  pub fn new(file: HANDLE, size: usize, chunk_quantity: usize, callback: OvlComplFn) -> Self {
    let mut ret = OvlReader{
      hndl_: file,
      buffer: Vec::with_capacity(size),
      ovls_: Vec::with_capacity(chunk_quantity),
      callback_: callback,
      ok_read: 0,
      active_readers: 0
    };
    //println!("{:#?}::OvlReader", &mut ret as *mut OvlReader);
    ret.buffer.resize(size, 0xBD);
    ret.ovls_.resize_with(chunk_quantity, TOVL::default);
    ret
  }
  
  pub(self) fn start_chunk(&mut self, off: u64, size: u32, mem_off: usize, chunk_idx: usize) -> DWORD {
    unsafe {
      let ovl : *mut TOVL = self.ovls_.as_mut_ptr().add(chunk_idx);
      //println!("{:#?}::start_chunk {:#?}", self as *mut OvlReader, ovl);
      let buff : *mut u8 = self.buffer.as_mut_ptr().add(mem_off);
      (*ovl).host_ = self;
      (*ovl).ovl_.u.s_mut().Offset = (off & 0xFFFFFFFF) as u32;
      (*ovl).ovl_.u.s_mut().OffsetHigh = (off >> 32) as u32;
      extern "system" fn fncompl(err_code: DWORD, transfered: DWORD, prm: *mut OVERLAPPED) {
        let ovl : *mut TOVL = prm as *mut TOVL;
        unsafe {
          let host = (*ovl).host_;
          (*ovl).host_ = null_mut();
          //println!("{:#?}::fncompl {:#?}", ovl, host);
          (*host).active_readers -= 1;
          if ERROR_SUCCESS!=err_code {
            println!("{}", str_win32err(err_code));
            (*host).cancel();
          } else {
            (*host).ok_read+=transfered as usize;
          }
          if 0==(*host).active_readers {
            ((*host).callback_)(host);
          }
        }
      }
      if(0==ReadFileEx(self.hndl_, buff as *mut _, size, ovl as LPOVERLAPPED, Some(fncompl))) {
        let ec: DWORD = GetLastError();
        if ERROR_IO_PENDING!=ec {
          self.cancel();
          return ec
        }
      }
      self.active_readers += 1
    }
    ERROR_SUCCESS
  }
  pub fn cancel(&mut self) {
    unsafe {
      SleepEx(1, 1);
      if 0!=self.active_readers {
        self.active_readers = 0;
        let ovlptr = self.ovls_.as_ptr();
        for it in 0..self.ovls_.len()-1 {
          let ovl : *const TOVL = ovlptr.add(it);
          if null_mut()==(*ovl).host_ {
            CancelIoEx(self.hndl_, ovl as *mut OVERLAPPED);
          }
        }
      }
    }
  }
}
impl Drop for OvlReader {
  fn drop(&mut self) {
    //println!("{:#?}::~OvlReader", self as *mut OvlReader);
    self.cancel();
  }
}

impl Drop for TOVL {
  fn drop(&mut self) {
    //println!("{:#?}::~TOVL", self as *mut TOVL);
  }
}

impl TOVL {
  pub fn default() -> Self { TOVL{ ovl_: unsafe{mem::zeroed()}, host_: std::ptr::null_mut() } }
}
