use crate::{config::Config, packet::Packet};
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{fs::File, io::Read, net::UdpSocket};

mod config;
mod packet;
mod util;
fn main() {
    let (tx, rx) = channel();

    let config = Config::read("config.json");
    let socket =
        UdpSocket::bind(format!("127.0.0.1:{}", config.UDPPort)).expect("创建UDP套接字失败");
    socket.set_nonblocking(true).expect("设置套接字非阻塞失败");
    let shared_socket = Arc::new(Mutex::new(socket));

    // 创建接收ACK的线程
    let recv_socket = shared_socket.clone();
    let recv_trd = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(500)); // TODO:在config文件中指定轮询时间
        let socket = recv_socket.lock().unwrap();
        let mut buf = [0u8; std::mem::size_of::<i32>()];
        println!("recv线程正在尝试接收ACK...");
        match socket.recv_from(&mut buf) {
            Ok((_, source)) => {
                let seq_num = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                println!("接收到来自{}的ACK, ACK确认序号: {}", source, seq_num);
                tx.send(seq_num).unwrap();
            }
            Err(ref err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                } else {
                    print!("接收ACK失败: {}", err);
                    return;
                }
            }
        }
    });

    // 创建发送线程
    let send_trd = thread::spawn(move || {
        let config = Config::read("config.json");
        let mut file = File::open(&config.FileToSend).expect("无法打开要发送的文件");
        let mut file_size = std::fs::metadata(&config.FileToSend)
            .expect("获取文件元数据错误")
            .len() as usize; // 计算要发送文件的大小
        let mut frame_cnt = 0;
        if file_size as usize % config.DataSize == 0 {
            // 刚好放入 file_size / config.DataSize 帧
            frame_cnt = (file_size / config.DataSize) as i32;
        } else {
            // 否则帧数得+1
            frame_cnt = (file_size / config.DataSize) as i32 + 1;
        }
        let mut seq_num = 0;
        loop {
            if seq_num > frame_cnt {
                println!("发送完成");
                break;
            } else {
                println!("正在发送第{}帧...", seq_num);
            }
            let send_socket = shared_socket.clone();
            let mut packet = Packet {
                seq_num,
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
                let packet_trd = thread::spawn(move || loop {
                    thread::sleep(Duration::from_millis(config.Timeout as u64));
                    let config = Config::read("config.json");
                    let socket = send_socket.lock().unwrap();
                    if let Ok(b) = stp_recv.try_recv() {
                        if b {
                            break;
                        }
                    }
                    util::send_packet(&packet, &socket, &config);
                });
                packet_trd.join().expect("packet_trd线程启动失败");
                let seq_num_from_ack_trd = rx.recv().expect("从channel中获取ACK失败"); // 从ACK接收线程收取seq_num
                if seq_num_from_ack_trd == seq_num {
                    println!("确认接收方已收到第{}帧, 正在停止第{}帧的发送线程...", seq_num, seq_num);
                    stp_sn.send(true).unwrap();
                    break;
                }
                seq_num += 1;
                continue;
            }

            if file_size < config.DataSize {
                // 最后一帧
                packet.data_size = file_size;
            } else {
                file_size -= config.DataSize;
            }

            packet.data.resize(config.DataSize, 0);
            match file.read_exact(&mut packet.data) {
                Ok(_) => {}
                Err(_) => {}
            }

            packet.checksum = packet.crc(); // 读取文件数据后重新计算crc

            let (stp_sn, stp_recv) = channel();
            let packet_trd = thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(config.Timeout as u64));
                let config = Config::read("config.json");
                let socket = send_socket.lock().unwrap();

                if let Ok(b) = stp_recv.try_recv() {
                    if b {
                        break;
                    }
                }
                util::send_packet(&packet, &socket, &config);
            });
            packet_trd.join().expect("packet_trd线程启动失败");

            let seq_num_from_ack_trd = rx.recv().unwrap(); // 从ACK接收线程收取seq_num
            if seq_num_from_ack_trd == seq_num {
                stp_sn.send(true).unwrap();
                break;
            }

            seq_num += 1;
        }
    });

    recv_trd.join().unwrap();
    send_trd.join().unwrap();
}
