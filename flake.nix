{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    crate2nix = {
      url = "github:nix-community/crate2nix";

      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      systems,
      crate2nix,
      ...
    }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
      pkgs = eachSystem (
        system:
        import nixpkgs {
          inherit system;
        }
      );
    in
    {
      packages = eachSystem (system: {
        backend =
          (crate2nix.tools.${system}.appliedCargoNix {
            name = "biasdo-backend";
            src = ./.;
          }).rootCrate.build;
        frontend = pkgs.${system}.callPackage ./packages/client { };
      });

      devShells = eachSystem (system: {
        default = pkgs.${system}.mkShell {
          packages = with pkgs.${system}; [
            nodejs
            pnpm
            cargo
            rustc
            sqlx-cli

            pkg-config
            openssl
          ];

          shellHook = ''
            pnpm install
          '';
        };
      });
    };
}
