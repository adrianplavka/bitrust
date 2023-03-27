mod token;

pub mod de;
pub mod error;
pub mod ser;

#[doc(inline)]
pub use self::de::{from_slice, from_str, Deserializer};

#[doc(inline)]
pub use self::ser::{to_string, to_vec, Serializer};

#[doc(inline)]
pub use self::error::{Error, Result};
