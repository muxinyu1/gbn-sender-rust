use std::net::UdpSocket;
use crc_any::CRCu16;

#[repr(C)]
struct Data {
    seq_num: i32,
    data_size: usize,
    data: [u8; 1024],
    checksum: u16
}

fn main() {
    let mut data = Data{
        seq_num: 0,
        data_size: 1024,
        data: [52u8; 1024],
        checksum: 0
    };
    const SZ: usize = std::mem::size_of::<Data>();
    let bytes: [u8; std::mem::size_of::<Data>()] = unsafe {
        std::mem::transmute_copy(&data)
    };

    let mut crc_ccitt = CRCu16::crc16ccitt_false();
    crc_ccitt.digest(&bytes[..(SZ - std::mem::size_of::<u16>())]);
    data.checksum = crc_ccitt.get_crc();

    let bytes: [u8; std::mem::size_of::<Data>()] = unsafe {
        std::mem::transmute_copy(&data)
    };
    let socket = UdpSocket::bind("127.0.0.1:0").expect("create udp socket error");
    socket.send_to(&bytes, "127.0.0.1:42695").expect("send error");
    println!("发送的校验码: {}", crc_ccitt.get_crc());
}
