// SPDX-License-Identifier: MIT

//! Fahras appearance choices: the application icon and the Explorer skins.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AppIcon {
    #[default]
    Manila,
    Forest,
    Ink,
    Clay,
}

impl AppIcon {
    pub(crate) const ALL: [Self; 4] = [Self::Manila, Self::Forest, Self::Ink, Self::Clay];

    /// The bundled icon-theme name, under `icons/scalable/apps/`.
    pub(crate) fn icon_name(self) -> &'static str {
        match self {
            Self::Manila => "fahras-manila",
            Self::Forest => "fahras-forest",
            Self::Ink => "fahras-ink",
            Self::Clay => "fahras-clay",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Manila => "Manila",
            Self::Forest => "Forest",
            Self::Ink => "Ink",
            Self::Clay => "Clay",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ExplorerSkin {
    #[default]
    Off,
    Windows10,
    Windows11,
    /// The first Explorer looks, kept for those who prefer Fahras's own icons,
    /// dates and address bar inside Explorer's layout.
    Windows10Legacy,
    Windows11Legacy,
}

impl ExplorerSkin {
    pub(crate) const ALL: [Self; 5] = [
        Self::Off,
        Self::Windows10,
        Self::Windows11,
        Self::Windows10Legacy,
        Self::Windows11Legacy,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Windows10 => "windows10",
            Self::Windows11 => "windows11",
            Self::Windows10Legacy => "windows10-legacy",
            Self::Windows11Legacy => "windows11-legacy",
        }
    }

    /// The Windows version whose layout and colors this look uses.
    pub(crate) fn base(self) -> Self {
        match self {
            Self::Windows10Legacy => Self::Windows10,
            Self::Windows11Legacy => Self::Windows11,
            skin => skin,
        }
    }

    /// Whether this look also copies Explorer's details: Windows' own icons,
    /// its date and size formats and its address bar.
    pub(crate) fn is_polished(self) -> bool {
        matches!(self, Self::Windows10 | Self::Windows11)
    }
}

impl AppIcon {
    fn id(self) -> &'static str {
        self.icon_name().trim_start_matches("fahras-")
    }
}

/// Stored by name; a name from a newer or older Fahras falls back to the default
/// instead of discarding the whole settings file.
macro_rules! stored_by_id {
    ($type:ty) => {
        impl Serialize for $type {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.id())
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let id = String::deserialize(deserializer)?;
                Ok(Self::ALL
                    .into_iter()
                    .find(|choice| choice.id() == id)
                    .unwrap_or_default())
            }
        }
    };
}

stored_by_id!(AppIcon);
stored_by_id!(ExplorerSkin);

#[cfg(test)]
mod tests;
