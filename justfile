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
  cd api && cargo watch -d 0.1 -x 'lrun serve --dev'

dev-web:
  cd htmx && bun run dev
