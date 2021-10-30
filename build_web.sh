#!/bin/bash
set -eu

# ./setup_web.sh # <- call this first!

FOLDER_NAME=${PWD##*/}
CRATE_NAME=$FOLDER_NAME # assume crate name is the same as the folder name
CRATE_NAME_SNAKE_CASE="${CRATE_NAME//-/_}" # for those who name crates with-kebab-case
OUT_DIR=docs
BUILD=release
TARGET_NAME="${CRATE_NAME_SNAKE_CASE}.wasm"
TARGET_PATH="${OUT_DIR}/${TARGET_NAME}"

# This is required to enable the web_sys clipboard API which egui_web uses
# https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
# https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
export RUSTFLAGS=--cfg=web_sys_unstable_apis

echo "Building rust…"
cargo build --${BUILD} -p ${CRATE_NAME} --lib --target=wasm32-unknown-unknown

echo "Generating JS bindings for wasm…"
# Clear output from old stuff:
rm -f ${TARGET_PATH}
wasm-bindgen "target/wasm32-unknown-unknown/${BUILD}/${TARGET_NAME}" \
  --out-dir ${OUT_DIR} --no-modules --no-typescript

# to get wasm-opt:  apt/brew/dnf install binaryen
# echo "Optimizing wasm…"
# wasm-opt ${CRATE_NAME_SNAKE_CASE}_bg.wasm -O2 --fast-math -o ${CRATE_NAME_SNAKE_CASE}_bg.wasm # add -g to get debug symbols

echo "Finished: ${TARGET_PATH}"
