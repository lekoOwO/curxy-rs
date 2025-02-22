# curxy-rs

#### _cursor_ + _proxy_ = **curxy**

A proxy worker for using ollama in cursor.

[繁體中文](README.zh-tw.md)

## What is this?

This is a proxy worker for using ollama in cursor. It is a simple server that forwards requests to the ollama server and returns the response.

## Why do you need this?

When we use llm prediction in cursor editor, the editor sends the data to the official cursor server, and the server sends the data to the ollama server.
Therefore, even if the endpoint is set to localhost in the cursor editor configuration, the cursor server cannot communicate with the local server.
So, we need a proxy worker that can forward the data to the ollama server.

## Requirements

- Rust 1.75+ (install via `rustup`)
- Ollama server

## Installation

```bash
cargo install --git https://github.com/lekoOwO/curxy-rs
```

Or build from source:

```bash
git clone https://github.com/lekoOwO/curxy-rs
cd curxy-rs
cargo build --release
```

## How to use

1. Launch the ollama server

2. Launch curxy

   ```bash
   curxy
   ```

   If you want to limit access to the ollama server, you can set the `OPENAI_API_KEY` environment variable:

   ```bash
   OPENAI_API_KEY=your_openai_api_key curxy
   ```

   You will see output like this:

   ```
   Starting server on 127.0.0.1:62192
   Server running at: https://remaining-chen-composition-dressed.trycloudflare.com
   ```

3. Enter the URL provided by curxy (with /v1 appended) into the "Override OpenAI Base URL" section of the cursor editor configuration.

4. Add model names you want to use to the "Model Names" section of the cursor editor configuration.

5. (Optional): If you want to restrict access to this Proxy Server for security reasons, you can set the OPENAI_API_KEY environment variable, which will enable access restrictions based on the key.

6. **Enjoy!**

You can also see help message by running `curxy --help`.

## Command Line Options

```
Options:
  -e, --endpoint <URL>          Ollama server endpoint [default: http://localhost:11434]
  -o, --openai-endpoint <URL>   OpenAI server endpoint [default: https://api.openai.com]
  -p, --port <PORT>             Server port [default: random]
      --hostname <HOSTNAME>     Server hostname [default: 127.0.0.1]
      --cloudflared <BOOL>      Use cloudflared tunnel [default: true]
  -h, --help                    Show help message
```

## Features

- 🚀 Written in Rust for excellent performance
- 🔒 Optional API key authentication
- 🌐 Automatic cloudflared tunnel support
- 🔄 Smart request routing
- 🛠 Complete error handling

## Development

```bash
# Run tests
cargo test

# Run development version
cargo run

# Build release version
cargo build --release
```

## License

MIT

## Acknowledgments

This project is inspired by [curxy](https://github.com/ryoppippi/curxy). 