use std::collections::HashMap;

use indexmap::{indexset, IndexSet};

use colored::Colorize;
use toml::Table;
use unicase::UniCase;

use super::path_str::PathString;

pub type EnvMap = HashMap<UniCase<String>, IndexSet<PathString>>;

#[derive(Debug, Default)]
struct EnvContainer {
    add_block: EnvMap,
    remove_block: EnvMap,
}

#[derive(Debug)]
pub struct Config {
    pub toml: Table,
}

#[derive(Debug)]
struct Profile {
    name: String,
    index: i64,
    add: Vec<String>,
    remove: Vec<String>,
    env: EnvMap,
}

impl Profile {
    fn new(config: &Table, verbose: bool) -> Result<Vec<Profile>, String> {
        let mut profiles = Vec::new();

        for (key, value) in config {
            if key == "profile" {
                if let Some(value) = value.as_array() {
                    for (i, v) in value.iter().enumerate() {
                        profiles.push(Self::parse(v, i, verbose)?);
                    }
                } else {
                    return Err(format!("{} Key {} must be an array.", "Error:".red(), key));
                }
            } else {
                eprintln!("{} Key {} is unused.", "Warning:".yellow(), key);
            }
        }

        Ok(profiles)
    }

    fn parse(v: &toml::Value, i: usize, verbose: bool) -> Result<Profile, String> {
        let mut profile = Profile {
            name: String::new(),
            index: 0,
            add: Vec::new(),
            remove: Vec::new(),
            env: HashMap::new(),
        };

        if let Some(v) = v.as_table() {
            for (k, v) in v {
                match k.as_str() {
                    "name" => {
                        if let Some(s) = v.as_str() {
                            profile.name = s.to_string();
                        } else {
                            return Err(format!(
                                "{} Key profile[{i}].name is not a string.",
                                "Error:".red()
                            ));
                        }
                    }
                    "index" => {
                        if let Some(i) = v.as_integer() {
                            profile.index = i;
                        } else {
                            return Err(format!(
                                "{} Key profile[{i}].index is not an integer.",
                                "Error:".red()
                            ));
                        }
                    }
                    "add" | "remove" => {
                        if let Some(v) = v.as_array() {
                            for (ii, v) in v.iter().enumerate() {
                                if let Some(s) = v.as_str() {
                                    if k == "add" {
                                        profile.add.push(s.to_string());
                                    } else {
                                        profile.remove.push(s.to_string());
                                    }
                                } else {
                                    return Err(format!(
                                        "{} Key profile[{i}].{k}[{ii}] is not a string.",
                                        "Error:".red()
                                    ));
                                }
                            }
                        } else {
                            return Err(format!(
                                "{} Key profile[{i}].{k} is not an array.",
                                "Error:".red()
                            ));
                        }
                    }
                    "env" => {
                        if let Some(t) = v.as_table() {
                            for (k, v) in t {
                                match v {
                                    toml::Value::String(s) => {
                                        insert_to_env_map(
                                            &mut profile.env,
                                            k.clone().into(),
                                            s.to_string().into(),
                                            verbose,
                                        );
                                    }
                                    toml::Value::Array(a) => {
                                        for (ii, v) in a.iter().enumerate() {
                                            if let Some(s) = v.as_str() {
                                                insert_to_env_map(
                                                    &mut profile.env,
                                                    k.clone().into(),
                                                    s.to_string().into(),
                                                    verbose,
                                                );
                                            } else {
                                                return Err(format!(
                                                    "{} Key profile[{i}].env.{k}[{ii}] is not a string.",
                                                    "Error:".red()
                                                ));
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(format!(
                                        "{} Key profile[{i}].env.{k} is not an array or a string.",
                                        "Error:".red()
                                    ))
                                    }
                                }
                            }
                        } else {
                            return Err(format!(
                                "{} Key profile[{i}].env is not an object.",
                                "Error:".red()
                            ));
                        }
                    }
                    _ => eprintln!("{} Key profile[{i}].{} is unused.", "Warning:".yellow(), k),
                }
            }

            if profile.name.is_empty() {
                return Err(format!(
                    "{} Key profile[{i}].name is missing.",
                    "Error:".red()
                ));
            }
        } else {
            return Err(format!(
                "{} Key profile[{i}] must be an array.",
                "Error:".red()
            ));
        }

        Ok(profile)
    }

    fn get_profile<'a>(profiles: &'a Vec<Profile>, name: &str) -> Option<&'a Profile> {
        for p in profiles {
            if p.name == name {
                return Some(p);
            }
        }

        None
    }

