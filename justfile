_list:
  @just --list

run:
  cd web && bun run build
  cargo run --release serve

filigree:
  ../filigree/target/debug/filigree write

prepare:
  cd web && bun install && bun run build

dev-api:
  cargo watch -d 0.1 -x 'lrun serve --dev'

dev-web:
  cd web && bun run dev
