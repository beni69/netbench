# { lib, fetchFromGitHub, rustPlatform }:

# { pkgs ? (import <nixpkgs> { }).pkgsMusl }:
# with pkgs;

# with (import <nixpkgs> { });

{ ... }:
with (import <nixpkgs> {
  overlays = [
    # (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
  ];
  # crossSystem = (import <nixpkgs/lib>).systems.examples.musl64 // {
  crossSystem = rec {
    system = "x86_64-unknown-linux-musl";
    rustc.config = system;
  };
});

rustPlatform.buildRustPackage rec {
  pname = "netbench";
  version = "0.1.0";

  # src = fetchFromGitHub
  #   {
  #     owner = "beni69";
  #     repo = pname;
  #     rev = "main";
  #     sha256 = "sha256-zWzAyGymLi6L49n9sNS00SC3Je4blapLj1IVdnUsHxQ=";
  #   };

  src = fetchCrate {
    inherit pname version;
    sha256 = "sha256-cS4E6ceNEBkrWczdDRfzZz+cKSzI3Itzr3w91UxKJIg=";
  };
  # src = ./.;

  cargoLock.lockFile = ./Cargo.lock;
  doCheck = false;
}
