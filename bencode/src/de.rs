
//! Bencode deserializer.

use std::io;
use std::marker::PhantomData;
use std::result;
use std::str::FromStr;
use std::{i32, u64};

use serde::de::{self, Expected, Unexpected};

use super::error::{Error, ErrorCode, Result};

use read::{self, Reference};
