# Dailybible-rs – a Telegram bot for daily Bible reading–cover to cover in one year.

Dailybible is a Telegram bot which will send you a Bible reading notification for every day of the year. If you follow this Bible reading plan 365 days along, you will have made it through the whole Bible.

The source code is written in Rust. Feel free to download and run the bot by yourself. If you don't want that, you can find a bot instance on Telegram: [@biblereadingscheduler_bot](https://t.me/biblereadingscheduler_bot).

## Environment Variables

In order for the bot to run, you'll need to set a few environment variables:

 - `TELOXIDE_TOKEN`: The token which you received from Telegram "Bot father"
 - `RUST_LOG`: The log level which you would like to enable (`error`, `warning`, `info` are possible)
 - `TELOXIDE_USERSTATEFILE`: The file path of the file where the user states will be saved

## Run with docker compose (recommended)

The easiest way to run the bot is via the official Docker image which is generated automatically from the master branch. 
To do that, create a file `docker-compose.yml` and an empty folder `userdata`. Fill the `docker-compose.yml` as following:

```yml
version: "3.8"

services:
  dailybible-rs:
    image: archchem/dailybible-rs:latest
    restart: always
    environment:
      TELOXIDE_TOKEN: ${TELOXIDE_TOKEN} # Replace this with your Telegram bot token, or use an .env file
    volumes:
      - ./userdata:/app/userdata
```
Now build and start the container via `docker compose up -d` (no root required).

## Compile and run it in Rust

```sh
cargo run --release
```
