pub mod cstr;
mod dir;
pub mod file;
pub mod logging;
mod mount;
pub mod result;



use std::{fs, ptr};
use crate::cstr::Utf8CString;

pub struct OverlayAttr(Utf8CString, Utf8CString);
