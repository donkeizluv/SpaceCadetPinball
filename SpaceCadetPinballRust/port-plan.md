# SpaceCadetPinball Rust Port Plan

Last Updated: 2026-04-26

## Goals

- Keep the Rust port organized around stable module boundaries instead of mirroring every C++ file one-to-one.
- Preserve buildability while moving code out of the legacy `GameState` namespace.
- Reach a first playable vertical slice before spending time on UI-only polish.

## Architecture Guardrails

- Keep dependency flow one-way: `platform` and `assets` feed `engine`, `engine` feeds `gameplay`, and UI/debug sit on top.
- Keep a single Cargo crate for now; do not split into multiple crates unless compile-time or dependency pressure justifies it.
- Prefer Rust-native ownership boundaries and explicit runtime context over mirroring each C++ header as a separate module.
- Keep SDL, mixer, window policy, and toolkit glue in `platform` so engine and gameplay code remain testable without window creation.
- Treat `TPinballTable` as gameplay ownership/orchestration, not as a low-level engine type.

## Completed

- [x] Introduced top-level Rust modules for `assets`, `engine`, `platform`, and `gameplay`.
- [x] Moved the DAT parser into the new assets layer at `src/assets/dat.rs`.
- [x] Added `src/assets/mod.rs` as the export surface for the assets layer.
- [x] Moved runtime state and table-input bridge state into `src/engine/runtime.rs`.
- [x] Added `src/engine/mod.rs` to export runtime types.
- [x] Added `src/platform/input.rs` for shared action names and impulse classification.
- [x] Added `src/platform/mod.rs` as the platform integration root.
- [x] Added `src/platform/sdl_app.rs` for SDL bootstrap, window/canvas ownership, and frame helpers.
- [x] Added `src/platform/fullscreen.rs` and moved fullscreen policy out of `main.rs`.
- [x] Expanded `src/platform/input.rs` into SDL event translation helpers.
- [x] Added `src/platform/input_bindings.rs` for default keyboard-to-action bindings.
- [x] Added `src/platform/options.rs` for platform-facing settings state.
- [x] Added `src/platform/audio.rs` and `src/platform/ui.rs` as platform integration landing zones.
- [x] Added `src/gameplay/mod.rs` as the gameplay root placeholder.
- [x] Added `src/assets/group.rs` for typed group and entry lookup helpers.
- [x] Added `src/assets/loader.rs` for initial loader-style metadata extraction.
- [x] Added `src/assets/embedded.rs` for asset-path and embedded-resource discovery helpers.
- [x] Moved group assembly, spliced-bitmap splitting, and zMap normalization out of `src/assets/dat.rs` and into `src/assets/group.rs`.
- [x] Added `src/engine/math.rs`, `src/engine/geom.rs`, `src/engine/bitmap.rs`, and `src/engine/time.rs` as Phase 3 engine foundations.
- [x] Added `src/engine/render/` with `mod.rs`, `sprite.rs`, `scene.rs`, and `texture_cache.rs` for the initial render path.
- [x] Moved shared bitmap and zMap primitives into `src/engine/bitmap.rs` so assets and render share one ownership boundary.
- [x] Wired `src/main.rs` through `engine::render::RenderState` so loaded DAT bitmaps can be presented by the new render layer.
- [x] Added `src/gameplay/components/mod.rs`, `table.rs`, `group.rs`, and `messages.rs` to start the Phase 4 gameplay scaffold.
- [x] Added a Rust `PinballTable` landing zone that can consume `engine::TableBridgeState` as gameplay-facing messages.
- [x] Added `src/gameplay/components/component.rs` with a `GameplayComponent` trait and table-owned trait-object dispatch.
- [x] Added `src/engine/physics/` with initial `ball`, `edge`, and `collision` primitives.
- [x] Added `src/engine/physics/ball.rs` and connected it to a simple launch -> flip -> drain simulation loop.
- [x] Wired runtime input, fixed-step updates, and `PinballTable` ownership together so the main loop advances gameplay state.
- [x] Added `src/gameplay/mechanics/mod.rs` plus `flipper.rs`, `plunger.rs`, and `drain.rs` for the first concrete mechanic components.
- [x] Moved the initial launch, flipper, and drain behavior out of `PinballTable` and into `GameplayComponent` implementations backed by shared simulation state.
- [x] Added `src/engine/physics/edge_manager.rs` and `src/engine/physics/flipper_edge.rs` for reusable table-boundary and flipper-edge collision management.
- [x] Routed the active ball through simple segment collision resolution instead of relying only on open-loop motion.
- [x] Repaired the DAT parser header layout so the Rust port can successfully load `PINBALL.DAT` again.
- [x] Added startup diagnostics in the window title and status overlay so DAT-load and render-path failures are visible at runtime.
- [x] Kept `src/GameState/mod.rs` and `src/GameState/assets.rs` as compatibility facades so the refactor remains incremental.
- [x] Updated `src/main.rs` to act as a bootstrap-only entrypoint that delegates platform behavior.
- [x] Split `src/gameplay/components/table.rs` into smaller gameplay-table submodules for input state, text-box ownership, and visual composition while preserving the existing gameplay/render export surface.
- [x] Verified the refactor with `cargo check` from `SpaceCadetPinballRust`.

