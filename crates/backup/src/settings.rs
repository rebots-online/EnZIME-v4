//! §14.2 `BackupSettings` DTO. Mirrors the TypeScript interface field names
//! and types verbatim.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackupSettings {
    pub reminder_interval_days: u32,
    pub include_conversations: bool,
    pub include_lore: bool,
    pub encrypt_by_default: bool,
}

impl Default for BackupSettings {
    fn default() -> Self {
        Self {
            reminder_interval_days: 7,
            include_conversations: true,
            include_lore: true,
            encrypt_by_default: true,
        }
    }
}
