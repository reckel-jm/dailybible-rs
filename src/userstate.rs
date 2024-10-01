use teloxide::types::ChatId;
use std::{error::Error, ops::Deref, path::Path, sync::Arc};
use tokio::sync::RwLock;

use crate::localize::*;
use serde::{ Serialize, Deserialize };


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserState {
    pub chat_id: ChatId,
    pub language: Language,
    pub timer: Option<chrono::NaiveTime>,
}


pub type UserStateVector = Arc<RwLock<Vec<UserState>>>;


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
            user_states: Arc::new(RwLock::new(Vec::new())),
        }
    }

    
    pub async fn user_state_exists(&self, chat_id: ChatId) -> bool {
        for u in self.user_states.read().await.iter() {
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
        
        for u in self.user_states.read().await.iter() {
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
        for u in self.user_states.write().await.iter_mut() {
            if u.chat_id == user_state.chat_id {
                *u = user_state.clone();
                
                // End the function if a UserState already exists which has been updated
                return true;
            }
        };

        // If there has been no user_state saved, the function will get here and add a new UserState element
        self.user_states.write().await.push(user_state);
        
        false
    }

    
    pub async fn write_states_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        match serde_json::to_string_pretty(self.user_states.read().await.deref()) {
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

    pub async fn load_states_from_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        match tokio::fs::read_to_string(file_path).await {
            Ok(file_string) => {
                match serde_json::from_str(&file_string) {
                    Ok(object) => {
                        let mut userstates: Vec<UserState> = object;
                        let mut userstate_lock = self.user_states.write().await;
                        userstate_lock.clear();
                        userstate_lock.append(&mut userstates);
                        Ok(())
                    },
                    Err(error) => Err(Box::new(error))
                }
            },
            Err(error) => Err(Box::new(error))
        }
        
    }

}

#[cfg(test)]
mod tests {
    const TEST_FILE_PATH: &str = "testfile.json";

    use std::fs;

    use super::*;

    struct TestfileHandling;

    impl Drop for TestfileHandling {
        fn drop(&mut self) {
            if fs::remove_file(TEST_FILE_PATH).is_err() {
                println!("Warning: Test File couldn't be removed because it most likely did not exist.");
            }
        }
    }

    
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
        // This ensures that the test file will be deleted after this test.
        let _tfh = TestfileHandling;
        
        let user_state_wrapper = UserStateWrapper::new();
        let userstate = user_state_wrapper.find_userstate(ChatId(123456));
        assert_eq!(userstate.await.language, Language::English);

        let user_state = UserState {
            chat_id: ChatId(654321),
            language: Language::German,
            timer: None
        };
        user_state_wrapper.update_userstate(user_state).await;

        assert!(user_state_wrapper.write_states_to_file(&TEST_FILE_PATH).await.is_ok());
        assert!(Path::new(TEST_FILE_PATH).exists());
    }

    #[tokio::test]
    async fn test_load_userstate() {
        let user_state_wrapper = UserStateWrapper::new();
        assert!(user_state_wrapper.load_states_from_file("testdata/test_userstate_loading.json").await.is_ok());

        assert_eq!(user_state_wrapper.user_states.read().await.len(), 2);
        assert_eq!(user_state_wrapper.find_userstate(ChatId(654321)).await.language, Language::German);
    }
}