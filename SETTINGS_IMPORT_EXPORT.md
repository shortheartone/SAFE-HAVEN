# Settings Import/Export Feature

## Overview

The Settings page allows users to export and import their deposit templates and frequently used tokens, with optional encryption for security.

## Features Implemented

### Export Settings
- **Unencrypted JSON export**: Export settings as plain JSON for easy sharing
- **Encrypted export**: Password-protected export using AES-256-GCM encryption
- Download as file with timestamped filename
- Shows current count of templates and tokens

### Import Settings
- Load settings from unencrypted or encrypted files
- Automatic detection of encrypted files (via base64 validation)
- Password prompt for encrypted files
- Two import modes:
  - **Replace**: Clear all existing settings and replace with imported
  - **Merge**: Combine imported settings with existing (deduplicates automatically)
- Full validation before applying

### Settings Preview
- Show detailed preview of imported settings before applying
- Display merge information (how many new items will be added)
- Show capacity warnings if limits are exceeded
- Display export metadata (version, export date)

### Security
- **Encryption**: AES-256-GCM with PBKDF2 key derivation
  - Random salt (128 bits) per encryption
  - Random IV (128 bits) per encryption
  - 100,000 PBKDF2 iterations for key derivation
- **Local storage**: Settings are never stored in the cloud
- **No tracking**: No telemetry or usage analytics

## File Structure

### New Files Created

```
frontend/src/
├── pages/SettingsPage.tsx              # Main settings page component
├── components/SettingsPreview.tsx      # Preview component for imports
├── lib/encryption.ts                   # AES-GCM encryption utilities
├── lib/settingsExport.ts               # Export/import logic
└── __tests__/
    ├── settingsExport.test.ts          # 26 unit tests for export/import
    └── encryption.test.ts              # 20 unit tests for encryption
```

### Modified Files

```
frontend/src/
├── types.ts                            # Added UserSettings, DepositTemplate types
├── App.tsx                             # Added SettingsPage route and import
└── components/TabNav.tsx               # Added Settings tab with icon
```

## Type Definitions

### UserSettings
```typescript
interface UserSettings {
  version: string                    // "1.0.0"
  exportedAt: number                 // Unix timestamp
  depositTemplates: DepositTemplate[]
  frequentTokens: string[]
}
```

### DepositTemplate
```typescript
interface DepositTemplate {
  tokenAddress: string               // Stellar token address
  lockDurationSeconds: number        // Lock duration in seconds
  penaltyBps: number                 // 0-10000 (basis points)
  label?: string                     // Optional friendly name
}
```

## API Reference

### settingsExport.ts

#### `exportSettingsAsJson(templates, frequentTokens): string`
- Exports settings as formatted JSON string
- Returns pretty-printed JSON with 2-space indentation

#### `exportSettingsEncrypted(templates, frequentTokens, password): Promise<string>`
- Encrypts settings with password
- Returns base64-encoded encrypted data
- Throws if password is empty

#### `validateSettings(data): SettingsValidation`
- Validates imported settings structure
- Checks array types, value ranges, required fields
- Returns validation result with errors and warnings

#### `parseSettingsFromJson(jsonString): SettingsWithMergeInfo`
- Parses JSON string and validates
- Throws on invalid JSON or structure

#### `parseSettingsFromEncrypted(encryptedData, password): Promise<SettingsWithMergeInfo>`
- Decrypts and parses encrypted settings
- Throws on decryption failure or invalid password

#### `mergeSettings(existing, imported, mode): MergedSettings`
- `mode='replace'`: Returns imported settings as-is
- `mode='merge'`: Combines with existing, removes duplicates, respects capacity limits

#### `generateExportFilename(encrypted): string`
- Generates timestamped filename
- Format: `safe-haven-settings-YYYY-MM-DD.json` or `.enc.json`

### encryption.ts

#### `encryptData(data: string, password: string): Promise<string>`
- Encrypts data using AES-256-GCM
- Returns base64-encoded result containing: salt (16B) + IV (16B) + ciphertext
- Uses PBKDF2 with 100,000 iterations for key derivation

#### `decryptData(encryptedBase64: string, password: string): Promise<string>`
- Decrypts base64-encoded encrypted data
- Throws if wrong password or corrupted data
- Returns original plaintext string

#### `isValidBase64(str: string): boolean`
- Checks if string is valid base64

## Usage

### Export Settings (Unencrypted)
```
1. Go to Settings tab
2. Click "Export Settings" button
3. JSON file downloads with all templates and tokens
```

### Export Settings (Encrypted)
```
1. Go to Settings tab
2. Check "Encrypt export with password"
3. Enter a password
4. Click "Export Settings" button
5. Encrypted JSON file downloads (password required to import)
```

