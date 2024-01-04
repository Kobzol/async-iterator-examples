use std::pin::Pin;
use std::task::{Context, Poll};

/// `async fn next` using AFIT
pub trait AfitAsyncIter {
    type Item;

    async fn next(&mut self) -> Option<Self::Item>;
}

/// `poll_next` with explicit `Pin`
pub trait PollNextAsyncIter {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}

/// Lending version of AFIT `async fn next`
pub trait LendAfitAsyncIter {
    type Item<'a>
    where
        Self: 'a;

    async fn next(&mut self) -> Option<Self::Item<'_>>;
}
