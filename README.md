# CLI Backgammon

A terminal-based backgammon game written in Rust, featuring a complete implementation of traditional backgammon rules with save/load functionality, replay system, and leaderboard.

## Features

- **Full Backgammon Implementation**: Complete rule set including checker movement, hitting, bearing off, and doubling dice
- **Interactive Terminal UI**: Clean ASCII-based game board with intuitive controls
- **Save/Load System**: Save your games and continue later
- **Replay System**: Record and watch replays of completed games
- **Leaderboard**: Track wins and maintain player statistics
- **Two-Player Local Play**: Play against another human player on the same computer

## Installation

### Prerequisites
- Rust 1.70+ (uses 2024 edition)
- Terminal with support for ANSI escape codes

### Dependencies
- **crossterm** (0.29.0) - Cross-platform terminal manipulation
- **rand** (0.9.2) - Random number generation for dice
- **chrono** (0.4) - Date/time handling for save files

### Building from Source
```bash
git clone <repository-url>
cd cli-backgammon
cargo build
```

### Running
```bash
cargo run
```

Or run the compiled binary:
```bash
./target/release/cli-backgammon
```

## Game Board Layout
- **White pieces (●)**: Move from 24 → 1, bear off at 0
- **Black pieces (○)**: Move from 1 → 24, bear off at 25
- **Bar**: Where captured pieces go (25 for white, 0 for black)
- **Tray**: Where pieces go when borne off (0 for white, 25 for black)


## Game Rules
1. **Starting**: Each player rolls one die; highest roll goes first
2. **Movement**: Move checkers according to dice rolls
3. **Hitting**: Landing on an opponent's single checker sends it to the bar
4. **Entering from Bar**: Must enter all pieces from bar before making other moves
5. **Bearing Off**: When all pieces are in home board (1-6 for white, 19-24 for black), can bear off
6. **Winning**: First player to bear off all 15 checkers wins

## File Structure
The game automatically creates a `saves/` directory with:
- `saves/games/` - Saved game states
- `saves/replays/` - Complete game replays
- `saves/leaderboard.txt` - Player statistics

## Technical Details

### Architecture
- **`src/main.rs`** - Entry point
- **`src/game.rs`** - Core game logic and state management
- **`src/utils.rs`** - Terminal utilities and display functions

### Save Format
Game states are saved in a simple text format:
```
# Board state (24 numbers representing each point)
2 0 0 0 0 5 0 3 0 0 0 20 5 0 0 0 18 0 20 0 0 0 0 2
# Current player (0=white, 1=black)
0
# Bar state (white_count black_count)
0 0
# Tray state (white_count black_count)
0 0
```

## Future Enhancements

- Improve UI, consider switching to TUI with Ratatui library
