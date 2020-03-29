#![windows_subsystem = "console"]

#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(unused_mut)]
#![allow(unused_variables)]

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
    if("z"==filepath) {filepath = String::from("C:\\Users\\dkord\\Downloads\\wix311.exe");}
    else {
      match FileWin32::getFullPath(&filepath) {
        Err(ec) => { println!("{}", str_win32err(ec)); continue }
        Ok(fp) => {  filepath=fp.0 }
      }
    }
    println!("{}", filepath);
    let mut thefile = FileWin32::new();
    match thefile.open(&filepath) {
      Err(ec) => { println!("{}", str_win32err(ec)); continue }
      Ok(_) => { }
    }
    let mut flsize = thefile.getSize();
    println!("{} opened, {} bytes", filepath, flsize);
    if(flsize<=0) {
      continue;
    }
    let mut tests: Vec<Box<OvlReader>> = Vec::with_capacity(64);
    loop {
      fn start_read(thefile: &FileWin32, tests: &mut Vec<Box<OvlReader>>, size:isize, chunk:isize) {
        match thefile.read(0, size as u64, chunk as u64, Ovl_op_complete) {
          Err(ec) => { println!("{}", str_win32err(ec)) }
          Ok(rdr) => { tests.push(rdr) }
        }
      }
      let uinp: String = gets(op_promt);
      match uinp.to_lowercase().as_ref() {
        "0" | "" => { break }
        "w" => { println!("{}", time_mark()); unsafe { SleepEx(1000, TRUE); } },
        "2" => { start_read(&thefile, &mut tests, flsize, flsize) },
        "1" => { start_read(&thefile, &mut tests, flsize, 4096) },
        "c" => { tests.clear() },
        _ => { }
      }
    }
  }
}

fn Ovl_op_complete(callee: *const OvlReader) {
  println!("{}, IO completed.", time_mark())
}

