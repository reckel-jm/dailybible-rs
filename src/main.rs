use teloxide::{prelude::*, utils::command::BotCommands, RequestError};

mod biblereading;


#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description="Show the start message")]
    Start,
    #[command(description="Send the daily reminder with the verses once")]
    SendDailyReminder,
    #[command(description="Setup a Timer")]
    SetupTimer,
    #[command(description="Show help message")]
    Help,
    #[command(description="Send user/chat information (for debugging purposes)")]
    UserInformation,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting DailyBible Bot...");
    let bot = Bot::from_env();

    Command::repl(bot, answer).await;
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::SendDailyReminder => send_daily_reminder(bot, msg).await?,
        Command::Start => bot.send_message(msg.chat.id, "This bot helps you to read your Bible daily. Type /help for more information").await?,
        Command::SetupTimer => send_not_implemented(bot, msg).await?,
        Command::UserInformation => bot.send_message(msg.chat.id, msg.chat.id.to_string()).await?
    };  
    Ok(())
}

async fn send_daily_reminder(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    match biblereading::get_todays_biblereading() {
        Ok(todays_biblereading) => {
            bot.send_message(
                msg.chat.id, 
                format!(
                    "*Today's Bible reading*: \n\nAT: {}\nNT: {}", 
                    todays_biblereading.old_testament_reading,
                    todays_biblereading.new_testament_reading
                )
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await
        },
        Err(error) => {     
            log::error!("{}", error.to_string());

            bot.send_message(msg.chat.id, "This is a reminder to read your bible!").await
        }
    }
}

async fn send_not_implemented(bot: Bot, msg: Message) -> Result<Message, RequestError> {
    log::warn!("User {} called something which has not been implemented yet.", msg.chat.username().unwrap_or("unknown"));
    bot.send_message(msg.chat.id, "Not implemented yet").await
}