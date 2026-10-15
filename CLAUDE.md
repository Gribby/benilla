# Working rules

- Consult the advisor before locking a plan, after the same error happens twice, and
  before calling a task done.
- The server (`Gribby/mangos-classic`) is built with MSVC on Windows: never use `near`, `far` or
  `small` as C++ identifiers there (Windows macros), and run the check in that repo's CLAUDE.md
  before every server commit.
