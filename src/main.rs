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
//use std::fs::File; //for creating output file
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
    //println!("{:?}", scanned_reg_ex);

    //Parse reg_ex: method call, input regex, output transition diagram if failed parse print error and exit
    let trans_table = parse_regex(scanned_reg_ex);
    println!("Trans_table: \n {:?}", trans_table);

    //Print the state diagram to stdut
    //Also do so in another method input is diagram, no return, it creates the file
    //To create file: let mut output = File::create("stdout.txt").expect("Unable to create file");
 
    //Read from stdin and print to stderr
    process_input(trans_table);

}


/// For scanning the input regex into a vector of symbols easier to parse
/// - Input: Regex string to scan
/// - Output: Vector containing scanned string
/// - KEY:
///     - SIGMA -> SIGMA
///     - \w -> !
///     - \d -> @
///     - (,{ -> (
///     - ),} -> {
///     - '*' -> *
///     - '+' -> +
///     - | -> |
fn scan_regex(reg: &str) -> Vec<char>{
    let mut scanned = Vec::new();
    let mut special_char = false;
    let mut paren_count = 0; //used to make sure each ( and { has a matching ) or }
    for char in reg.chars(){
        if special_char { // if the previous symbol was a '\'
            if char == 'w' {
                scanned.push('!');
                special_char = false;
            }
            else if char == 'd' {
                scanned.push('@');
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
            scanned.push('(');
            paren_count += 1; //enter a paren
        }
        else if char == ')' || char == '}' {
            scanned.push(')');
            paren_count -= 1; //exit a paren
        }
        else if char == '*'{
            scanned.push('*');
        }
        else if char == '+'{
            scanned.push('+');
        }
        else if char == '|'{
            scanned.push('|');
        }
    }
    if paren_count != 0 {
        //not every parenthasis closes, error
        writeln!(std::io::stderr(), "Invalid Input").unwrap();
        std::process::exit(1);
    }
    scanned
}

/// For parsing the scanned regex input and translating it to a DFA
/// - Input: Vector containing the scanned regex
/// - Output: Vector of vectors containing the transition diagram
/// - KEY:
///     - SIGMA -> SIGMA
///     - \w -> !
///     - \d -> @
///     - (,{ -> (
///     - ),} -> {
///     - '*' -> *
///     - '+' -> +
///     - | -> |
fn parse_regex(reg: Vec<char>) -> Vec<Vec<String>>{
    let invalid_start_symbols = ['*', '+', ')', '|']; //Close parens/brackets, stars, plus, and bar are invalid start symbols
    if invalid_start_symbols.contains(&reg[0]) {
        //The regex cannot start with those symbols, throw an error
        writeln!(std::io::stderr(), "Invalid Input").unwrap();
        std::process::exit(1);
    }
    //Set up the transition table
    let mut transition_table = Vec::new();
    transition_table.push(new_table_row());
    let mut accept_states = Vec::new();
    accept_states.push("X".to_string());

    //Variables for parsing
    let mut current_state = 0; //for knowing what row is being worked with
    let mut index = 0;
    let mut paren_layer; //for finding the layer of nested paren
    let mut paren_index; //for finding the index of the closing paren
    let mut cont; //for use in while loops for finding closing paren
    let max_index = reg.len() - 1;
    
    //Loop through the scanned input and parse it
    while index <= max_index {
        //println!("Index: {}, Symbol: {}", index, reg[index]);
        //Check for symbols that can start statements
        //This means either plain alphabet characters, \d, \w, (, or }
        //All other characters are somehow part of a sequence started by these characters
        if SIGMA.contains(&reg[index]) || reg[index] == '!' || reg[index] == '@'{
            //symbol is an alphabet character, \w, or \d
            if index < max_index && reg[index+1] == '*' {
                if index + 1 < max_index && reg[index+2] == '|'{
                    index = or(index, &reg, &mut current_state, &mut transition_table, 'S', &mut accept_states);
                }
                else {
                    star(&reg[index], &mut current_state, &mut transition_table, &mut accept_states);
                    index += 2; //skip to after star symbol
                }
            }
            else if index < max_index && reg[index+1] == '+' {
                if index + 1 < max_index && reg[index+2] == '|'{
                    index = or(index, &reg, &mut current_state, &mut transition_table, 'P', &mut accept_states);
                }
                else {
                    plus(&reg[index], &mut current_state, &mut transition_table, &mut accept_states);
                    index += 2; //skip to after plus symbol
                }
            }
            else if index < max_index && reg[index+1] == '|' {
                index = or(index, &reg, &mut current_state, &mut transition_table, 'N', &mut accept_states); //skip to after or statement
            }
            else {
                add(&reg[index], &mut current_state, &mut transition_table, &mut accept_states); //Add this character to the transition diagram
                index += 1; //go to next symbol
            }
        }
        else if reg[index] == '(' {
            //symbol is an open bracket or parens
            if !invalid_next('(', &reg[index+1]) {
                //if the next character is invalid throw an error
                writeln!(std::io::stderr(), "Invalid Input").unwrap();
                std::process::exit(1);
            }
            
            paren_index = index + 1; //start checking for closing paren in next index
            paren_layer = 0; //in case there are nested parens
            cont = true; //for continuing until the closing paren is found
            while cont {
                //find the matching paren index
                if reg[paren_index] == '(' {
                    paren_layer += 1; //enter a nested paren
                }
                else if reg[paren_index] == ')' && paren_layer == 0 {
                    cont = false; //found the closing paren
                }
                else if reg[paren_index] == ')' && paren_layer > 0 {
                    paren_layer -= 1; //wrong closing paren, exit nested paren
                }
                paren_index += 1; //check next index
            }
            // paren_index now contains the index of the closing paren and it can be passed to a helper
            // first check for *, +, or | after the parens close
            if paren_index < max_index && reg[paren_index+1] == '*' {
                //call paren_star(reg, index, paren_index)
                index = paren_index + 2; //skip to after star symbol
            }
            else if paren_index < max_index && reg[paren_index+1] == '+' {
                //call paren_plus(reg, index, paren_index)
                index = paren_index + 2; //skip to after plus symbol
            }
            else if paren_index < max_index && reg[paren_index+1] == '|' {
                //index = paren_or(reg, index, paren_index); //skip to after or statement
            }
            else {
                //call paren_add(reg, index, paren_index)
                index = paren_index + 1; //go to symbol after parens close
            }

        }
        else {
            //There's been a parsing error and an invalid character has been found
            writeln!(std::io::stderr(), "Error Parsing input").unwrap();
            std::process::exit(1);
        }
    }
    //add a final vec holding the accept state
    accept_states.push(current_state.to_string());
    transition_table.push(accept_states);
    transition_table
}

/// A helper for parse_regex
/// - Input: The current character and the next character
/// - Output: Boolean value, true if the next character is valid and false if not
fn invalid_next(first: char, next: &char) -> bool {
    // A { or ( cannot be followed by a *, +, or |
    if first == '(' {
        if *next == '*' || *next == '+' || *next == '|' {
            false
        }
        else {true}
    }
    // A | cannot be followed by a *, +, |, ), or }
    else if first == '|' {
        if *next == ')' || *next == '*' || *next == '+' || *next == '|'{
            false
        }
        else {true}
    }
    // A + or * cannot be followed by a * or +
    else if first == '*' || first == '+' {
        if *next == '*' || *next == '+' {
            false
        }
        else {true}
    }
    else {true} //Other characters can have any character follow them
}

/// A function to add a single input character to the transition table
/// - Input: A character that must be input to reach a state, the current state as a mut value, and the transition table
/// - Output: None, the mut parameters are changed as needed
fn add(symbol: &char, state: &mut u32, table: &mut Vec<Vec<String>>, accept: &mut Vec<String>) {
    let next_state = *state + 1;
    let alpha = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let num = ['0','1','2','3','4','5','6','7','8','9'];
    if *symbol == '!' {
        //symbol is \w so any alpha value will do
        if accept.len() == 1 || accept[1] == state.to_string() {
            //There is only one accept state 
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if alpha.contains(char) {
                    //This is a char needed to transition to the next state
                    table[*state as usize][i] = next_state.to_string();
                }
                i += 1;
            }
        }
        else {
            //There is more than one accept state right now
            for st in &*accept {
                if st == "X" { continue;}
                let mut i = 0;
                for char in SIGMA.iter() {
                    if alpha.contains(char) {
                        //This is a char needed to transition to the next state
                        table[st.parse::<u32>().unwrap() as usize][i] = next_state.to_string();
                    }
                    i += 1;
                }
            }
        }
    }
    else if *symbol == '@' {
        if accept.len() == 1 || accept[1] == state.to_string() {
            //symbol is \d so any num value will do
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if num.contains(char) {
                    //This is a char needed to transition to the next state
                    table[*state as usize][i] = next_state.to_string();
                }
                i += 1;
            }
        }
        else {
            //There is more than one accept state right now
            for st in &*accept {
                if st == "X" { continue;}
                let mut i = 0;
                for char in SIGMA.iter() {
                    if num.contains(char) {
                        //This is a char needed to transition to the next state
                        table[st.parse::<u32>().unwrap() as usize][i] = next_state.to_string();
                    }
                    i += 1;
                }
            }
        }
    }
    else {
        //symbol is specific
        if accept.len() == 1 || accept[1] == state.to_string() {
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if symbol == char {
                    //This is the char needed to transition to the next state
                    table[*state as usize][i] = next_state.to_string();
                }
                i += 1;
            }
        }
        else {
            for st in &*accept {
                if st == "X" {continue;}
                let mut i = 0;
                for char in SIGMA.iter() {
                    if symbol == char {
                        table[st.parse::<u32>().unwrap() as usize][i] = next_state.to_string();
                    }
                    i += 1;
                }
            }
        }
    }
    let mut holder = Vec::<String>::new();
    holder.push("X".to_string());
    *accept = holder; //resets the accept states
    *state = next_state;
    table.push(new_table_row()); //add next row for next state
}

