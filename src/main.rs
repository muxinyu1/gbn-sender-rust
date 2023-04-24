use crate::{config::Config, packet::Packet};
use crc_any::CRCu16;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::{fs::File, io::Read, net::UdpSocket};

#[repr(packed)]
struct Data {
    seq_num: i32,
    data_size: usize,
    data: [u8; 1024],
    checksum: u16,
}
mod config;
mod packet;
mod util;
fn main() {

    let (tx, rx) = channel();

    // 创建接收ACK的线程
    let recv_trd = thread::spawn(move || loop {
        let config = Config::read("config.json");
        let socket =
            UdpSocket::bind(format!("127.0.0.1:{}", config.UDPPort)).expect("创建UDP套接字失败");
        let mut buf = [0u8; std::mem::size_of::<i32>()];
        socket.recv_from(&mut buf).expect("接收ACK分组失败");
        let seq_num = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        tx.send(seq_num).unwrap();
    });

    // 创建发送线程
    let send_trd = thread::spawn(move || {
        let config = Config::read("config.json");
        let mut file = File::open(&config.FileToSend).expect("无法打开要发送的文件");
        let mut buf = vec![0u8; config.DataSize];
        let file_size = std::fs::metadata(&config.FileToSend)
            .expect("获取文件元数据错误")
            .len(); // 计算要发送文件的大小
        let mut frame_cnt = 0;
        if file_size as usize % config.DataSize == 0 {
            // 刚好放入 file_size / config.DataSize 帧
            frame_cnt = (file_size as usize / config.DataSize) as i32;
        } else {
            // 否则帧数得+1
            frame_cnt = (file_size as usize / config.DataSize) as i32 + 1;
        }
        let mut seq_num = 0;
        loop {
            let mut packet = Packet {
                seq_num: seq_num,
                data_size: config.DataSize,
                data: vec![],
                checksum: 0,
            };
            if packet.seq_num == 0 {
                // 第0帧发送基本数据
                packet.data.extend_from_slice(&frame_cnt.to_le_bytes());
                packet
                    .data
                    .extend_from_slice(&(config.FileToSend.len() as i32).to_le_bytes());
                packet.data.extend_from_slice(&config.FileToSend.as_bytes());
                packet.data.resize(config.DataSize, 0);
                packet.checksum = packet.crc();

                let (stp_sn, stp_recv) = channel();
                let packet_trd = thread::spawn(move || {
                    let config = Config::read("config.json");
                    let socket =
                    UdpSocket::bind(format!("127.0.0.1:{}", config.UDPPort)).expect("创建UDP套接字失败");
                    loop {
                        match stp_recv.try_recv() {
                            Ok(_) => {break;}
                            Err(_) => {}
                        }
                        socket.send_to(&packet.as_bytes(),format!("{}:{}", config.Where, config.WhichPort)).expect("UDP发送错误");
                        thread::sleep(Duration::from_millis(config.Timeout as u64));
                    }
                });
                packet_trd.join().expect("packet_trd线程启动失败");
                let seq_num_from_ack_trd = rx.recv().unwrap(); // 从ACK接收线程收取seq_num
                if seq_num_from_ack_trd == seq_num {
                    stp_sn.send(true).unwrap();
                }
                seq_num += 1;
                continue;
            }
        }
    });
    let mut data = Data {
        seq_num: 0,
        data_size: 1024,
        data: [52u8; 1024],
        checksum: 0,
    };
    const SZ: usize = std::mem::size_of::<Data>();
    let bytes: [u8; std::mem::size_of::<Data>()] = unsafe { std::mem::transmute_copy(&data) };

    let mut crc_ccitt = CRCu16::crc16ccitt_false();
    crc_ccitt.digest(&bytes[..(SZ - std::mem::size_of::<u16>())]); // checksum
    data.checksum = crc_ccitt.get_crc();

    let bytes: [u8; std::mem::size_of::<Data>()] = unsafe { std::mem::transmute_copy(&data) };
    let socket = UdpSocket::bind("127.0.0.1:0").expect("create udp socket error");
    socket
        .send_to(&bytes, "127.0.0.1:42695")
        .expect("send error");
    println!("发送的校验码: {}", crc_ccitt.get_crc());
}