### Import Settings (Unencrypted)
```
1. Go to Settings tab
2. Click file input and select a .json file
3. Click "Load Settings"
4. Choose Merge or Replace mode
5. Review preview
6. Click "Apply Settings"
```

### Import Settings (Encrypted)
```
1. Go to Settings tab
2. Click file input and select a .enc.json file
3. Enter the encryption password
4. Click "Load Settings"
5. Choose Merge or Replace mode
6. Review preview
7. Click "Apply Settings"
```

## Constraints & Limits

| Constraint | Value | Reason |
|---|---|---|
| Max templates per export | 50 | Prevent excessive JSON size |
| Max frequent tokens | 20 | Prevent excessive JSON size |
| Template label length | Unlimited | Used for display only |
| Max lock duration | 5 years | Contract constraint |
| Min lock duration | 60 seconds | Contract constraint |
| Penalty range | 0-10,000 bps | Contract constraint |
| Encryption password | No length limit | Any password accepted |
| File size limit | Browser dependent | Usually > 100MB |

## Data Validation

### On Import

1. **Structure validation**
   - Must be valid JSON object
   - Required fields: `version`, `depositTemplates`, `frequentTokens`

2. **Type validation**
   - `depositTemplates` must be array
   - `frequentTokens` must be array
   - Each template must have correct field types

3. **Value validation**
   - `penaltyBps` must be 0-10,000
   - `lockDurationSeconds` must be > 0
   - Token addresses must be strings (not validated for format)

4. **Warnings (non-fatal)**
   - Version mismatch
   - Empty arrays
   - Missing export timestamp

### On Merge

1. Deduplicates templates by (tokenAddress, lockDurationSeconds, penaltyBps)
2. Deduplicates frequent tokens by address
3. Respects capacity limits (50 templates, 20 tokens)
4. Removes excess items with warning

## Testing

### Test Coverage

**settingsExport.test.ts** (26 tests)
- ✓ Export data creation with timestamp
- ✓ JSON serialization
- ✓ Encryption option
- ✓ Filename generation (unencrypted/encrypted)
- ✓ Validation (structure, types, values)
- ✓ JSON parsing and error handling
- ✓ Settings merging (replace vs merge modes)
- ✓ Deduplication logic
- ✓ Capacity limit enforcement
- ✓ Sanitization of incomplete settings
- ✓ Full workflow (export → parse → merge → import)

**encryption.test.ts** (20 tests)
- ✓ Encryption with password
- ✓ Random salts per encryption
- ✓ Random IVs per encryption
- ✓ Decryption with correct password
- ✓ Decryption fails with wrong password
- ✓ Handles empty strings
- ✓ Handles long data (100KB+)
- ✓ Handles special characters and emoji
- ✓ Handles non-ASCII characters (Chinese, Russian, etc.)
- ✓ Base64 validation
- ✓ Corrupted data detection
- ✓ Round-trip encryption/decryption

### Run Tests

```bash
# Run settings export tests
npx vitest run src/__tests__/settingsExport.test.ts

# Run encryption tests
npx vitest run src/__tests__/encryption.test.ts

# Run all tests in watch mode
npm run test:watch
```

## Browser Compatibility

- **Web Crypto API**: Required for encryption (supported in all modern browsers)
- **Chrome/Edge/Firefox/Safari**: Full support
- **IE 11**: Not supported (Web Crypto API not available)

## Security Considerations

1. **Passwords are never stored**: Only used during encryption/decryption session
2. **No network transmission**: All encryption/decryption happens locally in browser
3. **Strong encryption**: AES-256-GCM is industry-standard authenticated encryption
4. **Proper key derivation**: PBKDF2 with 100,000 iterations (equivalent to 4+ seconds on modern hardware)
5. **Random IVs**: Each encryption produces different ciphertext even for same input
6. **No recovery mechanism**: Lost passwords cannot be recovered (intentional design)

## Future Enhancements

- Custom password strength requirements
- Support for more file formats (CSV, YAML)
- Backup to browser's local storage
- Settings sync across devices (with cloud storage)
- Settings versioning and history
- Bulk template operations (add, edit, delete)
- Template categories/groups
- Export as template library (shareable)

## Troubleshooting

### Encrypted file won't decrypt
- Verify password is correct
- Ensure file was not corrupted during transfer
- Try exporting again with same password

### Import preview shows fewer items than expected
- Check capacity limits (50 templates, 20 tokens max)
- Review warnings in preview for capacity overflow
- Consider multiple imports or remove existing settings first

### Settings don't persist after import
- Settings are stored locally in browser (check privacy settings)
- Browser privacy mode doesn't persist storage
- Use persistent browser profile to keep settings

### File download didn't start
- Check browser's download settings
- Try a different browser
- Check browser console for errors
