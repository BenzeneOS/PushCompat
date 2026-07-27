{ pkgs }:

let
  composition = pkgs.androidenv.composeAndroidPackages {
    platformVersions = [ "36" ];
    buildToolsVersions = [ "36.0.0" ];
    includeNDK = true;
  };
  sdk = composition.androidsdk;
  sdkRoot = "${sdk}/libexec/android-sdk";
  ndkRoot = "${sdkRoot}/ndk-bundle";
  ndkHostTag = if pkgs.stdenv.hostPlatform.isDarwin then "darwin-x86_64" else "linux-x86_64";
in
{
  inherit
    composition
    ndkHostTag
    ndkRoot
    sdk
    sdkRoot
    ;

  ndkToolchain = "${ndkRoot}/toolchains/llvm/prebuilt/${ndkHostTag}/bin";
}
