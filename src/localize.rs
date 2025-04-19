use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use teloxide::utils::markdown::escape;

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
                escape(&translated_bible_reference(lang, &biblereading.old_testament_reading)),
                escape(&translated_bible_reference(lang, &biblereading.new_testament_reading))
            )
        },
        Language::German => {
            format!(
                "*📖 Dies ist eine Erinnerung, heute in der Bibel zu lesen*: \n\nAT: {}\nNT: {}", 
                escape(&translated_bible_reference(lang, &biblereading.old_testament_reading)),
                escape(&translated_bible_reference(lang, &biblereading.new_testament_reading))
            )
        }
    }
}

fn translated_bible_reference(lang: &Language, original_reference: &str) -> String {
    match lang {
        Language::English => {
            bibleref::translate(original_reference, "en").unwrap_or_else(|_| original_reference.to_string())
        }
        Language::German => {
            bibleref::translate(original_reference, "de").unwrap_or_else(|_| original_reference.to_string())
        }
    }
}

pub fn msg_biblereading_not_found(lang: &Language) -> String {
    match lang {
        Language::English => "This is a reminder to read your bible!".to_string(),
        Language::German => "Dies ist eine Erinnerung, heute in der Bibel zu lesen.".to_string()
    }
}

pub fn msg_language_set(lang: &Language) -> String {
    match lang {
        Language::English => "Language set to English.".to_string(),
        Language::German => "Die Sprache wurde auf Deutsch umgestellt.".to_string()
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

#[allow(dead_code)]
pub fn msg_not_implemented_yet(lang: &Language) -> String {
    match lang {
        Language::English => "This feature has not been implemented yet.".to_string(),
        Language::German => "Diese Funktion wurde noch nicht implementiert.".to_string()
    }
}

pub fn msg_select_language(lang: &Language) -> String {
    match lang {
        Language::English => String::from("Please choose which language you would like to set."),
        Language::German => String::from("Bitte wählen Sie die Sprache aus, die sie einstellen möchten.")
    }
}

pub fn msg_timer_updated(lang: &Language, time: &NaiveTime) -> String {
    match lang {
        Language::English => format!("The daily timer has been updated to {}.", time.to_string()),
        Language::German => format!("Die tägliche Erinnerung wurde auf {} gesetzt.", time.to_string())
    }
}

pub fn msg_timer_unset(lang: &Language) -> String {
    match lang {
        Language::English => format!("The daily timer has been unset"),
        Language::German => format!("Die tägliche Erinnerung wurde deaktiviert"),
    }
}


pub fn msg_error_timer_update(lang: &Language) -> String {
    match lang {
        Language::English => String::from("The format was not valid. Please use the function with a valid time (for example /settimer 08:00)."),
        Language::German => String::from("Ungültiges Format. Bitte benutze die Funktion mit einer gültigen Zeitangabe, zum Beispiel /settimer 08:00.")
    }
}