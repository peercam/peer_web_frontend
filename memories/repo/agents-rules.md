# Repo conventions

- Per AGENTS.md: never append `tail`, `head`, `grep` to terminal commands. The harness
  paginates results automatically. Use `tee` to a temp file for later analysis if needed.
