use json_line_parser::Message;
use std::io::Write;
use std::net::TcpStream;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let count = 100000;
    let msg = Message::Hello("Hello".to_string(), 12);
    let msg_bytes = serde_json::to_vec(&msg).unwrap();

    let mut stream = TcpStream::connect("127.0.0.1:5555").unwrap();
    let start = Instant::now();
    for _ in 0..count {
        stream.write_all(&msg_bytes).unwrap();
        stream.write(b"\n").unwrap();
    }
    stream.flush().unwrap();
    println!(
        "Message/s: {}",
        count as f64 / start.elapsed().as_secs_f64()
    );
}
