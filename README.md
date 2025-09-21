# Quixo Player

This repository contains a rust-only version of the Quixo player from the
[compint](https://github.com/fgiacome/compint) repository, without the Python
interface, and with the addition of a nice Terminal User Interface (TUI). Enjoy
playing!

Just run:
```bash
cargo run --release
```

## Rules
The rules for Quixo are simple:
1. Two players take turns moving tiles.
1. You can move any tile from the edges that belongs to you or to no one yet.
1. Inner tiles cannot be moved.
1. Tiles that do not belong to you cannot be moved.
1. If you move a tile that does not belong to anybody, it becomes yours.
1. Tiles can be moved to occupy one of the terminal positions of the row or column to which they belong, other tiles in that row or column shift accordingly.
1. First player that can place 5 of his tiles in a row, column or diagonal wins.

## Usage
After you launch the program, it will print the board on your terminal. You can
select a tile with arrows, and move the tile with shift + arrows. If the move
you request is not valid, nothing happens.  You can request a move from the
computer (computed with MCTS) with the c key.  Quit with q, reset with r. If one
of the players wins, nothing happens: you only see that the winner appears in
the status bar, but you can keep making moves or reset the board.