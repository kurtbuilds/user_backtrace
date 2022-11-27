set dotenv-load := true

help:
    @just --list --unsorted

build:
    cargo build
alias b := build

test:
    cargo test -- --nocapture