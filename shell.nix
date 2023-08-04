{ pkgs ? import <nixpkgs> { }
, unstable ? import <unstable> { }
}:

pkgs.mkShell {
  LLVM_SYS_140_PREFIX = "${pkgs.llvm_14.dev}/";
  RUSTC_LLVM_14 = "${pkgs.rustc}/bin/rustc";
  hardeningDisable = [ "all" ];
  buildInputs = [
    pkgs.llvm_14.dev
    pkgs.libxml2
  ];
  nativeBuildInputs = [ 
    pkgs.lit
    pkgs.rust-analyzer 
    pkgs.rustup 
    pkgs.rustc
  ];
}
