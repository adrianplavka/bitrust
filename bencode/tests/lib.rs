#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use bitrust_bencode::{from_str, to_string};

    #[quickcheck]
    fn circular(value: String) {
        let bencode = to_string(&value).unwrap();
        let de_value = from_str::<&str>(&bencode).unwrap();

        assert_eq!(value, de_value);
    }
}
