use anyhow::anyhow;
use bytes::BufMut;

use bilrost::Message;

pub mod test_messages;

pub enum RoundtripResult {
    /// The roundtrip succeeded.
    Ok(Vec<u8>),
    /// The data could not be decoded. This could indicate a bug in prost,
    /// or it could indicate that the input was bogus.
    DecodeError(bilrost::DecodeError),
    /// Re-encoding or validating the data failed.  This indicates a bug in `prost`.
    Error(anyhow::Error),
}

impl RoundtripResult {
    /// Unwrap the roundtrip result.
    pub fn unwrap(self) -> Vec<u8> {
        match self {
            RoundtripResult::Ok(buf) => buf,
            RoundtripResult::DecodeError(error) => {
                panic!("failed to decode the roundtrip data: {}", error)
            }
            RoundtripResult::Error(error) => panic!("failed roundtrip: {}", error),
        }
    }

    /// Unwrap the roundtrip result. Panics if the result was a validation or re-encoding error.
    pub fn unwrap_error(self) -> Result<Vec<u8>, bilrost::DecodeError> {
        match self {
            RoundtripResult::Ok(buf) => Ok(buf),
            RoundtripResult::DecodeError(error) => Err(error),
            RoundtripResult::Error(error) => panic!("failed roundtrip: {}", error),
        }
    }
}

pub fn roundtrip<M>(data: &[u8]) -> RoundtripResult
where
    M: Message,
{
    // Try to decode a message from the data. If decoding fails, continue.
    let message = match M::decode(data) {
        Ok(decoded) => decoded,
        Err(error) => return RoundtripResult::DecodeError(error),
    };

    let encoded_len = message.encoded_len();

    let mut buf1 = Vec::new();
    if let Err(error) = message.encode(&mut buf1) {
        return RoundtripResult::Error(error.into());
    }
    if encoded_len != buf1.len() {
        return RoundtripResult::Error(anyhow!(
            "expected encoded len ({}) did not match actual encoded len ({})",
            encoded_len,
            buf1.len()
        ));
    }

    let prepend_buf = message.encode_fast();

    if encoded_len != prepend_buf.len() {
        return RoundtripResult::Error(anyhow!(
            "expected encoded len ({}) did not match actual prepended len ({})",
            encoded_len,
            prepend_buf.len()
        ))
    }

    let mut prepended = Vec::new();
    prepended.put(prepend_buf);
    if prepended != buf1 {
        return RoundtripResult::Error(anyhow!(
            "encoded and prepended messages were different",
        ))
    }

    let roundtrip = match M::decode(buf1.as_slice()) {
        Ok(roundtrip) => roundtrip,
        Err(error) => return RoundtripResult::Error(anyhow::Error::new(error)),
    };

    let buf2 = roundtrip.encode_to_vec();
    let buf3_rev = roundtrip.encode_fast();
    let mut buf3 = Vec::new();
    buf3.put(buf3_rev);

    /*
    // Useful for debugging:
    eprintln!(" data: {:?}", data.iter().map(|x| format!("0x{:x}", x)).collect::<Vec<_>>());
    eprintln!(" buf1: {:?}", buf1.iter().map(|x| format!("0x{:x}", x)).collect::<Vec<_>>());
    eprintln!("a: {:?}\nb: {:?}", all_types, roundtrip);
    */

    if buf1 != buf2 {
        return RoundtripResult::Error(anyhow!("roundtripped encoded buffers do not match"));
    }

    if buf1 != buf3 {
        return RoundtripResult::Error(anyhow!(
            "roundtripped encoded buffers do not match with `encode_to_vec`"
        ));
    }

    RoundtripResult::Ok(buf1)
}
