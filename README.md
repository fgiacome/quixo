# Quixo Player

This repository contains a rust-only version of the Quixo player from the
compint repository, without the Python interface, and with the addition of a
nice Terminal User Interface (TUI). Enjoy playing!

Just run:
```bash
cargo run --release
```

## Rules
The rules for Quixo are simple:
1. You can move any tile from the edges that belongs to you or to no one yet.
2. Inner tiles cannot be moved.
3. Tiles that do not belong to you cannot be moved.
4. If you move a tile that does not belong to anybody, it becomes yours.
5. Tiles can be moved to occupy one of the terminal positions of the row or column to which they belong, other tiles in that row or column shift accordingly.

## Usage
After you launch the program, it will print the board on your terminal. You can
select a tile with arrows, and move the tile left, bottom, top, right with h, j,
k, l respectively.  If the move you request is not valid, nothing happens.  You
can request a move from the computer (computed with MCTS) with the c key.  Quit
with q, reset with r. If one of the players win, nothing happens: you only see
that the winner appears in the status bar, but you can keep making moves or
reset the board.