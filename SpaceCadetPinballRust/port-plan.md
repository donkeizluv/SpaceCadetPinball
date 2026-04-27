# SpaceCadetPinball Rust Port Plan

Last Updated: 2026-04-27

This is the working tracker for the Rust port in `SpaceCadetPinballRust`.
The original C++ source lives in `../SpaceCadetPinball` and comes from a
decompilation, so the plan tracks behavior and data ownership rather than
requiring a one-to-one file mirror.

## Tracking Format

Use one checklist item per small, reviewable port slice. Each item should name:

- Source: original C++ file(s) or function family being matched.
- Rust target: module or file that owns the ported behavior.
- Parity check: the concrete runtime/build/test observation that proves the slice.

Status tags:

- `[x]` done and verified at least with `cargo check`.
- `[~]` scaffolded or approximate, needs source-level parity work.
- `[ ]` not started.
- `[?]` design decision or source behavior still needs investigation.

Prefer adding new items under the subsystem tracker below instead of adding long
free-form notes. Move items to done only when the parity check is recorded in
the Validation Log.

## Current Assessment

The Rust project has a usable module skeleton, DAT loading, a render path, SDL
bootstrap, typed visual snapshots, and a simplified launch/flipper/drain loop.
Compared with the C++ source, most remaining work is not rendering coverage; it
is behavior parity:

- The C++ table (`TPinballTable`) owns scoring, player/ball lifecycle, tilt,
  timers, ball lists, component lookup, gravity/ramp state, and replay/bonus
  bookkeeping. Rust currently owns one optional ball plus simplified counters.
- The C++ component model uses `MessageCode`, `TPinballComponent`, component
  tags, and `control.cpp` handler families. Rust now has an initial C++-named
  `MessageCode` surface and shared component base state, but the control graph
  and most component-specific message behavior are still unported.
- The C++ physics path uses `TBall`, `TCollisionComponent`, `TEdgeManager`,
  `TEdgeBox`, `TEdgeSegment`, `TFlipperEdge`, and field effects. Rust currently
  has simple segment reflection, table-boundary walls, and flipper segments.
- The C++ gameplay rules are concentrated in `control.cpp` plus light, target,
  kickout, sink, gate, timer, score, high-score, and demo classes. Rust has
  visual approximations for many table objects, but not the original rule graph.

## Architecture Guardrails

- Keep dependency flow one way: `platform` and `assets` feed `engine`; `engine`
  feeds `gameplay`; UI/debug sit on top.
- Keep SDL, mixer, window policy, and toolkit glue in `platform`.
- Keep DAT parsing and typed asset lookup in `assets`; rendering should consume
  typed lookup results, not raw DAT internals.
- Treat `TPinballTable` as gameplay orchestration, not a low-level engine type.
- Do not mirror every C++ header mechanically. Mirror behavior, state, message
  contracts, and data ownership.
- Keep compatibility facades only while they reduce migration risk; track their
  removal explicitly.

## Source Coverage Map

