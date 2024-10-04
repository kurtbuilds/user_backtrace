use std::fmt::{Display, Formatter};

const HIDDEN_PACKAGES: &[&str] = &[
    "F",
    "alloc",
    "anyhow",
    "axum",
    "backtrace",
    "core",
    "futures",
    "futures_core",
    "futures_util",
    "hyper",
    "hyper_util",
    "std",
    "test",
    "tokio",
    "tower",
    "tower_service",
    "tracing",
];

pub struct DecodedFrame {
    frame: String,
    location: Option<String>,
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
                    writeln!(f, "{}", frame.frame)?;
                    if let Some(line2) = &frame.location {
                        writeln!(f, "{}", line2)?;
                    }
                }
                Ok(())
            }
            DecodedUserBacktrace::Disabled => {
                writeln!(f, "disabled backtrace")
            }
        }
    }
}

fn decode_backtrace<Backtrace: Display>(
    b: &Backtrace,
    hide_packages: &[&str],
) -> DecodedUserBacktrace {
    let s = b.to_string();
    let mut lines = s.lines().peekable();
    let mut frames = Vec::new();
    if lines
        .peek()
        .map(|&s| s == "disabled backtrace")
        .unwrap_or(true)
    {
        return DecodedUserBacktrace::Disabled;
    }

    loop {
        let Some(&line) = lines.peek() else {
            break;
        };
        let line = &line[3..];
        if line.starts_with('1') {
            break;
        }
        lines.next();
    }

    while let Some(frame) = lines.next() {
        // skip the "  #: " portion
        let frame = &frame[6..];
        if frame.starts_with("__") {
            continue;
        }
        // get location, if its there
        let mut location = None;
        if let Some(&l) = lines.peek() {
            let l = l.trim_start_matches(' ');
            if l.starts_with("at ") {
                location = Some(lines.next().unwrap().to_string());
            }
        }

        if frame.starts_with("start_thread") || frame.starts_with("clone") {
            continue;
        }

        // decode
        if frame.starts_with('<') {
            let (left, right) = frame[1..].split_once(" as ").unwrap();
            let package1 = left.split(':').next().unwrap();
            let package2 = right.split(':').next().unwrap();
            if hide_packages.contains(&package1) && hide_packages.contains(&package2) {
                continue;
            }
        } else {
            let package = frame.split(':').next().unwrap();
            if hide_packages.contains(&package) {
                continue;
            }
        };
        frames.push(DecodedFrame {
            frame: frame.to_string(),
            location,
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
    use super::*;
    use anyhow::{anyhow, Result};

    fn nested2() -> Result<()> {
        Err(anyhow!("Not implemented"))
    }

    fn nested1() -> Result<()> {
        nested2()
    }

    #[test]
    fn test_anyhow_err() {
        let Err(e) = nested1() else {
            panic!("expected error");
        };
        // println!("{:?}", decode_backtrace(e.backtrace()));
        println!("backtrace: {}", e.backtrace());
        let user_backtrace = format!("{}", e.user_backtrace());
        println!("{}", user_backtrace);
        assert_eq!(user_backtrace.lines().count(), 8);
    }

    #[test]
    fn test_parse_backtrace1() {
        let s = include_str!("../data/backtrace1.txt");
        let r = decode_backtrace(&s, HIDDEN_PACKAGES);
        let r = r.to_string();
        println!("{}", r);
        assert_eq!(r.lines().count(), 3);
    }
}
