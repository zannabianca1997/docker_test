# Backend
This is a server for the chatroom, written in `Rust` using `Axum`

## Running
You can run it directly using
```bash
cargo run --addr 127.0.0.1:3000 --database postgresql://user:password@addr/db
```
or with 
```bash
DB_CONN_STRING=postgresql://user:password@addr/db cargo run --addr 127.0.0.1:3000
```