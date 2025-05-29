# Mini Redis Clone

A minimal Redis clone implemented in Rust for learning purposes. This project demonstrates core concepts of systems programming including:
- TCP networking
- Concurrent request handling
- In-memory data structures
- RESP (Redis Serialization Protocol) implementation

## Prerequisites

1. Install Rust and Cargo:
   - Windows: Visit https://www.rust-lang.org/tools/install and download rustup-init.exe
   - Follow the installation instructions
   - Verify installation with: `rustc --version` and `cargo --version`

## Features

- Basic Redis commands: GET, SET, DEL
- String data type support
- TCP server implementation
- RESP protocol parsing
- Concurrent client handling

## Project Structure

```
src/
├── main.rs          # Entry point, TCP server setup
├── command.rs       # Command parsing and execution
├── db.rs           # In-memory database implementation
└── resp.rs         # RESP protocol implementation
```

## Building and Running

1. Clone the repository
2. Run `cargo build` to compile
3. Run `cargo run` to start the server
4. Connect using `redis-cli` or any Redis client on port 6379

## Testing

Run tests with:
```bash
cargo test
``` 