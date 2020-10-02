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
const SIGMA: [char; 37] = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','0','1','2','3','4','5','6','7','8','9', ' '];
const REG_CHARS: [char; 8] = ['|', '{', '}', '(', ')', '*', '+', '\\'];

fn main() {

    //Grab input from command line
    let mut args = Vec::new();
    for input in std::env::args().skip(1) {
		args.push(input);
    }

    //Check that there is only one input
    if args.len() != 1 {
		writeln!(std::io::stderr(), "Invalid Input").unwrap();
		std::process::exit(1);
	}

    //Get the regular expression
    let reg_ex = &args[0];

    //check that input is valid
    for char in reg_ex.chars() {
        if !SIGMA.contains(&char) { //if the character is not in sigma
            if !REG_CHARS.contains(&char) { //and if the character is not in RegEx chars
                //Then contains unsupported characters and is invalid
                writeln!(std::io::stderr(), "Invalid Input").unwrap();
		        std::process::exit(1);
            }
        }
    }

    //Scan reg_ex: method call, input regex, output vec with translation
    let scanned_reg_ex = scan_regex(reg_ex);
    println!("{:?}", scanned_reg_ex);

    //Parse reg_ex: method call, input regex, output ????, if failed parse print error and exit

    //Build a state diagram for reg_ex
    //Do so in another method, input is ???? from parse, return diagram

    //Print the state diagram to stdut
    //Also do so in another method input is diagram, no return, it creates the file
    //To create file: let mut output = File::create("stdout.txt").expect("Unable to create file");
 
    //Read from stdin and print to stderr
    process_input();

}


/// For scanning the input regex into a vector of symbols easier to parse
/// - Input: Regex string to scan
/// - Output: Vector containing scanned string
/// - KEY:
///     - SIGMA -> SIGMA
///     - \w -> 1
///     - \d -> 2
///     - (,{ -> 3
///     - ),} -> 4
///     - '*' -> 5
///     - '+' -> 6
///     - | -> 7
fn scan_regex(reg: &str) -> Vec<char>{
    let mut scanned = Vec::new();
    let mut special_char = false;
    for char in reg.chars(){
        if special_char { // if the previous symbol was a '\'
            if char == 'w' {
                scanned.push('1');
                special_char = false;
            }
            else if char == 'd' {
                scanned.push('2');
                special_char = false;
            }
            else {
                //ERROR, not a \w or \d, not a valid regex
                writeln!(std::io::stderr(), "Invalid Input").unwrap();
		        std::process::exit(1);
            }
        }
        else if SIGMA.contains(&char){
            scanned.push(char); //push any alphabet characters straight to the vec
        }
        else if char == '\\' {
            special_char = true; //the next char must be a w or a d
        }
        else if char == '(' || char == '{' {
            scanned.push('3');
        }
        else if char == ')' || char == '}' {
            scanned.push('4');
        }
        else if char == '*'{
            scanned.push('5');
        }
        else if char == '+'{
            scanned.push('6');
        }
        else if char == '|'{
            scanned.push('7');
        }
    }

    scanned
}


/// For reading input from stdin and printing accept or reject for each line
/// - Input: ????
/// - Output: An accept or reject output followed by the string printed to stderr
fn process_input() {
    let mut stderr = std::io::stderr();
    let stdin = stdin();
    'outer: for line in stdin.lock().lines() {
        let string = line.unwrap();
        
        //check to make sure the string only containts symbols in the alphabet
        for char in string.chars() {
            if !SIGMA.contains(&char) {
                //if not, string is rejected
                writeln!(&mut stderr, "Reject {}", string).unwrap();
                continue 'outer; //continues the outer for loop to go to next string and skip regex
            }
        }
        
        //if string chars are valid, make sure it matches the regex
        if true{ //Check to see if it matches the regex here
            writeln!(&mut stderr, "Accept {}", string).unwrap();
        }else {
            writeln!(&mut stderr, "Reject {}", string).unwrap();
        }
    }
}
