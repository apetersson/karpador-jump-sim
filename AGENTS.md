# Commit Rule
Commit as "conventional commit" style. No co-author.
Any commit should clearly state the reason for the change - and be well enough defined that a very competent LLM could re-author the changes from the previous state. Take hints from the conversation of user intent. 
Ideally max 3 lines, not more. If that is not possible, split up the commit into multiple.

# Deploy to GitHub Pages

1. Build the web UI: `cd web && pnpm install && pnpm build`
2. Create a temp worktree on `gh-pages`: `git worktree add -B gh-pages /tmp/gh-pages HEAD`
3. Copy the built site into it, commit, and push:
   ```sh
   cd /tmp/gh-pages
   find . -mindepth 1 -maxdepth 1 ! -name '.git' -exec rm -rf {} +
   cp -R $REPO_ROOT/web/dist/.* .
   git add -A && git commit -m "chore: deploy web UI"
   git push origin gh-pages
   ```
4. Clean up: `git worktree remove /tmp/gh-pages`

The site is served from the `gh-pages` branch root at https://apetersson.github.io/karpador-jump-sim/
