# Tauri Updater
---
> ⚠️ This project is a working project. Expect breaking changes.
---

The updater is focused on making Tauri's application updates **as safe and transparent as updates to a website**.

Instead of publishing a feed of versions from which your app must select, Tauri updates to the version your server tells it to. This allows you to intelligently update your clients based on the request you give to Tauri.

The server can remotely drive behaviors like rolling back or phased rollouts.

The update JSON Tauri requests should be dynamically generated based on criteria in the request, and whether an update is required.

Tauri's installer is also designed to be fault-tolerant, and ensure that any updates installed are valid and safe.

# Configuration

Once you have your Tauri project ready, you need to configure the updater.

Add this in tauri.config.json
```json
"updater": {
    "active": true,
    "endpoints": [
        "https://releases.myapp.com/{target}}/{current_version}}"
    ],
    "dialog": true,
    "pubkey": ""
}
```

The required keys are "active" and "endpoints", others are optional.

"active" must be a boolean. By default, it's set to false.

"endpoints" must be an array. The string `{{target}}` and `{{current_version}}` are automatically replaced in the URL allowing you determine [server-side](#update-server-json-format) if an update is available. If multiple endpoints are specified, the updater will fallback if a server is not responding within the pre-defined timeout.

"dialog" if present must be a boolean. By default, it's set to true. If enabled, [events](#events) are turned-off as the updater will handle everything. If you need the custom events, you MUST turn off the built-in dialog.

"pubkey" if present must be a valid public-key generated with Tauri cli. See [Signing updates](#signing-updates).

## Update Requests

Tauri is indifferent to the request the client application provides for update checking.

`Accept: application/json` is added to the request headers because Tauri is responsible for parsing the response.

For the requirements imposed on the responses and the body format of an update, response see [Server Support](#server-support).

Your update request must *at least* include a version identifier so that the server can determine whether an update for this specific version is required.

It may also include other identifying criteria such as operating system version, to allow the server to deliver as fine-grained an update as you would like.

How you include the version identifier or other criteria is specific to the server that you are requesting updates from. A common approach is to use query parameters, [Configuration](#configuration) shows an example of this.

## Built-in dialog

By default, updater uses a built-in dialog API from Tauri.

![New Update](https://i.imgur.com/UMilB5A.png)

The dialog release notes is represented by the update `note` provided by the [server](#server-support).

If the user accepts, the download and install are initialized. The user will be then prompted to restart the application.

## Events

**Attention, you need to _disable built-in dialog_ in your [tauri configuration](#configuration), otherwise, events aren't emitted.**

To know when an update is ready to be installed, you can subscribe to these events:

### Initialize updater and check if a new version is available

#### If a new version is available, the event `tauri://update-available` is emitted.

Event : `tauri://update`

### Rust
```rust
dispatcher.emit("tauri://update", None);
```

### Javascript
```js
import { emit } from "@tauri-apps/api/event";
emit("tauri://update");
```

### Listen New Update Available

Event : `tauri://update-available`

Emitted data:
```
version    Version announced by the server
date       Date announced by the server
body       Note announced by the server
```

### Rust
```rust
dispatcher.listen("tauri://update-available", move |msg| {
    println("New version available: {:?}", msg);
})
```

### Javascript
```js
import { listen } from "@tauri-apps/api/event";
listen("tauri://update-available", function (res) {
    console.log("New version available: ", res);
});
```

### Emit Install and Download

You need to emit this event to initialize the download and listen to the [install progress](#listen-install-progress).

Event : `tauri://update-install`

### Rust
```rust
dispatcher.emit("tauri://update-install", None);
```

### Javascript
```js
import { emit } from "@tauri-apps/api/event";
emit("tauri://update-install");
```

### Listen Install Progress

Event : `tauri://update-status`

Emitted data:
```
status    [ERROR/PENDING/DONE]
error     String/null
```

PENDING is emitted when the download is started and DONE when the install is complete. You can then ask to restart the application.

ERROR is emitted when there is an error with the updater. We suggest to listen to this event even if the dialog is enabled.

### Rust
```rust
dispatcher.listen("tauri://update-status", move |msg| {
    println("New status: {:?}", msg);
})
```

### Javascript
```js
import { listen } from "@tauri-apps/api/event";
listen("tauri://update-status", function (res) {
    console.log("New status: ", res);
});
```

# Server Support

Your server should determine whether an update is required based on the [Update Request](#update-requests) your client issues.

If an update is required your server should respond with a status code of [200 OK](http://tools.ietf.org/html/rfc2616#section-10.2.1) and include the [update JSON](#update-server-json-format) in the body. To save redundantly downloading the same version multiple times your server must not inform the client to update.

If no update is required your server must respond with a status code of [204 No Content](http://tools.ietf.org/html/rfc2616#section-10.2.5).

## Update Server JSON Format

When an update is available, Tauri expects the following schema in response to the update request provided:

```json
{
    "url": "https://mycompany.example.com/myapp/releases/myrelease.tar.gz",
    "version": "0.0.1",
    "notes": "Theses are some release notes",
    "pub_date": "2020-09-18T12:29:53+01:00",
    "signature": ""
}
```

The only required keys are "url" and "version", the others are optional.

"pub_date" if present must be formatted according to ISO 8601.

"signature" if present must be a valid signature generated with Tauri cli. See [Signing updates](#signing-updates).

## Update File JSON Format

The alternate update technique uses a plain JSON file meaning you can store your update metadata on S3, gist, or another static file store. Tauri will check against the name/version field and if the version is smaller than the current one and the platform is available, the update will be triggered. The format of this file is detailed below:

```json
{
	"name":"v1.0.0",
	"notes":"Test version",
	"pub_date":"2020-06-22T19:25:57Z",
	"platforms": {
		"darwin": {
			"signature":"",
			"url":"https://github.com/lemarier/tauri-test/releases/download/v1.0.0/app.app.tar.gz"
		},
 		"linux": {
			"signature":"",
			"url":"https://github.com/lemarier/tauri-test/releases/download/v1.0.0/app.AppImage.tar.gz"
		},
		"win64": {
			"signature":"",
			"url":"https://github.com/lemarier/tauri-test/releases/download/v1.0.0/app.x64.msi.zip"
		}
	}
}
```


# Bundler (Artifacts)

The Tauri bundler will automatically generate update artifacts if the updater is enabled in your tauri.config.js

If the bundler can locate your private and pubkey, your update artifacts will be automatically signed.

The signature can be found in the `sig` file. The signature can be uploaded to GitHub safely or made public as long as your private key is secure.

You can see how it's [bundled with the CI](https://github.com/tauri-apps/tauri/blob/feature/new_updater/.github/workflows/artifacts-updater.yml#L44)

## macOS

On MACOS we create a .tar.gz from the whole application. (.app)

```
target/release/bundle
└── osx
    └── app.app
    └── app.app.tar.gz (update bundle)
    └── app.app.tar.gz.sig (if signature enabled)
```

## Windows

On Windows we create a .zip from the MSI, when downloaded and validated, we run the MSI install.

```
target/release
└── app.x64.msi
└── app.x64.msi.zip (update bundle)
└── app.x64.msi.zip.sig (if signature enabled)
```

## Linux

On Linux, we create a .tar.gz from the AppImage.

```
target/release/bundle
└── appimage
    └── app.AppImage
    └── app.AppImage.tar.gz (update bundle)
    └── app.AppImage.tar.gz.sig (if signature enabled)
```

# Signing updates

We offer a built-in signature to ensure your update is safe to be installed.

To sign your updates, you need two things.

The *Public-key* (pubkey) should be added inside your tauri.config.json to validate the update archive before installing.

The *Private key* (privkey) is used to sign your update and should NEVER be shared with anyone. Also, if you lost this key, you'll NOT be able to publish a new update to the current user base (if pubkey is set in tauri.config.json). It's important to save it at a safe place and you can always access it.

To generate your keys you need to use the Tauri cli. (will be built in Tauri main CLI soon)

```bash
tauri sign -g -w ~/.tauri/myapp.key
```

You have multiple options available
```bash
FLAGS:
        --force          Overwrite private key even if it exists on the specified path
    -g, --generate       Generate keypair to sign files
    -h, --help           Prints help information
        --no-password    Set empty password for your private key
    -V, --version        Prints version information

OPTIONS:
    -b, --binary <binary>                        Sign the specified binary
    -p, --password <password>                    Set private key password when signing
    -k, --private-key <private-key>              Load the private key from a string
    -f, --private-key-path <private-key-path>    Load the private key from a file
    -w, --write-keys <write-keys>                Write private key to a file
```

***

Environment variables used to sign:

`TAURI_PRIVATE_KEY`  Path or String of your private key

`TAURI_KEY_PASSWORD`  Your private key password (optional)
