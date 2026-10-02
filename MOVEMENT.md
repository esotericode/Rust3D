# Movement design / version 0.6

Stride aims for readable, responsive 3D platforming at low speed and deliberate,
flowing traversal once the player builds momentum. The controls should explain
why a jump worked, why a turn lost speed, and how to improve the next attempt.
This is a first tuning pass, with hardware playtesting still needed for feel.

## References and decisions

These are design references, not copied game implementations or an attempt to
reproduce proprietary Mario physics.

| Primary reference | Useful principle | Stride's interpretation |
| --- | --- | --- |
| [Nintendo's Super Mario Odyssey movement overview](https://supermario.nintendo.com/mario-cappy/) | A readable vocabulary of jumps, wall moves and downhill rolling | Distinct long jump, dive, slide, rollout and wall kick states; downhill routes offer a reason to combine them |
| [id Software's QuakeWorld movement source](https://github.com/id-Software/Quake/blob/master/QW/client/pmove.c) | Velocity, friction and acceleration are separate; input need not replace velocity | Earned speed survives directional input and air travel; braking, friction and collisions remove it |
| [Kyle Pittman's GDC jump-design talk](https://www.gdcvault.com/play/1023148/Math-for-Game-Programmers-Building) | Choose useful jump arcs and support variable jump height | Retain tap/hold jump height, predictable long-jump arcs and faster falling; tune against actual obstacle routes |
| [Maddy Thorson's Celeste & Forgiveness](https://www.mattmakesgames.com/articles/celeste_and_forgiveness/index.html) | Small timing allowances help the game read player intent | Preserve 100 ms coyote time and 90 ms jump buffering; a held wall slide allows a kick at any point, with 35 ms departure grace |

## Momentum contract

- Walking targets 7.2 m/s and sprinting targets 10.5 m/s. These are input-driven
  speeds, **not hard limits on earned speed**.
- Jump keeps full tap/hold height whether or not you sprint. The long jump is
  Crouch + Jump while moving (Ctrl or LT by default), as in Mario's crouch long
  jump, so sprinting never costs height.
- A long jump adds an impulse of 6, a dive 4, and a rollout 3.8. At speed `s`,
  the actual increment is `impulse / (1 + (s / 22)^1.5)`. Gains remain positive
  at high speed but become smaller. The chain counter is feedback; there is no
  hidden combo multiplier or timed score reward affecting acceleration.
- Strong input in the direction of travel preserves earned momentum. Airborne
  movement with no input coasts. Slight ground drag grows with speed; slides
  have greater friction. Landings offer 100 ms before ordinary running drag,
  allowing an intentional immediate jump chain.
- Opposite input brakes; lighter controller input targets a slower speed for
  precise landings. How much earned speed is kept blends smoothly with stick
  strength (fully braking at 55% or less, fully keeping at 90% or more) and with
  the angle from your heading (about 81 to 101 degrees), so there is no cliff
  where a slightly lighter push suddenly stops you. Ground stops remain quick
  at walking speed. Fast turns need wider arcs, and cannot instantly redirect a
  dive or long jump backwards.
- A dive is available once per flight. Landing becomes a slide; a fresh Jump
  or Dive press rolls out. Hold Sprint to extend a slide, including downhill
  slides, and steer it in gentle arcs that widen with speed. Repeated airborne
  Dive presses supply no extra boosts, and a dive does not brake a fast fall.
  A rollout that lands early, on raised ground, ends there so the next jump is
  never locked out.
- Holding toward a wall in the air slides down it at 4.5 m/s, and Jump kicks
  off at any point of the slide. Alternate walls to climb; the same wall cannot
  be kicked twice before landing. Brushing a wall without holding into it
  catches you for 120 ms, as before.
- Standing past a ledge on only the capsule's rounded base slips off, unless you
  are moving onto the solid (stepping up or landing short), so feet never sink
  visibly beside an edge.
- No gameplay rule clamps horizontal speed to 15 or 18 m/s. Drag, uphill travel,
  collisions, turn demands, finite space and diminishing gains create practical
  limits. This is an arcade controller, not an exact energy-conserving simulation.

## Slopes

Rendering and terrain collision use the same heightfield triangles, while
lighting uses smooth vertex normals. The capsule follows walkable grades and
projects velocity onto their tangent. Gravity projected onto the slope speeds
downhill travel and resists uphill travel. Faces steeper than approximately
50 degrees slide even while pushing uphill. Jumping from them springs 6 m/s
away from the face with 0.2 s of no air control and no slope boost, and landing
on one keeps only the motion along it, so hopping cannot climb what walking
cannot.

Launching uphill retains the slope's upward velocity in addition to the move's
vertical impulse; relative to the ground, an uphill jump is an ordinary jump.
At a ramp edge, losing support also retains upward velocity. Continuous hills
use ground adhesion for stable traversal until a crest curves away faster than
gravity can follow: when horizontal speed squared times the terrain's downward
curvature (measured across a 6 m cell) exceeds gravity, and the ground ahead
drops below the ballistic arc, you leave the ground and fly. Broad Highlands
hills need roughly 60 m/s; sharper rises need less. Landing retains horizontal
velocity rather than replacing it with the walking target, and landing impact
is measured into the surface, so a touchdown along a matching slope is soft.

Horizontal collision uses distance-based substeps no longer than half the
capsule radius, so high speed cannot skip thin obstacles. The simulation remains
120 Hz at every supported render cap. Moving-platform carry and inherited jump
velocity remain independent of the terrain controller.

## Try it

1. Pause and choose **Highlands / Momentum Routes → Long Run**.
2. Run forward with Sprint and repeatedly release/press Jump on landing.
3. During a long jump, press Dive. At its landing, press Jump for a rollout.
   After the rollout recovers, long jump again. Watch Speed, Peak and Chain.
4. Try **Ridge Descent**. Dive and hold Sprint down the hill; roll out into the
   basin. Use broad turns and brake before reaching blocks.
5. Revisit Skyway. Use light stick input for precise pads; take moving-platform
   velocity into account rather than carrying maximum speed into every landing.

## Validation and next tuning targets

Automated checks cover uphill/downhill speed changes, steep-face sliding,
uphill move launches, terrain/mesh agreement, long-jump and dive/rollout chains,
air coasting, analog braking, 300 m/s thin-wall collision, camera clearance and
the entire Skyway route at two initial platform timings. Flat-ground chains
exceed 55 m/s for long jumps and 50 m/s for dive/rollout in the test scenarios.
Those values are checks, not speed limits or guaranteed records.

Useful playtest feedback: whether high-speed steering is predictable; whether
rollout timing is readable on rough grades; whether lighter stick input brakes
too abruptly; and whether uphill launches feel too tall. Camera pullback and
field of view widen smoothly with speed, without changing physics.
