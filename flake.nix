{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";

    bun2nix = {
      url = "github:nix-community/bun2nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        systems.follows = "systems";
      };
    };
  };
  nixConfig = {
    extra-substituters = [
      "https://cache.nixos.org"
      "https://nix-community.cachix.org"
    ];
    extra-trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
  outputs =
    inputs:
    let
      eachSystem = inputs.nixpkgs.lib.genAttrs (import inputs.systems);
      pkgs = eachSystem (
        system:
        import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.bun2nix.overlays.default ];
        }
      );
    in
    {
      packages = eachSystem (system: {
        backend =
          (pkgs.${system}.callPackage ./Cargo.nix {
            inherit (inputs) nixpkgs;
            pkgs = pkgs.${system};
          }).rootCrate.build;
        frontend = pkgs.${system}.callPackage ./packages/client { };
      });

      devShells = eachSystem (system: {
        default = pkgs.${system}.mkShell {
          nativeBuildInputs = with pkgs.${system}; [
            cargo
            rustc
          ];

          packages = with pkgs.${system}; [
            crate2nix
            bun
          ];

          shellHook = ''
            crate2nix generate
            bun install --frozen-lockfile
          '';
        };
      });
    };
}
