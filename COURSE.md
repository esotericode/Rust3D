# Skyway — course guide

Start from **Pause → Skyway / Section Practice**. The broad orange ramp east of
the movement lab also leads to the start. The 360 × 420 m yard leaves room below
the elevated route for the original lab and four new ramp-and-block gardens.

The route has **46 ordered components: 30 fixed decks and 16 moving platforms**.
It starts 8 m above the yard and finishes at 44 m. Touch each numbered target
in order; skipping a component does not advance the run. The next three targets
are shown to help you plan, with the immediate target highlighted.

| Section | Components* | Challenge | Design purpose |
| --- | --- | --- | --- |
| 1 · Boarding | 01–03 | Wide shuttle and fixed dock | Learn to board, ride and leave at a predictable endpoint |
| 2 · Crosswind | 04–11 | Three sideways pads separated by fixed islands | Learn timing with generous landing areas |
| 3 · Lift Works | 12–17 | Vertical lift and rising zigzag steps | Learn lift exits, elevation and controlled turning |
| 4 · Diagonal Dock | 18–21 | Long shuttle, rest island and diagonal ride | Combine riding, direction changes and endpoint jumps |
| 5 · Clockwork | 22–30 | Smaller pads, staggered cycles and a direct moving transfer | Test precise takeoffs and calm landings |
| 6 · Leap and Dive | 31–35 | Running long jump, low arch and raised exit | Test speed control, dive clearance and rollout timing |
| 7 · High Harbor | 36–39 | Two diagonal shuttles and a high fixed dock | Use inherited momentum while waiting for safe exits |
| 8 · Summit | 40–46 | Lift, narrow fixed decks, swaying pad and finish shuttle | Combine earlier skills at height |

\* Each section starts from the previous violet checkpoint deck. Component 01
is the initial start deck, and each section ends at a fixed checkpoint.

Amber pads move. Mint edges mark fixed surfaces. Violet rings and edges mark
checkpoints. Triangles point toward the next component; dotted rails and endpoint
rings show each mover's full path. Vertical lifts use a circle on their top.

Use short, controlled jumps for the small islands. On a shuttle, let the pad do
the travel and jump when the destination is close. The pads ease into their
endpoints and dwell there for 0.9 seconds. Their cycles are deterministic,
including the offset cycles in Clockwork. There are no random hazards.

For the long-jump lane, run along the striped launch deck, hold Crouch (Ctrl or
LT) and press Jump near its edge. On the following runway, press Dive before the low arch.
Stay low through it, then press Jump or Dive during the landing slide to roll
out toward the raised exit. Jump once more if you recover to standing first.

Falls return you promptly to the most recent fixed checkpoint. You repeat the
components after that checkpoint; earlier sections stay completed. The camera
faces toward the route on respawn. Retrying keeps the platform cycles and timer
running. Restarting resets both and requires menu confirmation. Pause freezes
the course, and practice runs can start at any section without recording a best.

The timer uses actual unpaused time, including retries, and begins with your
first movement, jump or dive. Full-course best times are kept for this session.
Practice ends at the next violet deck. Choose another section or a full run in
the pause menu when finished.

## Validation

The course is traversed in tests from two initial platform phases using normal
game inputs and the actual capsule collision. The pilot walks to departure
edges, waits for reachable windows, runs before long jumps, dives through the
arch and reaches the finish without teleports or velocity overrides. These
checks establish reachability; hands-on play still guides future feel tuning.
Additional tests cover rider stability through full horizontal/vertical cycles,
platform momentum, lift landings, blocked carrying and checkpoint/practice rules.
