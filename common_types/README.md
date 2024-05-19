# common_types

This contains the types the server has to send to the client. 
They are factored out so one can build the client without building the server (or the server without generating the typescript bindings)

# bindgen

If you need the bindings for the server you can generate them with
```bash
cargo run --features bindgen > schema.json
```
