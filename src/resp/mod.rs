use bytes::{Buf, Bytes, BytesMut};

use self::{
    array::{array, array_with_partial, RespArray, RespArrayConcrete, RespArrayPartial},
    int::{int, RespInt, RespIntConcrete, RespIntPartial},
    string::{string, string_with_partial, RespString, RespBulkStringConcrete, RespBulkStringPartial},
};

pub mod array;
pub mod int;
pub mod string;

#[derive(Debug)]
pub enum RespError {
    StringParseFailure,
    // UnexpectedEnd,
    UnknownStartingByte(u8),
    // IOError(std::io::Error),
    IntParseFailure,
    BadBulkStringSize(i64),
    BadArraySize(i64),
}

#[derive(Debug)]
pub enum Resp {
    Concrete(RespConcreteType),
    Partial(RespTypePartialable),
}

#[derive(Debug)]

pub enum RespConcreteType {
    Array(RespArrayConcrete),
    Int(RespIntConcrete),
    BulkString(RespBulkStringConcrete),
}

#[derive(Debug)]
pub enum RespTypePartialable {
    Array(RespArrayPartial),
    Int(RespIntPartial),
    BulkString(RespBulkStringPartial),
}

#[derive(Debug)]
pub enum Word {
    Concrete(Bytes),
    Partial(Bytes),
}

pub fn word(buf: &mut BytesMut) -> Word {
    for (i, b) in buf.iter().enumerate() {
        if *b == b'\r' {
            let output = buf.split_to(i);
            if 1 == buf.remaining() {
                // edge case when \r was read but not \n
                buf.clear();
                return Word::Concrete(output.into());
            }
            buf.advance(2);
            return Word::Concrete(output.into());
        }

        if *b == b'\n' {
            let output = Bytes::new();
            buf.advance(1);
            return Word::Partial(output);
        }
    }
    Word::Partial(buf.split().into())
}

pub fn parse(buf: &mut BytesMut, partial: Option<RespTypePartialable>) -> Result<Resp, RespError> {
    match partial {
        Some(partial) => match partial {
            RespTypePartialable::Array(partial_array) => {
                match array_with_partial(buf, partial_array)? {
                    RespArray::Concrete(r) => Ok(Resp::Concrete(RespConcreteType::Array(r))),
                    RespArray::Partial(partial) => {
                        Ok(Resp::Partial(RespTypePartialable::Array(partial)))
                    }
                }
            }
            RespTypePartialable::Int(partial_int) => match int(buf, Some(partial_int))? {
                RespInt::Concrete(r) => Ok(Resp::Concrete(RespConcreteType::Int(r))),
                RespInt::Partial(partial) => Ok(Resp::Partial(RespTypePartialable::Int(partial))),
            },
            RespTypePartialable::BulkString(partial_string) => {
                match string_with_partial(buf, partial_string)? {
                    RespString::Concrete(r) => Ok(Resp::Concrete(RespConcreteType::BulkString(r))),
                    RespString::Partial(partial) => {
                        Ok(Resp::Partial(RespTypePartialable::BulkString(partial)))
                    }
                }
            }
        },
        None => match buf.get_u8() {
            b'*' => match array(buf)? {
                RespArray::Concrete(r) => Ok(Resp::Concrete(RespConcreteType::Array(r))),
                RespArray::Partial(partial) => {
                    Ok(Resp::Partial(RespTypePartialable::Array(partial)))
                }
            },
            b':' => match int(buf, None)? {
                RespInt::Concrete(r) => Ok(Resp::Concrete(RespConcreteType::Int(r))),
                RespInt::Partial(partial) => Ok(Resp::Partial(RespTypePartialable::Int(partial))),
            },
            b'$' => match string(buf)? {
                RespString::Concrete(r) => Ok(Resp::Concrete(RespConcreteType::BulkString(r))),
                RespString::Partial(partial) => {
                    Ok(Resp::Partial(RespTypePartialable::BulkString(partial)))
                }
            },
            b'\n' => {
                buf.advance(1);
                parse(buf, None)
            }
            b => Err(RespError::UnknownStartingByte(b)),
        },
    }
}
