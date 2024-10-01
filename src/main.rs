use std::{ops::Deref, sync::Arc, time, env};

use chrono::{NaiveTime, Timelike};
use localize::msg_biblereading_not_found;
use teloxide::{ prelude::*, types::ParseMode::*, utils::command::BotCommands, RequestError };
use tokio::signal;

mod biblereading;
mod userstate;
mod localize;
use crate::localize::*;
use crate::userstate::*;



/// The default file path for the file where the user states will be saved
const DEFAULT_USER_STATE_FILE_PATH: &str = "userstates.json";

/// The name of the environment variable where the path of the user_state_file_path can be specified
const USER_STATE_ENV: &str = "TELOXIDE_USERSTATEFILE";


/// This enum contains all the commands which are available
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



#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting DailyBible Bot...");

    let user_state_wrapper: UserStateWrapper = UserStateWrapper::new();

    // Check whether we can load the latest user_states from a file
    let user_state_file = env::var(USER_STATE_ENV).unwrap_or(DEFAULT_USER_STATE_FILE_PATH.to_string());
    match user_state_wrapper.load_states_from_file(&user_state_file).await {
        Ok(_) => log::info!("Previous user states successfully loaded."),
        Err(error) => log::warn!("Could not load previous user states: {}", error.to_string()),
    }

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
    tokio::spawn(async move { run_timer_thread_loop(bot_arc_thread.clone(), user_state_wrapper_arc_thread.clone()).await } );

    let user_state_wrapper_arc_thread = user_state_wrapper_arc.clone();
    tokio::spawn(async move { run_save_userstate_loop(user_state_wrapper_arc_thread.clone()).await } );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![user_state_wrapper_arc.clone()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

}   



/// This function handles the answers which the bot can give depending on the command issued by the user.
/// It is automatically called by the dispatcher.
/// 
/// # Arguments
/// - bot: The telegram bot (it can be cloned)
/// - cmd: The Command which has been issued
/// - user_state_wrapper: An Arc of the UserStateWrapper
/// 
/// # Return
/// A ResponseResult (just await this function)
/// 
/// # Note
/// The Arc of the UserStateWrapper should be cloned every time passing it to a function to make sure that always enough references of that live.
async fn answer(bot: Bot, msg: Message, cmd: Command, user_state_wrapper: Arc<UserStateWrapper>) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::SendDailyReminder => send_daily_reminder(bot, msg.chat.id, user_state_wrapper.clone()).await?,
        Command::Start => bot.send_message(msg.chat.id, "This bot helps you to read your Bible daily. Type /help for more information").await?,
        Command::SetTimer { timer_string } => bot_set_timer(bot, msg, user_state_wrapper.clone(), timer_string).await?,
        Command::UserInformation => send_user_information(bot, msg, user_state_wrapper.clone()).await?,
        Command::SetLang { lang_string } => set_language(bot, msg, user_state_wrapper.clone(), lang_string).await?,
    };  
    Ok(())
}


