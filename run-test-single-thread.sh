#!/usr/bin/env bash
#
# disable parallel test execution because
# unit tests contains both image creating, modification and reading.
#
# https://doc.rust-lang.org/book/ch11-02-running-tests.html
cargo test -- --test-threads=1 --show-output
