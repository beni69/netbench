{ pkgs ? import <nixpkgs> { } }:

let
  netbench = import ./default.nix { };
in
pkgs.dockerTools.buildImage {
  name = "netbench-docker";
  config = {
    Entrypoint = [ "${netbench}/bin/netbench" ];
    Cmd = [ "server" ];
  };
}
