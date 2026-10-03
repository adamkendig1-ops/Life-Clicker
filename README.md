# Life Clicker Update Feed

This public repository hosts the Life Clicker launcher update feed.

## Stable channel

Manifest:
`https://raw.githubusercontent.com/adamkendig1-ops/Life-Clicker/main/stable/latest.json`

The Windows launcher checks the configured manifest once at startup and may be manually rechecked from the launcher UI.

## Release order

1. Build and validate the .lcupdate package.
2. Run save-compatibility tests.
3. Calculate exact SHA-256 and byte size.
4. Publish the .lcupdate package.
5. Verify the published package.
6. Publish `stable/latest.json` **last**.

Publishing the manifest last prevents launchers from seeing a release before its package is available.

## Save safety

Life Clicker updates use versioned save schemas, pre-update backups, validation, and rollback/recovery protections.
