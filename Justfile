set dotenv-load
set positional-arguments
set export

help:
    @just --list --unsorted

build:
    cargo build
alias b := build

test *ARGS:
    cargo test $ARGS