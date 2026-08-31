#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use std::sync::atomic::AtomicBool;

pub static Verbose: AtomicBool = AtomicBool::new(true);
pub static DebugMode: AtomicBool = AtomicBool::new(false);