# SpaceCadetPinball Rust Port Plan

Last Updated: 2026-05-10

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

- MUST DO: Use the decompilation references in `../Doc` and `../SpaceCadetPinball`
  when porting behavior or DAT semantics. Do not guess at coordinate spaces,
  attribute meaning, control flow, timer behavior, or scoring rules when source
  evidence is available.
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
| `TLight.*`, `TLightGroup.*`, `TLightBargraph.*`, `TLightRollover.*` | `src/gameplay/mechanics/*`, future `src/gameplay/rules/lights.rs` | `[~]` | Shared light, light-group, bargraph, and light-rollover mechanics now cover timed/flasher state, per-player backups, first countdown behavior, and the first linked target-light-group families (`top_circle_tgt_lights`, `ramp_tgt_lights`, `lchute_tgt_lights`, `bpr_solotgt_lights`); broader animation/query/control routing still remains. |
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
- [~] Source: `GroupData.*` edge/visual record layout; Rust target: `src/assets/group.rs`; parity check: typed collision-edge decoding exists for `600`/`603` wall arrays, for primary visual point payloads used by `TOneway`, and for `TKickout`-style visual circles driven by attribute `306`; field regions, scoring records, and visual offsets still need dedicated accessors.
- [ ] Source: all DAT parser structs; Rust target: `src/assets/dat.rs`; parity check: unit tests cover header layout, record counts, bitmap/zMap dimensions, and selected known group names.

### 2. Rendering and Visual Composition

- [x] Source: `render.*`, `gdrv.*`, `zdrv.*`; Rust target: `src/engine/render/*`; parity check: palette-aware DAT sprites draw through a texture cache.
- [x] Source: table/sidebar positioning; Rust target: `src/engine/render/mod.rs`; parity check: playfield and scoreboard appear in the correct relative positions.
- [x] Source: base table shell; Rust target: `src/gameplay/components/table/visuals.rs`; parity check: gameplay-owned visual snapshot includes `background` and `table`.
- [x] Source: visible table object groups; Rust target: `src/gameplay/components/group_name.rs`, `visuals.rs`; parity check: broad sequence/light banks are in gameplay-owned visual composition.
- [x] Source: `TTextBox.*`, `TTextBoxMessage.*`; Rust target: `src/gameplay/components/table/text_box.rs`; parity check: info and mission text boxes render with DAT-backed font and clipping.
- [?] Source: `proj.*`, `TTableLayer.*`, `TBall::Repaint`, plunger/ball feed path; Rust target: `src/gameplay/components/table.rs`, `src/engine/render/*`, future coordinate-space helpers; parity check: one documented conversion path exists between DAT/world/projection space and table-local 2D space, and ready-ball placement, ball rendering, and component sprite anchoring all use it consistently.
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
- [~] Source: `TEdgeSegment.*`, `TFlipperEdge.*`; Rust target: `src/engine/physics/edge.rs`, `flipper_edge.rs`; parity check: flipper segments plus component-owned DAT line/circle walls now collide, including slot-filtered rollover-style `600`/`603` ownership and create-wall offset registration, but broad-phase and source-style trigger callbacks remain partial.
- [~] Source: `TEdgeManager.*`; Rust target: `src/engine/physics/edge_manager.rs`; parity check: current manager resolves table-boundary, flipper, and slot-filtered DAT line/circle contacts without grid partitioning yet.
- [~] Source: `TEdgeManager::box_x`, `box_y`, `TestGridBox`; Rust target: physics broad-phase grid; parity check: static table/component edges now register into broad-phase boxes and candidate lookups come from overlapping cells, but source-style processed-flag/ray-walk distance selection still remains.
- [ ] Source: `FindCollisionDistance` methods on edge/collision components; Rust target: ray/contact solver; parity check: ball motion uses time-of-impact style distance selection.
- [~] Source: `TBall::already_hit`, `not_again`, collision reset flags; Rust target: ball collision memory; parity check: ball collision memory now suppresses immediate repeat contacts for static walls/bounds/flippers with source-shaped short-lived reset semantics, owned solid edges now consume component elasticity/smoothness/threshold/boost metadata, and default solid collision callbacks are impact-threshold aware, but component-triggered `not_again` parity and fuller source collision response still remain.
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
- [~] Source: `TPinballTable::AddBall`, `BallList`, `BallCountInRect`; Rust target: table ball collection; parity check: table-owned ball list, add-ball/drain removal flow, region ball counts, and drain-timer-driven player ball-count plus feed-timer updates now exist; inactive-ball reuse and fuller multiball rule parity still remain.
- [~] Source: `TPinballTable::AddScore`, score structs, player state; Rust target: rules/scoring plus table state; parity check: the table now keeps per-player score state, current-player HUD/widgets, source-shaped `AddScore` multiplier/base-award logic, bonus/jackpot accumulation, normal and mission-style `SpecialAddScore` behavior, live drain-bonus scoring, basic `SwitchToNextPlayer` score/jackpot restoration, and drain-timer-driven player ball-count plus shoot-again/feed-timer handoff on top of component-driven scoring from ported wall/target families, but richer rule flow still remains.
- [ ] Source: `TPinballTable::tilt`, tilt timers, tilt lock; Rust target: table lifecycle/rules; parity check: repeated nudge can lock controls and recover on timeout.
- [~] Source: `TPinballTable` light show/endgame/replay callbacks; Rust target: gameplay timers/rules; parity check: table-owned `GameOver` state and the endgame restart timeout now exist, but replay and light-show callback behavior still remain.

