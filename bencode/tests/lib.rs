#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use arbitrary::{Arbitrary, Unstructured};
    use quickcheck_macros::quickcheck;

    use bitrust_bencode::{from_str, to_string};
    use rand::Rng;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
    #[serde(untagged)]
    enum Value {
        Num(usize),

        String(String),

        List(Vec<Value>),

        Map(HashMap<String, Value>),
    }

    impl Value {
        fn _arbitrary<'a>(
            u: &mut arbitrary::Unstructured<'a>,
            rng: &mut rand::rngs::ThreadRng,
            depth: u8,
        ) -> arbitrary::Result<Value> {
            if depth >= 3 {
                return Ok(Value::Num(u.arbitrary()?));
            }

            match rng.gen_range(0, 10) {
                0 => Ok(Value::Num(u.arbitrary()?)),
                1 => Ok(Value::String(u.arbitrary()?)),
                2 => {
                    let size = rng.gen_range(0, 10);
                    let mut vec = Vec::with_capacity(size);

                    for _ in 0..size {
                        vec.push(Self::_arbitrary(u, rng, depth + 1)?);
                    }

                    Ok(Value::List(vec))
                }
                _ => {
                    let size = rng.gen_range(0, 10);
                    let mut map = HashMap::with_capacity(size);

                    for _ in 0..size {
                        map.insert(u.arbitrary()?, Self::_arbitrary(u, rng, depth + 1)?);
                    }

                    Ok(Value::Map(map))
                }
            }
        }
    }

    impl<'a> Arbitrary<'a> for Value {
        fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            Self::_arbitrary(u, &mut rand::thread_rng(), 0)
        }
    }

    #[quickcheck]
    fn circular_property(fuzz_data: Vec<u8>) {
        let mut u = Unstructured::new(fuzz_data.as_slice());
        let value = Value::arbitrary(&mut u).unwrap();

        assert_eq!(
            value,
            from_str::<Value>(&to_string(&value).unwrap()).unwrap()
        );
    }
}
