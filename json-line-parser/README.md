# JSON message parser
This crate implements an async iterator that reads lines from a `tokio` TCP socket and parses each line as a JSON message
using `serde_json`.

## Run server
1) `poll_next`
    ```
    $ cargo run --release --bin server -- poll
    ```
2) `async gen`
    ```
    $ cargo run --release --bin server -- async-gen
    ```
3) AFIT (`async fn next`)
    ```
    $ cargo run --release --bin server -- afit
    ```
4) AFIT (`async fn next`)
    ```
    $ cargo run --release --bin server -- lend-afit
    ```

## Run client (benchmark)
```bash
$ cargo run --release --bin client
```
