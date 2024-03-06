use bytes::{Bytes, BytesMut};
use std::str;

use super::{word, RespError, Word};

#[derive(Debug)]
pub enum RespInt {
    Concrete(RespIntConcrete),
    Partial(RespIntPartial),
}

#[derive(Debug)]
pub struct RespIntPartial(Bytes);

pub type RespIntConcrete = i64;

impl RespInt {
    fn from_partial(partial_int: Bytes) -> RespInt {
        RespInt::Partial(RespIntPartial(partial_int))
    }
}

pub fn int(buf: &mut BytesMut, partial: Option<RespIntPartial>) -> Result<RespInt, RespError> {
    let RespIntPartial(partial_bytes) = partial.unwrap_or(RespIntPartial(Bytes::new()));

    match word(buf) {
        // if the current word is complete, add to existing partial partial and return concrete
        Word::Concrete(word) => {
            let concatenated_bytes = Bytes::from([partial_bytes.as_ref(), word.as_ref()].concat());
            let raw =
                str::from_utf8(&concatenated_bytes).map_err(|_| RespError::IntParseFailure)?;

            let i = raw.parse::<i64>().map_err(|_| RespError::IntParseFailure)?;

            Ok(RespInt::Concrete(i))
        }
        // return partial
        Word::Partial(word) => {
            let concatenated_bytes = Bytes::from([partial_bytes.as_ref(), word.as_ref()].concat());
            Ok(RespInt::from_partial(concatenated_bytes))
        }
    }
}
