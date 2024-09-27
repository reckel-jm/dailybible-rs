use std::sync::{Arc,Mutex};

use localize::msg_biblereading_not_found;
use serde::{ Serialize, Deserialize };
use teloxide::{ prelude::*, utils::command::BotCommands, RequestError };

mod biblereading;

mod localize;
use crate::localize::*;

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
    #[command(description="Setup the language", parse_with="split")]
    SetLang { lang_string: String }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserState {
    pub chat_id: ChatId,
    pub language: localize::Language,
}

type UserStateVector = Arc<Mutex<Vec<UserState>>>;

/// The UserStateWrapper handles the managing of user state and can be savely used by the commands to read
/// or write user states.
/// Define any needed userstate in the UserState struct.
#[derive(Clone)]
struct UserStateWrapper {
    user_states: UserStateVector,
}

impl UserStateWrapper {
    pub fn new() -> Self {
        UserStateWrapper {
            user_states: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a `UserState` by a given `ChatId`. This function is save, that means, if no UserSate for a
    /// given ChatId is saved, the default UserState will be returned.
    /// 
    /// # Params
    /// - `chat_id` A `ChatId`
    /// # Returns
    /// The saved `UserState` if one is saved, or the default `UserState` if no one is found.
    pub fn find_userstate(&self, chat_id: ChatId) -> UserState {
        let default_user_state = UserState {
                chat_id: chat_id,
                language: Language::English,
        };
        
        let user_state_reference = self.user_states.lock().unwrap();
        for u in user_state_reference.iter() {
            if u.chat_id == chat_id {
                return u.clone();
            }
        }
        default_user_state
    }

    pub fn update_userstate(&self, user_state: UserState) -> bool {
        let mut user_state_reference = self.user_states.lock().unwrap();

        for u in user_state_reference.iter_mut() {
            if u.chat_id == user_state.chat_id {
                *u = user_state.clone();
                return true;
            }
        };

        user_state_reference.push(user_state);
        false
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting DailyBible Bot...");

    let user_state_wrapper: UserStateWrapper = UserStateWrapper::new();

    let bot: Bot = Bot::from_env();

    let bot_commands = Command::bot_commands();
    if bot.set_my_commands(bot_commands).await.is_err() {
        log::warn!("Could not set up the commands.");
    }

    let handler = Update::filter_message()
            .branch(dptree::entry()
                .filter_command::<Command>()
                .endpoint(answer)
            );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![user_state_wrapper])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}   

async fn answer(bot: Bot, msg: Message, cmd: Command, user_state_wrapper: UserStateWrapper) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::SendDailyReminder => send_daily_reminder(bot, msg, user_state_wrapper).await?,
        Command::Start => bot.send_message(msg.chat.id, "This bot helps you to read your Bible daily. Type /help for more information").await?,
        Command::SetupTimer => send_not_implemented(bot, msg, user_state_wrapper).await?,
        Command::UserInformation => bot.send_message(msg.chat.id, msg.chat.id.to_string()).await?,
        Command::SetLang { lang_string } => set_language(bot, msg, user_state_wrapper, lang_string).await?,
    };  
    Ok(())
}

async fn send_daily_reminder(bot: Bot, msg: Message, user_state_wrapper: UserStateWrapper) -> Result<Message, RequestError> {
    let userstate = user_state_wrapper.find_userstate(msg.chat.id);

    match biblereading::get_todays_biblereading() {
        Ok(todays_biblereading) => {
            let _ = bot.send_message(
                msg.chat.id,
                msg_biblereading(&userstate.language, todays_biblereading)
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await;
        },
        Err(error) => {     
            log::error!("{}", error.to_string());

            let _ = bot.send_message(
                msg.chat.id,
                msg_biblereading_not_found(&userstate.language)
            ).await;
        }
    };

    let question_strings = msg_poll_text(&userstate.language);
    bot.send_poll(
        msg.chat.id, 
        question_strings.get(0).unwrap(), 
        vec![
            question_strings.get(1).unwrap().clone(), 
            question_strings.get(2).unwrap().clone()
        ],
    )
    .is_anonymous(false)
    .await
}       

async fn send_not_implemented(bot: Bot, msg: Message, user_state_wrapper: UserStateWrapper) -> Result<Message, RequestError> {
    let language: Language = user_state_wrapper.find_userstate(msg.chat.id).language;
    
    log::warn!("User {} called something which has not been implemented yet.", msg.chat.username().unwrap_or("unknown"));
    bot.send_message(msg.chat.id, msg_not_implemented_yet(&language)).await
}

async fn set_language(bot: Bot, msg: Message, user_state_wrapper: UserStateWrapper, lang_str: String) -> Result<Message, RequestError> {
    let mut user_state = user_state_wrapper.find_userstate(msg.chat.id);
    match lang_str.to_lowercase().as_str() {
        "de" => { user_state.language = Language::German; },
        "en" => { user_state.language = Language::English; },
        _ => {
                return bot.send_message(
                    msg.chat.id, 
                    msg_error_enter_language(&user_state.language)
                ).await;
        }
    };
    user_state_wrapper.update_userstate(user_state.clone());
    bot.send_message(msg.chat.id, msg_language_set(&user_state.language)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_userstate_wrapper() {
        let user_state_wrapper = UserStateWrapper::new();
        let userstate = user_state_wrapper.find_userstate(ChatId(123456));
        assert_eq!(userstate.language, Language::English);

        let user_state = UserState {
            chat_id: ChatId(654321),
            language: Language::German
        };
        user_state_wrapper.update_userstate(user_state);
        let userstate = user_state_wrapper.find_userstate(ChatId(654321));
        assert_eq!(userstate.language, Language::German);
    }
}