/// A function to add a starred input character to the transition table
/// - Input: Char that is starred, current state as a mut value, and the transition table
/// - Output: None, the mut parameters are changed as needed
fn star(symbol: &char, state: &mut u32, table: &mut Vec<Vec<String>> , accept: &mut Vec<String>){
    let alpha = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let num = ['0','1','2','3','4','5','6','7','8','9'];
    if *symbol == '!' {
        //symbol is \w so any alpha value will do
        if accept.len() == 1 || accept[1] == state.to_string() {
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if alpha.contains(char) {
                    //This is a char needed to transition to the next state
                    table[*state as usize][i] = state.to_string();
                }
                i += 1;
            }
        }
        else {
            for st in &*accept {
                if st == "X" {continue;}
                let mut i = 0; //index for keepting track of where you are in the transition table row
                for char in SIGMA.iter() {
                    if alpha.contains(char) {
                        //This is a char needed to transition to the next state
                        table[st.parse::<u32>().unwrap() as usize][i] = state.to_string();
                    }
                    i += 1;
                }
            }
        }
    }
    else if *symbol == '@' {
        //symbol is \d so any num value will do
        if accept.len() == 1 || accept[1] == state.to_string() {
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if num.contains(char) {
                    //This is a char needed to transition to the next state
                    table[*state as usize][i] = state.to_string();
                }
                i += 1;
            }
        }
        else {
            for st in &*accept {
                if st == "X" {continue;}
                let mut i = 0; //index for keepting track of where you are in the transition table row
                for char in SIGMA.iter() {
                    if num.contains(char) {
                        //This is a char needed to transition to the next state
                        table[st.parse::<u32>().unwrap() as usize][i] = state.to_string();
                    }
                    i += 1;
                }
            }
        }
    }
    else {
        //symbol is specific
        if accept.len() == 1 || accept[1] == state.to_string() {
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if symbol == char {
                    //This is the char needed to transition to the next state
                    table[*state as usize][i] = state.to_string();
                }
                i += 1;   
            }
        }
        else {
            for st in &*accept {
                if st == "X" {continue;}
                let mut i = 0; //index for keepting track of where you are in the transition table row
                for char in SIGMA.iter() {
                    if symbol == char {
                        //This is the char needed to transition to the next state
                        table[st.parse::<u32>().unwrap() as usize][i] = state.to_string();
                    }
                    i += 1;   
                }
            }
        }
    }
}

