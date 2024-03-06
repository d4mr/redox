use bytes::BytesMut;

use super::{
    int::{int, RespInt},
    parse, Resp, RespConcreteType, RespError, RespTypePartialable,
};

pub enum RespArray {
    Concrete(RespArrayConcrete),
    Partial(RespArrayPartial),
}

pub type RespArrayConcrete = Vec<RespConcreteType>;

#[derive(Debug)]
pub struct RespArrayPartial {
    length: RespInt,
    concrete_array: Option<Vec<RespConcreteType>>,
    partial_item: Option<Box<RespTypePartialable>>,
}

pub fn array_with_partial(
    buf: &mut BytesMut,
    partial: RespArrayPartial,
) -> Result<RespArray, RespError> {
    let RespArrayPartial {
        length,
        concrete_array,
        partial_item,
    } = partial;

    // length at this point could still be partial
    let length = match length {
        // if partial, try reading the rest of the length
        RespInt::Partial(partial) => match int(buf, Some(partial))? {
            // if concrete length received, then proceed to read array
            RespInt::Concrete(i) => i,
            // if partial length received, then return partial array with updated partial length
            RespInt::Partial(partial) => {
                return Ok(RespArray::Partial(RespArrayPartial {
                    length: RespInt::Partial(partial),
                    concrete_array: None,
                    partial_item: None,
                }))
            }
        },
        // if concrete length received, then proceed to read array
        RespInt::Concrete(i) => i,
    };

    // length is ready, now read the array
    // look at the last item in the array, check if it is a partial
    let mut concrete_array = concrete_array.unwrap_or(Vec::with_capacity(length as usize));

    // if partial item exists, try to parse
    if let Some(partial) = partial_item {
        match parse(buf, Some(*partial))? {
            // if the parse result is concrete, then add to array and continue
            Resp::Concrete(r) => {
                concrete_array.push(r);
            }
            // if parse result is partial, return partial array, with updated last partial element
            Resp::Partial(partial) => {
                return Ok(RespArray::Partial(RespArrayPartial {
                    length: RespInt::Concrete(length),
                    concrete_array: Some(concrete_array),
                    partial_item: Some(Box::new(partial)),
                }))
            }
        }
    }

    array_with_length(
        buf,
        concrete_array,
        TryInto::<usize>::try_into(length).map_err(|_| RespError::BadArraySize(length))?,
    )
}

fn array_with_length(
    buf: &mut BytesMut,
    mut concrete_array: RespArrayConcrete,
    length: usize,
) -> Result<RespArray, RespError> {
    while concrete_array.len() < length && !buf.is_empty() {
        match parse(buf, None)? {
            // if the parse result is concrete, then add to array and continue
            Resp::Concrete(r) => {
                concrete_array.push(r);
            }
            // if parse result is partial, return partial array, with updated last partial element
            Resp::Partial(partial) => {
                return Ok(RespArray::Partial(RespArrayPartial {
                    length: RespInt::Concrete(length as i64),
                    concrete_array: Some(concrete_array),
                    partial_item: Some(Box::new(partial)),
                }))
            }
        }
    }

    // if array length is equal to length, return concrete array
    // if we exit the loop without early return then the array is complete
    match concrete_array.len() < length {
        true => Ok(RespArray::Partial(RespArrayPartial {
            length: RespInt::Concrete(length as i64),
            concrete_array: Some(concrete_array),
            partial_item: None,
        })),

        false => Ok(RespArray::Concrete(concrete_array)),
    }
}

pub fn array(buf: &mut BytesMut) -> Result<RespArray, RespError> {
    // length at this point could still be partial
    let length = match int(buf, None)? {
        // if concrete length received, then proceed to read array
        RespInt::Concrete(i) => i,
        // if partial length received, then return partial array with updated partial length
        RespInt::Partial(partial) => {
            return Ok(RespArray::Partial(RespArrayPartial {
                length: RespInt::Partial(partial),
                concrete_array: None,
                partial_item: None,
            }))
        }
    };

    // length is ready, now read the array
    // look at the last item in the array, check if it is a partial
    let length = length
        .try_into()
        .map_err(|_| RespError::BadArraySize(length))?;

    let concrete_array = Vec::with_capacity(length);
    array_with_length(buf, concrete_array, length)
}