| C++ source | Current Rust target | Status | Notes |
| --- | --- | --- | --- |
| `loader.*`, `GroupData.*`, `partman.*`, `EmbeddedData.*` | `src/assets/*` | `[~]` | DAT load and typed lookup exist; remaining work is parser parity tests and embedded fallback completeness. |
| `gdrv.*`, `zdrv.*`, `render.*`, `TTableLayer.*`, sprite helpers | `src/engine/bitmap.rs`, `src/engine/render/*` | `[~]` | Real DAT textures render; dirty-rect/background restore and z-order parity still need targeted checks. |
| `maths.*`, `proj.*` | `src/engine/math.rs`, `src/engine/geom.rs` | `[~]` | Basic vector/rect helpers exist; verify projection and normalization behavior before physics parity. |
| `TBall.*`, `TEdge*.*`, `TCollisionComponent.*`, `TFlipperEdge.*` | `src/engine/physics/*` | `[~]` | Simplified ball and edge collision exist; grid, ray collision, field effects, ramps, and collision masks remain. |
| `TPinballComponent.*`, `TComponentGroup.*` | `src/gameplay/components/*` | `[~]` | Trait-object scaffold and group registry exist; original message field, active flag, control tag, scoring, and sprite helpers remain. |
| `TPinballTable.*` | `src/gameplay/components/table*` | `[~]` | Table owns simulation and visuals; original player, scoring, timers, tilt, ball list, multiball, and component lookup remain. |
| `TFlipper.*`, `TPlunger.*`, `TDrain.*` | `src/gameplay/mechanics/*` | `[~]` | First playable approximations exist; original message semantics and collision/control behavior remain. |
| `TBumper.*`, `TBlocker.*`, `TGate.*`, `TKickback.*`, `TKickout.*`, `THole.*`, `TSink.*`, `TRamp.*`, `TWall.*`, `TOneway.*` | `src/gameplay/mechanics/*` | `[ ]` | Visuals are present for many groups, but behavior components are mostly absent. |
| `TLight.*`, `TLightGroup.*`, `TLightBargraph.*`, `TLightRollover.*` | `src/gameplay/rules/lights.rs` | `[ ]` | Visual light banks exist; original timed/flasher/group messages are not ported. |
| `TRollover.*`, `TPopupTarget.*`, `TSoloTarget.*`, `TTripwire.*`, `TFlagSpinner.*`, `TCircle.*`, `TLine.*` | `src/gameplay/rules/targets.rs` or mechanics split | `[ ]` | Needs collision-triggered rule behavior, not only frame selection. |
| `control.*`, `score.*` | `src/gameplay/rules/*` | `[ ]` | Original mission/rank/scoring controller graph is not yet represented. |
| `timer.*`, `TTimer.*` | `src/engine/time.rs`, `src/gameplay/rules/timers.rs` | `[~]` | Fixed-step timing exists; original scheduled callback semantics remain. |
| `Sound.*`, `midi.*`, `TSound.*` | `src/platform/audio.rs` | `[ ]` | Platform audio is a placeholder. |
| `options.*`, `fullscrn.*`, `winmain.*`, `pb.*`, `high_score.*`, `TDemo.*` | `src/platform/*`, `src/gameplay/*` | `[~]` | SDL/input/fullscreen scaffolds exist; persistence, menus, high score, demo/attract remain. |
| `DebugOverlay.*`, ImGui files | `src/platform/ui.rs`, future debug module | `[ ]` | Debug UI has not been ported. |

## Subsystem Tracker

### 1. Assets and DAT Data

- [x] Source: `loader.*`, `GroupData.*`; Rust target: `src/assets/dat.rs`, `src/assets/group.rs`; parity check: `PINBALL.DAT` loads and table/background groups resolve.
- [x] Source: DAT palette and bitmap records; Rust target: `src/engine/bitmap.rs`, `src/engine/render/texture_cache.rs`; parity check: DAT-backed background and table render in `cargo run`.
- [x] Source: named group traversal; Rust target: `src/assets/group.rs`; parity check: HUD, sequence, number-widget, bitmap, and debug-label helpers are used outside render internals.
- [ ] Source: `loader.cpp` metadata extraction; Rust target: `src/assets/loader.rs`; parity check: enumerate component controls, scoring records, and group indexes needed by table construction.
- [ ] Source: `partman.*` and `EmbeddedData.*`; Rust target: `src/assets/embedded.rs`; parity check: asset discovery works both from external files and embedded/fallback resources.
- [ ] Source: `GroupData.*` edge/visual record layout; Rust target: `src/assets/group.rs`; parity check: typed accessors exist for collision edges, fields, scoring, and visual offsets without ad-hoc byte parsing.
- [ ] Source: all DAT parser structs; Rust target: `src/assets/dat.rs`; parity check: unit tests cover header layout, record counts, bitmap/zMap dimensions, and selected known group names.

### 2. Rendering and Visual Composition

- [x] Source: `render.*`, `gdrv.*`, `zdrv.*`; Rust target: `src/engine/render/*`; parity check: palette-aware DAT sprites draw through a texture cache.
- [x] Source: table/sidebar positioning; Rust target: `src/engine/render/mod.rs`; parity check: playfield and scoreboard appear in the correct relative positions.
- [x] Source: base table shell; Rust target: `src/gameplay/components/table/visuals.rs`; parity check: gameplay-owned visual snapshot includes `background` and `table`.
- [x] Source: visible table object groups; Rust target: `src/gameplay/components/group_name.rs`, `visuals.rs`; parity check: broad sequence/light banks are in gameplay-owned visual composition.
- [x] Source: `TTextBox.*`, `TTextBoxMessage.*`; Rust target: `src/gameplay/components/table/text_box.rs`; parity check: info and mission text boxes render with DAT-backed font and clipping.
- [ ] Source: `TPinballComponent::SpriteSet`, `SpriteSetBall`; Rust target: visual snapshot API; parity check: component state can request sprites through a common component-facing API instead of hard-coded visual banks.
- [ ] Source: `render.cpp` dirty rect/background restore; Rust target: `src/engine/render/scene.rs` or render state; parity check: moving sprites/text boxes redraw cleanly without relying only on full-frame redraw.
- [ ] Source: zMap/depth behavior; Rust target: `src/engine/render/*`; parity check: ball and table-object layering matches representative C++ scenes.
- [ ] Source: debug overlay/sprite inspection; Rust target: future `src/gameplay/debug.rs` or `src/platform/ui.rs`; parity check: active group names, frame indexes, and component state can be inspected at runtime.

