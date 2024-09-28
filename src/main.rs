use std::{ops::Deref, rc, str::FromStr, sync::{Arc,Mutex}, thread, time};

use chrono::NaiveTime;
use localize::msg_biblereading_not_found;
use serde::{ Serialize, Deserialize };
use teloxide::{ prelude::*, utils::command::BotCommands, RequestError, types::ParseMode::* };

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
    #[command(description="Setup a daily timer for a given time (hh:mm)", parse_with="split")]
    SetTimer { timer_string: String },
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
    pub timer: Option<chrono::NaiveTime>,
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

    pub fn user_state_exists(&self, chat_id: ChatId) -> bool {
        for u in self.user_states.clone().lock().unwrap().iter() {
            if u.chat_id == chat_id {
                return true;
            }
        }
        false
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
                chat_id,
                language: Language::English,
                timer: None,
        };
        
        let user_state_reference = self.user_states.lock().unwrap();
        for u in user_state_reference.iter() {
            if u.chat_id == chat_id {
                return u.clone();
            }
        }
        default_user_state
    }

    /// This updates a UserState internally and overrides an existing one if the ChatId does already exist
    /// # Params
    /// - `user_state`: The UserState which should be updated.
    /// # Returns
    /// A bool, `true` if the given ChatId had already a UserStage which have been updated.
    /// `false` if a UserState with the given ChatId has been saved for the first time.
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

    
    let bot_arc = Arc::new(bot.clone());
    let user_state_wrapper_arc = Arc::new(user_state_wrapper);

    let bot_arc_thread = bot_arc.clone();
    let user_state_wrapper_arc_thread = user_state_wrapper_arc.clone();
    thread::spawn(move || run_timer_thread_loop(&bot_arc_thread.clone(), &user_state_wrapper_arc_thread.clone()));


    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![user_state_wrapper_arc.clone()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}   

async fn answer(bot: Bot, msg: Message, cmd: Command, user_state_wrapper: Arc<UserStateWrapper>) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::SendDailyReminder => send_daily_reminder(bot, msg.chat.id, user_state_wrapper).await?,
        Command::Start => bot.send_message(msg.chat.id, "This bot helps you to read your Bible daily. Type /help for more information").await?,
        Command::SetTimer { timer_string } => bot_set_timer(bot, msg, user_state_wrapper, timer_string).await?,
        Command::UserInformation => send_user_information(bot, msg, user_state_wrapper).await?,
        Command::SetLang { lang_string } => set_language(bot, msg, user_state_wrapper, lang_string).await?,
    };  
    Ok(())
}

async fn send_daily_reminder(bot: Bot, chat_id: ChatId, user_state_wrapper: Arc<UserStateWrapper>) -> Result<Message, RequestError> {
    let userstate = user_state_wrapper.find_userstate(chat_id);

    match biblereading::get_todays_biblereading() {
        Ok(todays_biblereading) => {
            let _ = bot.send_message(
                chat_id,
                msg_biblereading(&userstate.language, todays_biblereading)
            )
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await;
        },
        Err(error) => {     
            log::error!("{}", error.to_string());

            let _ = bot.send_message(
                chat_id,
                msg_biblereading_not_found(&userstate.language)
            ).await;
        }
    };

    let question_strings = msg_poll_text(&userstate.language);
    bot.send_poll(
        chat_id, 
        question_strings.first().unwrap(), 
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

async fn bot_set_timer(bot: Bot, msg: Message, user_state_wrapper: UserStateWrapper, timer_string: String) -> Result<Message, RequestError> {
    let mut user_state = user_state_wrapper.find_userstate(msg.chat.id);

    match chrono::NaiveTime::parse_from_str(&timer_string, "%H:%M") {
        Ok(time) => { 
            user_state.timer = Some(time);
            user_state_wrapper.update_userstate(user_state.clone());
            bot.send_message(msg.chat.id, msg_timer_updated(&user_state.language, &time)).await
        }
        Err(_) => {
            bot.send_message(msg.chat.id, msg_error_timer_update(&user_state.language)).await
        }
    }
}

async fn send_user_information(bot: Bot, msg: Message, user_state_wrapper: UserStateWrapper) -> Result<Message, RequestError> {
    if user_state_wrapper.user_state_exists(msg.chat.id) {
        bot.send_message(
                msg.chat.id, 
                format!("The following data about you is saved on the server: \n\
                \n\
                ```\
                {}\
                ```\
                ", serde_json::to_string_pretty(&user_state_wrapper.find_userstate(msg.chat.id)).unwrap()
            )
        )
        .parse_mode(MarkdownV2).await
    } else {
        bot.send_message(msg.chat.id, "There is currently no data saved on the server concerning you.").await
    }
}

fn run_timer_thread_loop(bot_arc: &Arc<Bot>, user_state_wrapper_arc: &Arc<UserStateWrapper>) {
    let mut last_run: Option<NaiveTime> = None;

    loop {
        let now = chrono::offset::Local::now().naive_local().time();

        if last_run.is_none() || last_run.unwrap() != now {
            let unlocked_user_state_wrapper = user_state_wrapper_arc.clone();
            for u in unlocked_user_state_wrapper.user_states.clone().lock().unwrap().iter() {
                if u.timer.is_some() && u.timer.unwrap() == now {
                    send_daily_reminder(bot_arc.as_ref().clone(), u.chat_id, unlocked_user_state_wrapper.as_ref().clone());
                }
            }
        }
        last_run = Some(now);
        thread::sleep(time::Duration::from_secs(30));
    }
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
            language: Language::German,
            timer: None
        };
        user_state_wrapper.update_userstate(user_state);
        let userstate = user_state_wrapper.find_userstate(ChatId(654321));
        assert_eq!(userstate.language, Language::German);
    }
}