#![feature(gen_blocks)]
#![feature(async_iterator)]

extern crate octocrab;

use octocrab::models::Repository;
use octocrab::{Octocrab, Page};
use pin_project_lite::pin_project;
use shared::{AfitAsyncIter, PollNextAsyncIter};
use std::async_iter::AsyncIterator;
use std::collections::VecDeque;
use std::future::{poll_fn, Future};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use octocrab::orgs::ListReposBuilder;

// AFIT version
struct RepoIterAfit<'a> {
    client: &'a Octocrab,
    page: Option<u32>,
    repos: std::vec::IntoIter<Repository>,
}

/// The ergonomics here are not great, as the splitting of batches into individual items
/// requires me to manually store the state in `RepoIterAfit`.
impl<'a> AfitAsyncIter for RepoIterAfit<'a> {
    type Item = Repository;

    async fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.repos.next();
            if let Some(next) = next {
                return Some(next);
            }
            if let Some(page) = self.page {
                let repos = self
                    .client
                    .orgs("rust-lang")
                    .list_repos()
                    .per_page(100)
                    .page(page)
                    .send()
                    .await
                    .unwrap();
                if repos.next.is_none() {
                    self.page = None;
                } else {
                    self.page = Some(page + 1);
                }
                self.repos = repos.into_iter();
            } else {
                return None;
            }
        }
    }
}

async fn iterate_repos_afit(client: &Octocrab) -> u32 {
    let mut count = 0;
    let mut iterator = RepoIterAfit {
        client,
        page: Some(0),
        repos: Default::default(),
    };

    while let Some(_repo) = iterator.next().await {
        count += 1;
    }
    count
}

// async-gen version
async fn iterate_repos_async_gen(client: &Octocrab) -> u32 {
    let mut count = 0;
    let mut iter = async gen {
        let mut page = 0u32;
        loop {
            let mut repos = client
                .orgs("rust-lang")
                .list_repos()
                .per_page(100)
                .page(page)
                .send()
                .await
                .unwrap();
            for repo in repos.take_items() {
                yield repo;
            }
            if repos.next.is_none() {
                break;
            } else {
                page += 1;
            }
        }
    };

    let mut iter = pin!(iter);
    while let Some(msg) = poll_fn(|cx| iter.as_mut().poll_next(cx)).await {
        count += 1;
    }
    count
}

// poll version
// Impossible due to octocrab using double lifetime?
// If anyone knows how to write this manually, I'd be glad for help :)

// pin_project! {
//     struct RepoIterPoll<'a> {
//         client: &'a Octocrab,
//         #[pin]
//         future: Option<Box<dyn Future<Output=octocrab::Result<octocrab::Page<Repository>>> + 'a>>,
//         page: Option<u32>,
//         repos: std::vec::IntoIter<Repository>,
//     }
// }

// impl<'a> PollNextAsyncIter for RepoIterPoll<'a> {
//     type Item = Repository;
//
//     fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let mut this = self.project();
//         let client: &'a Octocrab = this.client;
//
//         if let Some(item) = this.repos.next() {
//             return Poll::Ready(Some(item));
//         }
//         let Some(page) = this.page else { return Poll::Ready(None); };
//
//         if this.future.is_none() {
//             let handler = client.orgs("rust-lang");
//             let handler: ListReposBuilder<'a, '_> = handler.list_repos().per_page(100)
//                 .page(*page);
//             let handler: Box<dyn Future<Output=octocrab::Result<Page<Repository>>>> = Box::new(handler.send());
//             this.future.as_mut().set(Some(handler));
//         }
//
//         // this.future.as_pin_mut().unwrap().poll(cx);
//
//         todo!();
//         Poll::Pending
//     }
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = octocrab::OctocrabBuilder::default().build()?;
    let count = iterate_repos_afit(&client).await;
    // let count = iterate_repos_async_gen(&client).await;

    println!("{count}");

    Ok(())
}
