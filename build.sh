#!/bin/sh
wasm-pack build
wasm-opt -Oz pkg/pathypath_bg.wasm -o pkg/pathypath_bg.wasm
cd www
npm run build
cd ..