/// This function is used to send the daily reminder to the user
/// 
/// # Arguments
/// - bot: The telegram bot (it can be cloned)
/// - chat_id: the ChatId of the user (where to send the message to)
/// - user_state_wrapper_arc: An Arc of the UserStateWrapper
/// 
/// # Return
/// A ResponseResult (just await this function)
/// 
/// # Note
/// The Arc of the UserStateWrapper should be cloned every time passing it to a function to make sure that always enough references of that live.
async fn send_daily_reminder(bot: Bot, chat_id: ChatId, user_state_wrapper_arc: Arc<UserStateWrapper>) -> Result<Message, RequestError> {
    let userstate = user_state_wrapper_arc.find_userstate(chat_id).await;

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


#[allow(dead_code)]
async fn send_not_implemented(bot: Bot, msg: Message, user_state_wrapper: Arc<UserStateWrapper>) -> Result<Message, RequestError> {
    let language: Language = user_state_wrapper.find_userstate(msg.chat.id).await.language;
    
    log::warn!("User {} called something which has not been implemented yet.", msg.chat.username().unwrap_or("unknown"));
    bot.send_message(msg.chat.id, msg_not_implemented_yet(&language)).await
}


async fn set_language(bot: Bot, msg: Message, user_state_wrapper: Arc<UserStateWrapper>, lang_str: String) -> Result<Message, RequestError> {
    let mut user_state = user_state_wrapper.find_userstate(msg.chat.id).await;
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
    user_state_wrapper.update_userstate(user_state.clone()).await;
    bot.send_message(msg.chat.id, msg_language_set(&user_state.language)).await
}


async fn bot_set_timer(bot: Bot, msg: Message, user_state_wrapper: Arc<UserStateWrapper>, timer_string: String) -> Result<Message, RequestError> {
    let mut user_state = user_state_wrapper.find_userstate(msg.chat.id).await;

    match chrono::NaiveTime::parse_from_str(&timer_string, "%H:%M") {
        Ok(time) => { 
            user_state.timer = Some(time);
            user_state_wrapper.update_userstate(user_state.clone()).await;
            bot.send_message(msg.chat.id, msg_timer_updated(&user_state.language, &time)).await
        }
        Err(_) => {
            bot.send_message(msg.chat.id, msg_error_timer_update(&user_state.language)).await
        }
    }
}


async fn send_user_information(bot: Bot, msg: Message, user_state_wrapper: Arc<UserStateWrapper>) -> Result<Message, RequestError> {
    if user_state_wrapper.user_state_exists(msg.chat.id).await {
        bot.send_message(
                msg.chat.id, 
                format!("The following data about you is saved on the server: \n\
                \n\
                ```\
                {}\
                ```\
                ", serde_json::to_string_pretty(&user_state_wrapper.find_userstate(msg.chat.id).await).unwrap()
            )
        )
        .parse_mode(MarkdownV2).await
    } else {
        bot.send_message(msg.chat.id, "There is currently no data saved on the server concerning you.").await
    }
}


async fn run_timer_thread_loop(bot_arc: Arc<Bot>, user_state_wrapper_arc: Arc<UserStateWrapper>) {
    let mut last_run: Option<NaiveTime> = None;
    log::info!("Start Timer thread");
    
    let control_c_pressed = tokio::spawn(
        async {
            let _ = signal::ctrl_c().await;
            log::info!("Shutdown the timer");
        }
    );
    log::info!("Start the Loop");
    while !control_c_pressed.is_finished() {
        let now = chrono::offset::Local::now().naive_local().time();
        log::info!(
            "Start timer for {}", now.to_string()
        );

        // We make sure that the real timer task is only runned once per minute.
        if last_run.is_none() || last_run.unwrap().hour() != now.hour() || last_run.unwrap().minute() != now.minute() {
            let unlocked_user_state_wrapper = user_state_wrapper_arc.clone();
            
            for u in unlocked_user_state_wrapper.user_states.read().await.iter() {
                if u.timer.is_some() && u.timer.unwrap().hour() == now.hour() && u.timer.unwrap().minute() == now.minute() {
                    log::info!("Send Reminder");

                    // We have to clone all the variables which are needed for the `send_daily-reminder`-function because they will be consumed 
                    // by the spawned task.
                    let bot_arc_clone = bot_arc.clone();
                    let user_state_wrapper_arc_clone = user_state_wrapper_arc.clone();
                    let u_clone = u.clone();
                    tokio::spawn(
                        async move { 
                            match send_daily_reminder(bot_arc_clone.deref().clone(), u_clone.chat_id, user_state_wrapper_arc_clone).await {
                                Ok(_) => log::info!("Sending completed"),
                                Err(_) => log::info!("There was an error"),
                            } 
                        } 
                    );
                }   
            }
        }
        last_run = Some(now);
        tokio::time::sleep(time::Duration::from_secs(5)).await;
    }
}

async fn run_save_userstate_loop(user_state_wrapper_arc: Arc<UserStateWrapper>) {
    let control_c_pressed = tokio::spawn(
        async {
            let _ = signal::ctrl_c().await;
            log::info!("Shutdown the user state saver timer");
        }
    );

    loop {
        let cloned_user_state_wrapper_arc = user_state_wrapper_arc.clone();
        tokio::spawn(
            async move {
                handle_save_current_userstates(cloned_user_state_wrapper_arc).await;
            }
        );

        tokio::time::sleep(time::Duration::from_secs(30)).await;
        if control_c_pressed.is_finished() {
            handle_save_current_userstates(user_state_wrapper_arc.clone()).await;               
            break;
        }
    }
}

async fn handle_save_current_userstates(user_state_wrapper_arc: Arc<UserStateWrapper>) {
    let user_state_file = env::var(USER_STATE_ENV).unwrap_or(DEFAULT_USER_STATE_FILE_PATH.to_string());

    match user_state_wrapper_arc.write_states_to_file(&user_state_file).await {
        Ok(_) => log::info!("Saved user states to {}", user_state_file),
        Err(error) => log::warn!("Could not save user state file: {}", error.to_string())
    }
}