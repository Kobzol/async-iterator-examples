#![feature(gen_blocks)]
#![feature(async_iterator)]

use pin_project_lite::pin_project;
use shared::{AfitAsyncIter, PollNextAsyncIter};
use std::async_iter::AsyncIterator;
use std::future::{poll_fn, Future};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use std::time::Duration;

// AFIT version
struct ForwardIterAfit<It> {
    iter: It,
}

async fn transform(item: String) -> String {
    tokio::time::sleep(Duration::from_millis(1)).await;
    item
}

impl<It: AfitAsyncIter<Item = String>> AfitAsyncIter for ForwardIterAfit<It> {
    type Item = String;

    async fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next().await?;
        let item = transform(item).await;
        Some(item)
    }
}

struct StringStream {
    count: usize,
}

impl AfitAsyncIter for StringStream {
    type Item = String;

    async fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            None
        } else {
            self.count -= 1;
            Some(String::from("Hello world"))
        }
    }
}

async fn forward_afit() {
    let stream = StringStream { count: 5 };
    let mut iter = ForwardIterAfit { iter: stream };
    while let Some(item) = iter.next().await {
        println!("{item}");
    }
}

// async gen
async fn forward_async_gen() {
    let stream = async gen {
        for i in 0..5 {
            yield String::from("Hello world");
        }
    };
    let mut stream = pin!(stream);
    let iter = async gen {
        while let Some(s) = poll_fn(|cx| stream.as_mut().poll_next(cx)).await {
            let s = transform(s).await;
            yield s;
        }
    };
    let mut iter = pin!(iter);
    while let Some(s) = poll_fn(|cx| iter.as_mut().poll_next(cx)).await {
        println!("{s}");
    }
}

// poll
pin_project! {
    struct ForwardIterPoll<It> {
        #[pin]
        inner: It,
        #[pin]
        transform_fut: Option<Pin<Box<dyn Future<Output=String>>>>,
    }
}

impl<It: PollNextAsyncIter<Item = String>> PollNextAsyncIter for ForwardIterPoll<It> {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            if let Some(fut) = this.transform_fut.as_mut().as_pin_mut() {
                return match fut.poll(cx) {
                    Poll::Ready(ret) => {
                        *this.transform_fut = None;
                        Poll::Ready(Some(ret))
                    }
                    Poll::Pending => Poll::Pending,
                };
            }
            match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(item) => match item {
                    Some(item) => {
                        *this.transform_fut = Some(Box::pin(transform(item)));
                        continue;
                    }
                    None => return Poll::Ready(None),
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

struct PollStream {
    count: usize,
}

impl PollNextAsyncIter for PollStream {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.count == 0 {
            Poll::Ready(None)
        } else {
            self.count -= 1;
            Poll::Ready(Some(String::from("Hello world")))
        }
    }
}

async fn forward_poll() {
    let stream = PollStream { count: 5 };
    let iter = ForwardIterPoll {
        inner: stream,
        transform_fut: None,
    };
    let mut iter = pin!(iter);
    while let Some(s) = poll_fn(|cx| iter.as_mut().poll_next(cx)).await {
        println!("{s}");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // forward_afit().await;
    // forward_async_gen().await;
    forward_poll().await;

    Ok(())
}
