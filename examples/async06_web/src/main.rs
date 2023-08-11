use std::fs;
use std::time::Duration;

// one: use std::{net::{TcpListener, TcpStream}, io::{Write, Read}};
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use futures::stream::StreamExt;
use async_std::prelude::*;
use async_std::task;

#[async_std::main]
async fn main() {
    // let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //     handle_connection(stream).await;
    // }

    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    listener.incoming().for_each_concurrent(None, |tcpstream| async move {
        let tcpstream = tcpstream.unwrap();
        task::spawn(handle_connection(tcpstream));
    }).await;

}

async fn handle_connection(mut stream: TcpStream) {
    // 从连接中读取1024字节数据
    let mut buffer = [0; 1024];
    // stream.read(&mut buffer).unwrap();
    stream.read(&mut buffer).await.unwrap();
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // println!("接收内容为: {:?}", str::from_utf8(&buffer));

    // 处理HTTP协议头
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    }else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HHTTP/1.1 200 OK\r\n\r\n", "sleep.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap_or_else(|err| {
         println!("无法找到文件: {:?}", err);
         String::from("")
    });

    // 将恢复内容写入到连接缓存中
    let response = format!("{status_line}{contents}");
    // stream.write_all(response.as_bytes()).unwrap();
    // stream.flush().unwrap();
    stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}
