# Cursed Tetris

A terminal tetris game made using Ratatui and Crossterm in Rust, with some powerups for extra variety.

The implementation is pretty barebones, and mostly just an experience for me to learn Rust.

**Note: Maximize your terminal when running the program!**

| ![Title Screen](title.png) | ![Gameplay](gameplay.png) |
|---|---|

## Download instructions

**Linux/MacOS**

1. Download the Linux or MacOS build of the [latest release](https://github.com/orangishcat/cursed-tetris/releases/latest)
2. Extract the tarball.
3. Open the terminal and change directory into the extracted directory using `cd`.
4. Run `/cursed-tetris` from the command line.
  - (On MacOS, you may need to self-sign and unquarantine the binary first.)

**Windows**

1. Download the Windows exe of the [latest release](https://github.com/orangishcat/cursed-tetris/releases/latest).
2. Double click the `.exe`.

## Online leaderboard

Run the game with a shared SQLite database and a stable player ID to enable the online
leaderboard:

```sh
cursed-tetris --online /var/lib/cursed-tetris/scores.sqlite3 --id PLAYER_ID
```

Both options are required together. For SSH hosting, the forced-command launcher should derive
`PLAYER_ID` from the authenticated public key. The ID is private to the database; players choose
a display name when they first submit a qualifying score.
