#![windows_subsystem = "console"]

//#![allow(unused_imports)]
//#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate winapi;
//extern crate widestring;

//use winapi::um::minwinbase::*;

//use std::cmp;
//use std::io;
use winapi::shared::minwindef::*;
use winapi::um::synchapi::*;

mod common;
mod filewin32;

use crate::common::*;
use crate::filewin32::*;

fn main() {
let op_promt = r#" test:
  1: read by 4k
  2: read whole file
  w: alertable sleep 1sec
  c: cleanup
  0: another file
? "#;

  loop {
    let mut filepath: String = gets("file name? ");
    if(""==filepath) {
      break;
    }
    if("z"==filepath) {filepath = String::from("C:\\Users\\dkord\\Downloads\\tz_ricom.doc");}
    let mut thefile = FileWin32::new();
    match thefile.open(&filepath) {
      Err(ec) => { println!("{}", str_win32err(ec)); continue }
      Ok(_)=>{ println!("{:?}", FileWin32::getFullPath(&filepath).0)}
    }

    loop {
      let uinp: String = gets(op_promt);
      match uinp.to_lowercase().as_ref() {
        "0" | "" => { break }
        "w" => { println!("{}", time_mark()); unsafe { SleepEx(1000, TRUE); } },
        _ => { }
      }
    }
  }
}

