# Highlands / version 0.6

The new terrain measures **1440 × 1680 metres**: **2.4192 km²**, or sixteen
times the previous 360 × 420 m yard's area. Each side is four times longer.
The original seven-beacon lab and eight-section Skyway remain inside it.

Broad rolling hills surround the lab, with four mountain masses rising to
over 200 metres. The terrain has a 6 m sampling grid, continuous shared edges,
smooth shading and grass-to-rock material colors. A reserved flat patch keeps
the old routes intact. An open north/south lane gives room to build momentum.

The seeded scatter adds **180 blocks and 40 ramps** beyond the existing
obstacles. Blocks range from 1.5 to 17 m in width/depth and 1 to 18 m above the
local surface. The scatter avoids the old routes, clear regional spawn zones,
steep mountain faces and the speed lane. Layout seed: `0x51de2026`.
The same layout returns every launch, making movement comparisons repeatable.

## Five starting regions

Open **Esc / Start → Highlands / Momentum Routes**. Travel resets your current
free-run timer, peak speed and chain, and sets the selected region as your
respawn point. Respawn is a pause-menu option. Return to Playground returns to
the beacon lab. Restart Course outside Skyway restarts the original beacon lab.

| Region | Starting X / Z | What to test |
| --- | --- | --- |
| Long Run | 100 / 740 | A roughly 550 m open approach toward the lab; long-jump/dive/rollout chains and braking distance |
| Rolling Basin | -350 / 350 | Gentle waves, broad turns, valleys and speed carried between hills |
| Ridge Descent | 550 / -570 | A high mountain starting point; downhill slides, high-speed steering and rollout exits |
| Mountain Pass | -420 / -510 | Uphill momentum loss, steep-face sliding, and finding gentler lines around a mountain |
| Block Fields | 470 / 380 | Varied blocks and ramps; choosing a line through scattered obstacles with enough open space to recover |

The HUD shows horizontal speed in m/s, peak speed, recent move-chain count,
nearest region, coordinates and the current support slope. The open environment
is a movement sandbox rather than a new ordered challenge course. Skyway still
provides the authored precision challenge and checkpoints.

The mountains are a heightfield: they have no caves or overhangs. Grass/rock
detail comes from the renderer's embedded surface texture, not displaced GPU
tessellation. Sun shadows remain local to the character; the longer view range
and lighter distance fog preserve distant landmarks. Terrain is split into
small GPU meshes, with bounds culling for the local shadow pass.
