use super::common::checksum;

pub trait FixSerializable {
    fn serialize_body_to_fix(&self) -> String;
}

#[inline]
pub fn serialize<T: FixSerializable>(t: &T) -> String {
    let body = t.serialize_body_to_fix();
    let header = format!("8=FIX.4.2\x019={}\x01", body.len());
    let chksum = checksum(header.as_bytes()) + checksum(body.as_bytes());
    format!("{}{}10={:03}\x01", header, body, chksum)
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(FixSerialize)]
    #[msg_type = "A"]
    struct Message {
        #[id = "50"] seq: u64,
        #[id = "51"] value: f64,
    }

    #[test]
    fn test() {
        let msg = Message { seq: 5, value: 1.23 };
        let fix = "8=FIX.4.2|9=18|35=A|50=5|51=1.23|10=038|";
        assert_eq!(serialize(&msg), fix.replace('|', "\x01"));
    }
}
