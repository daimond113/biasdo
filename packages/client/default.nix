{
  stdenv,
  bun2nix,
  lib,
  ...
}:
let
  package = lib.importJSON ./package.json;
in
stdenv.mkDerivation {
  pname = package.name;
  version = package.version;

  src = ./../..;

  nativeBuildInputs = [
    bun2nix.hook
  ];

  bunDeps = bun2nix.fetchBunDeps {
    bunNix = ./../../bun.nix;
  };

  bunNodeModulesInstallPhase = ''
    bun install --verbose
  '';

  buildPhase = ''
    cd packages/client
    bun run build
  '';

  installPhase = ''
    mkdir -p $out/build

    cp -R packages/client/build $out
  '';
}