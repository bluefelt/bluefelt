# Connect 4 Rules

## Overview
Connect 4 is a two-player connection game in which players take turns dropping colored discs into a vertical grid. The objective is to be the first to form a horizontal, vertical, or diagonal line of four of one's own discs.

## Players
- Minimum: 2
- Maximum: 2

## Equipment
- 1 vertical grid board (7 columns × 6 rows)
- 21 red discs for Player 1
- 21 yellow discs for Player 2

## Setup
The game begins with an empty grid. Player 1 (red) goes first.

## Gameplay
### Turn Structure
1. Current player chooses a column (1-7)
2. Player drops their disc into the chosen column
3. Disc falls to the lowest available position in that column
4. Turn passes to the other player

### Allowed Actions
- **Drop Disc**: Place a disc in any column that is not full (has fewer than 6 discs)

### Restrictions
- Cannot place a disc in a full column
- Discs are affected by gravity (always fall to the lowest available position)
- Once placed, discs cannot be moved or removed

## Win Conditions
- **Four in a Row**: First player to connect four discs in a line wins
  - Horizontal: Four discs in a row (e.g., columns 2-5 in the same row)
  - Vertical: Four discs in a column (e.g., rows 1-4 in the same column)
  - Diagonal: Four discs diagonally (ascending or descending)
- **Draw**: If all 42 spaces are filled with no winner, the game is a draw

## Special Rules
- Gravity applies: discs always fall to the lowest empty position in a column
- Players cannot skip turns
- The game ends immediately when a player achieves four in a row

## Quick Start
Players take turns dropping colored discs into a 7×6 grid. Discs fall to the bottom of the column. First player to connect four of their discs in any direction wins!

## Visual Example
```
Empty Board:         Example Game State:
. . . . . . .       . . . . . . .
. . . . . . .       . . . . . . .
. . . . . . .       . . . . . . .
. . . . . . .       . . R . . . .
. . . . . . .       . Y R Y . . .
. . . . . . .       R Y R Y . . .
1 2 3 4 5 6 7       1 2 3 4 5 6 7

R = Red (Player 1)
Y = Yellow (Player 2)
. = Empty
```

## Strategy Tips
- Control the center column (4) for more winning opportunities
- Block opponent's three-in-a-row formations
- Create multiple threats simultaneously
- Be careful not to set up your opponent for a win on the next level