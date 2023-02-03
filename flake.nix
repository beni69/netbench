{
  description = "netbench";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-22.11";

    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, }:
    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        mkCrate = { crossSystem ? localSystem, musl ? false }:
          let
            pkgs = import nixpkgs { inherit crossSystem localSystem; };
            abi = if musl then "musl" else "gnu";
            target = pkgs.stdenv.hostPlatform.qemuArch + "-unknown-linux-" + abi;
            TARGET = with pkgs.lib; with strings; pipe target [ (replaceChars [ "-" ] [ "_" ]) toUpper ]; # UPPERCASE_TARGET_FORMAT
            toolchain = with fenix.packages.${localSystem}; combine [ stable.cargo stable.rustc targets.${target}.stable.rust-std ];
            craneLib = crane.lib.${localSystem}.overrideToolchain toolchain;
          in
          with pkgs; craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;

            # depsBuildBuild = [ qemu ]; # unused because it's not cached for aarch64
            nativeBuildInputs = [ stdenv.cc pkg-config ];

            buildInputs = [ ]; # native dependencies like `openssl` go here

            CARGO_BUILD_TARGET = target;
            "CARGO_TARGET_${TARGET}_LINKER" = "${stdenv.cc.targetPrefix}cc";
            HOST_CC = "${stdenv.cc.nativePrefix}cc";

            # run checks on native build
            doCheck = localSystem == crossSystem;
          };
        mkDocker = { crossSystem ? localSystem, musl ? false, debug ? false }:
          let
            pkgs = (import nixpkgs { inherit localSystem crossSystem; });
            netbench = mkCrate { inherit crossSystem musl; };
            busybox = pkgs.busybox.override { enableStatic = true; useMusl = true; }; # unbloated "debug env"
          in
          with pkgs; with dockerTools; buildImage {
            name = "netbench";
            tag = "latest";
            config = {
              Cmd = [ "server" ];
              Entrypoint = [ "/bin/netbench" ];
            };
            copyToRoot = buildEnv {
              name = "image-root";
              paths = [ netbench ] ++ lib.optional debug busybox;
              pathsToLink = [ "/bin" ];
            };
          };
      in
      {
        packages = {
          default = mkCrate { };
          musl = mkCrate { musl = true; };
          docker = mkDocker { musl = true; };

          aarch64 = mkCrate { crossSystem = "aarch64-linux"; };
          musl-aarch64 = mkCrate { musl = true; crossSystem = "aarch64-linux"; };
          docker-aarch64 = mkDocker { musl = true; crossSystem = "aarch64-linux"; };
        };
      });
}
