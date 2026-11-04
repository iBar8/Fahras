// SPDX-License-Identifier: MIT

//! The data types of [ashpd](https://docs.rs/ashpd) that Strata's file
//! chooser builds on, without the D-Bus portal behind them. Fahras aliases
//! this crate as `ashpd` on Windows, where there is no desktop portal; the
//! chooser still serves Fahras's own destination dialogs there.

use std::{
    ffi::{CString, OsStr},
    fmt,
    path::Path,
    str::FromStr,
};

pub mod backend {
    pub type Result<T> = std::result::Result<T, crate::PortalError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortalError {
    Failed(String),
    InvalidArgument(String),
    NotFound(String),
    Exist(String),
    NotAllowed(String),
    Cancelled(String),
    WindowDestroyed(String),
}

impl fmt::Display for PortalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, message) = match self {
            Self::Failed(message) => ("Failed", message),
            Self::InvalidArgument(message) => ("InvalidArgument", message),
            Self::NotFound(message) => ("NotFound", message),
            Self::Exist(message) => ("Exist", message),
            Self::NotAllowed(message) => ("NotAllowed", message),
            Self::Cancelled(message) => ("Cancelled", message),
            Self::WindowDestroyed(message) => ("WindowDestroyed", message),
        };
        write!(f, "{kind}: {message}")
    }
}

impl std::error::Error for PortalError {}

/// A NUL-free path as the portal transfers it.
#[derive(Debug, Default, PartialEq)]
pub struct FilePath(CString);

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        Path::new(OsStr::new(self.0.to_str().unwrap_or_default()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WindowIdentifierType {
    X11(std::os::raw::c_ulong),
    Wayland(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    MissingScheme,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "empty URI",
            Self::MissingScheme => "URI has no scheme",
        })
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uri(String);

impl Uri {
    pub fn parse(uri: &str) -> Result<Self, ParseError> {
        Self::from_str(uri)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Uri {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }
        if !s.contains(':') || s.starts_with(':') {
            return Err(ParseError::MissingScheme);
        }
        Ok(Self(s.to_owned()))
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub mod desktop {
    pub mod file_chooser {
        use crate::Uri;

        #[derive(Clone, Debug, PartialEq)]
        enum FilterType {
            GlobPattern,
            MimeType,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct FileFilter(String, Vec<(FilterType, String)>);

        impl FileFilter {
            pub fn new(label: &str) -> Self {
                Self(label.to_owned(), Vec::new())
            }

            #[must_use]
            pub fn mimetype(mut self, mimetype: &str) -> Self {
                self.1.push((FilterType::MimeType, mimetype.to_owned()));
                self
            }

            #[must_use]
            pub fn glob(mut self, pattern: &str) -> Self {
                self.1.push((FilterType::GlobPattern, pattern.to_owned()));
                self
            }

            pub fn label(&self) -> &str {
                &self.0
            }

            pub fn mimetype_filters(&self) -> Vec<&str> {
                self.1
                    .iter()
                    .filter_map(|(kind, value)| (*kind == FilterType::MimeType).then_some(value.as_str()))
                    .collect()
            }

            pub fn pattern_filters(&self) -> Vec<&str> {
                self.1
                    .iter()
                    .filter_map(|(kind, value)| (*kind == FilterType::GlobPattern).then_some(value.as_str()))
                    .collect()
            }
        }

        #[derive(Clone, Debug)]
        pub struct Choice(String, String, Vec<(String, String)>, String);

        impl Choice {
            pub fn boolean(id: &str, label: &str, state: bool) -> Self {
                Self::new(id, label, &state.to_string())
            }

            pub fn new(id: &str, label: &str, initial_selection: &str) -> Self {
                Self(id.to_owned(), label.to_owned(), Vec::new(), initial_selection.to_owned())
            }

            #[must_use]
            pub fn insert(mut self, key: &str, value: &str) -> Self {
                self.2.push((key.to_owned(), value.to_owned()));
                self
            }

            pub fn id(&self) -> &str {
                &self.0
            }

            pub fn label(&self) -> &str {
                &self.1
            }

            pub fn pairs(&self) -> Vec<(&str, &str)> {
                self.2.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect()
            }

            pub fn initial_selection(&self) -> &str {
                &self.3
            }
        }

        use crate::FilePath;

        /// Portal requests never arrive on Windows, so options are always empty.
        #[derive(Debug, Default)]
        pub struct OpenFileOptions {
            filters: Vec<FileFilter>,
            choices: Vec<Choice>,
        }

        impl OpenFileOptions {
            pub fn accept_label(&self) -> Option<&str> {
                None
            }

            pub fn modal(&self) -> Option<bool> {
                None
            }

            pub fn multiple(&self) -> Option<bool> {
                None
            }

            pub fn directory(&self) -> Option<bool> {
                None
            }

            pub fn filters(&self) -> &[FileFilter] {
                &self.filters
            }

            pub fn current_filter(&self) -> Option<&FileFilter> {
                None
            }

            pub fn choices(&self) -> &[Choice] {
                &self.choices
            }

            pub fn current_folder(&self) -> Option<&FilePath> {
                None
            }
        }

        #[derive(Debug, Default)]
        pub struct SaveFileOptions {
            filters: Vec<FileFilter>,
            choices: Vec<Choice>,
        }

        impl SaveFileOptions {
            pub fn accept_label(&self) -> Option<&str> {
                None
            }

            pub fn modal(&self) -> Option<bool> {
                None
            }

            pub fn current_name(&self) -> Option<&str> {
                None
            }

            pub fn current_folder(&self) -> Option<&FilePath> {
                None
            }

            pub fn current_file(&self) -> Option<&FilePath> {
                None
            }

            pub fn filters(&self) -> &[FileFilter] {
                &self.filters
            }

            pub fn current_filter(&self) -> Option<&FileFilter> {
                None
            }

            pub fn choices(&self) -> &[Choice] {
                &self.choices
            }
        }

        #[derive(Debug, Default)]
        pub struct SaveFilesOptions {
            choices: Vec<Choice>,
            files: Vec<FilePath>,
        }

        impl SaveFilesOptions {
            pub fn accept_label(&self) -> Option<&str> {
                None
            }

            pub fn modal(&self) -> Option<bool> {
                None
            }

            pub fn choices(&self) -> &[Choice] {
                &self.choices
            }

            pub fn current_folder(&self) -> Option<&FilePath> {
                None
            }

            pub fn files(&self) -> &[FilePath] {
                &self.files
            }
        }

        #[derive(Debug, Default)]
        pub struct SelectedFiles {
            uris: Vec<Uri>,
            choices: Vec<(String, String)>,
            current_filter: Option<FileFilter>,
            writable: Option<bool>,
        }

        impl SelectedFiles {
            pub fn uris(&self) -> &[Uri] {
                &self.uris
            }

            pub fn choices(&self) -> &[(String, String)] {
                &self.choices
            }

            pub fn uri(mut self, value: Uri) -> Self {
                self.uris.push(value);
                self
            }

            pub fn choice(mut self, key: &str, value: &str) -> Self {
                self.choices.push((key.to_owned(), value.to_owned()));
                self
            }

            pub fn current_filter(mut self, value: impl Into<Option<FileFilter>>) -> Self {
                self.current_filter = value.into();
                self
            }

            pub fn writable(mut self, value: impl Into<Option<bool>>) -> Self {
                self.writable = value.into();
                self
            }
        }
    }
}
