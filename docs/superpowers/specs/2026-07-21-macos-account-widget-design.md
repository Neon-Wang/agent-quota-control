# macOS Account Widget Design

## Goal

Add a native macOS WidgetKit extension to Agent Quota Control. Each widget
instance selects one monitored upstream account in the system widget editor,
similar to choosing a location for the Weather widget. The widget renders the
same account quota snapshot used by the matching card in the app overview.

## Product Decisions

- One widget instance selects one monitored account. Multiple widget instances
  may select different accounts.
- Accounts may belong to Kimi Code or Codex.
- The widget supports `systemSmall`, `systemMedium`, and `systemLarge`.
- All sizes use the same card snapshot. Small and medium widgets show a
  deliberate subset; large shows every field from the overview card.
- The Tauri host app remains the only process that reads credentials or calls
  upstream providers.
- The widget is read-only. Tapping it opens the app overview.

## Architecture

### Host App

The Rust backend owns monitored accounts, credentials, provider requests,
estimates, proxy status, snapshot publication, and refresh scheduling. The
React overview renders one quota card per monitored account from the backend's
card snapshots.

### Shared Snapshot Directory

The host app writes a versioned, redacted JSON snapshot to
`~/Library/Application Support/io.ccswitch.agent-quota-control/` using a
temporary file followed by an atomic rename. The sandboxed widget receives a
read-only home-relative file exception for this exact directory. This avoids a
provisioning-profile dependency while keeping the extension unable to write
host state. The snapshot contains account display metadata and card data only.
It never contains API keys, OAuth tokens, upstream response bodies, or private
filesystem paths.

### Widget Extension

A native SwiftUI WidgetKit extension reads the shared JSON. A
`WidgetConfigurationIntent` has one `AppEntity` account parameter. Its
`EntityQuery` lists the accounts in the shared snapshot, providing the same
dynamic selection behavior used by configurable system widgets.

The extension is compiled against WidgetKit, SwiftUI, and AppIntents and
embedded in the Tauri application under `Contents/PlugIns`. The executable is
linked through `_NSExtensionMain`; linking it as a normal Swift executable
causes the process to exit before WidgetKit can return descriptors. The bundle
pipeline extracts App Intents metadata, signs nested code first, and signs the
host last.

## Account Model

`MonitorAccount` contains:

- a stable UUID;
- `service`: `kimi` or `codex`;
- an editable display name;
- an optional redacted provider account identifier;
- a credential reference, never the credential itself;
- an enabled flag;
- creation and last-success timestamps.

Kimi accounts are added with a display name and API key. Secrets are stored per
account in Keychain or the existing encrypted vault backend. Codex accounts are
added by importing the current Codex CLI login into an app-owned per-account
Keychain item. Importing again after changing the CLI login creates another
account rather than replacing an existing account.

Existing single-account Kimi and Codex configurations migrate to stable default
account records. Migration preserves the existing secret source and does not
delete or overwrite credentials. A legacy credential may be moved only after a
new per-account item is written and read back successfully.

Deleting an account removes it from the entity query. An installed widget that
still references that stable UUID shows an account-removed state and never
silently switches to another account.

## Shared Card Contract

The shared document has this logical shape:

```text
schemaVersion
generatedAt
accounts[]
  id
  service
  displayName
  providerIdentityHint
cardsByAccount{}
  accountId
  service
  serviceDisplayName
  accountDisplayName
  status
  tiers[]
  weeklyEstimate
  proxy
  queriedAt
  lastSuccessfulAt
  errorMessage
```

`CardSnapshot` is the canonical presentation contract. Rust constructs it once.
Both `QuotaCard.tsx` and the Swift widget map this contract to their native view
systems; neither recomputes quota state, estimate state, or error precedence.

The snapshot schema uses camelCase and a monotonically increasing version.
Swift rejects unsupported future versions with an update-required state.

## Approved Visual Direction

The selected direction is a data-instrument layout. It follows the system
light or dark appearance instead of forcing one color scheme. Light mode uses
a cool white surface and dark neutral text. Dark mode uses a charcoal surface
and soft white text. Green, amber, red, and gray appear only as semantic state
colors; the service color must not tint the entire card.

The hierarchy is:

1. provider icon and selected account;
2. current utilization as the primary value;
3. projected utilization and exhaustion risk as a separate region;
4. connection and freshness metadata at the bottom.

