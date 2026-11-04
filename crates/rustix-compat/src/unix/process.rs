// SPDX-License-Identifier: MIT

use std::{
    io,
    os::windows::process::CommandExt as _,
    process::{Command, ExitStatus},
};

const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

pub trait CommandExt {
    fn process_group(&mut self, pgroup: i32) -> &mut Command;
    /// Windows cannot replace the running image: run the command, then exit
    /// with its status. Returns only if it could not be started.
    fn exec(&mut self) -> io::Error;
}

impl CommandExt for Command {
    fn process_group(&mut self, _pgroup: i32) -> &mut Command {
        self.creation_flags(CREATE_NEW_PROCESS_GROUP)
    }

    fn exec(&mut self) -> io::Error {
        match self.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(error) => error,
        }
    }
}

pub trait ExitStatusExt {
    fn signal(&self) -> Option<i32>;
    fn from_raw(raw: i32) -> Self;
}

impl ExitStatusExt for ExitStatus {
    fn signal(&self) -> Option<i32> {
        None
    }

    fn from_raw(raw: i32) -> Self {
        std::os::windows::process::ExitStatusExt::from_raw(raw as u32)
    }
}