/// A function to add a 'plussed' input character to the transition table
/// - Input: Char that is starred, current state as a mut value, and the transition table
/// - Output: None, the mut parameters are changed as needed
fn plus(symbol: &char, state: &mut u32, table: &mut Vec<Vec<String>>,  accept: &mut Vec<String>) {
    add(symbol, state, table, accept); // character must be entered at least once
    star(symbol, state, table, accept); // it can then be treated like a starred character
}

/// A function to add an or statement to the transition table
/// - Input: Char that is starred, current state as a mut value, and the transition table
/// - Output: New index to jump to (the index after the last char of the or statement)
fn or(index: usize, regex: &Vec<char>, state: &mut u32, table: &mut Vec<Vec<String>>, special: char,  accept: &mut Vec<String>) -> usize {
    let left_or = regex[index];
    let mut index_jump = 2;
    let mut next_state = *state + 1;
    let alpha = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let num = ['0','1','2','3','4','5','6','7','8','9'];
    
    let mut acc_states = Vec::new(); //for when things get complex
    acc_states.push("X".to_string());

    if special == 'S' {
        index // an or statement where the left is a symbol*
    }
    else if special == 'P' {
        index // an or statement where the left is a symbol+
    }
    else {

    //check that there is a right side of the or, and that it has valid input
    if (index+2) >= regex.len() || !invalid_next('|', &regex[index+2]){
        //if the next character is invalid throw an error
        writeln!(std::io::stderr(), "Invalid Input").unwrap();
        std::process::exit(1);
    }
    //Check what is on the other side of the or
    if regex[index+2] == '(' {
        // There is a statement on the other side bounded by parens

    }
    else{
        let right_or = regex[index+2];
        // There is not a parenthetical argument on the other side
        // Check if the next character has a *, +, or | operation on it
        if (index+3) < regex.len() && regex[index+3] == '*' {
            //left_or | starred right_or, check for another |
            index_jump = 4;
            if (index+4) < regex.len() && regex[index+4] == '|' {
                //call a helper for stacked_or, really complex
            }
            else {
                // This gets complicated here
                let state_holder = *state; //this will still be an accept state
                add(&left_or, state, table, accept); //add the first symbol, get back second accept state
                next_state = *state + 1;
                if right_or == '!' {
                    //symbol is \w so any alpha value will do
                    let mut i = 0; //index for keepting track of where you are in the transition table row
                    for char in SIGMA.iter() {
                        if alpha.contains(char) {
                            //This is a char needed to transition to the next state
                            table[state_holder as usize][i] = next_state.to_string();
                        }
                        i += 1;
                    }
                }
                else if right_or == '@' {
                    //symbol is \d so any num value will do
                    let mut i = 0; //index for keepting track of where you are in the transition table row
                    for char in SIGMA.iter() {
                        if num.contains(char) {
                            //This is a char needed to transition to the next state
                            table[state_holder as usize][i] = next_state.to_string();
                        }
                        i += 1;
                    }
            
                }
                else {
                    let mut i = 0; //index for keepting track of where you are in the transition table row
                    for char in SIGMA.iter() {
                        if right_or == *char {
                            //This is the char needed to transition to the next state
                            table[state_holder as usize][i] = next_state.to_string();
                        }
                        i += 1;
                    }
                }
                let state_holder_2 = *state; //holds second accepting state
                *state = next_state;
                table.push(new_table_row());
                star(&right_or, state, table, accept);
                next_state = *state;
                acc_states.push(state_holder.to_string());
                acc_states.push(state_holder_2.to_string());
                acc_states.push(state.to_string());
                //accept states are currently in state_holder, state_holder_2, and state
            }

        }
        else if (index+3) < regex.len() && regex[index+3] == '+' {
            //left_or | plus right_or, check for another |
            index_jump = 4;
            if (index+4) < regex.len() && regex[index+4] == '|' {
                //call a helper for stacked_or, really complex
            }
            else {
                // this gets complicated here
                // This gets complicated here
                let state_holder = *state; //this will NOT still be an accept state
                add(&left_or, state, table, accept); //add the first symbol, get back second accept state
                next_state = *state + 1;
                if right_or == '!' {
                    //symbol is \w so any alpha value will do
                    let mut i = 0; //index for keepting track of where you are in the transition table row
                    for char in SIGMA.iter() {
                        if alpha.contains(char) {
                            //This is a char needed to transition to the next state
                            table[state_holder as usize][i] = next_state.to_string();
                        }
                        i += 1;
                    }
                }
                else if right_or == '@' {
                    //symbol is \d so any num value will do
                    let mut i = 0; //index for keepting track of where you are in the transition table row
                    for char in SIGMA.iter() {
                        if num.contains(char) {
                            //This is a char needed to transition to the next state
                            table[state_holder as usize][i] = next_state.to_string();
                        }
                        i += 1;
                    }
            
                }
                else {
                    
                    let mut i = 0; //index for keepting track of where you are in the transition table row
                    for char in SIGMA.iter() {
                        if right_or == *char {
                            //This is the char needed to transition to the next state
                            table[state_holder as usize][i] = next_state.to_string();
                        }
                        i += 1;
                    }
                }
                let state_holder_2 = *state; //holds second accepting state
                *state = next_state;
                table.push(new_table_row());
                star(&right_or, state, table, accept);
                next_state = *state; //to make double sure
                acc_states.push(state_holder_2.to_string());
                acc_states.push(state.to_string());
                //accept states are currently in state_holder_2, and state
            }
        }
        else if (index+3) < regex.len() && regex[index+3] == '|' {
            //left_or | right_or | another_thing, check for another |
            // call a helper for stacked_or
            index_jump = 4;
        }
        else {
            index_jump = 3;
            //simple left_or char | right_or char
            if left_or == '!' || right_or == '!'{
                //symbol is \w so any alpha value will do
                let mut i = 0; //index for keepting track of where you are in the transition table row
                for char in SIGMA.iter() {
                    if alpha.contains(char) {
                        //This is a char needed to transition to the next state
                        table[*state as usize][i] = next_state.to_string();
                    }
                    i += 1;
                }
            }
            if left_or == '@' || right_or == '@'{
                //symbol is \d so any num value will do
                let mut i = 0; //index for keepting track of where you are in the transition table row
                for char in SIGMA.iter() {
                    if num.contains(char) {
                        //This is a char needed to transition to the next state
                        table[*state as usize][i] = next_state.to_string();
                    }
                    i += 1;
                }
            }
            if left_or != '!' || left_or != '@' || right_or != '!' || right_or != '@' {
                //symbol is specific
                let mut i = 0; //index for keepting track of where you are in the transition table row
                for char in SIGMA.iter() {
                    if left_or == *char {
                        //This is a char needed to transition to the next state
                        table[*state as usize][i] = next_state.to_string();
                    }
                    else if right_or == *char {
                        //This is a char needed to transition to the next state
                        table[*state as usize][i] = next_state.to_string();
                    }
                    i += 1;
                }
            }
            table.push(new_table_row()); //add next row for next state
        }
    }
    *state = next_state;
    *accept = acc_states;
    index + index_jump
    }   
}

