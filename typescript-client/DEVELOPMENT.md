# TypeScript Client Development Guide

## Generated Code Strategy

This TypeScript client uses a **proto-first** approach where all types and API definitions are generated from Protocol Buffer (`.proto`) files.

### Current Configuration: Generated Files Excluded from Git

The `generated/` directory is currently **excluded from version control** (see `.gitignore`).

**Pros:**
- Proto files remain the single source of truth
- No risk of generated files being out of sync
- Smaller repository size
- Clear separation between source and generated code

**Cons:**
- Developers must run `npm run generate` after cloning
- CI/CD must generate files before building
- Cannot browse types on GitHub without generating first

### Alternative: Include Generated Files in Git

If you prefer to include generated files in version control, you can:

1. Remove `generated/` from `.gitignore`
2. Run `npm run generate`
3. Commit the generated files
4. Add a pre-commit hook to ensure files stay in sync:

```bash
# .git/hooks/pre-commit
#!/bin/bash
cd typescript-client
./scripts/check-proto-sync.sh
```

## Development Workflow

### Initial Setup
```bash
cd typescript-client
npm install
npm run generate  # Generate TypeScript from proto files
npm run build     # Compile TypeScript to JavaScript
```

### After Modifying Proto Files
```bash
# Regenerate TypeScript types
npm run generate

# Verify everything compiles
npm run build

# Run the example to test
npm run example
```

### Before Committing

If generated files are tracked in git:
```bash
# Ensure generated files are up to date
./scripts/check-proto-sync.sh

# Or regenerate them
npm run generate
```

## File Structure

```
typescript-client/
├── src/                    # Source TypeScript files (committed)
│   ├── index.ts           # Main exports
│   ├── nats-client.ts     # NATS client wrapper
│   └── services/          # Service-specific clients
├── generated/             # Generated from proto (gitignored)
│   ├── *.ts              # TypeScript types from proto
│   └── nats-config.ts    # NATS routing configuration
├── dist/                  # Compiled JavaScript (gitignored)
├── scripts/              # Build and utility scripts (committed)
├── examples/             # Usage examples (committed)
└── package.json          # Package configuration (committed)
```

## Proto File Locations

The TypeScript generator reads proto files from:
- `../catalog/proto/` - Catalog service definitions
- `../shared-proto/proto/` - Shared message definitions

## Continuous Integration

The GitHub Actions workflow (`.github/workflows/typescript-client.yml`) automatically:
1. Generates TypeScript from proto files
2. Builds the TypeScript client
3. Runs tests
4. Publishes to npm (if version changed)

## Publishing to NPM

The package is configured to publish as `@rust-commerce/client`.

Before publishing:
1. Update version in `package.json`
2. Ensure generated files are up to date
3. Run `npm run build`
4. Test with `npm run example`

The CI will automatically publish when:
- Changes are pushed to `main` branch
- The version in `package.json` has changed
- The `NPM_TOKEN` secret is configured

## Troubleshooting

### "Cannot find module '../generated/...'"
Run `npm run generate` to create the generated files.

### "Generated files are out of sync"
Run `npm run generate` to regenerate from latest proto files.

### "protoc: command not found"
Install Protocol Buffers compiler:
```bash
# macOS
brew install protobuf

# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# Verify installation
protoc --version
```