#[cfg(any(feature = "chrono", feature = "time"))]
pub(crate) mod time_proxies {
    use crate::buf::ReverseBuf;
    use crate::encoding::underived::{
        underived_decode, underived_decode_distinguished, underived_encode, underived_encoded_len,
        underived_prepend,
    };
    use crate::encoding::value_traits::empty_state_via_default;
    use crate::encoding::{
        Capped, DecodeContext, DistinguishedValueEncoder, Fixed, General, ValueEncoder, WireType,
        Wiretyped,
    };
    use crate::DecodeErrorKind::InvalidValue;
    use crate::{Canonicity, DecodeError};
    use bytes::{Buf, BufMut};

    #[derive(Debug, Default, PartialEq, Eq)]
    pub(crate) struct TimeDeltaProxy {
        pub(crate) secs: i64,
        pub(crate) nanos: i32,
    }

    empty_state_via_default!(TimeDeltaProxy);

    impl Wiretyped<General> for TimeDeltaProxy {
        const WIRE_TYPE: WireType = WireType::LengthDelimited;
    }

    impl ValueEncoder<General> for TimeDeltaProxy {
        fn encode_value<B: BufMut + ?Sized>(value: &Self, buf: &mut B) {
            underived_encode!(TimeDelta {
                1: General => secs: &value.secs,
                2: Fixed => nanos: &value.nanos,
            }, buf)
        }

        fn prepend_value<B: ReverseBuf + ?Sized>(value: &Self, buf: &mut B) {
            underived_prepend!(TimeDelta {
                2: Fixed => nanos: &value.nanos,
                1: General => secs: &value.secs,
            }, buf)
        }

        fn value_encoded_len(value: &Self) -> usize {
            underived_encoded_len!(TimeDelta {
                1: General => secs: &value.secs,
                2: Fixed => nanos: &value.nanos,
            })
        }

        fn decode_value<B: Buf + ?Sized>(
            value: &mut Self,
            mut buf: Capped<B>,
            ctx: DecodeContext,
        ) -> Result<(), DecodeError> {
            underived_decode!(TimeDelta {
                1: General => secs: &mut value.secs,
                2: Fixed => nanos: &mut value.nanos,
            }, buf, ctx)?;
            if value.secs.signum() as i32 * value.nanos.signum() == -1 {
                Err(DecodeError::new(InvalidValue))
            } else {
                Ok(())
            }
        }
    }

    impl DistinguishedValueEncoder<General> for TimeDeltaProxy {
        const CHECKS_EMPTY: bool = true;

        fn decode_value_distinguished<const ALLOW_EMPTY: bool>(
            value: &mut Self,
            mut buf: Capped<impl Buf + ?Sized>,
            ctx: DecodeContext,
        ) -> Result<Canonicity, DecodeError> {
            underived_decode_distinguished!(TimeDelta {
                1: General => secs: &mut value.secs,
                2: Fixed => nanos: &mut value.nanos,
            }, buf, ctx)
        }
    }
}
