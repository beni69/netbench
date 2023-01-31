{}:
with import <nixpkgs>
{
  overlays = [
    (import "${fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz"}/overlay.nix")
    (import "${fetchTarball "https://github.com/nix-community/naersk/archive/master.tar.gz"}/overlay.nix")
  ];
};

let
  # target = "x86_64-unknown-linux-musl";
  # target = pkgs.lib.pipe builtins.currentSystem [ (builtins.split "-+") lib.lists.flatten lib.lists.head ]
  #   + "-unknown-linux-musl";
  target = stdenv.hostPlatform.linuxArch
    + "-unknown-linux-musl";
  rust = with fenix; combine [
    stable.cargo
    stable.rustc
    targets.${target}.stable.rust-std
  ];
  n = naersk.override { cargo = rust; rustc = rust; };
in
n.buildPackage {
  src = ./.;
  CARGO_BUILD_TARGET = target;
}
