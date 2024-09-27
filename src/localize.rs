use serde::{Deserialize, Serialize};

use crate::biblereading::BibleReading;

/// This enum contains the list of all supported languages for the bot
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Language {
    English,
    German
}

pub fn msg_biblereading(lang: &Language, biblereading: BibleReading) -> String {
    match lang {
        Language::English => {
            format!(
                "*📖 This is a reminder to read the Bible today*: \n\nOT: {}\nNT: {}", 
                biblereading.old_testament_reading,
                biblereading.new_testament_reading
            )
        },
        Language::German => {
            format!(
                "*📖 Dies ist eine Erinnerung, heute in der Bibel zu lesen*: \n\nAT: {}\nNT: {}", 
                biblereading.old_testament_reading,
                biblereading.new_testament_reading
            )
        }
    }
}

pub fn msg_biblereading_not_found(lang: &Language) -> String {
    match lang {
        Language::English => format!("This is a reminder to read your bible!"),
        Language::German => format!("Dies ist eine Erinnerung, auch heute in der Bibel zu lesen.")
    }
}

pub fn msg_language_set(lang: &Language) -> String {
    match lang {
        Language::English => format!("Language set to English."),
        Language::German => format!("Die Sprache wurde auf Deutsch umgestellt.")
    }
}

pub fn msg_poll_text(lang: &Language) -> Vec<String> {
    match lang {
        Language::English => vec![
            String::from("Have you read the Bible today?"),
            String::from("Yes"),
            String::from("No")
        ],
        Language::German => vec![
            String::from("Hast du heute in der Bibel gelesen?"),
            String::from("Ja"),
            String::from("Nein")
        ],
    }
}

pub fn msg_not_implemented_yet(lang: &Language) -> String {
    match lang {
        Language::English => format!("This feature has not been implemented yet."),
        Language::German => format!("Diese Funktion wurde noch nicht implementiert.")
    }
}

pub fn msg_error_enter_language(lang: &Language) -> String {
    match lang {
        Language::English => String::from("You need to specify a language, use either /setlang en or /setlang de"),
        Language::German => String::from("Du musst eine Sprache angeben, entweder /setlang de oder /setlang en")
    }
}