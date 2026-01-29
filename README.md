# WazirDrop: tournament winning board game AI engine

This an AI game engine for the game 0.1 that participated in the
[CodeCup 2026](https://www.codecup.nl/)
online tournament. WazirDrop [won](https://www.codecup.nl/competition.php?comp=344)!

- [WazirDrop: tournament winning board game AI engine](#wazirdrop-tournament-winning-board-game-ai-engine)
- [The game](#the-game)
  - [Pieces](#pieces)
  - [Setup phase](#setup-phase)
  - [Captures and drops](#captures-and-drops)
- [GUI](#gui)
- [Position representation](#position-representation)
- [Move representation](#move-representation)
- [Move generation](#move-generation)
  - [Setup moves](#setup-moves)
  - [Pseudomoves vs regular moves](#pseudomoves-vs-regular-moves)
  - [Check evasions](#check-evasions)
  - [Precomputed bitboards](#precomputed-bitboards)
  - [Checks](#checks)
  - [Check threats](#check-threats)
  - [Escape square attacks](#escape-square-attacks)
- [Alpha-beta search](#alpha-beta-search)
  - [Quiescence search](#quiescence-search)
  - [Move ordering](#move-ordering)
  - [Transposition table](#transposition-table)
  - [Killer moves](#killer-moves)
  - [PV table](#pv-table)
  - [Check extension](#check-extension)
  - [Null move pruning](#null-move-pruning)
  - [Futility pruning](#futility-pruning)
  - [Late move reductions](#late-move-reductions)
- [Time allocation](#time-allocation)
- [Hyperparameter tuning](#hyperparameter-tuning)
- [Repetitions](#repetitions)
  - [Detecting repetition](#detecting-repetition)
  - [Agressiveness factor](#agressiveness-factor)
- [Bootstrapping evaluation](#bootstrapping-evaluation)
  - [Simple material evaluation](#simple-material-evaluation)
  - [Linear features](#linear-features)
  - [Piece-square features](#piece-square-features)
  - [Wazir-piece-square features](#wazir-piece-square-features)
- [NNUE: efficiently updateable neural network](#nnue-efficiently-updateable-neural-network)
  - [Accumulator update](#accumulator-update)
  - [Quantization](#quantization)
  - [SIMD](#simd)
- [Self-play](#self-play)
- [Evaluation training](#evaluation-training)
  - [Evaluation as log-odds](#evaluation-as-log-odds)
- [Opening book](#opening-book)
  - [Reasonable setups](#reasonable-setups)
  - [Setup search](#setup-search)
  - [Book size](#book-size)
  - [Out of book search](#out-of-book-search)
- [Compressing NNUE weights and opening book](#compressing-nnue-weights-and-opening-book)
  - [Base 128 encoding in UTF-8](#base-128-encoding-in-utf-8)
  - [Encoding NNUE weights](#encoding-nnue-weights)
  - [Encoding setup moves](#encoding-setup-moves)


# The game

[0.1 (Zero Point One)](https://boardgamegeek.com/boardgame/114307/01-zero-point-one) is a board game designed by Jim Wickson. It is similar to [chess](https://en.wikipedia.org/wiki/Chess), and even more similar to [shogi](https://en.wikipedia.org/wiki/Shogi) or [crazyhouse](https://en.wikipedia.org/wiki/Crazyhouse).

![Screenshot of GUI](images/codecup_gui.png)

## Pieces

The game only uses unorthodox chess-like pieces, called [fairy chess pieces](https://en.wikipedia.org/wiki/Fairy_chess_piece). In particular, it only uses what are called ["leapers"](https://en.wikipedia.org/wiki/Fairy_chess_piece#Leapers).

A **leaper** is a piece that can jump a fixed offset away in any direction, potentially jumping over any other pieces. A chess knight is a leaper: it jumps by a [1, 2] vector in any of 8 directions (e.g. 1 square left, 2 squares down). In 0.1 terminology, a knight is a piece we can call "1.2".

The leapers used in 0.1 are:
* 0.1, [Wazir](https://en.wikipedia.org/wiki/Wazir_(chess)). It moves one square orthogonally. This is the equivalent of a chess King, but it can't move diagonally.
* 1.1, [Ferz](https://en.wikipedia.org/wiki/Ferz). It moves one square diagonally.
* 0.2, [Dababba](https://en.wikipedia.org/wiki/Dabbaba_(chess)). It moves or jumps two squares orthogonally.
* 1.2, [Knight](https://en.wikipedia.org/wiki/Knight_(chess)). This is just a regular chess knight.
* 2.2, [Alfil](https://en.wikipedia.org/wiki/Alfil). It moves or jumps two squares diagonally.

The goal of the game is to capture the opponent wazir. That's like in chess, except moving into check is allowed (but not advisable), and after giving a checkmate you still have to make an extra move to actually capture the wazir.

## Setup phase

Each player starts with 1 wazir, 2 ferzes, 4 dababbas, 1 knight and 8 alfils. The goal of the game is to capture the other wazir.

Each player can set up the pieces in any order they want in the first two rows on their side of the board. Red goes first, then blue.

## Captures and drops

Like in chess, you capture opponent pieces by landing on them. The difference is that after you capture an opponent piece, you can **drop** it onto any empty square as your own piece, in lieu of a regular move. This is similar to how shogi and crazyhouse are played.

# GUI

In order to be able to play against the engine, I created a graphical
interface. Rather than using circles with numbers, I found it a lot easier to
see what's going on if I use regular chess pieces (king = wazir, pawn = ferz, rook = dababba, bishop = alfil).

![GUI](images/gui.png)

# Position representation

The board is represented by a simple square -> color/piece mapping, as well
as a set of bitboards (for each piece, for each color, empty squares).

```rust
pub struct Board {
    squares: EnumMap<Square, Option<ColoredPiece>>,
    occupied_by: EnumMap<Color, Bitboard>,
    empty_squares: Bitboard,
    occupied_by_piece: EnumMap<ColoredPiece, Bitboard>,
    hash: u64,
}
```

The captured pieces are just a count for each color/piece:

```rust
pub struct CapturedOneSide {
    counts: EnumMap<Piece, u8>,
}

pub struct Captured {
    sides: EnumMap<Color, CapturedOneSide>,
    hash: u64,
}
```

# Move representation

There are four types of moves:
* setup moves (first two moves of the game)
* captures
* jumps (piece moves that don't capture anything)
* drops (put a previously captured piece back on the board)

Setup moves are represented by just a list of pieces in order:

```rust
pub struct SetupMove {
    pub color: Color,
    // From square 0 or square 63.
    pub pieces: [Piece; Self::SIZE],
}
```

All other moves are represented by this data type:
```rust
pub struct Move {
    pub colored_piece: ColoredPiece,
    pub from: Option<Square>,
    pub captured: Option<Piece>,
    pub to: Square,
}
```

If `from` is `None`, we have a drop move. If `captured` is not `None`, we have
a capture move.

This structure contains more information than what is implied by the CodeCup notation. This makes it easier to make moves, recognize invalid moves during search, and allows for easier to read notation. For example, instead of `b3c4` I internally use notation such as `Fb3xwc4`.


# Move generation

## Setup moves

There is a function that generates all possible setup moves. The number of such moves is:

$$ \frac{16!}{8!\ 4!\ 2!\ 1!\ 1!} = 10\ 810\ 800$$

During actual gameplay this function is not used however. Instead, we use the [opening book](#opening-book).

## Pseudomoves vs regular moves

Game rules don't technically distinguish "checks" and allow the wazir to move into check or ignore a check. But we don't normally generate such "suicide" moves at all. I call those "pseudomoves". The only time we need them is the last two moves of the game where the wazir is checkmated and both sides still have to make one (pseudo-)move each to capture it.

## Check evasions

When the wazir is in check, the only move we consider are check evasions. These
are (generated in this order):
* capture the checking piece
* wazir captures
* wazir jumps

## Precomputed bitboards

For each piece type and square, we store a precomputed bitboard of all destination squares. When generating jump moves and captures we use these bitboards to find all the moves efficiently.

## Checks

We generate checks separately. To generate checks, we have another set of precomputed bitboards: for each piece type and square, we store the set of squares that are reachable from that square in exactly **two** moves. Using
these tables starting with the opponent wazir square, we can quickly find
pieces that can possibly give a check. Intersecting two single-move bitboards
then gives the set of destination squares.

## Check threats

We also generate check **threats** separately. Those are move that threaten to
give a check next move. For this we precompute sets of squares that are reachable in exactly **three** moves by a given piece.

## Escape square attacks

Another kind of move we generate separately are "escape square attacks". Those are moves that attack a square that is next to the opponent wazir, restricting
its future escape paths. For this, for each piece and square, we precompute the set of squares that are reachable from a given square by:
* a wazir move + one piece move; these are the destination squares of escape square attacks
* a wazir move + two piece moves; these are the "from" squares of such attacks

# Alpha-beta search

We use a variant of alpha-beta search called [Principal Variation Search](https://en.wikipedia.org/wiki/Principal_variation_search).

## Quiescence search

When we reach the full search depth, we keep searching some moves in what's called [quiescence search](https://en.wikipedia.org/wiki/Quiescence_search). In this phase we only consider check escapes and captures.

## Move ordering

When not in check, we generate moves in the following order:
1. [Transposition table](#transposition-table) move
2. Captures (in an arbitrary order)
3. [Killer moves](#killer-moves)
4. Piece drop checks

## Transposition table

## Killer moves

## PV table

## Check extension

## Null move pruning

## Futility pruning

## Late move reductions

# Time allocation

# Hyperparameter tuning

# Repetitions

## Detecting repetition

## Agressiveness factor

# Bootstrapping evaluation

## Simple material evaluation

## Linear features

## Piece-square features

## Wazir-piece-square features

# NNUE: efficiently updateable neural network

## Accumulator update

## Quantization

## SIMD

# Self-play

# Evaluation training

## Evaluation as log-odds

# Opening book

## Reasonable setups

## Setup search

## Book size

## Out of book search

# Compressing NNUE weights and opening book

## Base 128 encoding in UTF-8

## Encoding NNUE weights

## Encoding setup moves