### 3. Platform, Window, Input, Options, Audio

- [x] Source: `winmain.*`, `fullscrn.*`; Rust target: `src/platform/sdl_app.rs`, `fullscreen.rs`; parity check: SDL app boots and fullscreen policy is isolated from `main.rs`.
- [x] Source: keyboard input path; Rust target: `src/platform/input.rs`, `input_bindings.rs`; parity check: flipper, plunger, start, and nudge inputs reach table bridge state.
- [ ] Source: `options.*`; Rust target: `src/platform/options.rs`; parity check: user options persist, load defaults, and affect runtime settings.
- [ ] Source: `Sound.*`, `midi.*`, `TSound.*`; Rust target: `src/platform/audio.rs`; parity check: WAV effects and MIDI/music settings play through SDL mixer or selected Rust backend.
- [ ] Source: menus/dialog behavior from `winmain.*`, `pb.*`; Rust target: `src/platform/ui.rs`; parity check: new game, pause/resume, options, high scores, and exit flows are available.
- [ ] Source: `nudge.*`; Rust target: `src/platform/input.rs`, gameplay table; parity check: keyboard/controller nudges apply bounded impulses and feed tilt logic.

### 4. Physics and Collision

- [x] Source: `TBall.*`; Rust target: `src/engine/physics/ball.rs`; parity check: ball can launch, move, and drain in the simplified loop.
- [x] Source: `TEdgeSegment.*`, `TFlipperEdge.*`; Rust target: `src/engine/physics/edge.rs`, `flipper_edge.rs`; parity check: wall/flipper segment collision changes ball velocity.
- [~] Source: `TEdgeManager.*`; Rust target: `src/engine/physics/edge_manager.rs`; parity check: current manager resolves table-boundary and flipper contacts only.
- [ ] Source: `TEdgeManager::box_x`, `box_y`, `TestGridBox`; Rust target: physics broad-phase grid; parity check: collision candidates come from table boxes instead of scanning only hard-coded segments.
- [ ] Source: `FindCollisionDistance` methods on edge/collision components; Rust target: ray/contact solver; parity check: ball motion uses time-of-impact style distance selection.
- [ ] Source: `TBall::already_hit`, `not_again`, collision reset flags; Rust target: ball collision memory; parity check: repeated edge contacts do not jitter or double-trigger.
- [ ] Source: `FieldEffects` and ramp force fields; Rust target: physics field/ramp model; parity check: ramp and gravity well style forces alter ball direction like C++.
- [ ] Source: collision masks/groups; Rust target: component collision registration; parity check: gates/blockers/one-way/ramp enable or disable collision by message state.
- [ ] Source: `TFlipperEdge` angular behavior; Rust target: flipper collision impulse model; parity check: active flipper imparts directional impulse instead of acting as a static segment.

### 5. Component Model and Table Orchestration

- [x] Source: `TPinballComponent`, `TComponentGroup`; Rust target: `src/gameplay/components/component.rs`, `group.rs`; parity check: table can register and tick component trait objects.
- [x] Source: `TPinballTable` landing zone; Rust target: `src/gameplay/components/table.rs`; parity check: table owns simulation, component slots, messages, and visual state.
- [x] Source: `MessageCode` enum; Rust target: `src/gameplay/components/messages.rs`; parity check: public and private message codes used by original components are represented or intentionally mapped.
- [x] Source: `TPinballComponent` fields (`ActiveFlag`, `MessageField`, `GroupName`, `Control`, `GroupIndex`, scoring); Rust target: shared component state struct; parity check: new components do not duplicate base fields.
- [~] Source: `control::make_links`, component tags; Rust target: component registry/linker; parity check: named DAT/component links resolve once during table construction.
- [ ] Source: `TPinballTable::find_component`; Rust target: table lookup API; parity check: lookup by name and group index is available to rules/control code.
- [ ] Source: `TPinballTable::AddBall`, `BallList`, `BallCountInRect`; Rust target: table ball collection; parity check: multiple balls can exist and collision/rule checks can query by region.
- [ ] Source: `TPinballTable::AddScore`, score structs, player state; Rust target: rules/scoring plus table state; parity check: score, E9 part, jackpot, ball count, extra balls, and player switching update the HUD.
- [ ] Source: `TPinballTable::tilt`, tilt timers, tilt lock; Rust target: table lifecycle/rules; parity check: repeated nudge can lock controls and recover on timeout.
- [ ] Source: `TPinballTable` light show/endgame/replay callbacks; Rust target: gameplay timers/rules; parity check: game over, replay, and light show timers transition through original states.

