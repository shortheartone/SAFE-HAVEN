# Contract Backups

`scripts/backup_contract.sh` creates a daily JSON snapshot of the deployed SAFE-HAVEN contract and stores it in IPFS or an S3-compatible cloud bucket.

The export contains the current logical state available through the contract's public read-only methods: configuration, active depositors, and both timestamp-based and ledger-based deposits. Soroban does not provide contract code with arbitrary storage iteration, so expired or removed records that are no longer returned by those methods are not included. Keep the contract ID, network, export timestamp, schema version, and storage version with every snapshot to support recovery decisions.

## Configuration

Required:

```bash
export CONTRACT_ID="C..."
export STELLAR_SOURCE="backup-reader"
```

Optional settings include `NETWORK` (default `testnet`), `RPC_URL`, `PAGE_SIZE` (default `25`), and `BACKUP_DIR` (default `backups`). The Stellar CLI identity must be configured locally and only needs read access for these queries.

For IPFS, run an IPFS API endpoint and optionally set `IPFS_API_URL` (default `http://127.0.0.1:5001`):

```bash
BACKUP_STORAGE=ipfs bash scripts/backup_contract.sh
```

For S3 or an S3-compatible service, configure the AWS CLI credentials and set `S3_BUCKET`; `S3_PREFIX` is optional:

```bash
BACKUP_STORAGE=s3 S3_BUCKET=my-safe-haven-backups bash scripts/backup_contract.sh
```

## Daily Schedule

Run the command from the repository root with cron. Use an absolute path and redirect output to an operational log:

```cron
15 2 * * * cd /opt/safe-haven && CONTRACT_ID="C..." STELLAR_SOURCE="backup-reader" NETWORK="testnet" BACKUP_STORAGE="ipfs" /opt/safe-haven/scripts/backup_contract.sh >> /var/log/safe-haven-backup.log 2>&1
```

Review the resulting IPFS CID or cloud object, monitor failed runs, and periodically test that a snapshot can be downloaded and parsed with `jq`.

## Encryption at Rest

Sensitive fields (for example social security numbers and credit card data) are encrypted at the field level before they are persisted, so a raw database dump or backup snapshot never contains them in plaintext. The application decrypts them on read and re-encrypts on write; encrypted values cannot be read directly from the database.

### Key Management

- The encryption key is supplied to the application through the `ENCRYPTION_KEY` environment variable and is never committed to the repository or written into backups.
- Store the key in a secrets manager (or the deployment platform's secret store) and inject it at runtime; restrict access to the application service only.
- Backups contain only ciphertext. To restore and use encrypted fields, the same key must be available to the restoring application; without it the encrypted columns remain unreadable.
- Key rotation is out of scope for this change; when rotating, re-encrypt existing values with the new key before retiring the old one.

### Search on Encrypted Fields

Exact-match lookups on encrypted fields use a deterministic blind index (a keyed hash of the normalized value) stored alongside the ciphertext, so search continues to work without exposing plaintext. Range or partial-match queries are not supported on encrypted fields.

### Backup / Restore with Encryption

1. Take a snapshot as described above; encrypted fields appear only as ciphertext plus their blind index.
2. Restore the snapshot into a database and start the application with the same `ENCRYPTION_KEY`.
3. Verify that the application can decrypt and display the sensitive fields and that exact-match search returns the expected records.