## Current State

- The crate now has a real module skeleton that matches the planned ownership boundaries.
- The DAT loading logic is no longer trapped inside the legacy `GameState` module.
- Runtime state is separated from asset parsing and no longer depends on the old module layout.
- `GameState` still exists as a facade, which keeps future moves lower-risk.
- `main.rs` now delegates SDL shell concerns into `platform::sdl_app`, `platform::fullscreen`, and `platform::input`.
- Phase 1 platform modules now exist, but audio, UI, and options persistence are only scaffolds so far.
- Phase 2 has started: typed group lookups and basic loader metadata are no longer defined in `src/assets/dat.rs`.
- DAT resource discovery now starts in `assets::embedded` instead of `engine::runtime`, which keeps fallback/resource concerns inside the assets layer.
- Group construction and normalization now live in `assets::group`, leaving `src/assets/dat.rs` focused on DAT models, record parsing, and low-level binary readers.
- Phase 3 foundations now exist under `src/engine/`, and `main.rs` no longer needs to know how assets become drawable sprites.
- The DAT loader is working again, and the current renderer can now display real asset textures instead of failing at startup.
- The render path now uses the DAT background palette, starts from the named `background` group, and can draw controlled DAT sprite sequences for the ball, plunger, and both flippers.
- Render now consumes a gameplay-owned table visual snapshot for ball/plunger/flipper presentation and a first HUD slice instead of having bootstrap code query those gameplay facts individually.
- The current DAT group-name mapping for that active render slice now also lives in gameplay-owned visual state rather than in `engine::render`.
- The first HUD slice is now represented as a generic gameplay-owned widget list, so `engine::render` no longer hard-codes the current score/ballcount/player widget set.
- The current mechanic-driven sprite set is now represented as a generic gameplay-owned sequence list, so `engine::render` no longer hard-codes the current plunger/left-flipper/right-flipper set either.
- The live bitmap sprite path is now represented as a generic gameplay-owned bitmap sprite list, so `engine::render` no longer has a ball-specific runtime sprite path either.
- The current composition and draw order for the active mechanic/HUD slice now lives in a gameplay-owned unified visual list, so `engine::render` no longer owns separate sequence, HUD, and bitmap passes for that slice.
- HUD widget layout decoding now lives in the asset lookup layer as a typed helper, so `engine::render` no longer parses raw DAT layout bytes for the current HUD slice.
- Named sequence traversal and frame selection now live in the asset lookup layer as a typed helper, so `engine::render` no longer owns that DAT walk for the current mechanic slice.
- Number-widget digit-to-group lookup now lives in the asset lookup layer as a typed helper, so `engine::render` no longer traverses font sequences for the current HUD slice.
- Generic bitmap group-name lookup now lives in the asset lookup layer as a typed helper, so `engine::render` no longer resolves named bitmap records for the current live bitmap path.
- Scene debug label lookup now also lives in the asset lookup layer, so `engine::render` no longer reaches into group internals even for the active-scene debug summary.
- A first gameplay-owned table-light slice now exists, with current mechanic state driving a small set of DAT-backed lights alongside the existing plunger/flipper/HUD visuals.
- The first gameplay-owned table-light slice now also includes plunger-charge indicator lights, so more of the active mechanic state is reflected through DAT-backed table elements.
- The gameplay-owned visual snapshot now uses a dynamic visual list and drives a much larger mechanic-backed light bank in one pass, substantially increasing the number of DAT-backed sprites active in the current slice.
- The current mechanic-backed light banks now extend further into higher score and launch/drain milestones, raising the sprite count again without adding new render-owned logic.
- The render path now matches the original C++ table/sidebar positioning model closely enough for the playfield and scoreboard to appear in the correct relative screen positions.
- The gameplay-owned visual snapshot now also carries a large default-state sequence bank for visible table objects such as bumpers, kickbacks, kickouts, gates, sinks, rebounders, rollovers, and the ramp family, substantially increasing sprite coverage in one pass without reopening render-owned selection logic.
- The gameplay-owned visual snapshot now also carries a large default-state sequence bank for DAT light-group and bargraph assets such as circle groups, target-light groups, trek lights, hyperspace lights, worm-hole lights, and the fuel bargraph, further increasing visible sprite coverage in one pass.
- That gameplay-owned DAT light-group sequence bank now also includes the remaining `right_target_lights` family, closing another visible gap without adding a new render or rules abstraction.
- The gameplay-owned visual snapshot now also carries a large default-state sequence bank for popup-target, solo-target, and tripwire assets, bringing another broad visible table-object family under gameplay-owned composition in one pass.
- The gameplay-owned visual snapshot now also carries a large remaining bank of individual DAT light groups and rollover-light groups through default light states, substantially widening table-light coverage in one pass without adding new rules ownership yet.
- The gameplay-owned visual snapshot now also carries the remaining currently-known standalone DAT light groups through default light states, further widening table-light coverage in one pass without adding new rules ownership yet.
- The gameplay-owned visual snapshot now also carries the remaining visible DAT one-way sequence variants, widening that table-object family in one pass without changing gameplay ownership or introducing a new render path.
- The gameplay-owned visual snapshot now includes a gameplay-owned text-box queue for `info_text_box` and `mission_text_box`, with DAT-driven bounds, queued/timed message ownership, a DAT-backed message-font render path with closer C++-style line fitting, and SDL clipping to the text-box bounds; in the current full-frame redraw renderer, that covers the practical clear/redraw path without needing a separate background-restore subsystem.
- The gameplay-owned visual snapshot now also carries the base `background` and `table` bitmaps, so the last remaining render-owned scene assembly for the table shell is no longer separate from gameplay-owned visual composition.
- The gameplay-owned visual snapshot now also drives a first subset of DAT light-group/bargraph sequences from live simplified gameplay state instead of leaving those groups pinned to frame `0.0`.
- The gameplay-owned visual snapshot now also drives the bumper sequence family from live simplified gameplay state instead of leaving those multi-frame sequences pinned to their first frame.
- The gameplay-owned visual snapshot now also drives the kickback sequence family from live simplified gameplay state instead of leaving those multi-frame sequences pinned to their first frame.
- The gameplay-owned visual snapshot now also drives the flag sequence family from live simplified gameplay state instead of leaving those multi-frame sequences pinned to their first frame.
- The gameplay-owned visual snapshot now also drives the gate sequence family from live simplified gameplay state instead of leaving those multi-frame sequences pinned to their first frame.
- The gameplay-owned visual snapshot now also drives the kickout sequence family from live simplified gameplay state instead of leaving those multi-frame sequences pinned to their first frame.
- The gameplay-owned visual snapshot now also drives the sink, one-way, rebounder, rollover, target, and tripwire sequence families from live simplified gameplay state instead of leaving those multi-frame sequences pinned to their first frames.
- The gameplay-owned visual snapshot now also drives the remaining static-table and light-group sequence banks from live simplified gameplay state instead of leaving those sequences pinned to their first frames.
- The gameplay-owned visual snapshot now also drives the remaining default table-light and rollover-light banks from live simplified gameplay state and live ball position instead of leaving those light states pinned to `0.0`.
- The gameplay-owned light-group sequence bank now also uses per-family region-aware progress derived from live ball position and simplified table state instead of one shared generic blend across all named groups.
- The gameplay-owned static-table sequence bank now also uses per-group region-aware progress, with the ramp entries following ramp semantics and `v_bloc1` following blocker-style lower-table semantics instead of one shared blend.
- The remaining generic lane-ready and ball-region signals now live in gameplay-owned `SimulationState` instead of being recomputed ad hoc inside the render-facing visual builder.
- The derived left/right/top/bottom/ramp table-region semantics now also live in gameplay-owned `SimulationState`, so the visual builder consumes gameplay-owned region signals instead of deriving those semantics itself.
- The remaining broad blend formulas for sequence and light families now also live in gameplay-owned `SimulationState` as named visual signals, so the visual builder consumes gameplay-owned focus/state channels instead of assembling those mixes inline.
- The gameplay table now also owns lightweight ramp-side and lower-hazard activity signals that persist across frames, so related sequence families can respond to simple gameplay activity instead of only static blends.
- The gameplay table now also owns lightweight orbit-side and target-side activity signals that persist across frames, so tripwire, target, and related light-group families can follow simple gameplay activity instead of only generic focus blends.
- The gameplay table now also owns a lightweight bumper activity signal that persists across frames, so the bumper sequence family can follow simple gameplay activity instead of only a generic focus blend.
- The gameplay table now also owns a lightweight lane/skill-shot activity signal that persists across frames, so the skill-shot and lane visuals can follow simple gameplay activity instead of only a generic progress blend.
- The gameplay table landing zone is no longer concentrated in a single `src/gameplay/components/table.rs` file: input-state translation, text-box queue/status ownership, and visual composition now live in focused gameplay-table submodules behind the same public API.
- Controlled Sprite Reintegration is well underway: plunger and flipper frames now come from named DAT sprite sequences, the window title exposes the active scene and controlled sprite selections for debugging, gameplay-owned visual state now covers the current table mechanics slice, an initial HUD slice, and a substantially larger table-light slice, and `engine::render` no longer owns the current group-name mapping, the current HUD widget set, the current mechanic sequence set, the current live bitmap sprite path, the current visual composition/order, the current HUD layout decode, the current sequence-frame lookup, the current HUD digit lookup, or the current named-bitmap/debug-label lookups for that slice.

