_list:
  @just --list

filigree:
  ../filigree/target/debug/filigree write

prepare:
  cd web && bun install && bun run build

dev-api:
  cargo watch -d 0.1 -x 'lrun serve --dev'

dev-web:
  cd web && bun run dev
