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
    //println!("{:?}", scanned_reg_ex);

    //Parse reg_ex: method call, input regex, output transition diagram if failed parse print error and exit
    let trans_table = parse_regex(scanned_reg_ex);
    println!("Trans_table: \n {:?}", trans_table);

    //Print the state diagram to stdout
    //Also do so in another method input is diagram, no return, it creates the file
    print_state_diagram(&trans_table);
 
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
    let mut index = 0; //for keeping your place in the regex
    let mut paren_layer; //for finding the layer of nested paren
    let mut paren_index; //for finding the index of the closing paren
    let mut cont; //for use in while loops for finding closing paren
    let mut bookmark = 0; //because concatination takes precidence, bookmarking state to loop back to for or statements. Heck
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
                    index = or(index, &mut bookmark, &reg, &mut current_state, &mut transition_table, 'S', &mut accept_states);
                }
                else {
                    star(&reg[index], &mut current_state, &mut transition_table, &mut accept_states);
                    index += 2; //skip to after star symbol
                }
            }
            else if index < max_index && reg[index+1] == '+' {
                if index + 1 < max_index && reg[index+2] == '|'{
                    index = or(index, &mut bookmark, &reg, &mut current_state, &mut transition_table, 'P', &mut accept_states);
                }
                else {
                    plus(&reg[index], &mut current_state, &mut transition_table, &mut accept_states);
                    index += 2; //skip to after plus symbol
                }
            }
            else if index < max_index && reg[index+1] == '|' {
                index = or(index, &mut bookmark, &reg, &mut current_state, &mut transition_table, 'N', &mut accept_states); //skip to after or statement
            }
            else {
                if bookmark == 0 { //makes sure not to reset accept states
                    add_or(&reg[index], &mut current_state, &mut transition_table, &mut accept_states); //Add this character to the transition diagram
                }
                else {
                    add(&reg[index], &mut current_state, &mut transition_table, &mut accept_states); //Add this character to the transition diagram
                }
                
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
            bookmark = current_state;
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
                    paren_index -= 1; //to account for the addition at the end of this loop
                }
                else if reg[paren_index] == ')' && paren_layer > 0 {
                    paren_layer -= 1; //wrong closing paren, exit nested paren
                }
                paren_index += 1; //check next index
            }
            index += 1; //set it to the first char inside the paren statement

            // paren_index now contains the index of the closing paren and it can be passed to a helper
            // first check for *, +, or | after the parens close
            if paren_index < max_index && reg[paren_index+1] == '*' {
                if paren_index + 1 < max_index && reg[paren_index+2] == '|'{
                    index = paren_or(index, &mut bookmark, paren_index, &reg, &mut current_state, &mut transition_table, &mut accept_states);
                }
                else {
                    paren_star_plus(index, &mut bookmark, paren_index, &reg, &mut current_state, &mut transition_table, &mut accept_states, "Star");
                    index = paren_index + 2; //skip to after star symbol
                }
            }
            else if paren_index < max_index && reg[paren_index+1] == '+' {
                if paren_index + 1 < max_index && reg[paren_index+2] == '|'{
                    index = paren_or(index, &mut bookmark, paren_index, &reg, &mut current_state, &mut transition_table, &mut accept_states);
                }
                else {
                    paren_star_plus(index, &mut bookmark, paren_index, &reg, &mut current_state, &mut transition_table, &mut accept_states, "Plus");
                    index = paren_index + 2; //skip to after plus symbol
                }
            }
            else if paren_index < max_index && reg[paren_index+1] == '|' {
                index = paren_or(index, &mut bookmark, paren_index, &reg, &mut current_state, &mut transition_table, &mut accept_states); 
            }
            else {
                paren_add(index, &mut bookmark, paren_index, &reg, &mut current_state, &mut transition_table, &mut accept_states); 
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
    if !accept_states.contains(&current_state.to_string()){
        accept_states.push(current_state.to_string());
    }
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
/// - Input: A character that must be input to reach a state, the current state as a mut value, the transition table, and vector of accept states
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
            //repeat for current state
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if alpha.contains(char) {
                    //This is a char needed to transition to the next state
                    table[*state as usize][i] = next_state.to_string();
                }
                i += 1;
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
            //repeat for current state
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if num.contains(char) {
                    //This is a char needed to transition to the next state
                    table[*state as usize][i] = next_state.to_string();
                }
                i += 1;
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
            //repeat for current state
            let mut i = 0;
            for char in SIGMA.iter() {
                if symbol == char {
                    //This is the char needed to transition to the next state
                    table[*state as usize][i] = next_state.to_string();
                }
                i += 1;
            }
        }
    }
    let mut holder = Vec::<String>::new();
    holder.push("X".to_string());
    *accept = holder; //resets the accept states
    *state = next_state;
    table.push(new_table_row()); //add next row for next state
}

