use std::fmt::{Display, Formatter};
use anyhow::{Result, anyhow};

const HIDDEN_PACKAGES: &[&str] = &[
    "backtrace",
    "anyhow",
    "core",
    "alloc",
    "std",
    "test",
    "tokio",
    "futures",
    "futures_util",
];

fn nested2() -> Result<()> {
    Err(anyhow!("Not implemented"))
}


pub fn nested1() -> Result<()> {
    nested2()
}


struct DecodedFrame {
    raw_line1: String,
    raw_line2: String,
}

/// Represents a best attempt at pulling out only user relevant information from a backtrace frame.
pub struct DecodedUserBacktrace {
    frames: Vec<DecodedFrame>,
}

impl Display for DecodedUserBacktrace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for frame in &self.frames {
            writeln!(f, "{}", frame.raw_line1)?;
            writeln!(f, "{}", frame.raw_line2)?;
        }
        Ok(())
    }
}

fn decode_backtrace<Backtrace: Display>(b: &Backtrace, hide_packages: &[&str]) -> DecodedUserBacktrace {
    let s = format!("{}", b);
    let mut lines = s.lines();
    let mut frames = Vec::new();

    while let Some(line1) = lines.next() {
        let frame = &line1[6..];
        if frame.starts_with("__") {
            continue
        }
        let line2 = lines.next().unwrap();
        if frame.starts_with('<') {
            let package1 = frame[1..].splitn(2, "::").next().unwrap();
            let package2 = frame.splitn(2, " as ").skip(1).next().unwrap().splitn(2, "::").next().unwrap();
            if hide_packages.contains(&package1) && hide_packages.contains(&package2) {
                continue;
            }
        } else {
            let package = frame.splitn(2, "::").next().unwrap();
            if hide_packages.contains(&package) {
                continue;
            }
        };
        frames.push(DecodedFrame {
            raw_line1: line1.to_string(),
            raw_line2: line2.to_string(),
        });
    }
    DecodedUserBacktrace { frames }
}

pub trait UserBacktrace {
    fn user_backtrace(&self) -> DecodedUserBacktrace;
}

impl UserBacktrace for anyhow::Error {
    fn user_backtrace(&self) -> DecodedUserBacktrace {
        decode_backtrace(self.backtrace(), HIDDEN_PACKAGES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let Err(e) = nested1() else { panic!("expected error"); };
        // println!("{:?}", decode_backtrace(e.backtrace()));
        // println!("backtrace: {}", e.backtrace());
        let user_backtrace = format!("{}", e.user_backtrace());
        assert_eq!(user_backtrace.lines().count(), 8);
    }
}
