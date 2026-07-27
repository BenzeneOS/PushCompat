{
  android,
  pkgs,
  toolchain,
}:

pkgs.mkShell {
  packages = toolchain ++ [
    pkgs.rust-analyzer
    android.sdk

    # nix/*.nu and the normalizer they hand off to.
    pkgs.nushell
    pkgs.python3
  ];

  ANDROID_SDK_ROOT = android.sdkRoot;
  ANDROID_NDK_ROOT = android.ndkRoot;
  CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "${android.ndkToolchain}/aarch64-linux-android21-clang";
  CC_aarch64_linux_android = "${android.ndkToolchain}/aarch64-linux-android21-clang";
  AR_aarch64_linux_android = "${android.ndkToolchain}/llvm-ar";

  shellHook = /* sh */ ''
    echo "PushCompat Development Shell"
    echo ""
    echo "  nix build .#pushcompat-bridge   - Build bridge server"
    echo "  cargo build -p pushcompat-jni --target aarch64-linux-android"
    echo "  nu nix/regen-proto.nu            - Regenerate the MCS protobuf bindings"
    echo "  nu nix/check-vendored-versions.nu - Check listener deps against the AOSP tree"
  '';
}
