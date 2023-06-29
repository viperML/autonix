{pkgs, ...}: {
  devShells.default = with pkgs;
    mkShell {
      packages = [
        rustc
        cargo
        rust-analyzer-unwrapped
        rustfmt
      ];
      RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
    };
}
