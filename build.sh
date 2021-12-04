#!/bin/sh
wasm-pack build
cd www
npm run build
cd ..
