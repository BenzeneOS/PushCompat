#!/usr/bin/env nu

# Soong can only link crates vendored into the Android tree, so every transitive
# dependency of the listener has to exist there at a semver-compatible version.
# Feature variants additionally have to match AOSP exactly, or a variant silently
# diverges from the base crate it patches.

def crate-version [dir: path]: nothing -> string {
   open ($dir | path join "Cargo.toml") | get package.version
}

def crate-name [dir: path]: nothing -> string {
   open ($dir | path join "Cargo.toml") | get package.name
}

def semver-compatible [requested: string, vendored: string]: nothing -> bool {
   let a = ($requested | split row -r '[-+]' | first | split row "." )
   let b = ($vendored | split row -r '[-+]' | first | split row ".")
   if ($a.0 != $b.0) {
      false
   } else if ($a.0 == "0") {
      ($a | get -o 1) == ($b | get -o 1)
   } else {
      true
   }
}

def main [] {
   let repo_root = ($env.FILE_PWD | path join ".." | path expand)

   let build_top = if "ANDROID_BUILD_TOP" in $env {
      $env.ANDROID_BUILD_TOP
   } else if ($repo_root | str ends-with "/external/rust/pushcompat") {
      $repo_root | path join ".." ".." ".." | path expand
   } else {
      print -e "error: set ANDROID_BUILD_TOP when running outside the Android tree"
      exit 2
   }

   let android_crates = ($env.ANDROID_CRATES_IO_ROOT?
      | default ($build_top | path join "external/rust/android-crates-io"))
   let benzeneos_crates = ($env.BENZENEOS_CRATES_ROOT?
      | default ($build_top | path join "external/rust/benzeneos-crates"))
   let variant_root = ($benzeneos_crates | path join "feature_variants/crates")
   let crate_roots = [
      ($android_crates | path join "crates")
      ($android_crates | path join "extra_versions/crates")
      ($benzeneos_crates | path join "crates")
      ($benzeneos_crates | path join "extra_versions/crates")
      $variant_root
   ]

   for root in $crate_roots {
      if ($root | path type) != "dir" {
         print -e $"error: vendored crate root does not exist: ($root)"
         exit 2
      }
   }

   let requested = (
      cargo tree --manifest-path ($repo_root | path join "Cargo.toml")
         -p pushcompat-listener -e no-dev,no-build --prefix none --format '{p}'
      | lines
      # cargo marks a subtree it already printed with a trailing "(*)".
      | each {|line| $line | str replace -r ' \(\*\)$' '' }
      | parse "{name} v{version}"
      | where name != "pushcompat-listener"
      | uniq-by name version
   )

   mut problems = []

   for dep in $requested {
      let vendored = (
         $crate_roots
         | each {|root| $root | path join $dep.name }
         | where {|dir| ($dir | path type) == "dir" }
         | each {|dir| crate-version $dir }
      )
      if ($vendored | is-empty) {
         $problems = ($problems | append $"MISSING ($dep.name) requested=($dep.version)")
      } else if not ($vendored | any {|v| semver-compatible $dep.version $v }) {
         $problems = ($problems | append
            $"INCOMPATIBLE ($dep.name) requested=($dep.version) vendored=($vendored | str join ' ')")
      }
   }

   for dir in (ls $variant_root | where type == dir | get name) {
      let name = (crate-name $dir)
      let variant = (crate-version $dir)
      let base = ($android_crates | path join "crates" $name)
      if ($base | path type) != "dir" {
         $problems = ($problems | append $"VARIANT_BASE_MISSING ($name) variant=($variant)")
         continue
      }
      let aosp = (crate-version $base)
      if $variant != $aosp {
         $problems = ($problems | append $"VARIANT_DRIFT ($name) variant=($variant) aosp=($aosp)")
      }
   }

   if ($problems | is-not-empty) {
      $problems | each {|line| print -e $line }
      exit 1
   }

   print "vendored listener dependencies are compatible and feature variants match AOSP exactly"
}
