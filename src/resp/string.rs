use bytes::{Bytes, BytesMut};
use std::{convert::From, str::from_utf8};

use super::{
    int::{int, RespInt},
    word, RespError, Word,
};

#[derive(Debug)]
pub enum RespString {
    Concrete(RespStringConcrete),
    Partial(RespStringPartial),
}

pub type RespStringConcrete = String;

#[derive(Debug)]
pub struct RespStringPartial {
    length: RespInt,
    string: Bytes,
}

pub fn string_with_partial(
    buf: &mut BytesMut,
    partial: RespStringPartial,
) -> Result<RespString, RespError> {
    let RespStringPartial {
        length,
        string: partial_string,
    } = partial;

    // length at this point could still be partial
    let length = match length {
        // if partial, try reading the rest of the length
        RespInt::Partial(partial) => match int(buf, Some(partial))? {
            // if concrete length received, then proceed to read string
            RespInt::Concrete(i) => i,
            // if partial length received, then return partial string with updated partial length
            RespInt::Partial(partial) => {
                return Ok(RespString::Partial(RespStringPartial {
                    length: RespInt::Partial(partial),
                    string: partial_string,
                }))
            }
        },
        // if concrete length received, then proceed to read string
        RespInt::Concrete(i) => i,
    };

    string_with_length(
        buf,
        partial_string,
        length
            .try_into()
            .map_err(|_| RespError::BadBulkStringSize(length))?,
    )
}

pub fn string_with_length(
    buf: &mut BytesMut,
    partial_string: Bytes,
    length: usize,
) -> Result<RespString, RespError> {
    // length is ready, now read the string
    match word(buf) {
        // if the current word is complete, add to existing partial partial and return concrete
        Word::Concrete(word) => {
            let concatenated_bytes = Bytes::from([partial_string.as_ref(), word.as_ref()].concat());
            if concatenated_bytes.len() != length {
                return Err(RespError::BadBulkStringSize(length as i64));
            }
            let raw = from_utf8(&concatenated_bytes).map_err(|_| RespError::StringParseFailure)?;
            Ok(RespString::Concrete(raw.to_string()))
        }
        // return partial string with concrete length but partial string
        Word::Partial(word) => {
            let concatenated_bytes = Bytes::from([partial_string.as_ref(), word.as_ref()].concat());
            Ok(RespString::Partial(RespStringPartial {
                length: RespInt::Concrete(length as i64),
                string: concatenated_bytes,
            }))
        }
    }
}

pub fn string(buf: &mut BytesMut) -> Result<RespString, RespError> {
    let partial_string = Bytes::new();

    // length at this point could still be partial
    let length = match int(buf, None)? {
        // if concrete length received, then proceed to read string
        RespInt::Concrete(i) => i,
        // if partial length received, then return partial string with updated partial length
        RespInt::Partial(partial) => {
            return Ok(RespString::Partial(RespStringPartial {
                length: RespInt::Partial(partial),
                string: partial_string,
            }))
        }
    };

    string_with_length(
        buf,
        partial_string,
        length
            .try_into()
            .map_err(|_| RespError::BadBulkStringSize(length))?,
    )
}
