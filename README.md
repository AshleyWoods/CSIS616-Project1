# Project 1 - Ashley Woods
## Purpose: 
     1. Accept a regular expression from the command line.
     2. Build an internal representation of the state diagram for the regular expression.
     3. Output to stdout the Graphiz definition of the state diagram.
     4. Read lines from stdin. The reason for using stdin is that you can either type in lines to test with or produce a text file that you redirect into the program.
     5. Each line from the file be a string that will be processed by the state machine.
     6. If the string is accepted by the state machine (it matches the regular expression), print “Accept” and the string to stderr.
     7. If the string is rejected by the state machine (it doesn’t match the regular expression), print “Reject” and the string to stderr
 
 ## Operation:
     - To run: cargo run RegEx
         - RegEx is the regular expression used to recognize strings
     - To test: cargo test

## Concerns and Caveats
    - It is exceedingly hard to make sure I have all possible input cases covered, I tried.
    - I am not fully sure if the dot notation for the output is correct
    - Currently doesn't support () or {}
    - CURRENTLY HAS NO TESTS