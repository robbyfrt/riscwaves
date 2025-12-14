This document records the decision to stay with a native `pixels`-based UI and not introduce Trunk or a browser frontend at this stage.

## ADR: Native Pixels UI, No Trunk / Browser Frontend (For Now)

### Context

- Current workflow:
  - `wasm-pack build` from VS Code (optionally via a task as a shortcut).
  - Simple static server (e.g. Python) serving the generated WASM and static files.
  - Manual browser reload is sufficient; the new WASM is picked up without extra tooling.
- Goal:
  - Experiment with simulations / visualizations (e.g. particles, pixel fields).
  - Add light interactivity:
    - Mouse hover position.
    - Simple “UI” elements (boxes, sliders, toggles).
    - On-screen metrics (FPS, particle counts, etc.).
- Constraints / preferences:
  - Focus on Rust, simulations, and reasoning about design, not on building a full web UI stack.
  - Prefer efficient, concise code and minimal tooling overhead.
  - Comfortable wiring low-level primitives (events, buffers) rather than adopting a heavy frontend framework too early.

### Decision

- Do **not** introduce a “real” browser-based frontend (HTML/CSS/JS/Rust web framework) at this stage.
- Do **not** adopt Trunk for now.
- Implement interaction and HUD-like UI **inside** the native `pixels` + `winit` stack:
  - Use `winit` events for mouse/keyboard input.
  - Use `pixels` frame buffer for:
    - Visualizing the simulation.
    - Drawing simple UI elements (boxes, sliders, buttons).
    - Rendering textual metrics (FPS, counts) via bitmap/primitive text drawing.

### Rationale

- The current `wasm-pack build` + static server + manual reload flow is already “good enough” ergonomically; adding Trunk would mostly replace existing glue without unlocking new core capabilities for the current scope.
- Trunk primarily adds value for:
  - Browser/WASM-centric frontends.
  - Asset bundling, automatic cache-busting, and hot reload.
  - Larger, Rust-first web UIs.
  These are out of scope for now.
- A native Pixels app matches the current goals better:
  - Single language, single event loop, predictable performance.
  - Direct control of rendering and input, similar to owning the whole game loop in a Python/OpenGL or Pygame setup.
  - Simple to add:
    - Hover logic by mapping window coordinates to pixel coordinates.
    - Sliders and toggles as hand-rolled rectangles reacting to `winit` mouse events.
    - Metrics overlay drawn into the frame buffer.
- Avoiding a “real” frontend avoids:
  - Additional complexity of DOM, CSS, and JS (or equivalent Rust web frameworks).
  - Extra build/dev-server tooling that does not directly advance the simulation/visualization goals.

### Consequences

- Short term:
  - Development remains focused on Rust, `pixels`, and `winit`.
  - No extra dependency on Trunk or a Rust web framework.
  - UI will be minimal and custom, but highly tailored and efficient for the use case.
- Medium / long term:
  - If the project evolves into a tool that needs:
    - Rich panel-based UIs.
    - Complex input widgets (forms, text inputs, multi-pane layouts).
    - Easy sharing via browser.
    then re-evaluating:
    - A browser/WASM frontend, and
    - A build/dev server tool (such as Trunk or alternatives)
    will be appropriate.
  - Until then, effort stays concentrated on core simulation/visualization logic and Rust ergonomics, not on frontend/infrastructure.