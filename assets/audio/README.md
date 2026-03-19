# Audio Assets

Drop gameplay audio files in this folder using these exact names:

- `pickup.ogg` - fruit pickup click/chime
- `near_miss.ogg` - near collision warning sound
- `combo.ogg` - combo streak accent
- `hit.ogg` - player elimination impact
- `ambience_calm.ogg` - low-intensity loop for early/mid round
- `ambience_tension.ogg` - medium-intensity loop for pressure moments
- `ambience_clutch.ogg` - high-intensity loop for endgame/combo spikes

The game auto-loads these files if present. Missing files are silently skipped.

Accessibility:

- Press `V` in-game to toggle reduced-audio profile.

Adaptive ambience tiers switch automatically based on:

- remaining time
- combo streak size
- near-miss pressure
- snakes left alive
