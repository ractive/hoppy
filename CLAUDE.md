# Documentation
Keep all the documentation in the folder ./hoppy-knowledgebase as *.md markdown files with frontmatter in yaml format with properties of type text, numbers, checkboxes, dates, dates and times, or lists.

Use is as your second brain and document there:
- outcome of online researches
- design decisions
- iteration planning with one file per iteration and a markdown task lists for steps, tasks, ACs

Organize content in suitable subfolders. Create markdown links to other related files.
Keep it compatible with obsidian.

# Rust
Use the rust-analyzer-lsp plugin with its rust language server for code intelligence and analysis like:
"analyze this Rust code", "find all references to this function", "go to the definition of this struct", or "check for clippy warnings in my project".

clippy (with -D warnings) and fmt must have run successfully before commiting or creating a PR.
