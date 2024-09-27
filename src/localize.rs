use serde::{Deserialize, Serialize};

use crate::biblereading::BibleReading;

/// This enum contains the list of all supported languages for the bot
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Language {
    English,
    German
}

pub fn msg_biblereading(lang: Language, biblereading: BibleReading) -> String {
    match lang {
        Language::English => {
            format!(
                "*Today's Bible reading*: \n\nOT: {}\nNT: {}", 
                biblereading.old_testament_reading,
                biblereading.new_testament_reading
            )
        },
        Language::German => {
            format!(
                "*Die heutige Bibellese*: \n\nAT: {}\nNT: {}", 
                biblereading.old_testament_reading,
                biblereading.new_testament_reading
            )
        }
    }
}

pub fn msg_biblereading_not_found(lang: Language) -> String {
    match lang {
        Language::English => format!("This is a reminder to read your bible!"),
        Language::German => format!("Dies ist eine Erinnerung, auch heute in der Bibel zu lesen.")
    }
}

pub fn msg_language_set(lang: Language) -> String {
    match lang {
        Language::English => format!("Language set to English."),
        Language::German => format!("Die Sprache wurde auf Deutsch umgestellt.")
    }
}
