extern crate num_traits;
extern crate serde;

#[doc(inline)]
pub use self::de::{from_str, Deserializer};
#[doc(inline)]
pub use self::error::{Error, Result};

pub mod de;
pub mod error;
