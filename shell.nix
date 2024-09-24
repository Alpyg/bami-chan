{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ cmake openssl libopus yt-dlp ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
