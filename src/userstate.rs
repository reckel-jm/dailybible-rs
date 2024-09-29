use teloxide::types::ChatId;
use std::{error::Error, ops::Deref, path::Path, sync::Arc};
use tokio::{sync::Mutex};

use crate::localize::*;

use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserState {
    pub chat_id: ChatId,
    pub language: Language,
    pub timer: Option<chrono::NaiveTime>,
}

pub type UserStateVector = Arc<Mutex<Vec<UserState>>>;

/// The UserStateWrapper handles the managing of user state and can be savely used by the commands to read
/// or write user states.
/// Define any needed user state in the UserState struct.
#[derive(Clone)]
pub struct UserStateWrapper {
    pub user_states: UserStateVector,
}

impl UserStateWrapper {
    pub fn new() -> Self {
        UserStateWrapper {
            user_states: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn user_state_exists(&self, chat_id: ChatId) -> bool {
        for u in self.user_states.clone().lock().await.iter() {
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
    pub async fn find_userstate(&self, chat_id: ChatId) -> UserState {
        let default_user_state = UserState {
                chat_id,
                language: Language::English,
                timer: None,
        };
        
        let user_state_reference = self.user_states.lock().await;
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
    pub async fn update_userstate(&self, user_state: UserState) -> bool {
        let mut user_state_reference = self.user_states.lock().await;

        for u in user_state_reference.iter_mut() {
            if u.chat_id == user_state.chat_id {
                *u = user_state.clone();
                return true;
            }
        };

        user_state_reference.push(user_state);
        false
    }

    pub async fn write_states_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        match serde_json::to_string_pretty(&self.user_states.lock().await.deref()) {
            Ok(json_string) => { 
                match tokio::fs::write(
                    &Path::new(file_path), 
                    json_string)
                    .await {
                        Ok(_) => Ok(()),
                        Err(error) => Err(Box::new(error)),
                }
            },
            Err(error) => Err(Box::new(error)),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_userstate_wrapper() {
        let user_state_wrapper = UserStateWrapper::new();
        let userstate = user_state_wrapper.find_userstate(ChatId(123456));
        assert_eq!(userstate.await.language, Language::English);

        let user_state = UserState {
            chat_id: ChatId(654321),
            language: Language::German,
            timer: None
        };
        user_state_wrapper.update_userstate(user_state).await;
        let userstate = user_state_wrapper.find_userstate(ChatId(654321));
        assert_eq!(userstate.await.language, Language::German);
    }

    #[tokio::test]
    async fn test_save_userstate() {
        let user_state_wrapper = UserStateWrapper::new();
        let userstate = user_state_wrapper.find_userstate(ChatId(123456));
        assert_eq!(userstate.await.language, Language::English);

        let user_state = UserState {
            chat_id: ChatId(654321),
            language: Language::German,
            timer: None
        };
        user_state_wrapper.update_userstate(user_state).await;

        let file_name: &str = "testfile.csv";
        assert!(user_state_wrapper.write_states_to_file(&file_name).await.is_ok());
        assert!(Path::new(file_name).exists());
    }
}