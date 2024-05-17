# Backend
This is a server for the chatroom, written in `Rust` using `Axum`

## Running
You can run it directly using
```bash
cargo run
```
If you want to change the title of the chat, or the port the server is listening on, use `cargo run --help` for info.

### Running into a container
The project is ready to be containerized. Just run
```bash
docker build . -t backend
docker run -p4000:3000 -it backend
```
You can customize both the port and the title:
```bash
docker run -p${PORT}:3000 -it backend --title ${TITLE}
```
