
// TODO: Remove allowances after successful implementation.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[doc(inline)]
pub use crate::error::{Error, Result};
#[doc(inline)]
pub use crate::decoder::*;

pub mod error;
pub mod decoder;
