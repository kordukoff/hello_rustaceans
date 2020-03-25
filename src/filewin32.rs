// FileWin32 struct

//use std::io;
//use std::cmp;
use std::ptr::*;
use std::mem;

use winapi::shared::minwindef::*;
use winapi::shared::ntdef::{ BOOLEAN, FALSE, HANDLE, TRUE };
use winapi::um::winnt::*;
use winapi::um::fileapi::*;
use winapi::um::debugapi::*;
use winapi::um::winbase::*;
//use winapi::shared::winerror::*;
//use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::errhandlingapi::*;

use crate::common::*;

pub struct FileWin32 {
  pub(self) hndl_: HANDLE
}

impl FileWin32 {
  pub fn default() -> Self { FileWin32{ hndl_: std::ptr::null_mut() } }
  pub fn new() -> Self { FileWin32{ hndl_: std::ptr::null_mut() } }
  pub fn row(&self) -> HANDLE { self.hndl_ }
  pub fn not_file(&self) -> bool { std::ptr::null_mut()==self.hndl_ }
  pub fn close(&mut self) {
    if(!self.not_file()) {
      unsafe {
        CloseHandle(self.row());
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
  pub fn getFullPath(rname: &String) -> Result<String, DWORD> {
    let mut tbuffer: Vec<u16> = Vec::with_capacity(5);
    tbuffer.resize(MAX_PATH, 0);
    let filePath: LPWSTR = tbuffer.as_mut_ptr();
    let mut filePart: LPWSTR = null_mut();
    let mut retVal: DWORD = 0;
    unsafe {
      let ufn = from_str_unchecked(rname);
      retVal = GetFullPathNameW(ufn.as_ptr(), tbuffer.len() as u32, filePath, &mut filePart as *mut LPWSTR);
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
      Ok(U16String::from_ptr_str(tbuffer.as_ptr()).to_string_lossy())
    }
  }
  pub fn getSize(&self) -> u64 {
    if(self.not_file()) {
      return -1i64 as u64
    }
    unsafe {
      let mut FileSize : LARGE_INTEGER = mem::zeroed();
      if(0==GetFileSizeEx(self.row(), &mut FileSize)) {
        return -1i64 as u64
      }
      *FileSize.QuadPart() as u64
    }
  }
}
impl Drop for FileWin32 {
  fn drop(&mut self) {
    self.close()
  }
}