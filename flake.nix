{
  description = "Yet Another Pointer Analysis for LLVM";

  inputs = {
    nixpkgs.url = github:nixos/nixpkgs/23.05;
    levers = {
      url = "github:kquick/nix-levers";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, levers }:
    {
      apps = levers.eachSystem (system: rec
        {
          default = yapall;
          yapall = {
            type = "app";
            program = "${self.packages.${system}.yapall}/bin/yapall";
          };
        });
      packages = levers.eachSystem (system:
        let pkgs = import nixpkgs { inherit system; };
        in rec {
          default = yapall;
          yapall = pkgs.rustPlatform.buildRustPackage {
            pname = "yapall";
            version = "0.0.0";
            src = self;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            LLVM_SYS_140_PREFIX = "${pkgs.llvm_14.dev}/";
            RUSTC_LLVM_14 = "${pkgs.rustc}/bin/rustc";
            buildInputs = [
              pkgs.llvm_14.dev
              pkgs.libxml2
              pkgs.zlib
            ];
            # Disable nix default of hardening, which will cause compilation of
            # various tests at -O0 to fail because _FORTIFY_SOURCE requires some
            # level of optimization.
            hardeningDisable = [ "all" ];
            nativeCheckInputs = [
              pkgs.clang_14
            ];
            meta = with pkgs.lib; {
              description = "Yet Another Pointer Analysis for LLVM";
              license = licenses.bsd3;
              homepage = "https://github.com/GaloisInc/yapall";
            };
          };
        });
      devShells = levers.eachSystem (system:
        let pkgs = import nixpkgs { inherit system; };
        in
          {
            # buildRustPackage doesn't provide assistance here, so simply
            # manually create a shell.
            default = pkgs.mkShell {
              packages = [
                pkgs.rustc
                pkgs.cargo
                pkgs.llvm_14
                pkgs.clang_14
                pkgs.llvm_14.dev
                pkgs.libxml2
                pkgs.zlib
                pkgs.lit
                pkgs.rust-analyzer
              ];
              LLVM_SYS_140_PREFIX = "${pkgs.llvm_14.dev}/";
              RUSTC_LLVM_14 = "${pkgs.rustc}/bin/rustc";
              hardeningDisable = [ "all" ];
            };
          });
    };
}