- `src/platform/`: SDL bootstrap, fullscreen policy, input translation, bindings/options, audio, and UI glue.
- `src/assets/`: DAT parsing, typed group lookup, loader-style metadata extraction, and embedded fallback resources.
- `src/engine/render/`: sprite records, scene assembly, texture conversion/cache, and z-aware presentation decisions.
- `src/engine/physics/`: ball state, edge registries, collision routines, and flipper-edge special handling.
- `src/engine/runtime.rs`: global runtime context, input dispatch entrypoints, active table ownership, and frame orchestration.
- `src/engine/time.rs`: fixed-step timing, scheduled callbacks, and frame cadence helpers.
- `src/gameplay/components/`: component base types, message routing, grouping, and the `TPinballTable` landing zone.
- `src/gameplay/mechanics/`: flipper, plunger, drain, hole, gate, kickback, kickout, ramp, one-way, and wall/blocker behavior.
- `src/gameplay/rules/`: scoring, lights, targets, timers, high score flow, and attract/demo logic.
- `src/engine/debug.rs` or `src/gameplay/debug.rs`: overlay, sprite viewer, parity instrumentation, and diagnostics.

## In Progress

- [ ] Expand the new mechanic components beyond the current simplified launch/flipper/drain behavior.
- [ ] Deepen the new collision layer beyond simple segment reflection and table-boundary edges.
- [ ] Replace the new platform placeholders with real options persistence, audio wiring, and UI behavior as later slices need them.
- [x] Replace the placeholder texture conversion with palette-aware DAT rendering.
- [x] Narrow scene assembly so the renderer starts from the table background instead of compositing every bitmap-backed group.
- [ ] Layer gameplay-relevant sprites back in deliberately after the background-only path is stable.
- [x] Add controlled DAT sprite passes for plunger and flippers.
- [x] Add a render debug view that shows which named DAT groups are actively being drawn.
- [x] Replace ad-hoc gameplay overlays with a gameplay-owned visual snapshot for the current ball/plunger/flipper render state.
- [x] Expand gameplay-owned visual state with a first DAT-backed HUD slice for score, ball count, and player number.
- [x] Move the current ball/plunger/flipper/HUD group-name selection out of `engine::render` and into gameplay-owned visual state.
- [x] Move the current HUD widget list out of `engine::render` and into gameplay-owned visual state.
- [x] Move the current mechanic-driven sprite set out of `engine::render` and into gameplay-owned visual state.
- [x] Move the current live bitmap sprite path out of `engine::render` and into gameplay-owned visual state.
- [x] Move the current visual composition and draw order out of `engine::render` and into gameplay-owned visual state.
- [x] Move HUD layout decoding out of `engine::render` and into typed asset helpers.
- [x] Move named sequence traversal/frame selection out of `engine::render` and into typed asset helpers.
- [x] Move number-widget digit lookup out of `engine::render` and into typed asset helpers.
- [x] Move generic bitmap name lookup out of `engine::render` and into typed asset helpers.
- [x] Move scene debug label lookup out of `engine::render` and into typed asset helpers.
- [x] Add a first gameplay-owned table-light slice driven by current mechanic state.
- [x] Expand the first gameplay-owned table-light slice with plunger-charge indicator lights.
- [x] Expand the gameplay-owned table-light slice with ball-status indicator lights.
- [x] Expand the gameplay-owned table-light slice with score-threshold indicator lights.
- [x] Expand the gameplay-owned table-light slice with launch-count milestone lights.
- [x] Expand the gameplay-owned table-light slice with drain-count milestone lights.
- [x] Replace the fixed visual array with a dynamic gameplay-owned visual list and expand the current light bank substantially in one pass.
- [x] Extend the current score and launch/drain light banks with additional DAT-backed milestone sprites.
- [ ] Expand gameplay-owned visual state further so more table elements stop accumulating in render/bootstrap glue.