### 6. Mechanics Components

- [x] Source: `TFlipper.*`; Rust target: `src/gameplay/mechanics/flipper.rs`; parity check: input messages toggle flipper visual and simplified collision state.
- [x] Source: `TPlunger.*`; Rust target: `src/gameplay/mechanics/plunger.rs`; parity check: plunger input charges and launches the simplified ball.
- [x] Source: `TDrain.*`; Rust target: `src/gameplay/mechanics/drain.rs`; parity check: drain removes ball and increments the simplified drain counter.
- [ ] Source: `TFlipper::Message`, flipper control handlers; Rust target: `flipper.rs`; parity check: extend/retract/null messages match original timing and sounds.
- [ ] Source: `TPlunger::Message`, `PlungerFeedBall`, `PlungerLaunchBall`; Rust target: `plunger.rs`; parity check: feed timer, relaunch, and launch strength match original behavior.
- [ ] Source: `TDrain::Message`, `BallDrainControl`; Rust target: `drain.rs`; parity check: drain sequences update balls, bonus, player switching, and game over.
- [ ] Source: `TBlocker.*`, `TGate.*`, `TOneway.*`, `TWall.*`; Rust target: mechanics collision components; parity check: enable/disable messages alter collision and visuals.
- [ ] Source: `TKickback.*`, `TKickout.*`, `THole.*`, `TSink.*`; Rust target: capture/eject mechanics; parity check: ball capture, timers, sounds, and release vectors match C++.
- [ ] Source: `TBumper.*`, `TRamp.*`, `TFlagSpinner.*`; Rust target: impact/ramp/spinner mechanics; parity check: collisions trigger scoring, lights, sounds, and animation state.

### 7. Lights, Targets, Rules, Missions

- [ ] Source: `TLight.*`; Rust target: `src/gameplay/rules/lights.rs`; parity check: turn on/off, timed on/off, flasher, temporary override, bitmap index changes, and message field behavior.
- [ ] Source: `TLightGroup.*`; Rust target: `lights.rs`; parity check: step, animation, random saturation/desaturation, reset, count, toggle split index, and notify timer behavior.
- [ ] Source: `TLightBargraph.*`, `TLightRollover.*`; Rust target: `lights.rs`; parity check: bargraph/rollover light state follows original messages.
- [ ] Source: `TPopupTarget.*`, `TSoloTarget.*`; Rust target: `src/gameplay/rules/targets.rs`; parity check: enable/disable, collision, scoring, and visual state are rule-driven.
- [ ] Source: `TRollover.*`, `TTripwire.*`, `TCircle.*`, `TLine.*`; Rust target: target/sensor components; parity check: sensor collisions notify the correct control handlers.
- [ ] Source: `score.*`; Rust target: `src/gameplay/rules/scoring.rs`; parity check: score widgets, multipliers, bonus, jackpot, replay, and extra ball rules match original scoring calls.
- [ ] Source: `control.cpp` component handler families; Rust target: `src/gameplay/rules/control.rs`; parity check: each handler is ported in a small commit with source and Rust tests or runtime traces.
- [ ] Source: `control.cpp` mission controllers; Rust target: `src/gameplay/rules/missions.rs`; parity check: mission select, rank progress, mission completion, and mission text match original flow.
- [ ] Source: `TDemo.*`; Rust target: `src/gameplay/demo.rs`; parity check: attract/demo behavior starts after idle and can relinquish control to a new game.

### 8. Text, Scores, Persistence, Debug

