{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
  };
  outputs =
    inputs:
    let
      eachSystem = inputs.nixpkgs.lib.genAttrs (import inputs.systems);
      pkgs = eachSystem (
        system:
        import inputs.nixpkgs {
          inherit system;
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
            nodejs_24
            pnpm
          ];

          shellHook = ''
            crate2nix generate
            bun install --frozen-lockfile
          '';
        };
      });
    };
}
