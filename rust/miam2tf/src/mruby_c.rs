#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(improper_ctypes)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::approx_constant)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::transmute_int_to_bool)]
#![allow(clippy::useless_transmute)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
