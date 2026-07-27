#!/usr/bin/env nu

# Regenerates the MCS/checkin protobuf bindings.

const GENERATED = [ android_checkin checkin mcs ]
const HEADER = "// Regenerate from the repository root: nix develop -c nu nix/regen-proto.nu"

def main [] {
   let repo_root = ($env.FILE_PWD | path join ".." | path expand)
   let proto_dir = ($repo_root | path join "crates/listener/src/proto")
   let tool_root = ($repo_root | path join "target/proto-tools")

   # pb-rs 0.10.0 trips a lint that is an error on current toolchains.
   let rustflags = $"($env.RUSTFLAGS? | default '') -A dangerous_implicit_autorefs"
   with-env { RUSTFLAGS: $rustflags } {
      cargo install pb-rs --version 0.10.0 --locked --root $tool_root
   }

   let pb_rs = ($tool_root | path join "bin/pb-rs")
   for name in $GENERATED {
      (^$pb_rs -s -D -I $proto_dir
         -o ($proto_dir | path join $"($name).rs")
         ($proto_dir | path join $"($name).proto"))
   }
   rm -f ($proto_dir | path join "mod.rs")

   for name in $GENERATED {
      let file = ($proto_dir | path join $"($name).rs")
      mut text = (open --raw $file | decode utf-8)

      $text = ($text
         | str replace --all "checkin_proto::DeviceType" "DeviceType"
         | str replace --all "checkin_proto::AndroidCheckinProto" "super::android_checkin::AndroidCheckinProto")

      if $name == "mcs" {
         $text = ($text | str replace "#![allow(non_camel_case_types)]"
            "#![allow(non_camel_case_types)]\n#![allow(dead_code)]")
      }

      # Deriving Eq is what makes the derive_partial_eq_without_eq suppression
      # dead, so the two have to move together. The reason rewrite runs last so
      # it also covers the dead_code allow inserted above.
      $text = ($text
         | str replace --all "PartialEq, Clone" "PartialEq, Eq, Clone"
         | str replace --all --regex '(?m)^#\[(allow|expect)\(clippy::derive_partial_eq_without_eq[^\n]*\n' ''
         | str replace --all --regex '(?m)^((#*!\[allow\([^]]*))(\)\])$' '${2}, reason = "generated protobuf code"${3}')

      $text = ($text | lines | insert 1 $HEADER | str join "\n" | $in + "\n")
      $text | save --force --raw $file
   }

   let files = ($GENERATED | each {|name| $proto_dir | path join $"($name).rs" })
   python3 ($repo_root | path join "nix/normalize-proto.py") ...$files
}