## Next Steps

### Phase 1: Platform shell cleanup

- [x] Add `src/platform/sdl_app.rs` for SDL bootstrap and app-loop helpers.
- [x] Add `src/platform/fullscreen.rs` and move fullscreen policy out of `main.rs`.
- [x] Expand `src/platform/input.rs` from action constants into SDL event translation helpers.
- [x] Add `src/platform/input_bindings.rs` for options-backed action mapping.
- [x] Add `src/platform/options.rs` for persistent settings and bindings.
- [x] Add `src/platform/audio.rs` for `Sound` and MIDI adapter responsibilities.
- [x] Add `src/platform/ui.rs` for menu/dialog glue once gameplay-facing APIs settle.

### Phase 2: Asset layer split

- [x] Add `src/assets/group.rs` for typed group and entry lookup helpers.
- [x] Add `src/assets/loader.rs` for loader-style metadata extraction.
- [x] Keep `src/assets/dat.rs` focused on raw file structures and parsing.
- [x] Add `src/assets/embedded.rs` for bundled or fallback resource helpers if the port needs embedded data parity.

### Phase 3: Engine foundations and render path

- [x] Add `src/engine/math.rs` for vector and scalar primitives.
- [x] Add `src/engine/geom.rs` for rays, circles, rectangles, and projection helpers.
- [x] Add `src/engine/bitmap.rs` for bitmap, palette, and z-map primitives that render and assets can share.
- [x] Add `src/engine/time.rs` for fixed-step timing and callback scheduling.
- [x] Add `src/engine/render/mod.rs` as the render subsystem entrypoint.
- [x] Add `src/engine/render/sprite.rs` for sprite records and sprite state.
- [x] Add `src/engine/render/scene.rs` for scene assembly, layering, and dirty-region style presentation inputs.
- [x] Add `src/engine/render/texture_cache.rs` for asset bitmap to SDL texture conversion and caching.

### Immediate Render Stabilization

- [x] Apply the real DAT palette in `src/engine/render/texture_cache.rs` instead of the current placeholder color mapping.
- [x] Restrict `src/engine/render/scene.rs` to build an initial background-focused scene instead of dumping every bitmap-backed group.
- [x] Keep the debug ball overlay, but reintroduce gameplay and animated sprites in controlled passes once the base table image is correct.
- [x] Re-run `cargo run` after each render slice and capture the visible result in the plan notes if a new blocker appears.

