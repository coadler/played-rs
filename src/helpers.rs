use byteorder::{ByteOrder, LittleEndian};
use chrono::{DateTime, TimeZone, Utc};
use std::convert::TryInto;

pub(crate) fn bytes_to_date(raw: &[u8]) -> DateTime<Utc> {
    let secs = LittleEndian::read_u64(raw);
    Utc.timestamp(secs.try_into().unwrap(), 0)
}
