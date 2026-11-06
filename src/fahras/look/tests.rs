// SPDX-License-Identifier: MIT

use super::*;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Stored {
    icon: AppIcon,
    skin: ExplorerSkin,
}

#[test]
fn saved_choices_round_trip_and_unknown_values_fall_back() {
    for (text, expected) in [
        (
            "icon = \"forest\"\nskin = \"windows11\"\n",
            Stored {
                icon: AppIcon::Forest,
                skin: ExplorerSkin::Windows11,
            },
        ),
        (
            "icon = \"clay\"\nskin = \"windows10\"\n",
            Stored {
                icon: AppIcon::Clay,
                skin: ExplorerSkin::Windows10,
            },
        ),
        (
            "icon = \"neon\"\nskin = \"windows95\"\n",
            Stored {
                icon: AppIcon::Manila,
                skin: ExplorerSkin::Off,
            },
        ),
    ] {
        let stored: Stored = toml::from_str(text).expect("stored look");
        assert_eq!(stored, expected, "{text}");
        let saved = toml::to_string(&stored).expect("saved look");
        assert_eq!(toml::from_str::<Stored>(&saved).expect("reloaded"), stored);
    }
}

#[test]
fn legacy_looks_keep_the_layout_without_the_new_details() {
    for (legacy, base) in [
        (ExplorerSkin::Windows10Legacy, ExplorerSkin::Windows10),
        (ExplorerSkin::Windows11Legacy, ExplorerSkin::Windows11),
    ] {
        assert_eq!(legacy.base(), base);
        assert!(!legacy.is_polished());
        assert!(base.is_polished());
    }
    assert!(!ExplorerSkin::Off.is_polished());
}
