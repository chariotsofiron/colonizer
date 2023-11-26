- https://settlers-rl.github.io/

collecting data
- `document.getElementById("game-log-text").innerHTML`


webapp
- https://old.reddit.com/r/Colonist/comments/pdahc9/i_created_a_web_app_to_extract_game_data_from/
- https://settlersofcatan.anvil.app/


## Data collection

We want to collect games to analyze.
- Copy game chat at end of game


## Reverse engineer board position


[5, 2, 6, 3, 8, 10, 9, 12, 11, 4, 8, 10, 9, 4, 5, 6, 3, 11]

- Catan has 19 hex tiles. 18 tiles are resources, 1 is a desert.
- There are 18 numbers that are placed in a spiral around the board
- The official algorithm for generating a board:
  1. Place 19 terrain tiles randomly on the board
  2. Place 9 harbor tiles randomly on the board
  3. Sort the 18 numbers, start at a corner, and place them in a spiral counter-clockwise, skipping over desert tiles


Combinations:
- orient the board from the perspective of the 5, removing symmetry
- step 1: 19! / (4! 4! 4! 3! 3!) = 244_432_188_000
- step 2: 9! / 4! = 15_120
- step 3: 19 ways to do this (there are 19 ways to skip over desert tiles

total:
= 244_432_188_000 * 15_120 * 19
= 70_220_478_968_640_000
= 7.022047896864 × 10^16

Actually the answer is
3695814682560000

http://minimallysufficient.github.io/math/games/2016/01/24/how-many-catan-board-configurations-are-there.html


## Strategy + analysis

- [Settlers of Catan Data](https://boardgames.stackexchange.com/questions/2877/settlers-of-catan-data)
- https://developingcatan.wordpress.com/
  - blog that discusses strategy
- [Catan Base Game Masterclass - Overview with DandyDrew and Anora](https://youtu.be/nNdon0f-bwU)
- [Settlers of Catan Placements Study: Results From 754 Games](https://youtu.be/Dx5HZPJqMTc)
- https://www.alexcates.com/post/catan-breakdown-starting-settlement-numbers-decision



What are the top 5 moves a player can make in a turn
- Navigate this game tree using expectimax




## Misc

- https://github.com/zactodd/ColonistQLearning

