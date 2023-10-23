use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to Profiles.toml
    #[arg(short, long)]
    pub config_path: Option<String>,

    /// Use verbose output
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Sets one or more profiles to the parent process
    Set {
        /// List of profile names to set
        profile_names: Vec<String>,
    },
    Open {
        #[arg(short, long)]
        process: Option<String>,
    },
    // Unset { profile_names: Vec<String> },
}

pub fn get_profiles_toml_default() -> Option<String> {
    use std::ptr::{null, null_mut};

    #[link(name = "Kernel32")]
    extern "C" {
        fn SearchPathW(
            path: *const u16,
            file_name: *const u16,
            extension: *const u16,
            buffer_length: u32,
            buffer: *mut u16,
            file_part: *mut *mut u16,
        ) -> u32;
    }

    let mut buffer = [0u16; 512];

    let file_name = [0x50, 0x72, 0x6f, 0x66, 0x69, 0x6c, 0x65, 0x73, 0x0]; // Profiles
    let extension = [0x2e, 0x74, 0x6f, 0x6d, 0x6c, 0x0]; // .toml

    let path_size = unsafe {
        SearchPathW(
            null(),
            file_name.as_ptr(),
            extension.as_ptr(),
            buffer.len() as u32,
            buffer.as_mut_ptr(),
            null_mut(),
        ) as usize
    };

    if path_size == 0 {
        None
    } else if buffer.len() > path_size {
        if let Ok(s) = String::from_utf16(&buffer[..path_size]) {
            Some(s)
        } else {
            None
        }
    } else {
        let mut buffer: Vec<u16> = Vec::with_capacity(path_size);

        let path_size = unsafe {
            SearchPathW(
                null(),
                file_name.as_ptr(),
                extension.as_ptr(),
                buffer.len() as u32,
                buffer.as_mut_ptr(),
                null_mut(),
            ) as usize
        };

        if path_size == 0 {
            None
        } else if buffer.len() > path_size {
            if let Ok(s) = String::from_utf16(&buffer) {
                Some(s)
            } else {
                None
            }
        } else {
            None
        }
    }
}
