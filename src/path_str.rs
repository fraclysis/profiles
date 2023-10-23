use std::{
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[derive(Default, Clone)]
pub struct PathString(pub String);

impl Debug for PathString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for PathString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq for PathString {
    fn eq(&self, other: &Self) -> bool {
        let mut left = self.0.trim_end_matches(['/', '\\']).chars();
        let mut right = other.0.trim_end_matches(['/', '\\']).chars();

        loop {
            let x = match left.next() {
                None => return right.next().is_none(),
                Some(v) => v,
            };

            let y = match right.next() {
                None => return false,
                Some(v) => v,
            };

            let x = x.to_ascii_lowercase();
            let y = y.to_ascii_lowercase();

            if x != y {
                let is_x_path_sep = x == '/' || x == '\\';
                let is_y_path_sep = y == '/' || y == '\\';

                if !(is_x_path_sep && is_y_path_sep) {
                    return false;
                }
            }
        }
    }
}

impl Eq for PathString {}

impl Hash for PathString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let chars = self.0.trim_end_matches(['/', '\\']).chars();

        for mut c in chars {
            if c == '/' {
                c = '\\';
            }

            for l in c.to_lowercase() {
                state.write_u32(l as u32)
            }
        }
    }
}

impl From<String> for PathString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<std::ffi::OsStr> for PathString {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_ref()
    }
}
