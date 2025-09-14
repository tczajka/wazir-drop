# Zero Point One

## Introduction

Zero Point One is a two-player strategy game introduced in 2012. The game was designed and illustrated by Jim Wickson and published by BoardGameGeek.com. The game is played on an 8x8 square board.

## Starting Setup

The Red player places his pieces on rows a and b. The Blue player places his pieces on rows g and h. Both players have the following pieces:

* 1 Wazir (0.1)
* 1 Knight (1.2)
* 2 Ferzes (1.1)
* 4 Dabbabas (0.2)
* 8 Alfils (2.2)

Red player pieces are labeled: WNFFDDDDDAAAAAAA. Blue player pieces are labeled: wnffddddaaaaaaaa.

## Gameplay
The game is played in turns, with the Red player going first. Players take turns moving one of their pieces. Each piece moves according to its specific movement rules, which are indicated on the piece itself.

## Piece Movement

Wazir (0.1): Moves 0 steps orthogonally (horizontal or vertical) and then 1 step perpendicular to that direction, in any direction.

Knight (1.2): Moves 1 step orthogonally (horizontal or vertical) and then 2 steps perpendicular to that, in any direction. The Knight may jump over other pieces, like in chess.

Ferz (1.1): Moves 1 step orthogonally and 1 step perpendicular to that, in any direction.

Dabbaba (0.2): Moves 0 steps orthogonally and then 2 steps perpendicular, in any direction. It may jump.

Alfil (2.2): Moves 2 steps orthogonally and then 2 steps perpendicular, in any direction. It may jump.

## Capturing Pieces

Players can capture an opponent’s piece by landing on a square occupied by the opponent’s piece. When this happens, the captured piece is removed from the board and becomes part of the capturing player's collection. If a player captures the opponent’s Wazir (0.1), it immediately wins the game.

## Reviving Captured Pieces
Instead of moving a piece or capturing an opponent’s piece, a player can choose to revive a captured piece from its collection. The revived piece can be placed, without any restrictions, on any empty square on the board.

## Winning the Game

The game ends when a player captures its opponent’s Wazir (0.1), and that player is declared the winner.

For more information about the game, visit: https://boardgamegeek.com/boardgame/114307/01-zero-point-one.

## Protocol

Your program must follow the protocol when communicating with the judging software. Your player must read information from standard input, and output its requested respons to standard output. For more information, see the Technical rules. Do not forget to flush your output!

The red player is send "Start", indicating that he must play with red. Then he has to write his desired starting sequence in return. This sequence is send to the blue player and then he must write his starting sequence also in return. Red will receive this sequence from blue and has to write his first move. Blue receives red's first move and must do the same, and so on.

The format for a regular move is b5d3, indicating a piece is moving from b5 to d3. If a captured piece is put onto the board, for a red Dabbaba (0.2) you must use the format Df8 and for a blue Alfil (2.2) you use ac3. The first move places a Dabbaba (0.2) on spot f8 and the second move a blue Alfil (2.2) on spot c3.

When there is no winner after 102 moves (including the starting moves), the game ends in a draw. Both programs have 30 seconds thinking time.

## Getting points

The winner of the game receives 21 points, the loser gets 1 point only. In the case of a draw both players get 11 points.

If a player makes a mistake, crashes, leaves too early or runs out of time, he receives a "Quit" from the jury software and he loses with 0 - 21. In such a case the other player receives a "Quit" as input, and must terminate his program immediately. Of course he wins with 21 - 0.

## Competition

For the competition each player plays each other player exactly twice, once as the red player and once as the blue player. The overall winner of the competition is the player with the highest number of points overall. For more information, see the Competition rules.