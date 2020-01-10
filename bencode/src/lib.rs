extern crate num_traits;
extern crate serde;

#[doc(inline)]
pub use self::de::{from_slice, from_str, Deserializer};

#[doc(inline)]
pub use self::error::{Error, Result};

#[doc(inline)]
pub use self::ser::{to_string, Serializer};

pub mod de;
pub mod error;
pub mod ser;

mod read;