### 6. Mechanics Components

- [x] Source: `TFlipper.*`; Rust target: `src/gameplay/mechanics/flipper.rs`; parity check: input messages toggle flipper visual and simplified collision state.
- [x] Source: `TPlunger.*`; Rust target: `src/gameplay/mechanics/plunger.rs`; parity check: plunger input charges and launches the simplified ball.
- [x] Source: `TDrain.*`; Rust target: `src/gameplay/mechanics/drain.rs`; parity check: drain removes ball and increments the simplified drain counter.
- [x] Source: `TBlocker::Message`, `TGate::Message`, `TKickout::Message`, `TSink::Message`, `TLight::Message`; Rust target: `src/gameplay/mechanics/{blocker,gate,kickout,sink,light}.rs`; parity check: `cargo test` passes with per-component enable/disable/timer/player-state unit coverage.
- [x] Source: `TKickback::Message`, `TRollover::Message`, `TLightRollover::Message`, `TTripwire` reset/collision state; Rust target: `src/gameplay/mechanics/{kickback,rollover,light_rollover,tripwire}.rs`; parity check: `cargo test` passes with timer/rearm/toggle coverage for each mechanic.
- [x] Source: `TFlipper::Message`, `TPlunger::Message`, `TDrain::Message`; Rust target: `src/gameplay/mechanics/{flipper,plunger,drain}.rs`; parity check: `cargo test` passes with extend/retract, feed/release, and drain-timer unit coverage on the current single-ball table state.
- [~] Source: `TBlocker.*`, `TGate.*`, `TOneway.*`, `TWall.*`; Rust target: mechanics collision components; parity check: enable/disable messages alter collision and visuals, DAT wall arrays register component-owned line/circle edges with source-style create-wall offsets where available, `TOneway` visual-line geometry registers separate solid/trigger edges, rollover-style secondary walls use independent slot activity instead of one shared active bit, and rebounder `TWall` components now exist with source-shaped hard-hit flash/reset behavior.
- [~] Source: `TKickback.*`, `TKickout.*`, `THole.*`, `TSink.*`; Rust target: capture/eject mechanics; parity check: circle-backed geometry now registers for `TKickout`, but ball capture, timers, sounds, and release vectors still need full C++ behavior parity.
- [ ] Source: `TBumper.*`, `TRamp.*`, `TFlagSpinner.*`; Rust target: impact/ramp/spinner mechanics; parity check: collisions trigger scoring, lights, sounds, and animation state.

### 7. Lights, Targets, Rules, Missions

