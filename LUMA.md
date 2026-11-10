# GPUI for Luma

This is [Zed](https://github.com/zed-industries/zed)'s GPUI and the crates it
builds from, with Luma's changes on top. [Luma](https://github.com/lumashow/luma)
depends on it by commit: see the `[patch]` block in Luma's `gpui/Cargo.toml`.

## Branches

- **`upstream`** is Zed's history filtered to the paths in
  [`luma/paths.txt`](luma/paths.txt), with `git filter-repo`. Every commit on
  it is a Zed commit with only those paths kept; nothing else is ever
  committed to it. Its tip is Zed `32a0e813`.
- **`luma`** is `upstream` plus Luma's commits. `git log upstream..luma` is the
  complete list of what Luma changes, and `git diff upstream luma` the complete
  diff. The first commit trims the workspace's `members` to the kept crates.

## Refreshing to a newer Zed

`git filter-repo` is deterministic: filtering a newer Zed with the same path
list gives the same commits for the shared history, so the refresh is a rebase.

```sh
git clone --bare --single-branch --no-tags https://github.com/zed-industries/zed zed.git
git clone --no-local zed.git filtered && cd filtered
git checkout -b upstream <zed-commit> && git branch -D main && git remote remove origin
git filter-repo --paths-from-file ../gpui/luma/paths.txt --force
cd ../gpui
git fetch ../filtered upstream && git branch -f upstream FETCH_HEAD
git rebase upstream luma
```

- On a `Cargo.lock` conflict in the first commit, take upstream's and run
  `cargo check -p gpui`, which prunes it to this workspace again.
- If GPUI gained a dependency on another Zed crate, `cargo metadata` fails
  with a missing path. Add it to `luma/paths.txt` and filter again. A changed
  path list rewrites the filtered history, so the rebase becomes
  `git rebase --onto upstream <old upstream tip> luma`.
- Check, test and push both branches (`git push origin upstream luma`), then
  bump the `rev` in Luma's `gpui/Cargo.toml`.

```sh
cargo check -p gpui -p gpui_platform -p gpui_macros
cargo test -p gpui_wgpu
```

## What Luma changes

`git diff upstream luma` is the inventory; this is a map of it. A refresh that
drops one of these reintroduces the bug it fixes, or deletes the feature it is.

- **Zero-copy presentation of a foreign renderer's frames.** One
  `Window::paint_surface(bounds, SurfaceHandle)` on every platform
  (`gpui/src/scene.rs` defines `SurfaceHandle` and `PaintSurface.source`,
  `gpui/src/window.rs` the method), then per compositor:
  - Metal: `gpui_apple/src/metal_renderer.rs` (`bgra_surfaces_pipeline_state`
    and the BGRA branch in `draw_surfaces`, `SurfaceInputIndex::BgraTexture`),
    `gpui_apple/src/shaders.metal` (`bgra_surface_fragment`), and the
    `core_video` re-export in `gpui/src/gpui.rs`.
  - MetalFX: `gpui_apple/src/metal_upscaler.rs`, for surfaces painted with
    `Window::paint_upscaled_surface`.
  - wgpu (Linux): the renderer draws on the compositor's own device, so the
    surface is a plain texture. `gpui/src/gpui.rs` (`WgpuDevice`, the `wgpu`
    re-export), `gpui/src/platform.rs` (`PlatformWindow::wgpu_device`),
    `gpui_wgpu/src/wgpu_context.rs` (`shared_device`; the device also asks for
    `TIMESTAMP_QUERY` and WebGPU-default limits so a renderer can live on it),
    `gpui_wgpu/src/wgpu_renderer.rs` (`PrimitiveBatch::Surfaces` drawn as an
    instance array plus a per-draw texture), the three
    `gpui_wgpu/src/shaders*.wgsl`, and `gpui_linux/src/linux/{x11,wayland}/window.rs`.
- **wgpu 30** (upstream: 29), so the compositor and Luma's renderer share one
  `wgpu::Device` type. The API deltas are `SurfaceConfiguration::color_space`,
  `Queue::present` and `RequestAdapterOptions::apply_limit_buckets`.
- **HDR10 output on wgpu**: `gpui_wgpu/src/hdr.rs`, `hdr.wgsl` and
  `srgb_extended.wgsl`. The compositor draws into an `Rgba16Float` target in
  extended sRGB and one final pass PQ-encodes it.
- **Backdrop blur on wgpu**: `gpui_wgpu/src/backdrop.rs` and `backdrop.wgsl`,
  integrated at scene draw-order boundaries in `wgpu_renderer.rs`. Both Metal
  and wgpu blend between sharp and blurred pixels at a constant Gaussian
  radius, and `Window::paint_scaled_layer` / `Scene::append_scaled` scale a
  dismissed dialog's primitives and clips together.
- **A headless wgpu renderer** for pixel tests: `gpui_wgpu/src/wgpu_renderer/headless.rs`
  and `gpui_linux/src/headless_renderer.rs`.
- **Wayland pointer lock** with relative motion, in
  `gpui_linux/src/linux/wayland/{client,window}.rs`.
- **X11 Mailbox presentation**, matching Wayland's preference, in
  `gpui_linux/src/linux/x11/window.rs`. FIFO backpressure can block the X11
  refresh timer past its next deadline and halve the frame rate.
- **In-place atlas tile refresh**, so a live viewport publishes under one
  stable `ImageId`: `gpui/src/platform.rs` (the trait method), the Metal
  backend in `gpui_apple/src/metal_atlas.rs`, and `Window::update_image` in
  `gpui/src/window.rs`. All three are needed.
- **`set_maximum_drawable_count(2)`** in `gpui_apple/src/metal_renderer.rs`
  (upstream: 3), coupled to `luma_render::viewport::RESERVED`.
- **Color emoji font selection** in `gpui_wgpu/src/cosmic_text_system.rs`.
- **Resolved anchor offsets** in `gpui/src/elements/anchored.rs`, so popover
  motion follows the side the popover ended up on.
- **`filter_profile`** in `gpui_apple/src/metal_renderer.rs`.

`zlog`, `ztracing` and `ztracing_macro` are GPL-3.0-or-later, as upstream.
