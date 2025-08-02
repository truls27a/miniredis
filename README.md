# MiniRedis

MiniRedis is a mini version of Redis, a key value store. It features two binaries, a server and a client. The server binary starts up a server that accepts GET, SET, and DEL operations. The client binary starts up a client that reads operation commands from the user through the terminal, sends it to the server, and prints out the result.

## Demo

## Installation

**Prerequisites:**

- Rust toolchain (install from [rustup.rs](https://rustup.rs/))

**Build and run:**

1. Clone the repository:

```bash
git clone <your-repository-url>
cd miniredis
```

2. Build the project:

```bash
cargo build --release
```

3. Run the server:

```bash
cargo run --bin miniredis-server
```

4. In another terminal, run the client:

```bash
cargo run --bin miniredis-client
```

**Alternative - Install from source:**

```bash
cargo install --path .
```

This will install the `miniredis-server` and `miniredis-client` binaries to your Cargo bin directory.

## Usage

Once you have both the server and client running, you can use the following commands in the client terminal:

**SET** - Store a key-value pair:

```
SET mykey myvalue
```

Returns: `OK`

**GET** - Retrieve a value by key:

```
GET mykey
```

Returns: `myvalue` (or `nil` if key doesn't exist)

**DEL** - Delete a key:

```
DEL mykey
```

Returns: `OK`

**Example session:**

```
SET username john
OK
SET age 25
OK
GET username
john
GET age
25
GET nonexistent
nil
DEL username
OK
GET username
nil
```

Commands are case-insensitive, so `get`, `GET`, `set`, `SET`, etc. all work the same.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.
