# gbn-sender-rust

BIT计算机网络编程作业1 发送端 Rust实现

## 部署

在`config.json`中配置发送端的一些属性:

- `UDPPort`: 发送端程序绑定的端口
- `DataSize`: 每个分组中, 有效数据的大小(Byte)
- `ErrorRate`: 分组中某字节发送错误的概率(%)
- `SWSize`: 窗口大小
- `InitSeqNo`: 从哪个帧序号开始发送
- `Timeout`: 重传计时器(ms)
- `Where`: 发送到哪(IP)
- `WhichPort`: 哪个端口
- `FileToSend`: 要发送哪个文件

配置完成后, 命令行运行

```sh
cargo run
```