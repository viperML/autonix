{pkgs, ...}: {
  devShells.default = with pkgs;
    mkShell {
      packages = [
        rustc
        cargo
        rust-analyzer-unwrapped
        rustfmt
        pkg-config
        openssl
        cargo-nextest
      ];
      RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
    };
}