### Controlled Sprite Reintegration

- [x] Add a controlled DAT sprite pass for the plunger so launch-lane visuals come from assets instead of debug geometry.
- [x] Add controlled DAT sprite passes for left and right flippers with explicit mapping from gameplay state to named asset groups.
- [x] Add a small render debug surface that lists active sprite/group names so sprite-selection work stays inspectable.
- [x] Move the current ball/plunger/flipper render decisions behind a gameplay-owned table visual snapshot.
- [x] Expand that gameplay-owned visual snapshot to cover an initial HUD slice for score, ball count, and player number.
- [x] Move the current group-name mapping for the active mechanics/HUD render slice out of `engine::render` and into gameplay-owned visual state.
- [x] Generalize the first HUD slice into a gameplay-owned widget list so render no longer owns the current widget set.
- [x] Generalize the current mechanic slice into a gameplay-owned sequence list so render no longer owns the current plunger/flipper set.
- [x] Generalize the live bitmap sprite path into a gameplay-owned bitmap sprite list so render no longer owns the current ball-specific sprite path.
- [x] Generalize the current visual composition into a gameplay-owned unified visual list so render no longer owns separate passes or ordering for the active slice.
- [x] Move HUD layout decoding into a typed asset helper so render no longer parses raw layout records for the active HUD slice.
- [x] Move named sequence traversal and frame selection into a typed asset helper so render no longer owns that DAT lookup for the active mechanic slice.
- [x] Move number-widget digit lookup into a typed asset helper so render no longer owns font-sequence traversal for the active HUD slice.
- [x] Move generic bitmap name lookup into a typed asset helper so render no longer owns named bitmap resolution for the active live bitmap path.
- [x] Move scene debug label lookup into a typed asset helper so render no longer reaches into group internals for the active scene summary.
- [x] Add a first gameplay-owned light family so the active visual snapshot covers more than ball/flipper/plunger/HUD state alone.
- [x] Expand that first gameplay-owned light family with plunger-charge indicator lights.
- [x] Expand that gameplay-owned light family with ball-status indicator lights.
- [x] Expand that gameplay-owned light family with score-threshold indicator lights.
- [x] Expand that gameplay-owned light family with launch-count milestone lights.
- [x] Expand that gameplay-owned light family with drain-count milestone lights.
- [x] Switch the active visual snapshot to a dynamic list and use it to drive a substantially larger mechanic-backed light bank in one run.
- [x] Extend the current score and launch/drain light banks with another contiguous set of DAT-backed milestone sprites.
- [x] Bulk-add a large default-state sequence bank for visible table objects through the gameplay-owned visual snapshot.
- [x] Bulk-add a large default-state sequence bank for DAT light-group and bargraph assets through the gameplay-owned visual snapshot.
- [x] Expand the gameplay-owned DAT light-group sequence bank to include `right_target_lights`.
- [x] Bulk-add a large default-state sequence bank for popup-target, solo-target, and tripwire assets through the gameplay-owned visual snapshot.
- [x] Bulk-add a large remaining bank of individual DAT light groups through default gameplay-owned light states.
- [x] Expand that default gameplay-owned light bank with the remaining currently-known standalone DAT light groups.
- [x] Expand the gameplay-owned one-way sequence bank to include the remaining visible DAT one-way variants.
- [x] Add a first gameplay-owned text-box path for `info_text_box` and `mission_text_box` with DAT-driven layout and fallback text rendering.
- [x] Improve the text-box path toward practical C++ parity: DAT-backed font layout, queued/timed ownership, and clipped text-box redraw behavior.
- [x] Move the base `background` and `table` layer into the gameplay-owned visual snapshot.
- [x] Turn a first subset of DAT light-group/bargraph sequences into mechanic-backed gameplay-owned selections.
- [x] Turn the multi-frame bumper family into gameplay-driven sequence selections.
- [x] Turn the multi-frame kickback family into gameplay-driven sequence selections.
- [x] Turn the multi-frame flag family into gameplay-driven sequence selections.
- [x] Turn the multi-frame gate family into gameplay-driven sequence selections.
- [x] Turn the multi-frame kickout family into gameplay-driven sequence selections.
- [x] Turn the sink, one-way, rebounder, rollover, target, and tripwire families into gameplay-driven sequence selections.
- [x] Turn the remaining static-table and light-group sequence banks into gameplay-driven sequence selections.
- [x] Turn the remaining default table-light and rollover-light banks into gameplay-driven light selections.
- [x] Refine the gameplay-owned light-group sequence bank from one generic progress blend into per-family region-aware selection.
- [x] Refine the gameplay-owned static-table sequence bank from one generic progress blend into per-group region-aware selection.
- [x] Move the remaining generic lane-ready and ball-region progress signals out of the render-facing visual builder and into gameplay-owned simulation state.
- [x] Move the derived left/right/top/bottom/ramp region semantics out of the render-facing visual builder and into gameplay-owned simulation state.
- [x] Move the remaining broad sequence/light blend formulas out of the render-facing visual builder and into gameplay-owned simulation state as named visual signals.
- [x] Add lightweight gameplay-owned ramp and lower-hazard activity signals and route related families through them.
- [x] Add lightweight gameplay-owned orbit and target activity signals and route related families through them.
- [x] Add lightweight gameplay-owned bumper activity signal and route the bumper sequence family through it.
- [x] Add lightweight gameplay-owned lane/skill-shot activity signal and route skill-shot/lane visuals through it.
- [ ] Expand that gameplay-owned visual snapshot further to cover additional gameplay-driven table elements beyond the current simplified sequence/light-bank ownership.