/// A function route a specific symbol to a specific state in the transition table
/// - Input: A character that must be input to reach a state, the current state as a mut value, the transition table, vector of accept states, and a specified next state
/// - Output: None, the mut parameters are changed as needed
fn add_to(symbol:&char, state: u32, table: &mut Vec<Vec<String>>, accept: &mut Vec<String>, to_state: u32) {
    let alpha = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let num = ['0','1','2','3','4','5','6','7','8','9'];
    if *symbol == '!' {
        //symbol is \w so any alpha value will do
        //if accept.len() == 1 || accept[1] == state.to_string() {
            //There is only one accept state 
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if alpha.contains(char) {
                    //This is a char needed to transition to the next state
                    table[state as usize][i] = to_state.to_string();
                }
                i += 1;
            }
        //}
        /*else {
            //There is more than one accept state right now
            for st in &*accept {
                if st == "X" { continue;}
                let mut i = 0;
                for char in SIGMA.iter() {
                    if alpha.contains(char) {
                        //This is a char needed to transition to the next state
                        table[st.parse::<u32>().unwrap() as usize][i] = to_state.to_string();
                    }
                    i += 1;
                }
            }
        }*/
    }
    else if *symbol == '@' {
        //if accept.len() == 1 || accept[1] == state.to_string() {
            //symbol is \d so any num value will do
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if num.contains(char) {
                    //This is a char needed to transition to the next state
                    table[state as usize][i] = to_state.to_string();
                }
                i += 1;
            }
        //}
        /*else {
            //There is more than one accept state right now
            for st in &*accept {
                if st == "X" { continue;}
                let mut i = 0;
                for char in SIGMA.iter() {
                    if num.contains(char) {
                        //This is a char needed to transition to the next state
                        table[st.parse::<u32>().unwrap() as usize][i] = to_state.to_string();
                    }
                    i += 1;
                }
            }
        }*/
    }
    else {
        //symbol is specific
        //if accept.len() == 1 || accept[1] == state.to_string() {
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if symbol == char {
                    //This is the char needed to transition to the next state
                    table[state as usize][i] = to_state.to_string();
                }
                i += 1;
            }
        //}
        /*else {
            for st in &*accept {
                if st == "X" {continue;}
                let mut i = 0;
                for char in SIGMA.iter() {
                    if symbol == char {
                        table[st.parse::<u32>().unwrap() as usize][i] = to_state.to_string();
                    }
                    i += 1;
                }
            }
        }*/
    }
    let mut holder = Vec::<String>::new();
    holder.push("X".to_string());
    *accept = holder; //resets the accept states
    table.push(new_table_row()); //add next row for next state
}

