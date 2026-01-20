# Contributing

## Feature Development Workflow

1. Create a feature branch in both `openapi2mcp` and `mcp-server-template-ts` (use the same branch name)
2. Update the git submodule in `openapi2mcp` to track your feature branch:

   ```bash
   cd mcp-server-template-ts
   git checkout <feature-branch>
   cd ..
   git add mcp-server-template-ts
   ```

3. Iterate on changes in the generator and template as needed
4. Open a PR in `openapi2mcp` - it will be pinned to the feature branch in `mcp-server-template-ts`
5. Once approved and merged, create a release

## Release Process

1. Ensure the version in both `Cargo.toml` and `package.json` is the next released version on `main`. Do this by:
   1. Update lock files:
      ```bash
      cargo build
      npm i
      ```

   2. Open PR against main. Recommended commit message `release: v<version>`
2. Create the release (this triggers the workflow that publishes to npm):

   ```bash
   gh release create v0.8.0 --generate-notes --latest
   ```

## Releasing mcp-server-template-ts

1. Update `package.json` to use the latest released version of `openapi2mcp`
2. Bump the template version in `package.json`
3. Build and test:

   ```bash
   npm i && npm run build && npm test
   ```

4. Create a PR, get approval, and merge
5. Create the release:

   ```bash
   gh release create v0.x.0 --generate-notes --latest
   ```

6. (Optional) Update the git submodule in `openapi2mcp` to point to the new release
