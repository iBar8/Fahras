// SPDX-License-Identifier: MIT

//! Fahras: unlocking LUKS volumes and caching their passphrases in the Secret
//! Service are Linux features. On Windows nothing is ever cached.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ForgetCachedPasswordError {
    Failed(String),
}

impl ForgetCachedPasswordError {
    pub(super) fn user_message(&self) -> String {
        match self {
            Self::Failed(message) => message.clone(),
        }
    }
}

impl fmt::Display for ForgetCachedPasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(message) => f.write_str(message),
        }
    }
}

pub(super) fn volume_password_is_cached(_uuid: &str) -> bool {
    false
}

pub(super) fn forget_cached_volume_password(_uuid: &str) -> Result<(), ForgetCachedPasswordError> {
    Ok(())
}
