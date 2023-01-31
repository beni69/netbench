{ pkgs ? (import <nixpkgs> { }) }:

let
  # netbench = pkgs.callPackage ./default.nix { };
  netbench = import ./musl.nix { };
in
pkgs.dockerTools.buildImage {
  name = "netbench-docker";
  config = {
    Entrypoint = [ "${netbench}/bin/netbench" ];
    Cmd = [ "server" ];
  };
}
