#![feature(gen_blocks)]
#![feature(async_iterator)]

use std::collections::VecDeque;
use std::future::{Future, poll_fn};
use std::pin::{pin, Pin};
use octocrab::models::Repository;
use octocrab::Octocrab;
use shared::{AfitAsyncIter, PollNextAsyncIter};
use std::async_iter::AsyncIterator;
use std::task::{Context, Poll};
use pin_project_lite::pin_project;

// AFIT version
struct RepoIterAfit<'a> {
    client: &'a Octocrab,
    page: Option<u32>,
    repos: VecDeque<Repository>
}

/// The ergonomics here are not great, as the splitting of batches into individual items
/// requires me to manually store the state in `RepoIterAfit`. I also have to copy the data
/// from a `Vec` into `VecDeque` to enable efficient iteration in original order.
impl<'a> AfitAsyncIter for RepoIterAfit<'a> {
    type Item = Repository;

    async fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.repos.is_empty() {
                if let Some(page) = self.page {
                    let repos = self.client.orgs("rust-lang").list_repos()
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
                    self.repos = repos.into_iter().collect();
                    continue;
                } else {
                    return None;
                }
            }
            return Some(self.repos.pop_front().unwrap());
        }
    }
}

async fn iterate_repos_afit(client: &Octocrab) -> u32 {
    let mut count = 0;
    let mut iterator = RepoIterAfit {
        client,
        page: Some(0),
        repos: Default::default()
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
            let mut repos = client.orgs("rust-lang").list_repos()
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
pin_project! {
    struct RepoIterPoll<'a> {
        client: &'a Octocrab,
        #[pin]
        future: Option<Pin<Box<dyn Future<Output=octocrab::Result<octocrab::Page<Repository>>>>>>,
        page: Option<u32>,
        repos: VecDeque<Repository>,
    }
}

impl<'a> PollNextAsyncIter for RepoIterPoll<'a> {
    type Item = Repository;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut();
        // match &this.future {
        //     Some(fut) => {
        //     }
        //     None => {
        //
        //     }
        // }

        todo!();
        Poll::Pending
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = octocrab::OctocrabBuilder::default()
        .build()?;
    // let count = iterate_repos_afit(&client).await;
    let count = iterate_repos_async_gen(&client).await;

    println!("{count}");

    Ok(())
}
