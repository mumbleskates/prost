use anyhow::anyhow;
use bilrost::Canonicity::Canonical;
use bilrost::{DecodeError, DistinguishedMessage, Message, WithCanonicity};
use bytes::BufMut;
use regex::Regex;
use std::str::{from_utf8, FromStr};
use std::sync::LazyLock;

pub mod test_messages;

pub fn test_message(data: &[u8]) {
    let _ = roundtrip::<test_messages::TestAllTypes>(data).unwrap_error();
    let _ = roundtrip_distinguished::<test_messages::TestDistinguished>(data).unwrap_error();
}

pub fn test_parse_date(data: &[u8]) {
    static DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"^(\d{4}|[+-]\d+)-\d\d-\d\d([tT ]\d\d:\d\d:\d\d(\.\d{1,9})?( ?([+-]\d\d(:?\d\d)?)?|[zZ]))?$",
        )
        .unwrap()
    });
    // input must be text
    let Ok(s) = from_utf8(data) else {
        return;
    };
    // parse input as a datetime
    let Ok(t) = bilrost_types::Timestamp::from_str(s) else {
        return;
    };
    // check that it matches our regex pattern
    assert!(DATE_RE.is_match(s));
    // round trip from string again
    let s2 = format!("{t}");
    assert_eq!(Ok(&t), s2.parse().as_ref());
    // check that chrono has basically the same iso8601/rfc3339 date
    let Some(chrono_delta) = chrono::TimeDelta::new(t.seconds, t.nanos as u32) else {
        return;
    };
    let Some(chrono_time) =
        chrono::DateTime::<chrono::Utc>::UNIX_EPOCH.checked_add_signed(chrono_delta)
    else {
        return;
    };
    let s3 = chrono_time.to_rfc3339();
    assert_eq!(s2.strip_suffix("Z"), s3.strip_suffix("+00:00"));
    assert_eq!(Ok(&t), s3.parse().as_ref());
}

enum RoundtripResult {
    /// The roundtrip succeeded.
    Ok(Vec<u8>),
    /// The data could not be decoded. This could indicate a bug in bilrost,
    /// or it could indicate that the input was bogus.
    DecodeError(DecodeError),
    /// Re-encoding or validating the data failed.  This indicates a bug in `bilrost`.
    Error(anyhow::Error),
}

impl RoundtripResult {
    /// Unwrap the roundtrip result.
    #[allow(dead_code)]
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
    pub fn unwrap_error(self) -> Result<Vec<u8>, DecodeError> {
        match self {
            RoundtripResult::Ok(buf) => Ok(buf),
            RoundtripResult::DecodeError(error) => Err(error),
            RoundtripResult::Error(error) => panic!("failed roundtrip: {}", error),
        }
    }
}

fn roundtrip<M>(data: &[u8]) -> RoundtripResult
where
    M: Message,
{
    // Try to decode a message from the data. If decoding fails, continue.
    let message = match M::decode(data) {
        Ok(decoded) => decoded,
        Err(error) => return RoundtripResult::DecodeError(error),
    };

    let encoded_len = message.encoded_len();

    let buf1 = message.encode_to_vec();
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
        ));
    }

    let mut prepended = Vec::new();
    prepended.put(prepend_buf);
    if prepended != buf1 {
        return RoundtripResult::Error(anyhow!("encoded and prepended messages were different",));
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
            "roundtripped encoded buffers do not match with prepend-encoding"
        ));
    }

    RoundtripResult::Ok(buf1)
}

fn roundtrip_distinguished<M>(data: &[u8]) -> RoundtripResult
where
    M: DistinguishedMessage + Eq,
{
    // Try to decode a message from the data. If decoding fails, continue.
    let (message, canon) = match M::decode_distinguished(data) {
        Ok(decoded) => decoded,
        Err(error) => return RoundtripResult::DecodeError(error),
    };

    let encoded_len = message.encoded_len();

    let buf1 = message.encode_to_vec();
    if encoded_len != buf1.len() {
        return RoundtripResult::Error(anyhow!(
            "expected encoded len ({}) did not match actual encoded len ({})",
            encoded_len,
            buf1.len()
        ));
    }

    match canon {
        Canonical => {
            if buf1.as_slice() != data {
                return RoundtripResult::Error(anyhow!(
                    "decoded canonically but did not round trip"
                ));
            }
        }
        _ => {
            if buf1.as_slice() == data {
                return RoundtripResult::Error(anyhow!(
                    "decoded non-canonically but round tripped unchanged"
                ));
            }
        }
    }

    let prepend_buf = message.encode_fast();

    if encoded_len != prepend_buf.len() {
        return RoundtripResult::Error(anyhow!(
            "expected encoded len ({}) did not match actual prepended len ({})",
            encoded_len,
            prepend_buf.len()
        ));
    }

    let mut prepended = Vec::new();
    prepended.put(prepend_buf);
    if prepended != buf1 {
        return RoundtripResult::Error(anyhow!("encoded and prepended messages were different",));
    }

    let roundtrip = match M::decode_distinguished(buf1.as_slice()).canonical() {
        Ok(roundtrip) => roundtrip,
        Err(error) => return RoundtripResult::Error(anyhow::Error::new(DecodeError::new(error))),
    };

    if roundtrip != message {
        return RoundtripResult::Error(anyhow!("roundtripped message structs are not equal"));
    }

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
            "roundtripped encoded buffers do not match with prepend-encoding"
        ));
    }

    RoundtripResult::Ok(buf1)
}
