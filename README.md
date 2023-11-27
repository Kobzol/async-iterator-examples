# Async iterator examples
This repository contains an example (possibly more in the future :) ) of an asynchronous iterator implemented using three
specific approaches:
- Using the `poll_next` interface
- Using an async generator (backed by the `poll_next` interface)
- Using the `async fn next` interface

This is just a quick little project in which I wanted to find out how ergonomic to implement and performant these
approaches.

It's very much possible that the implementations are not as elegant as possible, possible improvements are welcome :)

Currently implemented examples:
- [JSON message parser](json-line-parser)

## Resources
These interfaces have been described in detail in several blog posts by [WithoutBoats](https://without.boats/blog/poll-next/)
and Yoshua Wuyts(https://blog.yoshuawuyts.com/async-iterator-trait/).
