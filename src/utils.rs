use std::path::Path;

pub fn ignore_file(p: &Path, pat: &str) -> bool {
    p.components().all(|c| {
        c.as_os_str()
            .to_str()
            .map(|s| !s.starts_with(pat))
            .unwrap_or(true)
    })
}
