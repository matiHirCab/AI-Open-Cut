# OpenCut Desktop

Built with [GPUI](https://www.gpui.rs).

> [!WARNING]
> Very early. Right now this is just a window that opens.

## Running

Rust is pinned in `.prototools` at the repo root (`proto use` installs it).

```sh
moon run desktop:dev     # cargo run
moon run desktop:check   # cargo check
moon run desktop:build   # cargo build --release
```

The first build compiles GPUI from source and takes a while. The root `Cargo.lock` is committed.

## Prototype warning boundary

The primitives under `src/components` are dormant prototypes for planned UI work. Their module has a narrowly scoped allowance for `dead_code` and `unused_imports` while they remain intentionally unused. Remove an allowance as soon as a component is used, or delete the component if the prototype is abandoned. Warnings everywhere else in the desktop crate and workspace remain errors under the strict workspace Clippy check.

## Platform requirements

- **macOS**: Xcode command line tools (Metal renderer).
- **Windows**: no extra dependencies (Win32 + DirectWrite).
- **Linux**: renders via Vulkan (Blade), windows via Wayland or X11 (both enabled by default). System packages (Debian/Ubuntu names): `libvulkan1` + working Vulkan drivers, `libwayland-dev`, `libx11-xcb-dev`, `libxkbcommon-x11-dev`, `libfontconfig-dev`, plus a C toolchain and `cmake`.
- **WSL2/WSLg**: uses XWayland automatically when available. GPUI 0.2.2 requires `xdg_wm_base` v2–5, while WSLg advertises v1.
