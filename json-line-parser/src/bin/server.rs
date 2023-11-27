use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll};

use async_gen::AsyncIter;
use pin_project_lite::pin_project;
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};
use tokio::net::{TcpListener, TcpStream};

use json_line_parser::Message;

/// Implementation using `poll_next`.
trait AsyncIteratorPollNext {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
                 -> Poll<Option<Self::Item>>;
}

pin_project! {
    struct MessageReaderPollNext {
        #[pin]
        stream: TcpStream,
        #[pin]
        buffer: [u8; 1024],
        read_amount: usize
    }
}

impl AsyncIteratorPollNext for MessageReaderPollNext {
    type Item = Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let read_amount = self.read_amount;
        let mut this = self.project();

        loop {
            let mut read_buf = ReadBuf::new(&mut this.buffer[read_amount..]);
            match this.stream.as_mut().poll_read(cx, &mut read_buf) {
                Poll::Ready(read) => read.unwrap(),
                Poll::Pending => {
                    return Poll::Pending;
                }
            };
            let read = read_buf.filled().len();
            if read == 0 {
                return Poll::Ready(None);
            }
            *this.read_amount += read;
            if let Some(newline_index) = this.buffer.iter().position(|&c| c == b'\n') {
                let line = &this.buffer[..newline_index];
                let msg: Message = serde_json::from_slice(&line).unwrap();
                this.buffer.copy_within(newline_index + 1.., 0);
                *this.read_amount -= newline_index + 1;
                return Poll::Ready(Some(msg));
            }
        }
    }
}

async fn message_reader_poll_next() -> u32 {
    let listener = TcpListener::bind("127.0.0.1:5555").await.unwrap();
    let (client, _addr) = listener.accept().await.unwrap();
    let mut reader = MessageReaderPollNext {
        stream: client,
        buffer: [0; 1024],
        read_amount: 0,
    };
    let mut iter = pin!(reader);

    let mut counter = 0;
    while let Some(msg) = poll_fn(|cx| iter.as_mut().poll_next(cx)).await {
        match msg {
            Message::Ping => {}
            Message::Hello(_, count) => {
                counter += count;
            }
        }
    }
    counter
}

/// Implementation using `poll_next` using generator syntax.
async fn message_reader_async_gen() -> u32 {
    let listener = TcpListener::bind("127.0.0.1:5555").await.unwrap();
    let (mut client, _addr) = listener.accept().await.unwrap();

    let iter = AsyncIter::from(async_gen::gen! {
        let mut buffer = [0; 1024];
        let mut read_amount = 0;
        loop {
            let read = client.read(&mut buffer[read_amount..]).await.unwrap();
            if read == 0 {
                return;
            }
            read_amount += read;
            if let Some(newline_index) = buffer.iter().position(|&c| c == b'\n') {
                let line = &buffer[..newline_index];
                let msg: Message = serde_json::from_slice(&line).unwrap();
                buffer.copy_within(newline_index + 1.., 0);
                read_amount -= newline_index + 1;
                yield msg;
            }
        }
    });
    let mut iter = pin!(iter);

    let mut counter = 0;
    while let Some(msg) = poll_fn(|cx| iter.as_mut().poll_next(cx)).await {
        match msg {
            Message::Ping => {}
            Message::Hello(_, count) => {
                counter += count;
            }
        }
    }
    counter
}

/// Implementation using `async fn next`.
trait AsyncIteratorAsyncNext {
    type Item;

    async fn next(&mut self) -> Option<Self::Item>;
}

struct MessageReaderAsyncNext {
    stream: TcpStream,
    buffer: [u8; 1024],
    read_amount: usize,
}

impl AsyncIteratorAsyncNext for MessageReaderAsyncNext {
    type Item = Message;

    async fn next(&mut self) -> Option<Self::Item> {
        loop {
            let read = self.stream.read(&mut self.buffer[self.read_amount..]).await.unwrap();
            if read == 0 {
                return None;
            }
            self.read_amount += read;
            if let Some(newline_index) = self.buffer.iter().position(|&c| c == b'\n') {
                let line = &self.buffer[..newline_index];
                let msg: Message = serde_json::from_slice(&line).unwrap();
                self.buffer.copy_within(newline_index + 1.., 0);
                self.read_amount -= newline_index + 1;
                return Some(msg);
            }
        }
    }
}

async fn message_reader_async_next() -> u32 {
    let listener = TcpListener::bind("127.0.0.1:5555").await.unwrap();
    let (client, _addr) = listener.accept().await.unwrap();
    let mut reader = MessageReaderAsyncNext {
        stream: client,
        buffer: [0; 1024],
        read_amount: 0,
    };

    let mut counter = 0;
    while let Some(msg) = reader.next().await {
        match msg {
            Message::Ping => {}
            Message::Hello(_, count) => {
                counter += count;
            }
        }
    }
    counter
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mode = args[1].as_str();
    loop {
        let ret = match mode {
            "async" => {
                println!("async next");
                message_reader_async_next().await
            }
            "gen" => {
                println!("gen");
                message_reader_async_gen().await
            }
            "poll" => {
                println!("poll next");
                message_reader_poll_next().await
            }
            _ => {
                return;
            }
        };
        println!("{ret}");
    }
}