### Phase 4: Gameplay scaffolding

- [x] Add `src/gameplay/components/mod.rs`.
- [x] Add `src/gameplay/components/table.rs` for the Rust `TPinballTable` landing zone.
- [x] Add `src/gameplay/components/group.rs` for component grouping and lookup helpers.
- [x] Add `src/gameplay/components/messages.rs` for table/component message contracts.
- [x] Split the gameplay-table landing zone into smaller `src/gameplay/components/table/` submodules so visual composition and text-box state stop accumulating in one file.

### Phase 5: First playable slice

- [x] Add `src/engine/physics/` with initial ball, edge, and collision primitives.
- [x] Add `src/engine/physics/ball.rs` for the `TBall` equivalent.
- [x] Add `src/engine/physics/edge.rs`, `edge_manager.rs`, `collision.rs`, and `flipper_edge.rs` for reusable collision infrastructure.
- [x] Add `src/gameplay/mechanics/flipper.rs`.
- [x] Add `src/gameplay/mechanics/plunger.rs`.
- [x] Add `src/gameplay/mechanics/drain.rs`.
- [x] Connect runtime, physics, and table ownership into a launch -> flip -> drain loop.

### Phase 6: Rules and progression

- [ ] Add `src/gameplay/rules/mod.rs`.
- [ ] Add `src/gameplay/rules/scoring.rs` for score state, multipliers, jackpots, and bonus bookkeeping.
- [ ] Add `src/gameplay/rules/lights.rs` for `TLight`, `TLightGroup`, `TLightBargraph`, and `TLightRollover` families.
- [ ] Add `src/gameplay/rules/targets.rs` for bumpers, popup targets, solo targets, rollovers, spinner, and tripwire behavior.
- [ ] Add `src/gameplay/rules/timers.rs` for gameplay-level timers layered on top of `engine::time`.
- [ ] Add `src/gameplay/high_score.rs` and `src/gameplay/demo.rs` once the first playable loop is stable.

### Phase 7: UI and debug parity

- [ ] Decide whether menus and overlays belong in `platform::ui` or `gameplay` based on whether they mostly express toolkit glue or game semantics.
- [ ] Add a dedicated debug module for overlays, sprite inspection, frame timing tools, and parity diagnostics.

## Current File Migration Map

- `src/main.rs`: keep as bootstrap-only entrypoint that delegates event handling and runtime work into `platform` and `engine`.
- `src/GameState/mod.rs`: continue shrinking toward a compatibility facade over `engine::runtime` plus `platform::input`.
- `src/GameState/assets.rs`: keep as a temporary re-export shim while `assets::group` and `assets::loader` come online.
- `src/assets/dat.rs`: now holds raw DAT structures, parse-time decoding, and binary readers only.
- `src/assets/group.rs`: now owns typed lookup helpers plus post-parse group assembly and bitmap/zMap normalization.
- `src/assets/embedded.rs`: own asset-path discovery and eventual embedded fallback resources instead of leaving that logic in runtime.
- `src/engine/bitmap.rs`: now owns shared bitmap/zMap primitives and resolution helpers used by both assets and render.
- `src/engine/render/`: now owns sprite extraction, scene ordering, and SDL texture caching instead of leaving draw preparation in bootstrap code.
- `src/gameplay/components/`: now owns the first Rust table/component/message scaffold instead of leaving gameplay as a placeholder.
- `src/gameplay/components/component.rs`: now defines the gameplay component trait boundary that future mechanics plug into.
- `src/engine/physics/`: now owns the first ball primitive and the initial collision/edge landing zone for gameplay motion.
- `src/gameplay/mechanics/`: now owns the first concrete gameplay mechanics instead of leaving launch/flipper/drain behavior inside `PinballTable`.
- `src/engine/physics/edge_manager.rs` and `src/engine/physics/flipper_edge.rs`: now own the reusable wall/flipper collision surface for the current playable slice.

## Risks

- The old `GameState` facade can become permanent debt if follow-up moves stall.
- `main.rs` still contains direct platform behavior that should migrate before gameplay code grows.
- The renderer now has a stable baseline, but sprite selection is still manually staged and can drift from gameplay semantics if more hooks accumulate in bootstrap/render glue.
- The component dispatch model is still undecided; choosing between trait objects, enum registries, or a hybrid approach will affect borrowing and ownership across the gameplay port.
- The current gameplay mechanics are intentionally simplified, so physics behavior is not yet parity-accurate.
- The new collision layer currently handles simple segment reflection only; ramps, complex edge chains, and richer contact ordering are still missing.
- UI and options code can sprawl into gameplay if the platform boundary is not kept strict.

## Verification Criteria

- Confirm that dependencies remain one-way: `platform/assets -> engine -> gameplay -> ui/debug`.
- Confirm that each major C++ subsystem has a Rust landing zone before implementation of that slice begins.
- Confirm that the first playable slice can be built without waiting on menu, dialog, or polish-only UI work.
- Keep using compatibility re-exports and facade modules to avoid a flag-day rewrite.