/// For creating a blank row to add to the transition table where all elements are defined
/// - Input: None
/// - Output: A blank row to add to the transition table, where for every element of SIGMA there is a " "
fn new_table_row() -> Vec::<String> {
    let mut row = Vec::<String>::new();
    for _char in SIGMA.iter() {
        row.push(" ".to_string());
    }
    row
}

/// For reading input from stdin and printing accept or reject for each line
/// - Input: Transition table
/// - Output: An accept or reject output followed by the string printed to stderr
fn process_input(table: Vec<Vec<String>>) {
    let mut stderr = std::io::stderr();
    let stdin = stdin();
    'outer: for line in stdin.lock().lines() {
        let string = line.unwrap();
        
        //check to make sure the string only containts symbols in the alphabet
        for char in string.chars() {
            if !SIGMA.contains(&char) {
                //if not, string is rejected
                writeln!(&mut stderr, "Reject {}", &string).unwrap();
                continue 'outer; //continues the outer for loop to go to next string and skip regex
            }
        }
        
        //if string chars are valid, make sure it matches the regex
        if check_string(&string, &table){ //Check to see if it matches the regex here
            writeln!(&mut stderr, "Accept {}", &string).unwrap();
        }else {
            writeln!(&mut stderr, "Reject {}", &string).unwrap();
        }
    }
}


/// For navigating the transition table and seeing if strings are valid
/// - Input: String and transition table
/// - Output: Boolean, true if string is valid, false if not
fn check_string(input: &str, table: &Vec<Vec<String>>) -> bool{
    let mut curr_state = 0;
    let len = table.len()-1;
    for char in input.chars() {
        if curr_state > len-1 {
            return false //something went wrong
        }
        else if table[curr_state][SIGMA.iter().position(|&x| x==char ).unwrap()] == " " {
            return false //there is no transition for this input from this state
        }
        else {
            curr_state = table[curr_state][SIGMA.iter().position(|&x| x==char ).unwrap()].parse::<u32>().unwrap() as usize;
        }
    }
    for state in &table[len] {
        if state == &curr_state.to_string() {return true} //valid end state reached!
    }
    false //end state not reached
}