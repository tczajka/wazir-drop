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
- [Bootstrapping position evaluation](#bootstrapping-position-evaluation)
  - [Evaluation as logit](#evaluation-as-logit)
  - [Training loop](#training-loop)
  - [Self play](#self-play)
  - [Simple material evaluation](#simple-material-evaluation)
  - [Linear model with piece-square features](#linear-model-with-piece-square-features)
  - [Wazir-piece-square features](#wazir-piece-square-features)
- [Training a model using tch](#training-a-model-using-tch)
- [NNUE: efficiently updateable neural network](#nnue-efficiently-updateable-neural-network)
  - [Accumulator update](#accumulator-update)
  - [Quantization](#quantization)
  - [SIMD](#simd)
- [Move generation](#move-generation)
  - [Setup moves](#setup-moves)
  - [Pseudomoves vs regular moves](#pseudomoves-vs-regular-moves)
  - [Check evasions](#check-evasions)
  - [Checks](#checks)
  - [Check threats](#check-threats)
  - [Escape square attacks](#escape-square-attacks)
- [Tree search](#tree-search)
  - [Quiescence search](#quiescence-search)
  - [Move ordering](#move-ordering)
  - [Transposition table](#transposition-table)
  - [Miscellaneous improvements](#miscellaneous-improvements)
- [Time allocation](#time-allocation)
- [Repetitions](#repetitions)
  - [Detecting repetition](#detecting-repetition)
  - [Agressiveness factor](#agressiveness-factor)
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

```rust
pub struct Position {
    stage: Stage,
    ply: Ply,
    board: Board,
    captured: Captured,
    null_move_counter: u8,
}
```

The board is represented by a simple square -> color/piece mapping, as well
as a set of bitboards (for each piece, for each color, empty squares):

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
pub struct Captured {
    sides: EnumMap<Color, CapturedOneSide>,
    hash: u64,
}

pub struct CapturedOneSide {
    counts: EnumMap<Piece, u8>,
}
```

Squares and pieces:

```rust
pub enum Square {
    A1, A2, A3, A4, A5, A6, A7, A8,
    B1, B2, B3, B4, B5, B6, B7, B8,
    C1, C2, C3, C4, C5, C6, C7, C8,
    D1, D2, D3, D4, D5, D6, D7, D8,
    E1, E2, E3, E4, E5, E6, E7, E8,
    F1, F2, F3, F4, F5, F6, F7, F8,
    G1, G2, G3, G4, G5, G6, G7, G8,
    H1, H2, H3, H4, H5, H6, H7, H8,
}
```

Why are squares represented as a large 64-element `enum` rather than a simple number such as `u8`? This is for memory efficiency. An enum tells the Rust compiler that only these 64 values are valid. This allows it to store `Option<Square>` in 1 byte: 0-63 to represent a square, 64 to represent `None`.

```rust
pub enum Color {
    Red,
    Blue,
}

pub enum Piece {
    Alfil,
    Dabbaba,
    Ferz,
    Knight,
    Wazir,
}

pub enum ColoredPiece {
    RedAlfil,
    BlueAlfil,
    RedDabbaba,
    BlueDabbaba,
    RedFerz,
    BlueFerz,
    RedKnight,
    BlueKnight,
    RedWazir,
    BlueWazir,
}
```

Again we have a separate type for `ColoredPiece` for memory efficiency. Storing `Color` and `Piece` separately would require 2 bytes rather than just 1.


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

All other moves (jumps, captures and drops) are represented by this data type:
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

This structure contains more information than what is implied by the CodeCup notation. This makes it easier to make moves, recognize invalid moves during search, and allows for easier to read notation. For example, instead of `b3c4` I internally use notation such as `Fb3xwc4`, so that we know it is a ferz capturing the wazir.

# Bootstrapping position evaluation

To do any reasonable tree search, we need an evaluation function that can estimate who is winning and by how much in any given position. But how to build
such a function? I had no idea how to play the game or how much the pieces are worth relative
to each other. So I decided to have the evaluation function be trained from self-play games.

## Evaluation as logit

Our goal is to have the evaluation function approximate the [logit](https://en.wikipedia.org/wiki/Logit) of predicted win probability. In other words,
we want the win probability to be approximated by the [sigmoid](https://en.wikipedia.org/wiki/Sigmoid_function) of the position evaluation.

If $v$ is the current evaluation, then we estimate win probability as:

$$ p = \sigma(v) = \frac{1}{1 + e^{-v}}$$

We treat draws as 50% win, 50% loss.

![sigmoid](images/sigmoid.svg)

So reasonable evaluations are normally somewhere in the range of [-5, 5].

In internal calculations, we usually scale these by a factor of 10 000 and use integer evaluations, so typically evaluations are in the range [-50 000, 50 000].

## Training loop

So how we do train an evaluation function? By having the program play against itself and learning from those games.

1. Take the current evaluation model.
2. Collect a lot of game positions and their evaluations using [self play](#self-play).
3. Train a new model.
4. Go to 1.

This went through many iterations using multiple different models:
* [simple material evaluation](#simple-material-evaluation)
* [linear model with piece-square features](#linear-model-with-piece-square-features]
* linear model with [wazir-piece-square features](#wazir-piece-square-features)
* [neural network](#nnue-efficiently-updateable-neural-network)

The last model, using the neural network was trained over 7 iterations of the training process.

I ran the last iteration when I went away for a skiing trip for a week. Playing 100 million games took 8 days on a 32-core workstation, and then it took 1 more day to train the model using that data.

## Self play

The goal of self play was to gather a diverse set of reasonable positions, and get their evaluations better than what the current evaluation function
can give us. We also generally want *quiet* positions, meaning positions in which captures aren't important. That's because our search can deal with captures anyway, we just want to be able to evaluate the resulting final positions after such sequences of captures.

Here is what I did:
1. Start with completely random setups.
2. Do a depth 4 [tree search](#tree-search).
3. Select the position at the end of the [best variation](https://www.chessprogramming.org/Principal_Variation) from the search. This will generally be a quiet position because of [quiescence search](#quiescence-search).
4. Do another, deeper search (extra 4 ply) to evaluate the selected position well.
5. Store the selected position and its evaluation in the output file.
6. At the root position, pick a move and play it.
7. Go to 2.

 To get some extra variation in the games beyond just the starting positions, I don't always pick the best move. Instead, I randomly pick a move, with better moves having higher probabilities, according to [soft max](https://en.wikipedia.org/wiki/Softmax_function) with a temperature $T$:

 $$ p_i = \frac{e^{v_i / T}}{\sum_i e^{v_i / T}} $$

## Simple material evaluation

But where do we begin the training process? I only had a very rough idea how to evaluate positions, I didn't even know which pieces are worth more than others. So I decided to just start with the simplest thing possible:

* every piece on the board gets value 0.1
* every captured piece also gets value 0.1

That's it. That was the only evaluation function that I created manually.

## Linear model with piece-square features

The next step was a linear model with piece-square features. The evaluation is a linear combination of the following features:
* 1 feature for each piece type / square combination
* 1 feature for each captured piece type and its number
* 1 feature for side to move (tempo bonus)

There are 64 squares, but because the rules of the game are symmetric
to rotations and reflections of the board, we only have 10 different "normalized squares":

```rust
pub enum NormalizedSquare {
    A1, A2, A3, A4,
        B2, B3, B4,
            C3, C4,
                D4,
}
```

In total we have 80 possible features for each side: 50 for pieces on the board, 30 for captured pieces. Plus a bonus for side to move.

```rust
pub static SCALE: f64 = 1000.0;
pub static TO_MOVE: i16 = 352;

pub static FEATURES: [i16; 80] = [
    // alfil
    83, 89, 163, 222, 112, 194, 203, 311, 313, 365,
    // dabbaba
    74, 60, 115, 185, 59, 181, 212, 320, 330, 309,
    // ferz
    18, 49, 62, 126, 206, 226, 247, 289, 286, 250,
    // knight
    188, 221, 282, 318, 265, 343, 419, 449, 442, 405,
    // wazir
    655, 626, 25, -148, 599, 44, -304, -283, -514, -701,
    // captured alfil
    186, 111, 80, 71, 64, 73, 70, 91, 53, 55, -107, -85, 0, 0, 0, 0,
    // captured dabbaba
    307, 148, 89, 40, 23, 68, -236, 0,
    // captured ferz
    324, 207, 154, 34,
    // captured knight
    442, 344,
];
```

All the values are multiplied by `SCALE = 1000`.

What can we notice here? Knights are the best piece. Pieces are much more valuable in the center, except the wazir which prefers corners. The first captured piece
is much more valuable than more captured pieces of the same type.

## Wazir-piece-square features

For the next, bigger model we still use a linear combination of features, but this time consider a larger set of features. I realized that piece values are very strongly dependend on where the wazirs are. We want to be attacking the opponent wazir, and protecting our own wazir. So we have features for each wazir position in combination with each other piece position (of the same or opposite color). But first we rotate and/or reflect the board so that the wazir square is normalized. 

There are in total 6360 features per side:
* 1 feature for each wazir position and other piece type and position. Total: 10 * 9 * 64 = 5760 features.
* 1 feature for each wazir position and capture piece type and number. Total: 600 features.

Plus a bonus for side to move.

Let's look at some of the weights: when a wazir is in A1 corner, here are the values for a
same-colored alfil:

```rust
// wazir: a1
    // same alfil
       0, -131,  387,  206,  315,  126,   33,   -5,
    -161,   47,  154,   75,  115,  116,   25,   22,
     364,  187,  410,  186,  173,  104,  131,   61,
     146,   99,  213,  294,  198,  152,   72,   36,
     272,  165,  192,  223,  274,  165,   51,  -45,
     117,  142,  116,  125,  173,  161,   73,    3,
      57,   76,  115,   74,   39,   76,   87,  144,
      56,   17,   93,   94,   98,   51,   93,  112,
```

Having an alfil right next to our own wazir actually has **negative** value! The alfil is blocking a potential escape square.

# Training a model using tch

I trained all movels using the [tch](https://crates.io/crates/tch) crate which is a Rust wrapper for [PyTorch](https://pytorch.org/).

# NNUE: efficiently updateable neural network

## Accumulator update

## Quantization

## SIMD


# Move generation

## Setup moves

We generate all possible setup moves by permuting the 16 pieces. The number of such moves is:

$$ \frac{16!}{8!\ 4!\ 2!\ 1!\ 1!} = 10\ 810\ 800$$

During actual gameplay this function is never used however. Instead, we use the [opening book](#opening-book).

## Pseudomoves vs regular moves

Game rules don't technically distinguish "checks" and allow the wazir to move into check or ignore a check. But we don't normally generate such "suicide" moves at all. I call those "pseudomoves". The only time we need them is the last two moves of the game where the wazir is checkmated and both sides still have to make one (pseudo-)move each to capture it.

## Check evasions

When the wazir is in check, the only move we consider are check evasions. These
are (generated in this order):
* capture the checking piece
* wazir captures
* wazir jumps

## Checks

We generate checks separately. A jump check always moves a piece from a square two moves away from the opponent wazir to a square one move away from the opponent wazir. To generate checks quickly, we have these sets of squares precomputed for each piece type and each square for the opponent wazir.

## Check threats

We also generate check **threats** separately. Those are move that threaten to
give a check next move. Jump checks move a piece from a square *three* moves away from the opponent wazir to a square *two* moves away from the opponent wazir.

## Escape square attacks

Another kind of move we generate separately are "escape square attacks". Those are moves that attack a square that is next to the opponent wazir, restricting
its future escape paths. For this, for each piece and square, we precompute the set of squares that are reachable from a given square by:
* a wazir move + one piece move; these are the destination squares of escape square attacks
* a wazir move + two piece moves; these are the "from" squares of such attacks


# Tree search

We use a variant of alpha-beta search called [Principal Variation Search](https://en.wikipedia.org/wiki/Principal_variation_search).

## Quiescence search

When we reach the full search depth, we keep searching some moves in what's called [quiescence search](https://en.wikipedia.org/wiki/Quiescence_search). In this phase we only consider check escapes and captures.

## Move ordering

When not in check, we generate moves in the following order:
1. [Transposition table](#transposition-table) move
2. Captures (in an arbitrary order)
3. [Killer moves](https://www.chessprogramming.org/Killer_Move)
4. Piece drop checks
5. Piece drop escape square attacks
6. Jump checks
7. Jump escape square attacks
8. Other jumps
9. Other drops

## Transposition table

The transposition table stores information about positions that have previously been searched with their score, depth searched, etc.

```rust
struct TTable {
    buckets: Vec<Bucket>,
    epoch: u8,
}

struct Bucket {
    entries: [PhysicalEntry; 4],
}

struct PhysicalEntry {
    hash: u32,
    epoch: u8,
    depth: Depth,
    mov: Option<Move>,
    score_type: TTableScoreType,
    score: Score,
}

enum TTableScoreType {
    None,
    Exact,
    LowerBound,
    UpperBound,
}
```

## Miscellaneous improvements

[Check extension](https://www.chessprogramming.org/Check_Extensions): every time there is a check we extend search 1 ply deeper. This allows searching forcing sequences with checks deeper.

[Null move pruning](https://www.chessprogramming.org/Null_Move_Pruning): if a position looks good for the side to move (evaluation > beta + 0.1), we try a null move and search 1 ply shallower. If this results in a beta cutoff, just use it.

[Futility pruning](https://www.chessprogramming.org/Futility_Pruning): at depth = 1, if evaluation looks bad (evaluation < alpha - 0.6), we don't even try boring moves (non-captures and non-checks).

[Late move reductions](https://www.chessprogramming.org/Late_Move_Reductions): at depth > 1, we search boring moves (other than the first 5) 1 ply shallower. If they turn out to be good move, we search again with full depth.

# Time allocation

The basic time allocation uses a simple geometric sequence. Each next move gets 5% less time than the previous.

One adjustment to this is *panic mode*. When the evaluation of the best move found so far drops by a significant amount (0.04) from the previous lower depth, we allocate up to 5x more time to try to find a better alternative move.

# Repetitions

## Detecting repetition

## Agressiveness factor


# Opening book

## Reasonable setups

## Setup search

## Book size

## Out of book search

# Compressing NNUE weights and opening book

## Base 128 encoding in UTF-8

## Encoding NNUE weights

## Encoding setup moves