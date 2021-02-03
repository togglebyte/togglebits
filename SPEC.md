# General idea

Some kind of multiplayer game in the terminal
run by chat.

Show eight bits.
Each player can choose a position on the byte with 0 - 7, and toggle that bit.

If all bits are matching the round is over.

* A round lasts for N seconds.
* First person to toggle the last correct bit wins.

# Game loop

* Listen to incoming messages.
* Tick a timer.

## On incoming message:

* Parse message, if it doesn't contain 0-7 discard the message
* Animate a falling bit
* Once the animation ends, change the bit.
* If it was the final bit required to make the byte complete show a winning
  screen and then restart the game after N seconds.

## Winning screen

Show the username of the winner and wait N seconds, then restart.
