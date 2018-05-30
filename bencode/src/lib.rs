
// TODO: Remove allowances after successful implementation.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[macro_use]
extern crate serde;

#[doc(inline)]
pub use self::error::{Error, Result};
#[doc(inline)]
pub use self::decoder::*;

pub mod error;
pub mod decoder;
pub mod de;
