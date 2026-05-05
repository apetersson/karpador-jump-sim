# Commit Rule
Commit as "conventional commit" style. No co-author.
Any commit should clearly state the reason for the change - and be well enough defined that a very competent LLM could re-author the changes from the previous state. Take hints from the conversation of user intent. 
Ideally max 3 lines, not more. If that is not possible, split up the commit into multiple.

# Deploy to GitHub Pages

GitHub Pages is configured to serve `main` from `/docs` at https://apetersson.github.io/karpador-jump-sim/.
Do not deploy through a `gh-pages` branch; that branch is obsolete and should not be recreated.

1. Build and validate the web UI:
   ```sh
   cd web
   pnpm install
   pnpm run typecheck
   pnpm run lint
   pnpm run build
   pnpm run test
   ```
2. Sync the built site into `/docs` from the repo root:
   ```sh
   rm -f web/dist/.DS_Store docs/.DS_Store
   rsync -a --delete --exclude '.DS_Store' web/dist/ docs/
   ```
3. Commit the docs deployment on `main`:
   ```sh
   git add docs
   git commit -m "chore(web): deploy frontend to docs"
   git push origin main
   ```
4. Optionally verify the Pages build and served asset hashes:
   ```sh
   gh api repos/apetersson/karpador-jump-sim/pages --jq '{status,source,html_url}'
   gh api repos/apetersson/karpador-jump-sim/pages/builds/latest --jq '{status,commit,error}'
   curl -fsSL 'https://apetersson.github.io/karpador-jump-sim/?cachebust='$(date +%s) | rg -o 'assets/[^" ]+'
   ```
