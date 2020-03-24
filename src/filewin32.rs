// FileWin32 struct

//use std::io;
//use std::cmp;
use std::ptr::*;

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
  pub fn close(&mut self) {
    if(self.hndl_!=std::ptr::null_mut()) {
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
  pub fn getFullPath(rname: &String) -> (String, u32) {
    (rname.to_string(),42u32)
  }
}
impl Drop for FileWin32 {
  fn drop(&mut self) {
    self.close()
  }
}