#![feature(gen_blocks)]
#![feature(async_iterator)]

use std::async_iter::AsyncIterator;
use std::future::poll_fn;
use std::pin::{Pin, pin};
use std::task::{Context, Poll};

use json_line_parser::Message;
use pin_project_lite::pin_project;
use shared::{AfitAsyncIter, LendAfitAsyncIter, PollNextAsyncIter};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};
use tokio::net::{TcpListener, TcpStream};

// Implementation using `poll_next`.
pin_project! {
    struct MessageReaderPoll {
        #[pin]
        stream: TcpStream,
        #[pin]
        buffer: [u8; 1024],
        read_amount: usize
    }
}

impl PollNextAsyncIter for MessageReaderPoll {
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

async fn message_reader_poll() -> anyhow::Result<u32> {
    let listener = TcpListener::bind("127.0.0.1:5555").await?;
    let (client, _addr) = listener.accept().await?;
    let mut reader = MessageReaderPoll {
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
    Ok(counter)
}

// Implementation using `async_gen`.
async fn message_reader_async_gen() -> anyhow::Result<u32> {
    let listener = TcpListener::bind("127.0.0.1:5555").await?;
    let (mut client, _addr) = listener.accept().await?;

    // Note: the async gen doesn't hold any borrows across a yield point
    let iter = async
    gen {
        let mut buffer = [0; 1024];
        let mut read_amount = 0;
        loop {
        let read = client.read( & mut buffer[read_amount..]).await.unwrap();
        if read == 0 {
        return;
        }
        read_amount += read;
        if let Some(newline_index) = buffer.iter().position( |& c | c == b'\n') {
        let line = & buffer[..newline_index];
        let msg: Message = serde_json::from_slice( &line).unwrap();
        buffer.copy_within(newline_index + 1.., 0);
        read_amount -= newline_index + 1;
        yield msg;
        }
        }
    };
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
    Ok(counter)
}

// Implementation using `async fn next`.
struct MessageReaderAfit {
    stream: TcpStream,
    buffer: [u8; 1024],
    read_amount: usize,
}

impl AfitAsyncIter for MessageReaderAfit {
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

async fn message_reader_afit() -> anyhow::Result<u32> {
    let listener = TcpListener::bind("127.0.0.1:5555").await?;
    let (client, _addr) = listener.accept().await?;
    let mut reader = MessageReaderAfit {
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
    Ok(counter)
}

// Implementation using lending `async fn next`
// This has the advantage of not hardcoding the deserialization inside the iterator
// and allows composition.
// It's also a bit annoying to implement though.
struct MessageReaderLendAfit {
    stream: TcpStream,
    buffer: [u8; 1024],
    read_amount: usize,
    skip: usize,
}

impl LendAfitAsyncIter for MessageReaderLendAfit {
    type Item<'a> = &'a [u8];

    async fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        // This is basically logic that should follow after `yield` in `async gen` block.
        // But here we can't use `yield`, so we need to implement the logic manually here.
        if self.skip > 0 {
            self.buffer.copy_within(self.skip.., 0);
            self.read_amount -= self.skip;
            self.skip = 0;
        }
        loop {
            let read = self.stream.read(&mut self.buffer[self.read_amount..]).await.unwrap();
            if read == 0 {
                return None;
            }
            self.read_amount += read;
            if let Some(newline_index) = self.buffer.iter().position(|&c| c == b'\n') {
                let line = &self.buffer[..newline_index];
                self.skip = line.len() + 1;
                return Some(line);
            }
        }
    }
}

async fn message_reader_lend_afit() -> anyhow::Result<u32> {
    let listener = TcpListener::bind("127.0.0.1:5555").await?;
    let (client, _addr) = listener.accept().await?;
    let mut reader = MessageReaderLendAfit {
        stream: client,
        buffer: [0; 1024],
        read_amount: 0,
        skip: 0,
    };

    let mut counter = 0;
    while let Some(msg) = reader.next().await.map(|v| serde_json::from_slice::<Message>(v)).transpose()? {
        match msg {
            Message::Ping => {}
            Message::Hello(_, count) => {
                counter += count;
            }
        }
    }
    Ok(counter)
}

// Implementation using lending `async_gen`.
// This does not compile in nightly, because async gen does not allow yielding
// references (to self).
// async fn message_reader_lend_async_gen() -> anyhow::Result<u32> {
//     let listener = TcpListener::bind("127.0.0.1:5555").await?;
//     let (mut client, _addr) = listener.accept().await?;
//
//     // Note: the async gen doesn't hold any borrows across a yield point
//     let iter = async gen {
//         let mut buffer = [0; 1024];
//         let mut read_amount = 0;
//         let x = 0;
//         loop {
//             let read = client.read(&mut buffer[read_amount..]).await.unwrap();
//             if read == 0 {
//                 return;
//             }
//             read_amount += read;
//             if let Some(newline_index) = buffer.iter().position(|& c | c == b'\n') {
//                 let line = &buffer[..newline_index];
//                 yield line;
//                 buffer.copy_within(newline_index + 1.., 0);
//                 read_amount -= newline_index + 1;
//             }
//         }
//     };
//     let mut iter = pin!(iter);
//
//     let mut counter = 0;
//     while let Some(msg) = poll_fn(|cx| iter.as_mut().poll_next(cx)).await {
//         let msg: Message = serde_json::from_slice(msg)?;
//         match msg {
//             Message::Ping => {}
//             Message::Hello(_, count) => {
//                 counter += count;
//             }
//         }
//     }
//     Ok(counter)
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mode = args[1].as_str();
    loop {
        let ret = match mode {
            "afit" => {
                println!("afit");
                message_reader_afit().await
            }
            "lend-afit" => {
                println!("lend afit");
                message_reader_lend_afit().await
            }
            "async-gen" => {
                println!("async gen");
                message_reader_async_gen().await
            }
            "poll" => {
                println!("poll");
                message_reader_poll().await
            }
            _ => {
                break;
            }
        }?;
        println!("{ret}");
    }
    Ok(())
}