- [x] Source: `TTextBox.*`; Rust target: `src/gameplay/components/table/text_box.rs`; parity check: text queues, timing, font lookup, fitting, and clipping work for current messages.
- [ ] Source: `TTextBoxMessage.*`; Rust target: text box queue/message catalog; parity check: original message IDs/text resources can be queued by gameplay rules.
- [ ] Source: `high_score.*`; Rust target: `src/gameplay/high_score.rs` or platform persistence; parity check: high-score read/write and name-entry flow work.
- [ ] Source: `translations.*`; Rust target: text resources module; parity check: UI/gameplay strings resolve without hard-coded English literals in rule code.
- [ ] Source: `DebugOverlay.*`; Rust target: debug module; parity check: overlay can show frame timing, active scene, component counts, and selected rule state.

## Near-Term Work Order

1. Component-specific message parity: route flipper, plunger, drain, light, gate, blocker, kickout, and sink behavior through the new `MessageCode`/`ComponentState` spine.
2. Convert registered collision metadata into simple line/circle edge records where DAT wall arrays are available.
3. Physics broad phase: port the `TEdgeManager` grid enough to register table edges and collision components from DAT-derived geometry.
4. Ball collection and lifecycle: replace single optional ball with table-owned ball list, drain/feed/add-ball flow, ball-in-rect queries, and player ball counts.
5. Rule-backed visuals: migrate current visual approximation banks behind real light/target/component state, one family at a time.
6. Audio and timers: add the original timer callback semantics and sound trigger path as soon as the first rule components need them.

## Completed Groundwork

- [x] Introduced top-level Rust modules for `assets`, `engine`, `platform`, and `gameplay`.
- [x] Moved DAT parsing and group assembly into the assets layer.
- [x] Added SDL bootstrap, fullscreen policy, input translation, bindings, and options/audio/UI landing zones.
- [x] Added engine foundations for math, geometry, bitmap/zMap primitives, time, render, and physics.
- [x] Wired `main.rs` through platform/runtime/table ownership so it acts mostly as a bootstrap entrypoint.
- [x] Added gameplay component, group, message, table, visual, and text-box scaffolding.
- [x] Added first mechanics for launch, flipper, and drain.
- [x] Added broad gameplay-owned visual composition for DAT-backed table shell, HUD, text boxes, lights, sequences, and live ball sprite.
- [x] Split the gameplay table landing zone into focused submodules for input, text boxes, and visual composition.

## Risks

- The current visual coverage can hide missing rule behavior because many sprites are driven by approximate progress signals.
- The simplified physics path may become harder to replace if more rules assume one optional ball and hard-coded bounds.
- The component dispatch model is still thinner than the original C++ `MessageCode` plus control-handler graph.
- Audio, options persistence, high scores, and demo/attract are still placeholders or absent.
- `GameState` compatibility facades should be removed once all consumers use the new module boundaries.

## Validation Log

- 2026-04-27: Reviewed plan against `TPinballTable.h`, `TPinballComponent.h`, `TBall.h`, `TEdgeManager.h`, `control.h`, and current Rust gameplay/physics modules; replaced broad phase plan with source-backed subsystem tracker.
- 2026-04-27: `cargo check` passed after adding the initial Rust `MessageCode` parity enum, shared `ComponentState`, group-index component registry lookup, and adapting flipper/plunger/drain mechanics to the new component state spine.
- 2026-04-27: `cargo test` passed after adding an initial source-shaped table builder/linker, default component definitions for the current playable slice, optional DAT group-index resolution, link reports, and main-loop construction through `PinballTable::from_dat`.
- 2026-04-27: `cargo test` passed after expanding table builder definitions with source-shaped placeholder records for `TBlocker`, `TGate`, `TKickback`, `TKickout`, and `TSink` control-tag families.
- 2026-04-27: `cargo test` passed after expanding table builder definitions with source-shaped placeholder records for `TOneway`, `TRollover`, `TLightRollover`, and `TTripwire` sensor/control-tag families.
- 2026-04-27: `cargo test` passed after adding DAT visual collision metadata extraction and registering component collision metadata from source-shaped table records.
- 2026-04-27: Previous runs recorded `cargo check` and `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` success after gameplay-owned activity/visual-signal/refactor passes.
- 2026-04-26: Previous runs recorded `cargo check` and `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` success after DAT-backed text boxes, base-layer visual composition, light/sequence bank expansion, and table submodule split.

## Notes

- Keep this file current after each implementation slice.
- Add new checklist rows where the work belongs; avoid growing another long narrative changelog.
- When a broad item is too large for one review, split it by C++ function family or by one Rust module ownership boundary.
