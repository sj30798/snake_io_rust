# Game Fonts

This directory contains fonts used by the Snake IO game.

## Required Font

Place your font file here:
- **Filename:** `FiraSans-Bold.ttf`
- **Recommended:** [Google Fonts - Fira Sans](https://fonts.google.com/specimen/Fira+Sans)

## Installation Instructions

1. Download `FiraSans-Bold.ttf` from Google Fonts
2. Move it to this directory (`assets/fonts/`)
3. Rebuild the game with `cargo build --release`

The game will render all UI elements using this font, including:
- Menu titles and instructions
- In-game HUD (score, time, controls)
- Game results screen

## Font Requirements

- Must be `.ttf` (TrueType Font) format
- Recommended: Support for extended character sets (emojis, arrows, special characters)
- Recommended size: 12-64pt range for good rendering quality

Alternative fonts you can use:
- **Roboto** from Google Fonts
- **JetBrains Mono** for monospace look
- Any other `.ttf` font file

Just rename it to `FiraSans-Bold.ttf` or update the `GAME_FONT` constant in the source code.
