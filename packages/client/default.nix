{
  stdenv,
  pnpm,
  nodejs,
  lib,
  ...
}:
let
  package = lib.importJSON ./package.json;
in
stdenv.mkDerivation (finalAttrs: {
  pname = package.name;
  version = package.version;

  src = ./../..;

  nativeBuildInputs = [
    nodejs
    pnpm.configHook
  ];

  buildPhase = ''
    runHook preBuild

    pnpm --filter=@biasdo/client build

    runHook postBuild
  '';

  installPhase = ''
    mkdir -p $out/build
    cp -R packages/client/build $out
  '';

  pnpmWorkspaces = [ "@biasdo/client" ];
  pnpmDeps = pnpm.fetchDeps {
    inherit (finalAttrs) pname version src;
    fetcherVersion = 2;
    hash = "sha256-OZuwS+QEgm8SS3KGotrkAThcca5inq+pE1ViSvrMuGc=";
  };
})
