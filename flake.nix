{
  inputs = {
    nixpkgs.url ="github:NixOS/nixpkgs/nixos-22.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =inputs@ {self, nixpkgs, flake-parts, ...}: flake-parts.lib.mkFlake {inherit inputs; }{
    systems = [
      "x86_64-linux"
      "aarch64-linux"
    ];

    perSystem.imports = [./per-system.nix];
  };
}
