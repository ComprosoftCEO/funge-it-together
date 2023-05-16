# Funge It Together

Congratulations! You have been hired as the new data engineer for our startup tech company: `FungeIt`.
Your job will be simple: programming artificial intelligence systems to handle data processing.
Everything you need to know will be described in the document below.
Complete all the levels and you can become a **Senior AI Engineer**!

<br />

## Compiling and Running

Our IDE uses the Rust [crossterm](https://docs.rs/crossterm/latest/crossterm/index.html) library to provide cross-platform support for terminal interfaces.
So it should work fine on Windows, Linux, and MacOS. It has been tested in the following terminals:

- Windows Command Prompt (Both old and new Windows terminals)
- Windows Subsystem for Linux
- MacOS Terminal
- Visual Studio Code Terminal

To run the code, follow [these instructions](https://www.rust-lang.org/tools/install) to install the latest version of Rust onto your computer.
Then run the command:

```bash
cargo run
```

<br />

## Command-Line Interface

The command-line interface allows you to select levels, write programs, and run the test cases.
In general, use the `Arrow Keys` and `Enter` to select options, and `Escape` to go back to the previous screen.
Press `Control-C` at any time to exit the program.

When using the program editor, a list of additional commands is shown on the right side of the terminal.
Navigate the grid with the arrow keys, and press the corresponding key to enter the command into the grid.
Use `b` to set the start location for the AI in the grid. Press `Delete` or `Backspace` to clear the highlighted grid cell.
The editor also has limited mouse support. You can `Left Click` to select a cell or `Right Click` to select and delete the contents of a cell.

Pressing `Tab` allows you to run your program step-by-step, or you can press `Space` to start automatic execution.
You can use the number keys `1` to `5` to set the execution speed.
Execution will continue indefinitely until you complete the level or an error occurs.
Pass all test cases to unlock the next level in sequence.

Your current solutions and level progress will be periodically saved during program execution.
So you can close the program using `Control-C` and know your progress will be saved.
However, closing the terminal with the close (X) button **may not** save your progress! You have been warned!

<br />

## The AI Engine

Our revolutionary AI system is based on the [Funge](https://en.wikipedia.org/wiki/Befunge) family of programming languages.
The AI operates in a 10x10 grid of cells. Each cell may contain one symbol with an instruction to execute.
The AI starts in one of the cells and moves right, executing each instruction in sequence.
Instructions may change the direction (up, down, left, right) that the AI moves through the grid.
If the AI reaches the edge of the grid, it wraps around back to the other side and continues executing instructions.

Data in the AI engine consists of integer values between `-999` and `999`.
Trying to compute values outside this range causes a computation error.

All data processing is done on a stack of values.
Data is read from the input stream and needs to be written to the output stream.
When you read a value from input, it is pushed onto the stack.
You can also program the grid to push constants onto a stack.
Executing a math instruction will pop two values off the stack, perform the operation, and push the new value onto the stack.
Writing to the output stream will pop the top value off the stack and write it to the output.
There are also instructions to duplicate the top stack value or discard the top stack value.
Finally, the AI has instructions to move around the order of items on the stack.
The stack is only allowed to contain **15 values** before it overflows, so any program should use 15 or fewer values on the stack.

The AI has conditional instructions used for branching in the program.
If the condition evaluates to true, then it executes the next instruction in sequence.
Otherwise, it skips the next instruction and continues execution two instructions ahead.
This allows the AI to make decisions based on the input data.

Each level consists of 25 test cases. Each test case will have a sequence of input values and expected output values.
Once your program reads all input values and writes the correct output values, it will move to the next test case.
If it passes all 25 test cases with the expected output, you will have completed the level.
Complete all levels to become a **Senior AI Engineer**!

<br/>

## Instructions

Instructions are categorized into five general categories:

### Directional

These instructions allow you to change the direction that the AI is moving through the 10x10 grid.
The AI moves in the same direction until an instruction updates it's direction.
If the AI reaches the edge of the grid, it wraps around back to the other side and continues executing instructions in the same direction.

| Instruction |       Symbol       | Description                                                              |
| :---------- | :----------------: | :----------------------------------------------------------------------- |
| Arrows      | `↑`, `↓`, `←`, `→` | Set the movement direction to up/down/left/right.                        |
| Mirrors     |      `/`, `\`      | Change the direction as though it was bounding off a mirror.             |
| Skip        |        `»`         | Skip the next instruction and continue execution two instructions ahead. |

### Stack Manipulation

The AI engine has a stack of values, where values can be pushed onto the top of the stack or popped off the top.
The stack is only allowed to contain **15 values** before it overflows, so any program should use 15 or fewer values on the stack.
There are also instructions to duplicate the top stack value or discard the top stack value.
Finally, the AI has instructions to move around the order of items on the stack.

| Instruction | Symbol | Description                                                                                                                                               |
| :---------- | :----: | :-------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pop         |  `☼`   | Discard the top item from the stack. Causes an error if the stack is empty                                                                                |
| Copy        |  `©`   | Duplicate the top item of the stack and push it onto the stack. Causes an error if it overflows the stack.                                                |
| Swap        |  `∫`   | Swap the order of the top two items on the stack. Causes an error if there are fewer than two items on the stack.                                         |
| Rotate Down |  `u`   | Moves the top item of the stack to the back of the stack, causing every item in the stack to shift down one position. Does nothing if the stack is empty. |
| Rotate Up   |  `∩`   | Moves the bottom item of the stack to the top of the stack, causing every item in the stack to shift up one position. Does nothing if the stack is empty. |

### Arithmetic

Data in the AI engine consists of integer values between `-999` and `999`.
Trying to compute values outside this range causes a computation error.

| Instruction |   Symbol   | Description                                                                                                                                                                                                                                                      |
| :---------- | :--------: | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Digit       | `0` to `9` | Push the constant onto the stack. If multiple constants are listed in sequence, it will push the larger number on the stack. (So `123` will push 123 onto the stack). Causes an error if it overflows the stack or you attempt to push a number larger than 999. |
| Add         |    `+`     | Pop off two items from the stack and pushes their sum back onto the stack. Causes an error if there are fewer than two items on the stack or the sum is outside the range \[-999,999\].                                                                          |
| Subtract    |    `-`     | Pops off two items from the stack and pushes their difference back onto the stack (second item minus top item of stack). Causes an error if there are fewer than two items on the stack or the sum is outside the range \[-999,999\].                            |

### Comparison

If the condition evaluates to true, then it executes the next instruction in sequence.
Otherwise, it skips the next instruction and continues execution two instructions ahead.

| Instruction     | Symbol | Description                                                                                                                                                                                                |
| :-------------- | :----: | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Less Than 0?    |  `<`   | Test if the top item of the stack is less than 0. If so, execute the next instruction, otherwise skip the next instruction. Does not pop the item off the stack. Causes an error if the stack is empty.    |
| Equal To 0?     |  `=`   | Test if the top item of the stack is equal to 0. If so, execute the next instruction, otherwise skip the next instruction. Does not pop the item off the stack. Causes an error if the stack is empty.     |
| Greater Than 0? |  `>`   | Test if the top item of the stack is greater than 0. If so, execute the next instruction, otherwise skip the next instruction. Does not pop the item off the stack. Causes an error if the stack is empty. |

### Input/Output

Each test case in a level will have a sequence of input values and expected output values.
Once your program reads all input values and writes the correct output values, it will move to the next test case.

| Instruction | Symbol | Description                                                                                                                                                              |
| :---------- | :----: | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Input       |  `Ї`   | Push the next item from the input stream onto the stack. Causes an error if it overflows the stack or there is no item to read in the input stream.                      |
| Has Input?  |  `?`   | Test if there is another input item to read. If so, execute the next instruction, otherwise skip the next instruction. Does **not** actually read from the input stream. |
| Output      |  `Θ`   | Pop the top item off the stack and send it to the output stream. Causes an error if the stack is empty or it exceeds the maximum number of allowed outputs.              |

<br />

## Creating Levels

Levels are written using the [Lua Programming Language](https://www.lua.org/docs.html) and executed at runtime.
All levels live inside the [levels](levels/) folder. To write a new level:

1. Create a Lua file for the level
2. Export a global function named `generateTestCase()` that randomly generates a new test case
3. Add an entry to the [pack.json](levels/pack.json) file. The file format should be self-explanatory, but here are a few guidelines to keep in mind:
   - The name should be short enough that it doesn't overflow the level select interface.
   - The description should fit into an 80 column x 24 row terminal window.
   - Be sure to generate a new unique UUID. [See Here](http://www.wasteaguid.info/).

All Lua levels must export a global `generateTestCase()` function. The function will get called 25 times consecutively to generate the test cases.
The code will not be reloaded between invocations, so you can use global variables to store state between invocations.
The function should return two arrays (inputs, outputs), where:

- All values in the array are integers between \[-999, 999\]
- Each array has no more than 15 elements
- The input array contains at least one element (output array is allowed to be empty)

When running the Lua code, the `levels` folder is added to the import path.
So you can `require()` additional Lua files from that folder if needed.

The Lua programs can use `math.random()`, but should not mess with `math.randomseed()`.
The game automatically sets the random seed to create reproducible test cases.
