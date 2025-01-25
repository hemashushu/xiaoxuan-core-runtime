#!/usr/bin/env bash
# https://doc.rust-lang.org/book/ch11-02-running-tests.html
cargo test -- --test-threads=1 --show-output
