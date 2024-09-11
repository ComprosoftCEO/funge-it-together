#!/bin/sh
BUILD_FILE=build.zip

cargo build --release

rm "$BUILD_FILE"
zip -r "$BUILD_FILE" README.md LICENSE levels
zip "$BUILD_FILE" -j target/release/funge-it-together