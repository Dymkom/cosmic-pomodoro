# Cosmic Pomodoro

A minimal, distraction-free Pomodoro applet for the COSMIC desktop.

## About
Cosmic Pomodoro is a small, straightforward pomodoro timer designed to fit naturally into the COSMIC desktop.
It provides a clear work/break cycle, simple controls, and a clean interface without distractions.  
The focus is on staying consistent and keeping the workflow predictable, without adding extra features you don’t really need.


## 🔽 Download

Latest Flatpak bundle:

- **Release page:**  
  https://github.com/petar030/cosmic-pomodoro/releases/tag/v0.1.0-flatpak-20260308

- **Direct download (.flatpak):**  
  https://github.com/petar030/cosmic-pomodoro/releases/download/v0.1.0-flatpak-20260308/io.github.petar030.cosmic-pomodoro-master.flatpak


## Features

Cosmic Pomodoro is designed to feel perfectly at home inside the **COSMIC desktop** — fast, native, and fully theme‑aware.

- **Fully native COSMIC applet** — integrates cleanly with COSMIC panel and UI conventions  
- **Automatic theme support** — adapts to system theme (light/dark, accent colors, community themes)
- **Minimal, focused popup UI** — clean Work/Break indicator with zero clutter  
- **Configurable durations** — customize work sessions, breaks, and long-break interval
- **Lightweight panel indicator** — compact progress icon showing session progress
- **Desktop notifications with sound cues** — sound is always played on session transitions


## Screenshots

| Theme | Preview |
|---|---|
| Pop!_OS Classic | img/PopOsClassic.png |
| Catppuccin | img/Catpuccin.png |
| Tokyo Night | img/TokyoNight.png |
| Gruvbox Dark | img/GruvboxDark.png |
| Gruvbox Light | img/GruvboxLight.png |
| Mono Dark | img/MonoDark.png |
| Settings | img/Config.png |


## Requirements

- Rust (`cargo`)
- https://github.com/casey/just
- `flatpak` + `org.flatpak.Builder`
- COSMIC session for full applet integration testing


## Local development

```sh
just run
```


## Flatpak build (local)

This project is prepared for the **COSMIC Flatpak ecosystem** (not Flathub-specific metadata/process).

```sh
# 1) Regenerate cargo sources used by manifest
just flatpak-cargo-sources

# 2) Build + install Flatpak locally
just flatpak-builder

# 3) Create distributable .flatpak bundle
just flatpak-bundle
```

Generated bundle:

```text
io.github.petar030.cosmic-pomodoro-master.flatpak
```

---

## Test installed Flatpak

```sh
flatpak run io.github.petar030.cosmic-pomodoro
```



