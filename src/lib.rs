use std::fmt::{Display, Formatter};

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


pub struct DecodedFrame {
    raw_line1: String,
    raw_line2: String,
}

/// Represents a best attempt at pulling out only user relevant information from a backtrace frame.
pub enum DecodedUserBacktrace {
    Frames(Vec<DecodedFrame>),
    Disabled,
}

impl Display for DecodedUserBacktrace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodedUserBacktrace::Frames(frames) => {
                for frame in frames {
                    writeln!(f, "{}", frame.raw_line1)?;
                    writeln!(f, "{}", frame.raw_line2)?;
                }
                Ok(())
            }
            DecodedUserBacktrace::Disabled => {
                writeln!(f, "disabled backtrace")
            }
        }
    }
}

fn decode_backtrace<Backtrace: Display>(b: &Backtrace, hide_packages: &[&str]) -> DecodedUserBacktrace {
    let s = b.to_string();
    let mut lines = s.lines().peekable();
    let mut frames = Vec::new();
    if lines.next().unwrap_or_default() == "disabled backtrace" {
        return DecodedUserBacktrace::Disabled;
    }
    loop {
        if lines.peek().map(|&l| l.trim_start().starts_with("1")).unwrap_or_default() {
            break;
        }
        lines.next();
    }
    while let Some(line1) = lines.next() {
        let frame = &line1[6..];
        if frame.starts_with("__") {
            continue;
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
    DecodedUserBacktrace::Frames(frames)
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
    use anyhow::{Result, anyhow};
    use super::*;

    fn nested2() -> Result<()> {
        Err(anyhow!("Not implemented"))
    }

    fn nested1() -> Result<()> {
        nested2()
    }

    #[test]
    fn test_anyhow_err() {
        let Err(e) = nested1() else { panic!("expected error"); };
        // println!("{:?}", decode_backtrace(e.backtrace()));
        // println!("backtrace: {}", e.backtrace());
        let user_backtrace = format!("{}", e.user_backtrace());
        println!("{}", user_backtrace);
        assert_eq!(user_backtrace.lines().count(), 8);
    }
}