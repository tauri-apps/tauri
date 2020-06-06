# Tauri Updater
---
> ⚠️ This project is a working project. Expect breaking changes.
---

Instead of publishing a feed of versions from which your app must select, Tauri updates to the version your server tells it to. This allows you to intelligently update your clients based on the request you give to Tauri.

The server can remotely drive behaviors like rolling back or phased rollouts.

The update JSON Tauri requests should be dynamically generated based on criteria in the request, and whether an update is required.

Tauri is also designed to be fault tolerant, and ensure that any updates installed are valid.

## API

```rust
use tauri_updater::{http::Update, CheckStatus};
let updater = Update::configure().url("URL").check()?;

match updater.status() {
    CheckStatus::UpdateAvailable(new_release) => {
      println!("New version available {:?}", new_release.version);
      updater.install()?;
    }
    CheckStatus::UpToDate => println!("App already up to date"),
}
```

## Update Requests

Your update request must at least include a version identifier and target so that the server can determine whether an update for this specific version is required.

## Todo
- [ ] Archive generator (generate tar.gz & zip automatically on release)
- [ ] Windows / Linux tests
- [ ] Cleanup
- [ ] Allow fallback URLS
- [ ] Add github updates (without update server)

