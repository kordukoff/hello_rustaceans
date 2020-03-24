// common - usable functions

use std::io;
use std::ptr::*;
use winapi::shared::minwindef::*;
use winapi::um::winbase::*;
use winapi::shared::ntdef::*;
use winapi::um::errhandlingapi::*;

pub type U16String = widestring::UCString<u16>;
pub fn from_str_unchecked(s: impl AsRef<str>) -> U16String {
  unsafe{
  let v: Vec<u16> = s.as_ref().encode_utf16().collect();
  return U16String::from_vec_unchecked(v);
}}

pub fn gets(prompt: &str) -> String {
  use std::io::{BufRead, Write};
  print!("{}", prompt);
  io::stdout().flush().unwrap();
  let mut ret = String::new();
  io::stdin().lock().read_line(&mut ret).unwrap();
  String::from(ret.trim())
}

pub fn time_mark() -> String {
  return chrono::Local::now().format("%F %H:%M:%S%.3f").to_string();
}

pub fn str_win32err(err_code: u32) -> String {
  let mut ec: DWORD = err_code as DWORD;
  let msg: U16String;
  unsafe {
    if 0==err_code {
      ec = GetLastError();
    }
    let mut buff: LPWSTR = null_mut();
    FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_IGNORE_INSERTS,
                   NULL, ec, MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL) as u32, 
                   (&mut buff as *mut LPWSTR) as LPWSTR, 0, null_mut());
    if(null_mut()==buff) {
      msg = from_str_unchecked(format!("Unknown error: {:#x}.\r\n", ec));
    }else{
      msg = U16String::from_ptr_str(buff);
      LocalFree(buff as /*HLOCAL*/*mut std::ffi::c_void);
    }
  }
  return msg.to_string_lossy();
}