## Open Decisions

- Decide whether ImGui remains the configuration/debug UI backend before `platform::ui` grows further.
- Decide whether gameplay component dispatch uses trait objects, enum-based registries, or a hybrid model before `gameplay::components` becomes large.
- If the assets layer keeps expanding quickly, split loader metadata from raw parsing early so parser code stays deterministic and independently testable.

## Validation Log

- 2026-04-27: `cargo check` passed after adding gameplay-owned orbit and target activity signals and routing related families through them.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-owned orbit/target activity pass.
- 2026-04-27: `cargo check` passed after adding gameplay-owned ramp and lower-hazard activity signals and routing related families through them.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-owned activity-signal pass.
- 2026-04-27: `cargo check` passed after moving the remaining broad sequence/light blend formulas into gameplay-owned simulation state as named visual signals.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-owned visual-signal pass.
- 2026-04-27: `cargo check` passed after moving the derived left/right/top/bottom/ramp region semantics into gameplay-owned simulation state.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-owned derived-region pass.
- 2026-04-27: `cargo check` passed after moving the remaining generic lane-ready and ball-region progress signals into gameplay-owned simulation state.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-owned region-signal pass.
- 2026-04-27: `cargo check` passed after refining the gameplay-owned static-table sequence bank to use per-group region-aware progress.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the region-aware static-table pass.
- 2026-04-27: `cargo check` passed after refining the gameplay-owned light-group sequence bank to use per-family region-aware progress.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the region-aware light-group pass.
- 2026-04-27: `cargo check` passed after turning the remaining default table-light and rollover-light banks into gameplay-driven selections.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the remaining-light-bank gameplay-driven pass.
- 2026-04-27: `cargo check` passed after turning the remaining static-table and light-group sequence banks into gameplay-driven selections.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the remaining-sequence-bank gameplay-driven pass.
- 2026-04-27: `cargo check` passed after turning the sink, one-way, rebounder, rollover, target, and tripwire sequence families into gameplay-driven selections.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the multi-family gameplay-driven sequence pass.
- 2026-04-27: `cargo check` passed after turning the kickout sequence family into gameplay-driven selections.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-driven kickout pass.
- 2026-04-27: `cargo check` passed after turning the gate sequence family into gameplay-driven selections.
- 2026-04-27: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-driven gate pass.
- 2026-04-26: `cargo check` passed after splitting `src/gameplay/components/table.rs` into smaller gameplay-table submodules.
- 2026-04-26: `cargo check` passed after bulk-adding a large remaining bank of individual DAT light groups through default gameplay-owned light states.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the bulk remaining-light-bank pass.
- 2026-04-26: `cargo check` passed after expanding the gameplay-owned one-way sequence bank to include the remaining visible DAT one-way variants.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the widened one-way sequence pass.
- 2026-04-26: `cargo check` passed after moving text-box queue and timing ownership into gameplay state.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-owned text-box queue/timing pass.
- 2026-04-26: `cargo check` passed after switching text-box rendering to the DAT-backed message font path with closer line fitting.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the DAT-backed text-box font pass.
- 2026-04-26: `cargo check` passed after clipping text-box rendering to the DAT-defined bounds.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the clipped text-box pass.
- 2026-04-26: `cargo check` passed after moving the base `background` and `table` layer into the gameplay-owned visual snapshot.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-owned base-layer pass.
- 2026-04-26: `cargo check` passed after turning a first subset of DAT light-group/bargraph sequences into mechanic-backed gameplay-owned selections.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the mechanic-backed light-group/bargraph pass.
- 2026-04-26: `cargo check` passed after turning the bumper sequence family into gameplay-driven selections.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-driven bumper pass.
- 2026-04-26: `cargo check` passed after turning the kickback sequence family into gameplay-driven selections.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-driven kickback pass.
- 2026-04-26: `cargo check` passed after turning the flag sequence family into gameplay-driven selections.
- 2026-04-26: `cargo run --manifest-path SpaceCadetPinballRust/Cargo.toml` launched successfully after the gameplay-driven flag pass.

## Notes