- [~] Source: `TLight.*`; Rust target: `src/gameplay/rules/lights.rs`; parity check: turn on/off, timed on/off, flasher, temporary override, bitmap index changes, and message field behavior are in place for the shared light mechanic, and the booster reward lights (`lite58`-`lite61`) now exist as real `TLight` components driven by popup-target control flow, but broader source light/control routing still remains.
- [~] Source: `TLightGroup.*`; Rust target: `lights.rs`; parity check: source-shaped per-player state, reset/on/off, notify timers, countdown decay, and basic group-wide flash-off behavior now exist for the medal and multiplier families (`bumper_target_lights`, `top_target_lights`) plus the first target-linked groups (`top_circle_tgt_lights`, `ramp_tgt_lights`, `lchute_tgt_lights`, `bpr_solotgt_lights`), but broader group animation/count/query behavior still remains.
- [~] Source: `TLightBargraph.*`, `TLightRollover.*`; Rust target: `src/gameplay/mechanics/{light_bargraph,light_rollover}.rs`, future `lights.rs`; parity check: `fuel_bargraph` now exists as a real `TLightBargraph` component with DAT `904` timer schedules, per-player countdown restore, queued countdown/timer-expired callbacks, shared runtime state consumed by the six fuel-rollover handlers, and `FuelSpotTargetControl` refuel completions that drive the real `lite70`-`lite72` lights plus bargraph refill, while broader per-light split/flasher parity and rule/control routing still remain.
- [~] Source: `TPopupTarget.*`, `TSoloTarget.*`; Rust target: `src/gameplay/rules/targets.rs` or mechanics split; parity check: `TSoloTarget` components (`a_targ10`-`a_targ22`) and `TPopupTarget` components (`a_targ1`-`a_targ9`) now exist with source-shaped hard-hit disable behavior plus timer/player-state handling, including DAT `407` popup-target re-enable timing, `FuelSpotTargetControl` now tracks its three-target progress per player and drives `lite70`-`lite72`, `top_circle_tgt_lights`, and the `fuel_bargraph` refill path on completion, `MissionSpotTargetControl` now drives `lite101`-`lite103` plus `ramp_tgt_lights` while preserving its source bitmask state per player, the left/right hazard solo-target trios now drive `lite104`-`lite109`, `lchute_tgt_lights`/`bpr_solotgt_lights`, and source-shaped `v_gate1`/`v_gate2` disable behavior on completion while tracking shared trio progress per player, the multiplier popup trio (`a_targ7`-`a_targ9`) now advances score multiplier state with source-shaped group completion/reset flow and linked `top_target_lights` light-group updates, the medal popup trio (`a_targ4`-`a_targ6`) now tracks per-player medal progression through level-one, level-two, and extra-ball rewards plus linked `bumper_target_lights` updates, and the booster popup trio (`a_targ1`-`a_targ3`) now drives staged flag/jackpot/bonus/bonus-hold progression plus the source-shaped top-tier double award and linked booster-light activation, while broader rule/control routing still remains.
- [~] Source: `TRollover.*`, `TTripwire.*`, `TCircle.*`, `TLine.*`; Rust target: target/sensor components; parity check: owned collision edges now dispatch targeted callbacks with slot/trigger context so rollover/tripwire/light-rollover/one-way mechanics can distinguish solid vs trigger hits, and the six fuel rollovers now apply source-shaped score/refuel-vs-indicator behavior against live `fuel_bargraph` state, but full rule/control-handler routing and other source collision side effects remain.
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

1. Coordinate-space cleanup: document and centralize DAT/world/projection/table-local conversions before more ball/ramp/kickout/sink behavior depends on ad hoc fixes.
2. Component-specific message parity: route flipper, plunger, drain, light, gate, blocker, kickout, and sink behavior through the new `MessageCode`/`ComponentState` spine.
3. Convert registered collision metadata into simple line/circle edge records where DAT wall arrays are available.
4. Physics broad phase: port the `TEdgeManager` grid enough to register table edges and collision components from DAT-derived geometry.
5. Ball collection and lifecycle: replace single optional ball with table-owned ball list, drain/feed/add-ball flow, ball-in-rect queries, and player ball counts.
6. Rule-backed visuals: migrate current visual approximation banks behind real light/target/component state, one family at a time.
7. Audio and timers: add the original timer callback semantics and sound trigger path as soon as the first rule components need them.

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
- The current port still mixes DAT/world/projection coordinates with table-local
  2D coordinates in a few places, which can produce "half-correct" fixes like
  ball placement moving closer without actually matching source behavior.
