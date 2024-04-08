_list:
  @just --list

filigree:
  ../filigree/target/debug/filigree write

sync-types:
  cd api && cargo run util sync-types

prepare:
  @just filigree
  @just sync-types

dev-api:
  cd api && cargo watch -s 'cargo run --release -- serve'

dev-web:
  cd web && bun run dev
