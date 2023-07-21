{pkgs, ...}: {
  devShells.default = with pkgs;
    mkShell.override {
      stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
    } {
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
