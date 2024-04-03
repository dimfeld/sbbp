_list:
  @just --list

filigree:
  ../filigree/target/debug/filigree write

sync-types:
  cd api && cargo run util sync-types

prepare:
  @just filigree
  @just sync-types
