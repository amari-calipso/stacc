# Stacc
A parserless stack-based programming language made in a night for fun

# Usage
To compile, use `cargo build --release`. To run directly, use `cargo run`.
```
stacc <filename>
```

# How does it work?
You have two stacks available. The code is composed of objects, operations and labels. 
Code is read from left to right: if an object is encountered, it gets pushed on the primary stack, if an operation is encountered, it's performed.

Objects are:
- Integers;
- Floats;
- Strings (denoted with `"`);
- Code objects (code surrounded by `{` and `}`).

Labels are denoted by surrounding their name with `[` and `]`, for example: `[myLabel]`.

Available operations are:
- `,`: pops the primary stack, and pushes the output on the secondary;
- `;`: pops the secondary stack, and pushes the output on the primary;
- `@`: pops the secondary stack, discarding the result;
    - this implies that `"insert comment here" ,@` is a valid way to create comments (the comment is pushed on the primary stack, moved to the secondary, and discarded).
- `#`: swaps the primary and secondary stacks (the primary becomes secondary, and viceversa);
- `.`: duplicates the last value on the primary stack (x = pop, push x, push x);
- `$`: pops the primary stack, and prints the output;
- `!`: pops the primary stack, if the popped value is truthy, pushes 0 on the primary stack, else 1;
- `+`: pops the primary stack twice, the first pop corresponds to the second operand, and the second pop corresponds to the first operand. The two operands are added together and the result is pushed on the primary stack. The operation is different depending on the operands' types:
    - `int + int` -> int (arithmetic addition, wraps around on overflow);
    - `int + float` -> float (arithmetic addition);
    - `float + int` -> float (arithmetic addition);
    - `float + float` -> float (arithmetic addition);
    - `string + string` -> string (string concatenation);
    - `int + string` or `float + string` -> string (turns first operand to string, and concatenates it to second operand);
    - `string + int` or `string + float` -> string (concatenates the first operand to the second operand, turned into a string);
    - `code + code` -> code (concatenates the code as if it were executed in sequence. If labels collide, an error is thrown);
    - Any other operation will throw an error.
- `-`, `/`, `*` and `%`: pop the primary stack twice, the first pop corresponds to the second operand, and the second pop corresponds to the first operand. Perform subtraction, division, multiplication and modulo, respectively, pushing the output on the primary stack. The operation is different depending on the operands' types:
    - `int + int` -> int (subtraction, division and multiplication wrap around on overflow);
    - `int + float` -> float;
    - `float + int` -> float;
    - `float + float` -> float;
    - Any other operation will throw an error.
- `&` and `|`: pop the primary stack twice, the first pop corresponds to the second operand, and the second pop corresponds to the first operand. Perform bitwise "and" and "or" operations respectively and push the result on the primary stack. They can only be applied to integers. Any other operation will throw an error;
- `=`: pops the primary stack twice, obtaining two operands. Compares the operands, and if they're equal, pushes 1 on the primary stack, otherwise 0;
- `<` and `>`: pop the primary stack twice, the first pop corresponds to the second operand, and the second pop corresponds to the first operand. Compare the operands, and if they're, respectively, first less than second, and first greater than second, push 1 on the primary stack, otherwise 0;
- `^`: pops the stack and "jumps" to the token indicated by the result. This operation jumps in different ways depending on the type of the operand:
    - int: jumps `n` tokens forward (or backwards, if the value is negative). Wraps around;
    - float: jumps `floor(n)` tokens forward (or backwards, if the value is negative). Wraps around;
    - string: jumps to a label with the name contained in the string. If the label doesn't exist, throws an error;
    - code: executes the code.
- `?`: pops the stack twice. The first pop corresponds to the location to jump to (this works the same way as the `^` operator), and the second pop corresponds to the condition. If the condition is truthy, the jump is performed, otherwise, it's not;
- `:`: defines a function: pops the stack twice. The first pop corresponds to the function name, which has to be a string (if it's not, an error is thrown). The second pop corresponds to the code that will be executed when the function is called, which has t be a code object (if it's not, an error is thrown). Functions can be called by simply referencing their name without quotes in the code. Calling an undefined function will result in an error;
- `~`: pops the primary stack, depending on the type of the popped value, it performs different operations:
    - int -> int: bitwise not;
    - float -> int: cast to int;
    - string -> code: parses the code contained in the string and returns a code object representing it. This means that the `~` operator can be used in different combinations for more complex operations:
        - `~^`: parse the code in the string and execute it, "eval" basically. This can also be used to parse integers and floats from strings;
        - `~^~`: if used on a string that contains a float, this parses the float and casts it to an integer.
    - Any other operation will throw an error.

## What values are truthy?
Strings, code objects, nonzero integers and floats.
