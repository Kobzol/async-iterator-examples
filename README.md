# Async iterator examples
This repository contains example(s) of an asynchronous iterator implemented using various specific approaches:
- Using the `poll_next` interface
- Using `async gen`
- Using the `async fn next` (AFIT) interface
- Using `async gen` without borrows across `yield` points (the proposed lowering-to-AFIT `async gen` blocks)
- Using lending AFIT (if it makes sense)
- Using `async gen` with lending (if it makes sense)

> Lending means yielding references that point into the iterator itself.

The goal is to compare the ergonomics (potentially also efficiency) properties of these approaches, and show how they
can be used to write async iterators that might occur "in the wild".

It's very much possible that the implementations are not as elegant as possible, possible improvements are welcome :)

Currently implemented examples:
- [JSON message parser](json-line-parser)
- [GitHub API pagination](github-api-pagination)

## Resources
These interfaces have been described in detail in several blog posts by [WithoutBoats](https://without.boats/blog/poll-next/)
and [Yoshua Wuyts](https://blog.yoshuawuyts.com/async-iterator-trait/).
