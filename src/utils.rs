use crate::config::RenovateFormatConfig;
use anyhow::Result;
use console::{style, Style};
use pg_query::NodeEnum;
use similar::{ChangeTag, TextDiff};
use std::{
    fmt::{self, Write},
    path::Path,
};

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn ignore_file(p: &Path, pat: &str) -> bool {
    p.components().all(|c| {
        c.as_os_str()
            .to_str()
            .map(|s| !s.starts_with(pat))
            .unwrap_or(true)
    })
}

pub fn create_diff(old: &NodeEnum, new: &NodeEnum) -> Result<String> {
    let format = RenovateFormatConfig::default().into();

    let old = sqlformat::format(&old.deparse()?, &Default::default(), format);
    let new = sqlformat::format(&new.deparse()?, &Default::default(), format);

    diff_text(&old, &new)
}

/// generate the diff between two strings. TODO: this is just for console output for now
fn diff_text(text1: &str, text2: &str) -> Result<String> {
    let mut output = String::new();
    let diff = TextDiff::from_lines(text1, text2);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut output, "{:-^1$}", "-", 80)?;
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new().dim()),
                };
                write!(
                    &mut output,
                    "{}{} |{}",
                    style(Line(change.old_index())).dim(),
                    style(Line(change.new_index())).dim(),
                    s.apply_to(sign).bold(),
                )?;
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        write!(&mut output, "{}", s.apply_to(value).underlined().on_black())?;
                    } else {
                        write!(&mut output, "{}", s.apply_to(value))?;
                    }
                }
                if change.missing_newline() {
                    writeln!(&mut output)?;
                }
            }
        }
    }

    Ok(output)
}