Repeated provider and account labels are allowed only when the user has kept
the provider name as the account display name. The layout must not introduce a
second decorative title.

Utilization colors are green below 70%, amber from 70% through 89%, and red at
90% or above. Sufficiency is green for enough, amber for tight, red for not
enough, and gray for unknown. A projection above 100% is always red. Stale or
failed refreshes retain the last successful values and use a bounded amber
status treatment rather than recoloring the whole widget.

## Display By Family

### Small

- service icon;
- account display name;
- current weekly utilization as the dominant value and meter;
- projected utilization or sufficiency conclusion;
- relative update time.

### Medium

- a two-column instrument layout;
- current weekly utilization and meter on the left;
- projected utilization, sufficiency, and exhaustion hint on the right;
- connection summary and relative update time in the right-column footer.

### Large

- all overview-card fields;
- every quota tier and reset label;
- projected utilization and estimate explanation;
- proxy detail;
- exact query/update time;
- full non-sensitive error state.

All layouts use stable family dimensions, truncation for account names, Dynamic
Type-aware text, semantic colors, monospaced numerals, and no decorative UI
that competes with quota status. Login-expired and no-data states replace
metrics with an explicit action message rather than showing invented zeros.

## Refresh And Error Behavior

The host keeps its five-minute scheduler and refreshes enabled accounts. After
each refresh attempt, account mutation, or rename, it writes one complete
snapshot and asks WidgetKit to reload the widget timeline. A small native helper
or bridge owned by the host performs the `WidgetCenter` reload call; failure to
notify does not invalidate the snapshot.

WidgetKit still controls actual delivery according to its system budget. The
extension does not perform provider network requests. A snapshot older than 15
minutes is marked stale while preserving the last successful values.

Error precedence:

1. unsupported snapshot version: update-required;
2. selected account missing: account-removed;
3. credential invalid and no usable values: login-expired;
4. no successful values: no-data;
5. refresh failed with successful history: stale values plus update-failed;
6. otherwise: fresh values.

Errors are represented by bounded enums and user-facing messages. Raw provider
bodies and secrets are logged only under existing redaction rules and never
cross the shared-snapshot boundary.

## Packaging

The `macos-widget/` package contains the WidgetKit extension, shared Swift
contract code, tests, metadata extraction inputs, assets, entitlements, and the
minimal WidgetCenter reload bridge. A deterministic script compiles the
extension for the current architecture, links `_NSExtensionMain`, extracts App
Intents metadata, and prepares the `.appex` before the Tauri bundle phase.
Tauri copies it to `Contents/PlugIns`.

The release artifact is signed with the available Apple Development identity.
The extension uses App Sandbox plus a read-only exception for the exact shared
snapshot directory. Verification must prove WidgetKit returns one descriptor,
resolves `AccountWidgetIntent`, precaches all supported family placeholders,
and serves account options from the current snapshot.

## Testing And Acceptance

Rust tests cover configuration migration, account CRUD, credential references,
per-account provider dispatch, snapshot state precedence, redaction, schema
serialization, and atomic publication.

Frontend tests cover adding/importing, renaming, deleting, refreshing, and
rendering account cards from `CardSnapshot` while preserving monitoring and
status-bar preferences.

Swift tests cover snapshot decoding, unsupported versions, real-user-home path
resolution, entity lookup, suggested/default accounts, deleted accounts,
stale/error precedence, semantic color thresholds, and the field set shown by
each family.

The release gate runs formatting, Rust tests, Clippy with warnings denied,
frontend typecheck/tests/build, Swift tests, WidgetKit extension build, Tauri bundle,
`codesign --verify --deep --strict`, entitlement inspection, and `pluginkit`
discovery.

The user-visible acceptance journey is:

1. create or import at least two monitored accounts;
2. refresh both and see separate overview cards;
3. replace the authorized installed app and add small, medium, and large widgets;
4. edit widget configuration and switch between the two accounts;
5. compare every displayed field against the matching overview card;
6. verify stale, login-expired, and deleted-account states;
7. verify a host refresh eventually updates the widget without exposing a
   credential.

## Out Of Scope

- provider requests from the widget extension;
- interactive quota-changing widget controls;
- iOS or watchOS widgets;
- cloud synchronization of account credentials;
- commit, push, release, or deployment without separate authorization.
