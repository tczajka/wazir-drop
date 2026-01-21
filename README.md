# WazirDrop: a (winning?) board game AI engine

This an AI game engine for the game 0.1, used in the CodeCup 2026
online tournament. WazirDrop placed ...

## The game

## GUI

## Board representation

## Move representation

## Move generation

### Setup moves
### Pseudomoves vs regular moves
### Check evasions
### Captures
### Checks
### Check threats
### Escape square attacks
### Boring moves
### Precomputed move tables

## Alpha-beta search

### PVS: Principal Variation Search

### Quiescence search

### Move ordering

### Transposition table

### Killer moves

### PV table

### Check extension

### Null move pruning

### Futility pruning

### Late move reductions

## Time allocation

## Hyperparameter tuning

## Repetitions

### Detecting repetition

### Agressiveness factor

## Bootstrapping evaluation

### Simple material evaluation

### Linear features

### Piece-square features

### Wazir-piece-square features

## NNUE: efficiently updateable neural network

### Accumulator update

### Quantization

### SIMD

## Self-play

## Evaluation training

### Evaluation as log-odds

## Opening book

### Reasonable setups

### Setup search

### Book size

### Out of book search

## Compressing NNUE weights and opening book

### Base 128 encoding in UTF-8

### Encoding NNUE weights

### Encoding setup moves