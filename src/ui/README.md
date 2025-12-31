# UI Module

> Terminal-based user interface using crossterm and ratatui.

## Module Structure

```
ui/
├── mod.rs       # Module exports
├── terminal.rs  # Terminal setup and rendering (stub)
├── input.rs     # Input handling (stub)
└── display.rs   # Display components (stub)
```

## Status: Stub Implementation

This module is planned but not yet implemented. Currently, the game uses a simple REPL in `main.rs`.

## Current Interface

```rust
// Simple REPL in main.rs
loop {
    display_status(&world);
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match input {
        "quit" | "q" => break,
        "tick" | "t" => run_simulation_tick(&mut world),
        s if s.starts_with("spawn ") => { /* spawn entity */ }
        _ => { /* parse with LLM */ }
    }
}
```

## Planned Design

### Terminal Layout

```
┌─────────────────────────────────────────────────────────────┐
│  ARC CITADEL                              Tick: 1234        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────┐  ┌─────────────────────────────┐  │
│  │      MAP VIEW       │  │       ENTITY PANEL          │  │
│  │                     │  │                             │  │
│  │   . . . . . . .     │  │  Marcus (Human)             │  │
│  │   . . @ . . . .     │  │  ├─ Health: ████████░░ 80%  │  │
│  │   . . . . . . .     │  │  ├─ Fatigue: ██░░░░░░ 20%   │  │
│  │   . . . . . . .     │  │  └─ Top Need: Social 45%    │  │
│  │                     │  │                             │  │
│  │  @ = Selected       │  │  Current Task: IdleObserve  │  │
│  └─────────────────────┘  └─────────────────────────────┘  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  EVENT LOG                                                  │
│  > Marcus noticed Elena nearby                              │
│  > Elena started task: TalkTo Marcus                        │
│  > Thomas is feeling hungry                                 │
├─────────────────────────────────────────────────────────────┤
│  > _                                                        │
└─────────────────────────────────────────────────────────────┘
```

### Components

```rust
// Terminal setup
pub struct Terminal {
    backend: CrosstermBackend<Stdout>,
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

// Input handling
pub struct InputHandler {
    mode: InputMode,
}

pub enum InputMode {
    Normal,
    Command,
    Selection,
}

// Display components
pub struct MapView { /* ... */ }
pub struct EntityPanel { /* ... */ }
pub struct EventLog { /* ... */ }
pub struct CommandLine { /* ... */ }
```

### Input Handling

```rust
pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

impl InputHandler {
    pub fn handle(&mut self, event: InputEvent) -> Option<Command> {
        match event {
            InputEvent::Key(key) => self.handle_key(key),
            InputEvent::Mouse(mouse) => self.handle_mouse(mouse),
            _ => None,
        }
    }
}
```

## Dependencies

```toml
crossterm = "0.27"  # Terminal manipulation
ratatui = "0.25"    # TUI framework
```

## Integration Points

### With `ecs/world.rs`
- Display entity states
- Show world tick

### With `simulation/`
- Display perception results
- Show thought generation

### With `llm/`
- Command input processing
- Display parsed intents

## Future Implementation

1. **Start with terminal setup** using crossterm
2. **Add basic layout** with ratatui
3. **Implement map view** showing entity positions
4. **Add entity panel** for selected entity details
5. **Implement event log** for game events
6. **Add command input** with LLM integration