- Treat this file as the working progress tracker for the Rust port.
- Update the Completed, In Progress, Next Steps, and Validation Log sections after each implementation slice.
- Immediate Render Stabilization now has a stable baseline: DAT palette conversion, background-only scene assembly, and a controlled DAT ball-sprite pass are in place.
- Controlled Sprite Reintegration is well underway: plunger and flipper frames now come from named DAT sprite sequences, the window title exposes the active scene and controlled sprite selections for debugging, gameplay-owned visual state now covers both the current table mechanics slice and an initial HUD slice, and the current group-name mapping for that slice no longer lives in `engine::render`.
- Controlled Sprite Reintegration is well underway: plunger and flipper frames now come from named DAT sprite sequences, the window title exposes the active scene and controlled sprite selections for debugging, gameplay-owned visual state now covers the current table mechanics slice, an initial HUD slice, and an expanding table-light slice, and `engine::render` no longer owns the current group-name mapping, the current HUD widget set, the current mechanic sequence set, the current live bitmap sprite path, the current visual composition/order, the current HUD layout decode, the current sequence-frame lookup, the current HUD digit lookup, or the current named-bitmap/debug-label lookups for that slice.
- The render baseline now also includes a validated positioning fix: the playfield is anchored in table-local space and the scoreboard/sidebar no longer overlaps it.
- Controlled Sprite Reintegration now also includes a validated bulk light-group/bargraph pass, bringing another large DAT-backed sprite family under the gameplay-owned visual snapshot in one slice.
- Controlled Sprite Reintegration now also includes a validated `right_target_lights` follow-up pass, closing the remaining obvious gap in the current light-group sequence bank with no new render-side logic.
- Controlled Sprite Reintegration now also includes a validated bulk default-state table-object pass, bringing a much larger visible sprite set under the gameplay-owned visual snapshot in one slice.
- Controlled Sprite Reintegration now also includes a validated bulk target/tripwire pass, bringing popup-target, solo-target, and tripwire families under the gameplay-owned visual snapshot in one slice.
- Controlled Sprite Reintegration now also includes a validated bulk remaining-light-bank pass, bringing a much wider set of individual table lights under gameplay-owned default light-state composition in one slice.
- Controlled Sprite Reintegration now also includes a validated remaining standalone-light pass, widening the default gameplay-owned light bank again by folding in the remaining currently-known DAT light groups from the dump.
- Controlled Sprite Reintegration now also includes a validated widened one-way pass, bringing the remaining visible DAT one-way variants under the gameplay-owned sequence bank in one slice.
- Controlled Sprite Reintegration now also includes a validated clipped text-box slice, bringing `info_text_box` and `mission_text_box` under gameplay-owned visual composition with DAT-driven bounds, queued/timed messages, closer C++-style bitmap-font layout, and bounded redraw behavior that fits the current full-frame renderer.
- Controlled Sprite Reintegration now also includes a validated gameplay-owned base-layer slice, bringing the `background` and `table` shell under the same gameplay-owned visual composition path as the rest of the current renderable table state.
- Controlled Sprite Reintegration now also includes a validated mechanic-backed light-group/bargraph slice, turning a first subset of those DAT sequence families from default-state placeholders into live gameplay-owned selections.
- Controlled Sprite Reintegration now also includes a validated gameplay-driven bumper slice, turning that multi-frame sequence family from a default-state placeholder into live gameplay-owned selections.
- Controlled Sprite Reintegration now also includes a validated gameplay-driven kickback slice, turning that multi-frame sequence family from a default-state placeholder into live gameplay-owned selections.
- Controlled Sprite Reintegration now also includes a validated gameplay-driven flag slice, turning that multi-frame sequence family from a default-state placeholder into live gameplay-owned selections.
- Controlled Sprite Reintegration now also includes a validated gameplay-driven gate slice, turning that multi-frame sequence family from a default-state placeholder into live gameplay-owned selections.
- Controlled Sprite Reintegration now also includes a validated gameplay-driven kickout slice, turning that multi-frame sequence family from a default-state placeholder into live gameplay-owned selections.
- Controlled Sprite Reintegration now also includes a validated multi-family slice, turning the sink, one-way, rebounder, rollover, target, and tripwire sequence families from default-state placeholders into live gameplay-owned selections in one pass because they shared the same visual-selection code path.
- Controlled Sprite Reintegration now also includes a validated remaining-sequence-bank slice, turning the static-table and light-group banks from default-state placeholders into live gameplay-owned selections in one pass because they shared the same visual-selection code path.
- Controlled Sprite Reintegration now also includes a validated remaining-light-bank slice, turning the default table-light and rollover-light banks from `0.0` placeholders into live gameplay-owned selections in one pass because they shared the same light-selection code path.
- Controlled Sprite Reintegration now also includes a validated region-aware light-group slice, replacing the old one-blend light-group approximation with per-family progress derived from left/right/top/ramp-style table semantics.
- Controlled Sprite Reintegration now also includes a validated region-aware static-table slice, replacing the old one-blend static-table approximation with per-group progress derived from ramp and blocker-style table semantics.
- Controlled Sprite Reintegration now also includes a validated gameplay-owned region-signal slice, moving the remaining generic lane-ready and ball-region progress ownership out of the render-facing visual builder and into gameplay state.
- Controlled Sprite Reintegration now also includes a validated gameplay-owned derived-region slice, moving the derived left/right/top/bottom/ramp table semantics out of the render-facing visual builder and into gameplay state.
- Controlled Sprite Reintegration now also includes a validated gameplay-owned visual-signal slice, moving the remaining broad sequence/light blend formulas out of the render-facing visual builder and into gameplay state as named focus channels.
- Controlled Sprite Reintegration now also includes a validated gameplay-owned activity-signal slice, adding lightweight ramp-side and lower-hazard activity channels that related families can follow instead of relying only on static blends.
- Controlled Sprite Reintegration now also includes a validated gameplay-owned orbit/target activity slice, adding lightweight orbit-side and target-side activity channels that tripwire, target, and related light-group families can follow instead of relying only on generic focus blends.
- The gameplay-table landing zone has now been structurally split into smaller submodules, reducing pressure on `src/gameplay/components/table.rs` before additional gameplay-owned sprite reintegration continues.