/// # Project 1 - Ashley Woods
/// ## Purpose: 
///     1. Accept a regular expression from the command line.
///     2. Build an internal representation of the state diagram for the regular expression.
///     3. Output to stdout the Graphiz definition of the state diagram.
///     4. Read lines from stdin. The reason for using stdin is that you can either type in lines to test with or produce a text file that you redirect into the program.
///     5. Each line from the file be a string that will be processed by the state machine.
///     6. If the string is accepted by the state machine (it matches the regular expression), print “Accept” and the string to stderr.
///     7. If the string is rejected by the state machine (it doesn’t match the regular expression), print “Reject” and the string to stderr
/// 
/// ## Operation:
///     - To run: cargo run RegEx
///         - RegEx is the regular expression used to recognize strings
///     - To exit: ctrl c
///     - To test: cargo test
/// 
/// ## Grammar for a Regular Expression:
///     E -> C|E    //The '|' character is part of the actual definition here
///     E -> C
///     C -> SC|S
///     S -> P*|P+|P
///     P -> (E)|{E}|L
///     L -> \w|\d|A
///     A -> All accepted characters (sigma)


use std::io::Write; //for writing to output file and stderr
use std::fs::File; //for creating output file
use std::io::stdin; //for reading from stdin
use std::io::prelude::*; //for reading from stdin

//define SIGMA and additionall acceptable chars for refrence
const SIGMA: [char; 36] = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','0','1','2','3','4','5','6','7','8','9'];
const REG_CHARS: [char; 8] = ['|', '{', '}', '(', ')', '*', '+', '\\'];

fn main() {

    //Grab input from command line
    let mut args = Vec::new();
    for input in std::env::args().skip(1) {
		args.push(input);
    }

    //Get the regular expression
    let reg_ex  = &args[0];
    
    //Parse reg_ex: method call, input regex, output ????, if failed parse print error and exit

    //Build a state diagram for reg_ex
    //Do so in another method, input is ???? from parse, return diagram

    //Print the state diagram to stdut
    //Also do so in another method input is diagram, no return, it creates the file
    //To create file: let mut output = File::create("stdout.txt").expect("Unable to create file");
 
    //Read from stdin and print to stderr
    process_input(reg_ex);

}


/// For reading input from stdin and printing accept or reject for each line
/// - Input: Regex regular expression to match input to
/// - Output: An accept or reject output followed by the string printed to stderr
fn process_input(reg: &str) {
    let mut stderr = std::io::stderr();
    let stdin = stdin();
    for line in stdin.lock().lines() {
        let string = line.unwrap();
        if true{ //Check to see if it matches the regex here
            writeln!(&mut stderr, "Accept {}", string).unwrap();
        }else {
            writeln!(&mut stderr, "Reject {}", string).unwrap();
        }
    }
}
