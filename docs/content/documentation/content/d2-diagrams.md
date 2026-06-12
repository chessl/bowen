+++
title = "D2 Diagrams"
weight = 85
+++

Bowen can render fenced `d2` code blocks as inline SVG diagrams during Markdown rendering.
This uses the built-in Rust renderer, so no external D2 binary, JavaScript bridge, or client-side rendering step is required.

## Configuration

D2 rendering is disabled by default. Enable it in `zola.toml`:

```toml
[markdown]
render_d2 = true
```

## Usage

After enabling `render_d2`, write a fenced code block with the `d2` language:

````
```d2
users -> api
api -> database
```
````

Bowen compiles the D2 source to SVG and inserts it into the page as HTML:

```html
<div class="d2-diagram">
  <svg>...</svg>
</div>
```

Use the `d2-diagram` class in your theme CSS if you need to control spacing, overflow, or alignment.

## Showing Source

If D2 rendering is enabled globally but one block should remain visible as source, add `render=false` to that fence:

````
```d2,render=false
users -> api
api -> database
```
````

This block is rendered as a normal code block, even when syntax highlighting is configured to error on missing languages.

## Build Errors

Invalid D2 source fails the build with a Markdown rendering error. This keeps broken diagrams visible during local builds and CI instead of silently emitting stale source or an empty image.
