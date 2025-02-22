# curxy-rs

#### _cursor_ + _proxy_ = **curxy**

一個讓你在 Cursor 編輯器中使用 Ollama 的代理伺服器。

## 這是什麼？

這是一個讓你在 Cursor 編輯器中使用 Ollama 的代理伺服器。它是一個簡單的伺服器，可以將請求轉發到 Ollama 伺服器並返回回應。

## 為什麼需要這個？

當我們在 Cursor 編輯器中使用 LLM 預測時，編輯器會將數據發送到官方的 Cursor 伺服器，然後伺服器將數據發送到 Ollama 伺服器。
因此，即使在 Cursor 編輯器配置中將端點設置為 localhost，Cursor 伺服器也無法與本地伺服器通信。
所以，我們需要一個代理伺服器來轉發數據到 Ollama 伺服器。

## 需求

- Rust 1.75+ (透過 `rustup` 安裝)
- Ollama 伺服器
- Cloudflared (可選，用於建立公開訪問的通道)

## 安裝

```bash
cargo install --git https://github.com/lekoOwO/curxy-rs
```

或從原始碼建置：

```bash
git clone https://github.com/lekoOwO/curxy-rs
cd curxy-rs
cargo build --release
```

## 如何使用

1. 啟動 Ollama 伺服器

2. 啟動 curxy

   ```bash
   curxy
   ```

   如果你想要限制對 Ollama 伺服器的訪問，可以設置 `OPENAI_API_KEY` 環境變數：

   ```bash
   OPENAI_API_KEY=your_openai_api_key curxy
   ```

   如果你想要使用 cloudflared 建立公開訪問的通道：

   ```bash
   curxy --cloudflared-path /path/to/cloudflared
   ```

   你會看到類似這樣的輸出：

   ```
   Starting server on 127.0.0.1:62192
   https://remaining-chen-composition-dressed.trycloudflare.com
   ```

3. 將 curxy 提供的 URL (加上 /v1) 輸入到 Cursor 編輯器配置的 "Override OpenAI Base URL" 部分。

4. 將你想要使用的模型名稱添加到 Cursor 編輯器配置的 "Model Names" 部分。

5. (可選)：如果你想要出於安全考慮限制對此代理伺服器的訪問，可以設置 OPENAI_API_KEY 環境變數，這將啟用基於金鑰的訪問限制。

6. **享受使用！**

你也可以透過運行 `curxy --help` 來查看幫助訊息。

## 命令列選項

```
選項：
  -e, --endpoint <URL>          Ollama 伺服器端點 [預設：http://localhost:11434]
  -o, --openai-endpoint <URL>   OpenAI 伺服器端點 [預設：https://api.openai.com]
  -p, --port <PORT>            伺服器端口 [預設：隨機]
      --hostname <HOSTNAME>     伺服器主機名 [預設：127.0.0.1]
      --cloudflared-path <PATH> Cloudflared 執行檔路徑
  -h, --help                    顯示幫助訊息
```

## 功能

- 🚀 使用 Rust 編寫，性能卓越
- 🔒 可選的 API 金鑰認證
- 🌐 支援使用 cloudflared 建立通道
- 🔄 智能請求路由
- 🛠 完整的錯誤處理

## 開發

```bash
# 運行測試
cargo test

# 運行開發版本
cargo run

# 建置發布版本
cargo build --release
```

## 授權

MIT

## 致謝

本專案受 [curxy](https://github.com/ryoppippi/curxy) 啟發。
