
# Colonizer

A program that tracks your opponent's cards in Settlers of Catan on the site [colonist.io](https://colonist.io).

![screenshot](images/screenshot.png)

Taking the first player, Dong, as an example. For the lumber resource, the probability `.31` represents the odds of receiving that card if someone were to rob Dong. The field is green because you have the best odds of receiving lumber by robbing Dong. The `1.57` represents the expected value for the number of lumber that Dong has.

There are two ways the expected value is represented. The `1.57` for Dong indicates they have 1 lumber, and an expected extra amount of `0.57`. The `3+1.42` for Brig indicates they definitely hve 3 wool, and are expected to have `1.42` additional wool. 

The value 17 in the bottom-right is the number of possible states the game could be in. It can be thought of as the amount of uncertainty in the game, with a value of 1 being complete certainty.


## Instructions

1. Build the project
2. Run Chrome or Firefox in debug mode
3. Run the program

For example, on macOS with Google Chrome:
```shell
$ /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome --remote-debugging-port=9222 --user-data-dir=/tmp/data
$ cargo build --release
$ ./target/release/catan_tracker <username>
```

## How it works

Events are recorded to the in-game chat log. Colonizer communicates with the browser to acess the page's HTML using Chrome's [DevTools protocol](https://chromedevtools.github.io/devtools-protocol/). The chat messages are parsed and the state of the game is updated. The basic events are adding cards, removing cards, monopoly, and robbing. The only event that adds uncertainty to the game state is robbing. The program keeps track of every state the game could be in, and future events narrow this space.

## Future plans

- Refreshing the page clears the chat log and the program is unable to resume. The ability to start tracking a game in progress is being worked on.
- Track development cards