/// A function to add a single input character to the transition table from an or statement
/// Doesn't reset the transition table
/// - Input: A character that must be input to reach a state, the current state as a mut value, the transition table, and vector of accept states
/// - Output: None, the mut parameters are changed as needed
fn add_or(symbol: &char, state: &mut u32, table: &mut Vec<Vec<String>>, _accept: &mut Vec<String>) {
    let next_state = *state + 1;
    let alpha = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let num = ['0','1','2','3','4','5','6','7','8','9'];
    if *symbol == '!' {
        //symbol is \w so any alpha value will do
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
    else if *symbol == '@' {
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
        //symbol is specific
            let mut i = 0; //index for keepting track of where you are in the transition table row
            for char in SIGMA.iter() {
                if symbol == char {
                    //This is the char needed to transition to the next state
                    table[*state as usize][i] = next_state.to_string();
                }
                i += 1;
            }
    }
    //let mut holder = Vec::<String>::new();
    //holder.push("X".to_string());
    //*accept = holder; //resets the accept states
    *state = next_state;
    table.push(new_table_row()); //add next row for next state
}

/// A function to add a starred input character to the transition table
/// - Input: Char that is starred, current state as a mut value, the transition table, and vector of accept states
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
/// - Input: Char that is plussed, current state as a mut value, the transition table, and vector of accept states
/// - Output: None, the mut parameters are changed as needed
fn plus(symbol: &char, state: &mut u32, table: &mut Vec<Vec<String>>,  accept: &mut Vec<String>) {
    add(symbol, state, table, accept); // character must be entered at least once
    star(symbol, state, table, accept); // it can then be treated like a starred character
}

/// A function to add an or statement to the transition table
/// - Input: Starting index, regex, current state as a mut value, the transition table, character for marking specific symbols before the |, and vector of accept states
/// - Output: New index to jump to (the index after the last char of the or statement)
fn or(index: usize, bookmark: &mut u32, regex: &Vec<char>, state: &mut u32, table: &mut Vec<Vec<String>>, special: char,  accept: &mut Vec<String>) -> usize {
    let left_or = regex[index];
    let mut index_jump = 2;
    let mut next_state = *state + 1;
    
    let mut acc_states = Vec::new(); //for when things get complex
    acc_states.push("X".to_string());

    if index + 3 < regex.len() && regex[index + 3] == '|' || index + 5 < regex.len() && regex[index + 5] == '|' {
        //In this situation, there is a 'stacked' or that does not include a parentheses
        stacked_or(index, bookmark, regex, state, table, accept)
    }
    else if index + 4 < regex.len() && regex[index + 4] == '|' && (regex[index + 1] == '*' || regex[index + 3] == '*' || regex[index + 1] == '+' || regex[index + 3] == '+'){
        stacked_or(index, bookmark, regex, state, table, accept)
    }
    else if special == 'S' {
        index_jump = 4;
        // a*|b* is essentially equivalent to a*|b+
        if (index+index_jump - 1) < regex.len() && regex[index+index_jump-1] == '(' {  
            let mut paren_index = index + 3; //start checking for closing paren in next index
            let mut paren_layer = 0; //in case there are nested parens
            let mut cont = true; //for continuing until the closing paren is found
            let held_state = *state;
            while cont {
                //find the matching paren index
                if regex[paren_index] == '(' {
                    paren_layer += 1; //enter a nested paren
                }
                else if regex[paren_index] == ')' && paren_layer == 0 {
                    cont = false; //found the closing paren
                    paren_index -= 1; //to account for the addition at the end of this loop
                }
                else if regex[paren_index] == ')' && paren_layer > 0 {
                    paren_layer -= 1; //wrong closing paren, exit nested paren
                }
                paren_index += 1; //check next index
            }
            if paren_index + 1 < regex.len() && regex[paren_index + 1] == '*' {
                paren_star_plus(index + 4, bookmark, paren_index, regex, state, table, accept, "S");
            }
            else if paren_index + 1 < regex.len() && regex[paren_index + 1] == '+' {
                paren_star_plus(index + 4, bookmark, paren_index, regex, state, table, accept, "P");
            }
            else {
                paren_add(index + 4, bookmark, paren_index, regex, state, table, accept);
            }
            add_to(&regex[index], held_state, table, accept, *state + 1);
            *state += 1;
            star(&regex[index], state, table, accept);
            table.remove(table.len()-1 as usize); //remove uneeded table row
            acc_states.push(held_state.to_string());
            acc_states.push(state.to_string());
            acc_states.push((*state+1).to_string());
            *accept = acc_states;
            return paren_index + 1;
        }
        else if (index+index_jump) < regex.len() && regex[index + index_jump] == '*' || (index+index_jump) < regex.len() && regex[index + index_jump] == '+'{
            //a*|b*
            index_jump += 1;
            let right_or = regex[index+3];
            let state_holder = *state; //this will still be an accept state
            acc_states = accept.clone(); //these will still be accept states
            add_or(&left_or, state, table, accept); //add the first symbol, get back second accept state
            let mut empty = Vec::<String>::new();
            empty.push("X".to_string());
            star(&left_or, state, table, &mut empty);
            let state_holder_2 = *state;
            next_state = *state + 1;
            let mut acc_holder = acc_states.clone();
            add_to(&right_or, *bookmark, table, &mut acc_holder, next_state);
            star(&right_or, &mut next_state, table, &mut empty);
            if !acc_states.contains(&state_holder.to_string()) {
                acc_states.push(state_holder.to_string());
            }
            for st in acc_holder {
                if !acc_states.contains(&st.to_string()) {
                    acc_states.push(st.to_string());
                }
            }
            acc_states.push(state_holder_2.to_string());
            acc_states.push(next_state.to_string());
            acc_states.push(bookmark.to_string());
            *state = next_state;
            *accept = acc_states;
            index + index_jump
        }
        else {
            //a*|b
            let right_or = regex[index+3];
            let state_holder = *state; //this will still be an accept state
            acc_states = accept.clone(); //these will still be accept states
            add_or(&left_or, state, table, accept); //add the first symbol, get back second accept state
            let mut empty = Vec::<String>::new();
            empty.push("X".to_string());
            star(&left_or, state, table, &mut empty);
            let state_holder_2 = *state;
            next_state = *state + 1;
            let mut acc_holder = acc_states.clone();
            add_to(&right_or, *bookmark, table, &mut acc_holder, next_state);
            if !acc_states.contains(&state_holder.to_string()) {
                acc_states.push(state_holder.to_string());
            }
            for st in acc_holder {
                if !acc_states.contains(&st.to_string()) {
                    acc_states.push(st.to_string());
                }
            }
            acc_states.push(bookmark.to_string());
            acc_states.push(state_holder_2.to_string());
            acc_states.push(next_state.to_string());
            *state = next_state;
            *accept = acc_states;
            index + index_jump
        }
    }
    else if special == 'P' {
        index_jump = 4;
        if (index+index_jump - 1) < regex.len() && regex[index+index_jump-1] == '(' {
            let mut paren_index = index + 3; //start checking for closing paren in next index
            let mut paren_layer = 0; //in case there are nested parens
            let mut cont = true; //for continuing until the closing paren is found
            let held_state = *state;
            while cont {
                //find the matching paren index
                if regex[paren_index] == '(' {
                    paren_layer += 1; //enter a nested paren
                }
                else if regex[paren_index] == ')' && paren_layer == 0 {
                    cont = false; //found the closing paren
                    paren_index -= 1; //to account for the addition at the end of this loop
                }
                else if regex[paren_index] == ')' && paren_layer > 0 {
                    paren_layer -= 1; //wrong closing paren, exit nested paren
                }
                paren_index += 1; //check next index
            }
            if paren_index + 1 < regex.len() && regex[paren_index + 1] == '*' {
                paren_star_plus(index + 4, bookmark, paren_index, regex, state, table, accept, "S");
            }
            else if paren_index + 1 < regex.len() && regex[paren_index + 1] == '+' {
                paren_star_plus(index + 4, bookmark, paren_index, regex, state, table, accept, "P");
            }
            else {
                paren_add(index + 4, bookmark, paren_index, regex, state, table, accept);
            }
            add_to(&regex[index], held_state, table, accept, *state + 1);
            *state += 1;
            star(&regex[index], state, table, accept);
            table.remove(table.len()-1 as usize); //remove uneeded table row
            acc_states.push(held_state.to_string());
            acc_states.push(state.to_string());
            acc_states.push((*state+1).to_string());
            *accept = acc_states;
            return paren_index + 1;
        }
        else if (index+index_jump) < regex.len() && regex[index + index_jump] == '*' {
            //a+|b*
            index_jump += 1;
            let right_or = regex[index+3];
            let state_holder = *state; //this will be an accept state
            acc_states = accept.clone(); //these will be accept states
            add_or(&left_or, state, table, accept); //add the first symbol, get back second accept state
            let mut empty = Vec::<String>::new();
            empty.push("X".to_string());
            star(&left_or, state, table, &mut empty);
            let state_holder_2 = *state;
            next_state = *state + 1;
            let mut acc_holder = acc_states.clone();
            add_to(&right_or, *bookmark, table, &mut acc_holder, next_state);
            star(&right_or, &mut next_state, table, &mut empty);
            if !acc_states.contains(&state_holder.to_string()) {
                acc_states.push(state_holder.to_string());
            }
            for st in acc_holder {
                if !acc_states.contains(&st.to_string()) {
                    acc_states.push(st.to_string());
                }
            }
            acc_states.push(state_holder_2.to_string());
            acc_states.push(next_state.to_string());
            acc_states.push(bookmark.to_string());
            acc_states.push(state.to_string());
            *state = next_state;
            *accept = acc_states;
            index + index_jump
        }
        else if (index+index_jump) < regex.len() && regex[index + index_jump] == '+' {
            //a+|b+
            index_jump += 1;
            let right_or = regex[index+3];
            //let state_holder = *state; //this will not be an accept state
            acc_states = accept.clone(); //these will not be accept states
            add_or(&left_or, state, table, accept); //add the first symbol, get back second accept state
            let mut empty = Vec::<String>::new();
            empty.push("X".to_string());
            star(&left_or, state, table, &mut empty);
            let state_holder_2 = *state;
            next_state = *state + 1;
            add_to(&right_or, *bookmark, table, &mut acc_states, next_state);
            star(&right_or, &mut next_state, table, &mut empty);
            acc_states.push(state_holder_2.to_string());
            //acc_states.push(next_state.to_string());
            *state = next_state;
            *accept = acc_states;
            index + index_jump
        }
        else {
            //a+|b
            let right_or = regex[index+3];
            let state_holder = *state; //this will not be an accept state
            acc_states = accept.clone(); //these will not be accept states
            add(&left_or, state, table, accept); //add the first symbol, get back second accept state
            let mut empty = Vec::<String>::new();
            empty.push("X".to_string());
            star(&left_or, state, table, &mut empty);
            let state_holder_2 = *state;
            next_state = *state + 1;
            add_to(&right_or, state_holder, table, &mut acc_states, next_state);
            acc_states.push(state_holder_2.to_string());
            acc_states.push(next_state.to_string());
            *state = next_state;
            *accept = acc_states;
            index + index_jump
        }
    }
    else {

        //check that there is a right side of the or, and that it has valid input
        if (index+2) >= regex.len() || !invalid_next('|', &regex[index+2]){
            //if the next character is invalid throw an error
            writeln!(std::io::stderr(), "Invalid Input").unwrap();
            std::process::exit(1);
        }
        //Check what is on the other side of the or
        if regex[index+index_jump] == '(' {
            let mut paren_index = index + 3; //start checking for closing paren in next index
            let mut paren_layer = 0; //in case there are nested parens
            let mut cont = true; //for continuing until the closing paren is found
            let held_state = *state;
            while cont {
                //find the matching paren index
                if regex[paren_index] == '(' {
                    paren_layer += 1; //enter a nested paren
                }
                else if regex[paren_index] == ')' && paren_layer == 0 {
                    cont = false; //found the closing paren
                    paren_index -= 1; //to account for the addition at the end of this loop
                }
                else if regex[paren_index] == ')' && paren_layer > 0 {
                    paren_layer -= 1; //wrong closing paren, exit nested paren
                }
                paren_index += 1; //check next index
            }
            if paren_index + 1 < regex.len() && regex[paren_index + 1] == '*' {
                paren_star_plus(index + 3, bookmark, paren_index, regex, state, table, accept, "S");
            }
            else if paren_index + 1 < regex.len() && regex[paren_index + 1] == '+' {
                paren_star_plus(index + 3, bookmark, paren_index, regex, state, table, accept, "P");
            }
            else {
                paren_add(index + 3, bookmark, paren_index, regex, state, table, accept);
                println!("TABLE {:?}", table);
            }
            add_to(&regex[index], held_state, table, accept, *state);
            table.remove(table.len()-1 as usize); //remove uneeded table row
            *accept = acc_states;
            return paren_index + 1

        }
        else{
            let right_or = regex[index+2];
            // There is not a parenthetical argument on the other side
            // Check if the next character has a *, +, or | operation on it
            if (index+3) < regex.len() && regex[index+3] == '*' {
                //left_or | starred right_or, check for another |
                index_jump = 4;
                // This gets complicated here
                //let state_holder = *state; //this will still be an accept state
                acc_states = accept.clone(); //these will still be accept states
                add_or(&left_or, state, table, accept); //add the first symbol, get back second accept state
                let state_holder_2 = *state;
                next_state = *state + 1;
                let mut acc_holder = acc_states.clone();
                add_to(&right_or, *bookmark, table, &mut acc_holder, next_state);
                let mut empty = Vec::<String>::new();
                empty.push("X".to_string());
                star(&right_or, &mut next_state, table, &mut empty);
                //if !acc_states.contains(&state_holder.to_string()) {
                //     acc_states.push(state_holder.to_string());
                //}
                for st in acc_holder {
                    if !acc_states.contains(&st.to_string()) {
                        acc_states.push(st.to_string());
                    }
                }
                acc_states.push(bookmark.to_string());
                acc_states.push(state_holder_2.to_string());
                acc_states.push(next_state.to_string());
            }
            else if (index+3) < regex.len() && regex[index+3] == '+' {
                //left_or | plus right_or, check for another |
                index_jump = 4;
                // This gets complicated here
                //let state_holder = *state; //this will not be an accept state
                acc_states = accept.clone(); //these will not be accept states
                add_or(&left_or, state, table, accept); //add the first symbol, get back second accept state
                let state_holder_2 = *state;
                next_state = *state + 1;
                add_to(&right_or, *bookmark, table, &mut acc_states, next_state);
                let mut empty = Vec::<String>::new();
                empty.push("X".to_string());
                star(&right_or, &mut next_state, table, &mut empty);
                acc_states.push(state_holder_2.to_string());
                acc_states.push(next_state.to_string());
            }
            else if (index+3) < regex.len() && regex[index+3] == '|' {
                //left_or | right_or | another_thing
                index_jump = stacked_or(index, bookmark, regex, state, table, accept);
            }
            else {
                index_jump = 3;
                //simple left_or char | right_or char
                //let mut st_holder = *state;
                acc_states = accept.clone();
                let mut acc_holder = accept.clone();
                add_or(&left_or, state, table, accept);
                acc_states.push(state.to_string());
                add_to(&right_or, *bookmark, table, &mut acc_holder, *state +1);
                //add(&right_or, &mut st_holder, table, &mut acc_states);
                //table.remove(table.len()-2 as usize); //remove uneeded table row
                next_state = *state + 1;
            }
        }
        *state = next_state;
        *accept = acc_states;
        index + index_jump
    }
       
}

/// A function to add an or statement to the transition table when there is more than one '|' involved
/// For example: a|b|c
/// - Input: Starting index, regex, current state as a mut value, the transition table, character for marking specific symbols before the |, and vector of accept states
/// - Output: New index to jump to (the index after the last char of the or statement) 
fn stacked_or(index: usize, bookmark: &mut u32, regex: &Vec<char>, state: &mut u32, table: &mut Vec<Vec<String>>,  accept: &mut Vec<String>) -> usize{
    //Set up working vars
    let mut index_jump = 0;
    let left_or = regex[index + index_jump];
    let mut right_or;
    let mut cont = true;
    let st_holder = *state;
    let mut acc_states = accept.clone();
    let mut acc_holder = Vec::<String>::new();
    let mut empty = Vec::<String>::new();
    empty.push("X".to_string());
    acc_holder.push("X".to_string());
    let mut next_state = *state;
    let mut use_next_state = false;

    //check that there is a right side of the or, and that it has valid input
    if !invalid_next('|', &regex[index]){
        //if the next character is invalid throw an error
        writeln!(std::io::stderr(), "Invalid Input").unwrap();
        std::process::exit(1);
    }

    //First statement in the or
    if regex[index + 1] == '*' {
        //first element is starred
        //bookmark is still an accept state
        acc_holder.push(bookmark.to_string());
        add_or(&left_or, state, table, accept);
        star(&left_or, state, table, &mut empty);
        index_jump += 3;
        acc_holder.push(state.to_string());
        next_state = *state + 1;
        use_next_state = true;
    }
    else if regex[index + 1] == '+' {
        //first element is plussed
        //st_holder is not an accept state
        add_or(&left_or, state, table, accept);
        star(&left_or, state, table, &mut empty);
        index_jump += 3;
        acc_holder.push(state.to_string());
        next_state = *state + 1;
        use_next_state = true;
    }
    else {
        //first element is only added
        //st_holder is not an accept state
        add_or(&left_or, state, table, accept);
        index_jump += 2;
        acc_holder.push(state.to_string());
    }

    //All the middle statemets in the or
    while cont {
        //check that there is a right side of the or, and that it has valid input
        if (index+index_jump) >= regex.len() || !invalid_next('|', &regex[index+index_jump]){
            //if the next character is invalid throw an error
            writeln!(std::io::stderr(), "Invalid Input").unwrap();
            std::process::exit(1);
        }

        if index + index_jump + 1 < regex.len() && regex[index + index_jump + 1]  == '|' || index + index_jump + 2 < regex.len() && regex[index + index_jump + 2]  == '|' {
            if regex[index + index_jump + 1] == '*' {
                // element is starred
                *accept = acc_states.clone();
                right_or = regex[index + index_jump];
                acc_holder.push(bookmark.to_string());
                if use_next_state {
                    //*state = st_holder;
                    //acc_holder.push(state.to_string());
                    add_to(&right_or, *bookmark, table, accept, next_state);
                    *state = next_state;
                    star(&right_or, state, table, &mut empty);
                }
                else {
                    next_state = *state + 1;
                    *state = st_holder;
                    //acc_holder.push(state.to_string());
                    add_to(&right_or, *bookmark, table, accept, next_state);
                    *state = next_state;
                    star(&right_or, state, table, &mut empty);
                }
                acc_holder.push(state.to_string());
                next_state = *state + 1;
                use_next_state = true;
                index_jump += 3;
            }
            else if regex[index + index_jump + 1] == '+' {
                // element is plussed
                *accept = acc_states.clone();
                acc_holder.push(state.to_string());
                right_or = regex[index + index_jump];
                if use_next_state {
                    *state = st_holder;
                    add_to(&right_or, *bookmark, table, accept, next_state);
                    *state = next_state;
                    star(&right_or, state, table, &mut empty);
                }
                else {
                    next_state = *state + 1;
                    *state = st_holder;
                    add_to(&right_or, *bookmark, table, accept, next_state);
                    *state = next_state;
                    star(&right_or, state, table, &mut empty);
                }
                acc_holder.push(state.to_string());
                next_state = *state + 1;
                use_next_state = true;
                index_jump += 3;
            }
            else if regex[index+index_jump + 1] == '(' {
                let mut paren_index = index + 3; //start checking for closing paren in next index
                let mut paren_layer = 0; //in case there are nested parens
                let mut cont = true; //for continuing until the closing paren is found
                while cont {
                    //find the matching paren index
                    if regex[paren_index] == '(' {
                        paren_layer += 1; //enter a nested paren
                    }
                    else if regex[paren_index] == ')' && paren_layer == 0 {
                        cont = false; //found the closing paren
                        paren_index -= 1; //to account for the addition at the end of this loop
                    }
                    else if regex[paren_index] == ')' && paren_layer > 0 {
                        paren_layer -= 1; //wrong closing paren, exit nested paren
                    }
                    paren_index += 1; //check next index
                }
                index_jump = paren_index - index + 1; //set it to the first char inside the paren statement
                if paren_index + 1 < regex.len() && regex[paren_index + 1] == '*' {
                    paren_star_plus(index + index_jump + 2, bookmark, paren_index, regex, state, table, accept, "S");
                }
                else if paren_index + 1 < regex.len() && regex[paren_index + 1] == '+' {
                    paren_star_plus(index + index_jump + 2, bookmark, paren_index, regex, state, table, accept, "P");
                }
                else {
                    paren_add(index + index_jump + 2, bookmark, paren_index, regex, state, table, accept);
                }
            }
            else {
                // element is only added
                *accept = acc_states.clone();
                *state = st_holder;
                right_or = regex[index + index_jump];
                if use_next_state {
                    add_to(&right_or, *bookmark, table, accept, next_state);
                    acc_holder.push(next_state.to_string());
                    next_state += 1;
                }
                else {
                    add_or(&right_or, bookmark, table, accept);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                }
                index_jump += 2;
            }

        } else {cont = false;}
    }

    //check that there is a right side of the or, and that it has valid input
    if (index+index_jump) >= regex.len() || !invalid_next('|', &regex[index+index_jump]){
        //if the next character is invalid throw an error
        writeln!(std::io::stderr(), "Invalid Input").unwrap();
        std::process::exit(1);
    }

    //Final statement in the or
    if index + index_jump + 1 < regex.len() && regex[index + index_jump + 1] == '*' {
        //final element is starred
        *accept = acc_states.clone();
        right_or = regex[index + index_jump];
        acc_holder.push(bookmark.to_string());
        if use_next_state {
            *state = st_holder;
            acc_holder.push(bookmark.to_string());
            add_to(&right_or, *bookmark, table, accept, next_state);
            *state = next_state;
            star(&right_or, state, table, &mut empty);
            *state = next_state;
        }
        else {
            next_state = *state + 1;
            *state = st_holder;
            acc_holder.push(bookmark.to_string());
            add_to(&right_or, *bookmark, table, accept, next_state);
            *state = next_state;
            star(&right_or, state, table, &mut empty);
            *state = next_state;
        }
        //acc_holder.push(state.to_string());
        index_jump += 2;
    }
    else if index + index_jump + 1 < regex.len() && regex[index + index_jump + 1] == '+' {
        //final element is plussed
        *accept = acc_states.clone();
        acc_holder.push(state.to_string());
        right_or = regex[index + index_jump];
        if use_next_state {
            *state = st_holder;
            add_to(&right_or, *bookmark, table, accept, next_state);
            *state = next_state;
            star(&right_or, state, table, &mut empty);
            *state = next_state;
        }
        else {
            next_state = *state + 1;
            *state = st_holder;
            add_to(&right_or, *bookmark, table, accept, next_state);
            *state = next_state;
            star(&right_or, state, table, &mut empty);
            *state = next_state;
        }
        //acc_holder.push(state.to_string());
        index_jump += 2;
    }
    else {
        //final element is only added
        *accept = acc_states.clone();
        *state = st_holder;
        right_or = regex[index + index_jump];
        if use_next_state {
            add_to(&right_or, *bookmark, table, accept, next_state);
            //acc_holder.push(next_state.to_string());
            *state = next_state;
        }
        else {
            add_or(&right_or, bookmark, table, accept);
            table.remove(table.len()-1 as usize); //remove uneeded table row
        }
        index_jump += 1;
    }

    //Filter out repeat states
    //acc_states = Vec::<String>::new();
    for st in acc_holder {
        if !acc_states.contains(&st.to_string()) {
            acc_states.push(st.to_string());
        }
    }

    *accept = acc_states;
    index + index_jump
}

/// A function to add a plain parenthetical statement to the transition table (the parens can be seen as useless in this situation ex: a(bc)d )
/// - Input: Starting index, ending index of the paren statement, regex, current state as a mut, mut transition table, and the accept states as a mut
/// - Output: None, the mut values are changed as needed
fn paren_add(index: usize, bookmark: &mut u32, p_index: usize, regex: &Vec<char>, state: &mut u32, table: &mut Vec<Vec<String>>, accept: &mut Vec<String>){
    let mut i = index;
    let mut special = 'N';

    while i < p_index {
        if i + 3 < p_index && regex[i + 3] == '|' || i + 4 < p_index && regex[i + 4] == '|' || i + 5 < p_index && regex[i + 5] == '|' {
            //statement is x|y|z or x*|y|z or x*|y*|z etc.
            i += stacked_or(i, bookmark, regex, state, table, accept);
        }
        else if i + 1 < p_index && regex[i + 1] == '|' || i + 2 < p_index && regex[i + 2] == '|' {
            //statement is a single or z|a, z*|a, etc.
            if regex[i+1] == '*' {special = 'S';}
            else if regex[i+1] == '+' {special = 'P';}
            i += or(i, bookmark, regex, state, table, special, accept);
        }
        else if regex[i + 1] == '*' {
            //statement is starred
            star(&regex[i], state, table, accept);
            i += 2;
        }
        else if regex[i + 1] == '+' {
            //statement is plussed
            plus(&regex[i], state, table, accept);
            i += 2;
        }
        else {
            //statement is plain
            add(&regex[i], state, table, accept);
            i += 1;
        }
    }

}

/// A function to add a starred parenthetical statement to the transition table
/// - Input: Starting index, ending index of the paren statement, regex, current state as a mut, mut transition table, and the accept states as a mut
/// - Output: None, the mut values are changed as needed
fn paren_star_plus(index: usize, bookmark: &mut u32, p_index: usize, regex: &Vec<char>, state: &mut u32, table: &mut Vec<Vec<String>>, accept: &mut Vec<String>, special: &str){
    let mut i = index;
    let mut sp = 'N'; //for or statements, serves as the special for that input
    let state_holder = *state;
    let mut acc_holder = accept.clone();
    if special != "Plus" {
        //The origional state is still an accept state in this case
        acc_holder.push(state_holder.to_string());
    }
    let mut cont = true;
    let mut add_final = true; //for adding the final element outside of special circumstances

    if p_index - i <= 1{
        star(&regex[i], state, table, accept);
    }
    else {
        if regex[i+2] == '|' {
            if regex[i+1] == '*' {sp = 'S';}
            else if regex[i+1] == '+' {sp = 'P';}
            i = or(i, bookmark, regex, state, table, sp, accept);
        }
        else if regex[i + 1] == '*' {//first statement is starred (some probs w/ ()+)
            plus(&regex[i], state, table, accept);
            i += 2;
            if p_index - i <= 2 && regex[p_index - 1] == '*' {
                cont = false;
                add_final = false;
                acc_holder.push(state.to_string());
                if special == "Plus" {
                    acc_holder.push(state_holder.to_string());
                    add(&regex[i], state, table, accept);
                    add_to(&regex[i], state_holder, table, accept, *state);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    add_to(&regex[i-2], *state, table, accept, state_holder + 1);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    star(&regex[i], state, table, accept);
                }
                else {
                    add_to(&regex[i], *state, table, accept, state_holder);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    star(&regex[i], state, table, accept);
                }
                //*state = state_holder;
            }
            else if p_index - i <= 2 && regex[p_index-1] == '+' || p_index - i <= 1 {
                cont = false;
                add_final = false;
                if special == "Plus" {
                    add(&regex[i], state, table, accept);
                    add_to(&regex[i], state_holder, table, accept, *state);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    add_to(&regex[i-2], *state, table, accept, state_holder + 1);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    star(&regex[i], state, table, accept);
                }else{
                    add_to(&regex[i], *state, table, accept, state_holder);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    star(&regex[i], state, table, accept);
                    //*state = state_holder;
                }
            }
            else {
                if regex[i + 1] == '*' {
                    //statement is starred
                    star(&regex[i], state, table, accept);
                    i += 2;
                    if p_index - i <= 2 && (regex[p_index - 1] == '*' || regex[p_index-1] == '+') {
                        cont = false;
                    }
                    else if p_index - i <= 1{
                        cont = false;
                        acc_holder.push(state.to_string());
                    }
                    else {cont = i<p_index;}
                }
                else if regex[i + 1] == '+' {
                    //statement is plussed
                    plus(&regex[i], state, table, accept);
                    add_to(&regex[i], state_holder, table, accept, *state);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    i += 2;
                    if p_index - i <= 2 && (regex[p_index - 1] == '*' || regex[p_index-1] == '+') {
                        cont = false;
                    }
                    else if p_index - i <= 1{
                        cont = false;
                    }
                    else {cont = i<p_index;}
                }
                else {
                    //statement is plain
                    add(&regex[i], state, table, accept);
                    add_to(&regex[i], state_holder, table, accept, *state);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                    i += 1;
                    if p_index - i <= 2 && (regex[p_index - 1] == '*' || regex[p_index-1] == '+') {
                        cont = false;
                    }
                    else if p_index - i <= 1{
                        cont = false;
                    }
                    else {cont = i<p_index;}
                }
            }
        }

        //Loop through central things
        while cont {
            if regex[i + 1] == '*' {
                //statement is starred
                star(&regex[i], state, table, accept);
                i += 2;
                if p_index - i <= 2 && (regex[p_index - 1] == '*' || regex[p_index-1] == '+') {
                    cont = false;
                }
                else if p_index - i <= 1{
                    cont = false;
                }
                else {cont = i<p_index;}
            }
            else if regex[i + 1] == '+' {
                //statement is plussed
                plus(&regex[i], state, table, accept);
                i += 2;
                if p_index - i <= 2 && (regex[p_index - 1] == '*' || regex[p_index-1] == '+') {
                    cont = false;
                }
                else if p_index - i <= 1{
                    cont = false;
                }
                else {cont = i<p_index;}
            }
            else {
                //statement is plain
                add(&regex[i], state, table, accept);
                i += 1;
                if p_index - i <= 2 && (regex[p_index - 1] == '*' || regex[p_index-1] == '+') {
                    cont = false;
                }
                else if p_index - i <= 1{
                    cont = false;
                }
                else {cont = i<p_index;}
            }
        }
        
        //add final element, if it wasn't added already
        if add_final {
            if regex[p_index - 1] == '*' {
                // Last symbol is starred
                star(&regex[i], state, table, accept);
                acc_holder.push(state.to_string());
                if regex[index+1] == '*' {
                    add_to(&regex[index], *state, table, accept, state_holder + 1);
                    add_to(&regex[index+2], *state, table, accept, state_holder+2);
                    //*state = state_holder;
                }
                else{
                    add_to(&regex[index], *state, table, accept, state_holder + 1);
                    //table.remove(table.len()-1 as usize); //remove uneeded table row
                    //*state = state_holder;
                }
                
            }
            else if regex[p_index-1] == '+' {
                // Last symbol is plussed 
                plus(&regex[i], state, table, accept);
                acc_holder.push(state.to_string());
                add_to(&regex[index], *state, table, accept, state_holder + 1);
                table.remove(table.len()-1 as usize); //remove uneeded table row
                //*state = state_holder;
            }
            else if i < p_index{
                //last symbol is plain
                if special == "Plus" {
                    add(&regex[i], state, table, accept);
                    add_to(&regex[index], *state, table, accept, state_holder + 1);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                }
                else {
                    add_to(&regex[i], *state, table, accept, state_holder);
                    table.remove(table.len()-1 as usize); //remove uneeded table row
                }
                //*state = state_holder;
            }
        }
        for st in acc_holder {
            if !accept.contains(&st.to_string()) {
                accept.push(st.to_string());
            }
        }
    }
}

/// A function to add an or expression that contains a parenthetical statent to the transition table
/// - Input: Starting index, regex, current state as a mut value, the transition table, character for marking specific symbols before the |, and vector of accept states
/// - Output: New index to jump to (the index after the last char of the or statement)
fn paren_or(index: usize, bookmark: &mut u32, p_index: usize, regex: &Vec<char>, state: &mut u32, table: &mut Vec<Vec<String>>, accept: &mut Vec<String>) -> usize{
    let state_holder = *state;
    let end_state;
    let mut acc_holder = accept.clone();
    let mut working_index;
    if regex[p_index + 1] == '*' {
        paren_star_plus(index, bookmark, p_index, regex, state, table, accept, "S");
    }
    else if regex[p_index + 1] == '+' {
        paren_star_plus(index, bookmark, p_index, regex, state, table, accept, "P");
    }
    else {
        paren_add(index, bookmark, p_index, regex, state, table, accept);
    }
    end_state = *state;
    working_index = p_index + 2;
    let mut loop_var = true; //for staying in the while loop

    while loop_var {
        if working_index < regex.len() && regex[working_index] == '(' {
            let mut paren_index = index + 1; //start checking for closing paren in next index
            let mut paren_layer = 0; //in case there are nested parens
            let mut cont = true; //for continuing until the closing paren is found
            while cont {
                //find the matching paren index
                if regex[paren_index] == '(' {
                    paren_layer += 1; //enter a nested paren
                }
                else if regex[paren_index] == ')' && paren_layer == 0 {
                    cont = false; //found the closing paren
                    paren_index -= 1; //to account for the addition at the end of this loop
                }
                else if regex[paren_index] == ')' && paren_layer > 0 {
                    paren_layer -= 1; //wrong closing paren, exit nested paren
                }
                paren_index += 1; //check next index
            }

            if regex[paren_index + 1] == '*' {
                paren_star_plus(working_index, bookmark, paren_index, regex, state, table, accept, "S");
            }
            else if regex[paren_index + 1] == '+' {
                paren_star_plus(working_index, bookmark, paren_index, regex, state, table, accept, "P");
            }
            else {
                paren_add(working_index, bookmark, paren_index, regex, state, table, accept);
            }

            working_index = paren_index + 1;
            if regex[working_index] != '|' { loop_var = false;}
        }
        else if working_index+1 < regex.len() && regex[working_index + 1] == '*' {
            acc_holder.push(state_holder.to_string());
            acc_holder.push(end_state.to_string());
            add_to(&regex[working_index], state_holder, table, accept, end_state+1);
            table.remove(table.len()-1 as usize); //remove uneeded table row
            accept.push((end_state+1).to_string());
            working_index += 2;
            if regex[working_index] != '|' { loop_var = false;}
        }
        else if working_index+1 < regex.len() && regex[working_index + 1] == '+' {
            acc_holder.push(end_state.to_string());
            add_to(&regex[working_index], state_holder, table, accept, end_state+1);
            table.remove(table.len()-1 as usize); //remove uneeded table row
            working_index += 2;
            if regex[working_index] != '|' { loop_var = false;}
        }
        else if working_index >= regex.len() {
            loop_var = false;
        }
        else {
            add_to(&regex[working_index], state_holder, table, accept, end_state);
            table.remove(table.len()-1 as usize); //remove uneeded table row
            working_index += 1;
            if working_index < regex.len() && regex[working_index] != '|' { loop_var = false;}
        }
    }

    *accept = acc_holder;
    *state = end_state;
    working_index
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

/// For printing the transition table as a state diagram to stdout.txt
/// - Input: Transition table
/// - Output: None
fn print_state_diagram(table: &Vec<Vec<String>>){
    let mut output = File::create("stdout.txt").expect("Unable to create file");
    //opening lines
    output.write(b"diagraph {\n\n\tnode [shape=point]; start;\n").expect("Unable to write to file");

    //insert end states for the double circle label
    output.write(b"\tnode [shape=doublecircle]; ").expect("Unable to write to file");
    let mut marker = 0;
    for state in &table[table.len()-1] { //loop through the accept state row
        if state == "X" {
            //This is just the marker for the accept state row, ignore
        }
        else {
            if marker == 0 {
                marker += 1;
            }
            else {
                output.write(b", ").expect("Unable to write to file");
            }
            output.write(state.as_bytes()).expect("Unable to write to file");
        }
    }
    output.write(b";\n").expect("Unable to write to file");

    //transition to next section of file
    output.write(b"\tnode [shape=circle];\n\n\tstart -> 0;\n").expect("Unable to write to file");

    //translate state diagram to transitions on a graph
    let mut row_num = 0;
    let mut i;
    for row in table{
        i = 0;
        if row_num == table.len()-1 {
            //This is the accept state row, should not be used here
            continue;
        }
        for transition in row {
            if transition != " " {
                output.write(b"\t").expect("Unable to write to file");
                output.write(row_num.to_string().as_bytes()).expect("Unable to write to file");
                output.write(b" -> ").expect("Unable to write to file");
                output.write(table[row_num][i].as_bytes()).expect("Unable to write to file");
                output.write(b" [label=\"").expect("Unable to write to file");
                output.write(SIGMA[i].to_string().as_bytes()).expect("Unable to write to file");
                output.write(b"\"];\n").expect("Unable to write to file");
            }
            i += 1;
        }
        row_num += 1;
    }

    //end and close file
    output.write(b"\n}").expect("Unable to write to file");
}

#[test]
fn test_scan_regex(){
    let mut solution = vec!['a','*','b','|','(','c','d','e',')','|','(', 'e','f','g',')'];
    assert_eq!(scan_regex("a*b|{cde}|(efg)"), solution);

    solution = vec!['!', '@', 'a', 'b', 'z', '8', '+'];
    assert_eq!(scan_regex("\\w\\dabz8+"), solution);
}

#[test]
fn test_invalid_next(){
    assert_eq!(invalid_next('(', &'*'), false);
    assert_eq!(invalid_next('|', &'b'), true);
    assert_eq!(invalid_next('*', &'*'), false);
    assert_eq!(invalid_next('+', &'a'), true);
}

#[test]
fn test_parse_regex(){
    let scanned_regex = vec!['a','*','b'];
    let mut table = Vec::<Vec::<String>>::new();
    let mut row = vec![String::from("0"), String::from("1"), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" ")]; 
    table.push(row);
    row = vec![String::from(" "), String::from(String::from(" ")), String::from(String::from(" ")), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" ")];
    table.push(row);
    row = vec![String::from("X"), String::from("1")];
    table.push(row);
    assert_eq!(parse_regex(scanned_regex), table);
}

#[test]
fn test_check_string(){
    let mut table = Vec::<Vec::<String>>::new();
    let mut row = vec![String::from("0"), String::from("1"), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" ")]; 
    table.push(row);
    row = vec![String::from(" "), String::from(String::from(" ")), String::from(String::from(" ")), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" "), String::from(" ")];
    table.push(row);
    row = vec![String::from("X"), String::from("1")];
    table.push(row);

    assert_eq!(check_string("aaaab", &table), true);
    assert_eq!(check_string("b", &table), true);
    assert_eq!(check_string("bbbbb", &table), false);
    assert_eq!(check_string("", &table), false);
}

#[test]
fn test_new_table_row(){
    let mut empty = Vec::<String>::new();
    for _char in SIGMA.iter() {
        empty.push(" ".to_string());
    }
    assert_eq!(new_table_row(), empty);
}