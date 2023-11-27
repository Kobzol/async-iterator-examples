# JSON message parser
This crate implements an async iterator that reads lines from a `tokio` TCP socket and parses each line as a JSON message
using `serde_json`.

## Run server
1) `poll_next`
    ```
    $ cargo run --release --bin server -- poll
    ```
2) Generator
    ```
    $ cargo run --release --bin server -- gen
    ```
3) `async fn next`
    ```
    $ cargo run --release --bin server -- async
    ```

## Run client (benchmark)
```bash
$ cargo run --release --bin client
```
