# Tauri CLI

## Licensing Errata:
Because of publishing issues upstream, we soft-forked (and patched) both [`console`](https://github.com/mitsuhiko/console/blob/278de9dc2bf0fa28db69adee351072f668beec8f/Cargo.toml#L7) and [`dialoguer`](https://github.com/mitsuhiko/dialoguer/blob/2c3fe6b64641cfb57eb0e1d428274f63976ec150/Cargo.toml#L12) crates because of untenable issues surrounding expected use on Windows. 

This soft fork was introduced to the Tauri Codebase [here](https://github.com/tauri-apps/tauri/pull/1610).

`console`
```
license = "MIT"
authors = [
	"Armin Ronacher <armin.ronacher@active-4.com>"
]
```

`dialoguer`
```
license = "MIT"
authors = [
	"Armin Ronacher <armin.ronacher@active-4.com>",
	"Pavan Kumar Sunkara <pavan.sss1991@gmail.com>"
]
```
