
#[derive(Clone)]
pub struct Packet {
    pub seq_num: i32,
    pub data_size: usize,
    pub data: Vec<u8>,
    pub checksum: u16
}

impl Packet {
    pub fn crc(&self) -> u16 {
        let mut bytes_to_check = vec![];
        bytes_to_check.extend_from_slice(&self.seq_num.to_le_bytes());
        bytes_to_check.extend_from_slice(&self.data_size.to_le_bytes());
        bytes_to_check.extend_from_slice(&self.data);
        bytes_to_check.shrink_to_fit();
        let mut crc_ccitt = crc_any::CRCu16::crc16ccitt_false();
        crc_ccitt.digest(&bytes_to_check);
        return crc_ccitt.get_crc();
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.seq_num.to_le_bytes());
        bytes.extend_from_slice(&self.data_size.to_le_bytes());
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        bytes.shrink_to_fit();
        return bytes;
    }
}

