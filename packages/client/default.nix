{
  stdenv,
  pnpm,
  nodejs,
  lib,
  apiUrl ? "https://biasdo-api.daimond113.com",
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

  VITE_API_URL = apiUrl;

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