- The simplified physics path may become harder to replace if more rules assume one optional ball and hard-coded bounds.
- The component dispatch model is still thinner than the original C++ `MessageCode` plus control-handler graph.
- Audio, options persistence, high scores, and demo/attract are still placeholders or absent.
- `GameState` compatibility facades should be removed once all consumers use the new module boundaries.

## Validation Log

- 2026-05-10: Updated the plan after tracing the ready-ball placement bug back to a broader coordinate-space mismatch between DAT/world/projection data and the current Rust table-local 2D runtime; future port slices should use `Doc/` plus the decompiled C++ source as the primary reference instead of guessing.
- 2026-05-10: `cargo test` passed after registering `lite104`-`lite109`, `lchute_tgt_lights`, and `bpr_solotgt_lights` as real light/light-group components, adding shared per-player left/right hazard trio progress to table state, and wiring `LeftHazardSpotTargetControl` plus `RightHazardSpotTargetControl` through those lights/groups and `v_gate1`/`v_gate2` disable messages.
- 2026-05-10: `cargo test` passed after registering `lite101`-`lite103`, `top_circle_tgt_lights`, and `ramp_tgt_lights` as real light/light-group components, extending `TLightGroup` with the first group-wide flash-off handling, and wiring `MissionSpotTargetControl` plus the refined `FuelSpotTargetControl` flow through those linked light groups.
- 2026-05-10: `cargo test` passed after porting the `FuelSpotTargetControl` rule into `TSoloTarget`, adding per-player fuel-target progress to table state, registering the `lite70`-`lite72` target lights as real `TLight` components, and covering the score/light/bargraph-refill path with unit tests.
- 2026-05-10: `cargo test` passed after exposing live `fuel_bargraph` index state through the table simulation, porting the six `FuelRollover*Control` score/refuel rules into `TRollover`, registering the rollover indicator lights (`literoll179`-`literoll184`) as real `TLight` components, and covering the new behavior with mechanic and builder tests.
- 2026-05-10: `cargo test` passed after adding a real `TLightBargraph` mechanic for `fuel_bargraph`, wiring DAT `904` float-attribute timer schedules through table construction, and covering countdown, player-restore, and builder hookup behavior with unit tests.
- 2026-04-29: `cargo test` passed after replacing the score HUD approximation with a real table score field and awarding source-linked points from the newly ported `TWall`, `TSoloTarget`, and `TPopupTarget` hard-hit paths.
- 2026-04-29: `cargo test` passed after promoting the table score into per-player storage, wiring HUD/text to the current player, and adding basic `SwitchToNextPlayer` score restoration plus `PlayerChanged` broadcast flow.
- 2026-05-07: `cargo test` passed after adding table-owned pending message flow so drain timer expiry now decrements the active player's ball count and requests source-shaped `SwitchToNextPlayer` handoff.
- 2026-05-07: `cargo test` passed after extending drain expiry to mirror source serve flow by queueing `PlungerStartFeedTimer` for shoot-again and post-switch cases, plus `GameOver` for the last-ball path.
- 2026-05-07: `cargo test` passed after making `GameOver` a real table-owned state with a source-shaped endgame timeout and restart prompt, and verifying last-ball drain transitions now enter that state.
- 2026-05-07: `cargo test` passed after extending table score bookkeeping with source-shaped bonus/jackpot accumulation in `AddScore` and per-player jackpot restoration plus transient-flag clearing on player switch.
- 2026-05-07: `cargo test` passed after adding source-shaped `AddScore` multiplier/base-award handling and a `SpecialAddScore` helper that temporarily suppresses multiplier and bonus/jackpot side effects like the C++ control path.
- 2026-05-07: `cargo test` passed after routing the first live `SpecialAddScore` gameplay use through `TDrain`, so last-ball drains now award the table bonus score unless tilt-lock is active.
- 2026-05-07: `cargo test` passed after threading Full Tilt mode into the table and adding mission-style `SpecialAddScore` behavior that applies the current jackpot and resets it to `500000` for future mission-rule consumers.
- 2026-05-07: `cargo test` passed after extending `TSoloTarget` with source-shaped `MissionSpotTargetControl` message-field bitmask tracking and per-player restoration for targets `a_targ13`-`a_targ15`.
- 2026-05-07: `cargo test` passed after extending the same `TSoloTarget` bitmask/per-player state path to `LeftHazardSpotTargetControl` and `RightHazardSpotTargetControl` for targets `a_targ16`-`a_targ21`.
- 2026-05-07: `cargo test` passed after extending `TPopupTarget` with source-shaped `MultiplierTargetControl` trio completion, score-multiplier advancement, per-player group progress, and targeted popup re-enable/reset flow for `a_targ7`-`a_targ9`.
- 2026-05-07: `cargo test` passed after extending `TPopupTarget` with source-shaped `MedalTargetControl` trio completion, per-player medal progression, level-one/level-two score awards, extra-ball reward, and targeted popup re-enable/reset flow for `a_targ4`-`a_targ6`.
- 2026-05-07: `cargo test` passed after extending `TPopupTarget` with source-shaped `BoosterTargetControl` trio completion, per-player staged reward progress, jackpot/bonus/bonus-hold activation, and the top-tier double-award behavior plus targeted popup re-enable/reset flow for `a_targ1`-`a_targ3`.
- 2026-05-07: `cargo test` passed after adding source-linked booster reward lights (`lite58`-`lite61`) as real `TLight` components and wiring booster popup-target completions to drive their timed/on reset messages.
- 2026-05-07: `cargo test` passed after adding real `TLightGroup` components for `bumper_target_lights` and `top_target_lights`, with per-player state, notify timers, multiplier decay, and popup-target-driven group reset/restart messages.
- 2026-04-29: `cargo test` passed after adding a small DAT float-attribute hook so `TPopupTarget` re-enable timers now source their delay from attribute `407` during table construction.
- 2026-04-29: `cargo test` passed after adding the `TPopupTarget` family (`a_targ1`-`a_targ9`) with hard-hit disable behavior, source control bindings, timer-based re-enable, and per-player message-field backup handling.
- 2026-04-29: `cargo test` passed after adding the `TSoloTarget` family (`a_targ10`-`a_targ22`) with hard-hit disable/rearm behavior, source control bindings, and builder coverage.
- 2026-04-29: `cargo test` passed after adding the first `TWall` mechanic family for rebounders (`v_rebo1`-`v_rebo4`) plus source-linked component definitions and wall flash/reset behavior on hard collision callbacks.
- 2026-04-29: `cargo test` passed after adding impact-speed and threshold-exceeded data to collision contacts, then making the default solid-edge component callback path promote only source-style hard impacts while leaving trigger edges immediate.
- 2026-04-29: `cargo test` passed after porting a first-pass shared component collision response that consumes owned edge elasticity, smoothness, threshold, and boost metadata during solid contacts.
- 2026-04-29: `cargo test` passed after letting owned solid edges resolve with per-component elasticity from registered collision metadata instead of one shared restitution value.
- 2026-04-29: `cargo test` passed after adding component collision hooks with slot/edge-role context and using them to keep `TOneway` trigger callbacks off the solid side while honoring tilt-lock in rollover-family mechanics.
- 2026-04-29: `cargo test` passed after routing owned collision contacts through targeted component `ControlCollision` callbacks, including trigger-only edges for `TOneway`-style geometry.
- 2026-04-29: `cargo test` passed after adding source-shaped `TBall` collision-memory helpers and wiring `EdgeManager` to suppress immediate repeat edge contacts using short-lived recent-collision state.
- 2026-04-29: `cargo test` passed after adding a first-pass `TEdgeManager` broad-phase grid to register static wall/circle geometry into table boxes and drive collision candidate lookup from overlapping cells.
- 2026-04-29: `cargo test` passed after replacing the single optional table ball with a table-owned ball list, wiring plunger/feed/drain/flipper/visual/text paths through the new lifecycle helpers, and adding region ball-count coverage.
- 2026-04-27: Reviewed plan against `TPinballTable.h`, `TPinballComponent.h`, `TBall.h`, `TEdgeManager.h`, `control.h`, and current Rust gameplay/physics modules; replaced broad phase plan with source-backed subsystem tracker.
- 2026-04-27: `cargo check` passed after adding the initial Rust `MessageCode` parity enum, shared `ComponentState`, group-index component registry lookup, and adapting flipper/plunger/drain mechanics to the new component state spine.
- 2026-04-27: `cargo test` passed after adding an initial source-shaped table builder/linker, default component definitions for the current playable slice, optional DAT group-index resolution, link reports, and main-loop construction through `PinballTable::from_dat`.
- 2026-04-27: `cargo test` passed after expanding table builder definitions with source-shaped placeholder records for `TBlocker`, `TGate`, `TKickback`, `TKickout`, and `TSink` control-tag families.
- 2026-04-27: `cargo test` passed after expanding table builder definitions with source-shaped placeholder records for `TOneway`, `TRollover`, `TLightRollover`, and `TTripwire` sensor/control-tag families.
- 2026-04-27: `cargo test` passed after adding DAT visual collision metadata extraction and registering component collision metadata from source-shaped table records.
- 2026-04-27: `cargo test` passed after fixing ready-ball behavior so unlaunched balls no longer fall under gravity and DAT plunger feed position attribute `601` is used for ball spawn when available.
- 2026-04-27: `cargo test` passed after splitting `PlaceholderMechanic` into source-shaped `TBlocker`, `TGate`, `TKickout`, `TSink`, and `TLight` message handlers, plus shared component sprite-state tracking for message-driven visuals.
- 2026-04-27: `cargo test` passed after splitting the remaining rollover-family placeholders into source-shaped `TKickback`, `TRollover`, `TLightRollover`, and `TTripwire` timer/reset handlers; `TOneway` remains deferred to the collision-geometry pass because its source behavior is almost entirely collision-path logic.
- 2026-04-27: `cargo test` passed after rewriting the simplified `TFlipper`, `TPlunger`, and `TDrain` mechanics around source-shaped message and timer state, plus minimal shared table fields for drain/tilt/multiball bookkeeping on the current single-ball runtime.
- 2026-04-27: `cargo test` passed after adding typed DAT wall-array decoding for `600`/`603` attributes, registering decoded line edges into the current `EdgeManager`, and covering the new asset/geometry path with unit tests.
- 2026-04-27: `cargo test` passed after making DAT-derived line walls component-owned and filtering them by component active state during collision resolution, which unblocks source-shaped geometry work for `TOneway`, blocker, and gate collision paths.
- 2026-04-27: `cargo test` passed after adding a typed primary-visual-point accessor, replacing `TOneway`'s placeholder with a dedicated mechanic, and registering its source-shaped solid/trigger line pair through component-owned edge roles.
- 2026-04-28: `cargo test` passed after extending collision-edge ownership from one component-level active bit to slot-aware filtering, allowing rollover and light-rollover `600`/`603` geometry to follow separate runtime flags during collision resolution.
- 2026-04-28: `cargo test` passed after adding first-pass circle-edge collision support and per-component wall-offset hooks, so create-wall mechanics like blocker/gate/kickback/drain/plunger/sink no longer import `600` geometry as zero-offset lines only.
- 2026-04-28: `cargo test` passed after adding a typed visual-circle accessor for the `visual.FloatArr + attr 306` pattern and registering `TKickout`-style circle geometry through a dedicated collision registration path.
- 2026-04-27: Previous runs recorded `cargo check` and `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` success after gameplay-owned activity/visual-signal/refactor passes.
- 2026-04-26: Previous runs recorded `cargo check` and `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` success after DAT-backed text boxes, base-layer visual composition, light/sequence bank expansion, and table submodule split.

## Notes

- Keep this file current after each implementation slice.
- Add new checklist rows where the work belongs; avoid growing another long narrative changelog.
- When a broad item is too large for one review, split it by C++ function family or by one Rust module ownership boundary.
