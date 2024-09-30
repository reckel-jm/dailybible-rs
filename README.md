# Dailybible-rs – a Telegram bot for daily Bible reading–cover to cover in one year.

Dailybible is a Telegram bot which will send you a Bible reading notification for every day of the year. If you follow this Bible reading plan 365 days along, you will have made it through the whole Bible.

The source code is written in Rust. Feel free to download and run the bot by yourself.

## Environment Variables

In order for the bot to run, you'll need to set a few environment variables:

 - `TELOXIDE_TOKEN`: The token which you received from Telegram "Bot father"
 - `RUST_LOG`: The log level which you would like to enable (`error`, `warning`, `info` are possible)
 - `TELOXIDE_USERSTATEFILE`: The file path of the file where the user states will be saved

# Compile 

```sh
cargo run --release
```