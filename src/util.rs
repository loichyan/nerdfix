#[cfg(test)]
macro_rules! icon {
    ($name:literal, $codepoint:literal) => {
        icon!($name, $codepoint, false)
    };
    ($name:literal, $codepoint:literal, $obsolete:literal) => {
        crate::db::Icon {
            name: $name.to_owned(),
            codepoint: $codepoint,
            obsolete: $obsolete,
        }
    };
}
