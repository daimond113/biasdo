{
  stdenv,
  pnpm,
  pnpmConfigHook,
  fetchPnpmDeps,
  nodejs,
  lib,
  apiUrl ? "https://biasdo-api.daimond113.com/v0",
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
    pnpm
    pnpmConfigHook
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
  pnpmDeps = fetchPnpmDeps {
    inherit (finalAttrs) pname version src;
    fetcherVersion = 4;
    hash = "sha256-0VI+DW55rfFe3RF1wmn0sn0HWPeb31Bd/Dvt8ORDdGI=";
  };
})
