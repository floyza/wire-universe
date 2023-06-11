{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell { buildInputs = [ wasm-pack openssl pkg-config binaryen ]; }