    fn get_env_block(profiles: &Vec<Profile>, name: &str, verbose: bool) -> EnvContainer {
        let p = Profile::get_profile(&profiles, name).unwrap();

        let mut new_env = EnvContainer::default();

        add_env_block(&mut new_env.add_block, &p.env);

        // add -> add_list
        // remove -> remove list
        for a in &p.add {
            let env_block = Profile::get_env_block(profiles, &a, verbose);

            add_env_block(&mut new_env.add_block, &env_block.add_block);
            add_env_block(&mut new_env.remove_block, &env_block.remove_block);
        }

        // add -> remove list
        // remove -> remove list
        for r in &p.remove {
            let env_block = Profile::get_env_block(profiles, &r, verbose);

            add_env_block(&mut new_env.remove_block, &env_block.add_block);
            add_env_block(&mut new_env.remove_block, &env_block.remove_block);
        }

        new_env
    }
}

impl Config {
    pub fn get_env(&self, profiles: &[String], verbose: bool) -> Result<EnvMap, String> {
        let env_profiles = Profile::new(&self.toml, verbose)?;

        let mut new_env = EnvContainer::default();

        for p in profiles {
            let profile_block = Profile::get_env_block(&env_profiles, &p, verbose);
            add_env_block(&mut new_env.add_block, &profile_block.add_block);
            add_env_block(&mut new_env.remove_block, &profile_block.remove_block);
        }

        if verbose {
            dbg!(&new_env);
        }

        // TODO:(fraclysis) Do not update every environmental variable update changing ones
        let system = get_system_env()?;
        // dbg!(&system);

        let mut update_list: EnvMap = EnvMap::default();

        for (k, v) in new_env.add_block {
            update_list.insert(k.clone(), v);

            if let Some(v) = system.get(&k) {
                let l = update_list.get_mut(&k).unwrap();

                for v in v {
                    l.insert(v.clone());
                }
            }
        }

        for (k, rv) in new_env.remove_block {
            if let Some(sv) = system.get(&k) {
                let l = match update_list.get_mut(&k) {
                    Some(l) => l,
                    None => {
                        update_list.insert(k.clone(), Default::default());
                        update_list.get_mut(&k).unwrap()
                    }
                };

                for v in sv {
                    if !rv.contains(v) {
                        l.insert(v.clone());
                    }
                }
            }

            if let Some(iv) = update_list.get_mut(&k) {
                for v in rv {
                    iv.remove(&v);
                }
            }
        }

        Ok(update_list)
    }
}

fn add_env_block(e0: &mut EnvMap, e1: &EnvMap) {
    for (k, v) in e1 {
        if let Some(block) = e0.get_mut(k) {
            for v in v {
                block.insert(v.clone());
            }
        } else {
            e0.insert(k.clone(), v.clone());
        }
    }
}

fn get_system_env() -> Result<EnvMap, String> {
    let mut system = HashMap::new();

    for (k, v) in std::env::vars_os() {
        let mut value = IndexSet::new();

        for p in std::env::split_paths(&v) {
            if let Some(val) = p.to_str() {
                value.insert(val.to_string().into());
            } else {
                return Err(format!(
                    "{} System environment variable is not valid utf8.",
                    "Error:".red()
                ));
            }
        }

        if let Some(key) = k.to_str() {
            system.insert(key.to_string().into(), value);
        } else {
            return Err(format!(
                "{} System environment variable is not valid utf8.",
                "Error:".red()
            ));
        }
    }

    Ok(system)
}

fn insert_to_env_map(map: &mut EnvMap, key: UniCase<String>, value: PathString, verbose: bool) {
    if let Some(container) = map.get_mut(&key) {
        if container.insert(value) && verbose {
            // eprintln!("{} Value is present.", "Warning:".yellow())
        }
    } else {
        map.insert(key.clone(), indexset![value]);
    }
}
