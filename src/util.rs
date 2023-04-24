use crate::config::Config;
use crate::packet::Packet;
use std::net::UdpSocket;
use rand::Rng;

pub fn send_packet(packet: &Packet, socket: &UdpSocket, config: &Config) {
    // 丢失
    let mut rng = rand::thread_rng();
    let random_num = rng.gen_range(1..=100);
    if random_num <= config.LostRate {
        return;
    }
    socket
        .send_to(
            &packet.as_bytes(),
            format!("{}:{}", config.Where, config.WhichPort),
        )
        .expect("UDP发送错误");
}
