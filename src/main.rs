mod args;
mod config;
mod path_str;

use args::*;
use clap::Parser;
use colored::Colorize;
use config::*;
use setenv::{SeResult, SetEnv};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let toml_str = std::fs::read_to_string(args.config_path.unwrap_or_else(|| {
        if let Some(p) = get_profiles_toml_default() {
            p
        } else {
            todo!("Error")
        }
    }))?;

    let config: Config = Config {
        toml: toml::from_str(&toml_str)?,
    };

    match args.command {
        Commands::Set { profile_names } => {
            let s = SetEnv::new()?;

            let env = match config.get_env(&profile_names, args.verbose) {
                Ok(env) => env,
                Err(e) => {
                    eprintln!("{e}");
                    return Ok(());
                }
            };

            for (k, v) in env {
                if args.verbose {
                    println!("{} Setting {k}", "Verbose:".blue());
                    for v in &v {
                        println!("\t{}", v.0);
                    }
                } else {
                    println!("{} Setting {k}", "Info:".green());
                }

                let env = std::env::join_paths(v.iter());
                if let Some(env) = env?.to_str() {
                    if let Err(e) = s.set_parent_var(&k, env) {
                        if args.verbose {
                            dbg!(k);
                            dbg!(env);
                        }
                        eprintln!("{} Setenv: {e}", "Error:".red())
                    }
                } else {
                    eprintln!("{} OsString::to_str() is None.", "Error:".red());
                }
            }
        }
        _ => todo!(),
    }

    Ok(())
}

extern "C" fn _warning_callback(result: SeResult) {
    eprintln!("{} {}", "Warning: ".yellow(), result)
}
