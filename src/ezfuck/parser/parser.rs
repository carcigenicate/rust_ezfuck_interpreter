use std::collections::{HashMap};
use std::fmt::{Display, Formatter};
use std::iter::Scan;
use std::string::ToString;
use strum_macros::Display;

// const COMMAND_SYMBOLS: [&str; 12] = ["+", "-", "*", "/", "<", ">", "[", "]", "^", ",", ".", "!"];
const COMMAND_SYMBOLS: &str = "+-*/<>[]^.,!@";
const VALUELESS_COMMAND_SYMBOLS: &str = "[],.!";
const NUMERIC_LITERAL_SYMBOLS: &str  = "1234567890";
const CURRENT_CELL_SYMBOLS: &str  = "V";
const VALUE_SYMBOLS: &str = "1234567890V";

fn is_command_lexeme(lexeme: &Lexeme) -> bool {
    let first_symbol = lexeme.symbols.chars().next().unwrap();
    return lexeme.symbols.len() == 1 && COMMAND_SYMBOLS.contains(first_symbol);
}

fn is_numeric_literal_lexeme(lexeme: &Lexeme) -> bool {
    let first_symbol = lexeme.symbols.chars().next().unwrap();
    return NUMERIC_LITERAL_SYMBOLS.contains(first_symbol);
}

fn is_current_cell_lexeme(lexeme: &Lexeme) -> bool {
    let first_symbol = lexeme.symbols.chars().next().unwrap();
    return CURRENT_CELL_SYMBOLS.contains(first_symbol);
}

#[derive(Debug, Eq, PartialEq)]
struct Lexeme {
    start: usize,
    symbols: String,
}

struct Scanner {
    code: Vec<char>,
    scan_i: usize,
    lexemes: Vec<Lexeme>,
}

impl Scanner {
    pub fn new(code: Vec<char>) -> Self {
        return Self {
            code: code,
            scan_i: 0,
            lexemes: vec![],
        };
    }

    pub fn next_symbol(self: &Self) -> Option<char> {
        return self.code.get(self.scan_i).map(| sym | *sym);
    }

    pub fn advance(self: &mut Self) {
        self.scan_i += 1;
    }

    pub fn is_exhausted(self: &Self) -> bool {
        return self.scan_i >= self.code.len();
    }

    pub fn next_is_command(self: &Self) -> bool {
        return self.next_symbol().map_or(false, | next_sym | COMMAND_SYMBOLS.contains(next_sym));
    }

    pub fn next_is_numeric_literal(self: &Self) -> bool {
        return self.next_symbol().map_or(false, | next_sym | NUMERIC_LITERAL_SYMBOLS.contains(next_sym));
    }

    pub fn next_is_current_cell(self: &Self) -> bool {
        return self.next_symbol().map_or(false, | next_sym | CURRENT_CELL_SYMBOLS.contains(next_sym));
    }
    pub fn consume_single_symbol(self: &mut Self) -> () {
        match self.next_symbol() {
            Some(symbol) => {
                let lexeme = Lexeme { start: self.scan_i, symbols: symbol.to_string() };
                self.lexemes.push(lexeme);
                self.advance();
            }
            None => {}
        }
    }

    pub fn consume_integer_literal(self: &mut Self) -> () {
        let start_i = self.scan_i;

        let mut symbols = String::new();
        loop {
            match self.next_symbol() {
                Some(chr) => {
                    if NUMERIC_LITERAL_SYMBOLS.contains(chr) {
                        symbols.push(chr);
                        self.advance();
                    } else {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }

        let lexeme = Lexeme { start: start_i, symbols: symbols.to_string() };
        self.lexemes.push(lexeme);
    }
}

fn scan_code(code: &Vec<char>) -> Vec<Lexeme> {
    let mut scanner = Scanner::new(code.clone());

    while scanner.is_exhausted() == false {
        if scanner.next_is_command() || scanner.next_is_current_cell() {
            scanner.consume_single_symbol();
        } else if scanner.next_is_numeric_literal() {
            scanner.consume_integer_literal();
        } else {
            scanner.advance();
        }
    }

    return scanner.lexemes;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum TokenKind {
    Command { value: char },
    IntegerLiteral { value: u8 },  // TODO: How big of integer?
    CurrentCellReference,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Token {
    kind: TokenKind,
    source_start: usize,
}

impl Token {
    fn from_parts(kind: TokenKind, source_start: usize) -> Self {
        return Self {
            kind: kind,
            source_start: source_start
        };
    }
    pub fn from_lexeme(lexeme: Lexeme) -> Self {
        if is_command_lexeme(&lexeme) {
            let first_char = lexeme.symbols.chars().next().unwrap();
            return Token::from_parts(TokenKind::Command { value: first_char }, lexeme.start);
        } else if is_numeric_literal_lexeme(&lexeme) {
            let parsed: u8 = lexeme.symbols.parse().expect(format!("Could not parse {} as integer literal", lexeme.symbols).as_str());
            return Token::from_parts(TokenKind::IntegerLiteral { value: parsed }, lexeme.start);
        } else if is_current_cell_lexeme(&lexeme) {
            return Token::from_parts(TokenKind::CurrentCellReference, lexeme.start);
        } else {
            panic!("Unknown lexeme: {}", lexeme.symbols);
        }
    }
}

fn evaluate_lexemes(lexemes: Vec<Lexeme>) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    for lexeme in lexemes {
        let token = Token::from_lexeme(lexeme);
        tokens.push(token);
    }

    return tokens;
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum Value {
    CurrentCell,
    Number(u8),
}

impl Value {
    pub fn determine_value(self, current_cell_value: u8) -> u8 {
        return match self {
            Value::Number(n) => n,
            Value::CurrentCell => current_cell_value,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Command {
    symbol: char,
    value: Option<Value>,
    source_start: usize,
}

impl Command {
    pub fn has_value(self: &Self) -> bool {
        return self.value.is_some();
    }

    pub fn get_defaulted_value(self: &Self) -> Value {
        return self.value.unwrap_or(Value::Number(1));
    }
}

fn parse_tokens(tokens: Vec<Token>) -> Vec<Command> {
    let mut commands: Vec<Command> = vec![];
    let mut command: Option<Command> = None;
    let mut last_source_start = 0;
    for Token { kind, source_start} in tokens {
        match kind {
            TokenKind::Command { value } => {
                if let Some(existing_command) = command {
                    commands.push(existing_command);
                }

                command = Some(Command { symbol: value, value: None, source_start: source_start });
            }
            TokenKind::IntegerLiteral { value } => {
                match command {
                    Some(mut existing_command) => {
                        existing_command.value = Some(Value::Number(value));
                        commands.push(existing_command);
                        command = None;
                    }
                    None => {
                        panic!("Integer literal {value} must come after a command.")
                    }
                }
            }
            TokenKind::CurrentCellReference => {
                match command {
                    Some(mut existing_command) => {
                        existing_command.value = Some(Value::CurrentCell);
                        commands.push(existing_command);
                        command = None;
                    }
                    None => {
                        panic!("\"V\" must come after a command.")
                    }
                }
            }
        }

        last_source_start = source_start;
    }

    if let Some(existing_command) = command {
        commands.push(existing_command);
    }

    return commands;
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum EqualityOperator {
    NotEqual,
    Equal,
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum MathOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum CellMoveOperator {
    Left,
    Right,
    Set,
}

#[derive(Copy, Clone, Debug, Display, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    ApplyOperatorToCell { operator: MathOperator, value: Value },
    ApplyOperatorToCellPtr { operator: CellMoveOperator, value: Value },
    JumpToIf { position: usize, operator: EqualityOperator, match_value: u8 },
    PrintOut,
    ReadIn,
    SetCell { value: Value },
    Breakpoint,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WrappedInstruction {
    pub instruction: Instruction,
    pub source_start: usize,
}

fn find_loop_indices(commands: &Vec<Command>) -> (HashMap<usize, usize>, HashMap<usize, usize>) {
    let mut start_to_end: HashMap<usize, usize> = HashMap::new();
    let mut end_to_start: HashMap<usize, usize> = HashMap::new();

    let mut loop_start_stack = vec![];

    for (i, token) in commands.iter().enumerate() {
        let symbol = token.symbol;
        if symbol == '[' {
            loop_start_stack.push(i);
        } else if symbol == ']' {
            let start_i = match loop_start_stack.pop() {
                Some(start_i) => start_i,
                None => panic!("] missing a matching [ at {i}"),
            };

            start_to_end.insert(start_i, i);
            end_to_start.insert(i, start_i);
        }
    }

    if loop_start_stack.len() > 0 {
        panic!("[ missing a matching ]: {loop_start_stack:?}");
    }

    return (start_to_end, end_to_start);
}

fn assert_valueless(command: Command) {
    if command.has_value() {
        panic!("Command {:?} cannot be given a value. Given {:?}.", command.symbol, command.value);
    }
}

fn compile_commands_to_intermediate(commands: Vec<Command>, allow_debugging: bool) -> Vec<WrappedInstruction> {
    let mut instructions: Vec<WrappedInstruction> = Vec::new();

    let (start_to_end, end_to_start) = find_loop_indices(&commands);
    for (i, command) in commands.iter().enumerate() {
        if VALUELESS_COMMAND_SYMBOLS.contains(command.symbol) {
            assert_valueless(*command);
        }

        let defaulted_value = command.get_defaulted_value();
        let instruction = match command.symbol {
            '+' => Some(Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: defaulted_value }),
            '-' => Some(Instruction::ApplyOperatorToCell { operator: MathOperator::Subtraction, value: defaulted_value }),
            '*' => Some(Instruction::ApplyOperatorToCell { operator: MathOperator::Multiplication, value: defaulted_value }),
            '/' => Some(Instruction::ApplyOperatorToCell { operator: MathOperator::Division, value: defaulted_value }),
            '<' => Some(Instruction::ApplyOperatorToCellPtr { operator: CellMoveOperator::Left, value: defaulted_value }),
            '>' => Some(Instruction::ApplyOperatorToCellPtr { operator: CellMoveOperator::Right, value: defaulted_value }),
            '@' => Some(Instruction::ApplyOperatorToCellPtr { operator: CellMoveOperator::Set, value: defaulted_value }),
            '[' => {
                let end_i = start_to_end.get(&i).unwrap();
                Some(Instruction::JumpToIf { position: *end_i, operator: EqualityOperator::Equal, match_value: 0 })
            },
            ']' => {
                let start_i = end_to_start.get(&i).unwrap();
                Some(Instruction::JumpToIf { position: *start_i, operator: EqualityOperator::NotEqual, match_value: 0 })
            },
            '.' => Some(Instruction::PrintOut),
            ',' => Some(Instruction::ReadIn),
            '^' => Some(Instruction::SetCell { value: defaulted_value }),
            '!' => if allow_debugging { Some(Instruction::Breakpoint) } else { None },
            _ => None,
        };

        match instruction {
            Some(inst) => {
                let wrapped_instruction = WrappedInstruction { instruction: inst, source_start: command.source_start };
                instructions.push(wrapped_instruction)
            },
            None => (),
        }
    }

    return instructions;
}

pub fn compile_to_intermediate(code: &str, allow_debugging: bool) -> Vec<WrappedInstruction> {
    let code_vec: Vec<char> = code.chars().collect();
    let lexemes = scan_code(&code_vec);
    let tokens = evaluate_lexemes(lexemes);
    let commands = parse_tokens(tokens);
    let instructions = compile_commands_to_intermediate(commands, allow_debugging);
    return instructions;
}

#[cfg(test)]
mod tests {
    mod scan_code {
        use super::super::*;

        #[test]
        fn it_should_produce_the_correct_lexemes() {
            let code = vec!['+', ' ', 'V', '1', '+', '2', '3', '+', '4', ' ', 'F'];
            let lexemes = scan_code(&code);

            assert_eq!(lexemes, vec![
                Lexeme { start: 0, symbols: String::from("+") },
                Lexeme { start: 2, symbols: String::from("V") },
                Lexeme { start: 3, symbols: String::from("1") },
                Lexeme { start: 4, symbols: String::from("+") },
                Lexeme { start: 5, symbols: String::from("23") },
                Lexeme { start: 7, symbols: String::from("+") },
                Lexeme { start: 8, symbols: String::from("4") },
            ]);
        }

        #[test]
        fn it_should_split_consecutive_commands_in_a_row_as_different_lexemes() {
            let code = vec!['+', '+', '-', '-'];
            let lexemes = scan_code(&code);

            assert_eq!(lexemes, vec![
                Lexeme { start: 0, symbols: String::from("+") },
                Lexeme { start: 1, symbols: String::from("+") },
                Lexeme { start: 2, symbols: String::from("-") },
                Lexeme { start: 3, symbols: String::from("-") },
            ]);
        }
    }

    mod evaluate_lexemes {
        use super::super::*;

        #[test]
        fn it_should_produce_the_correct_tokens() {
            let lexemes = vec![String::from("+"), String::from("123"), String::from("-"), String::from("V")];
            let lexemes = vec![
                Lexeme { start: 0, symbols: String::from("+") },
                Lexeme { start: 1, symbols: String::from("123") },
                Lexeme { start: 4, symbols: String::from("-") },
                Lexeme { start: 5, symbols: String::from("V") },
            ];

            let tokens = evaluate_lexemes(lexemes);

            assert_eq!(tokens, vec![
                Token::from_parts(TokenKind::Command { value: '+' }, 0),
                Token::from_parts(TokenKind::IntegerLiteral { value: 123 }, 1),
                Token::from_parts(TokenKind::Command { value: '-' }, 4),
                Token::from_parts(TokenKind::CurrentCellReference, 5),
            ]);
        }

        #[test]
        #[should_panic]
        fn it_should_panic_if_an_unknown_lexeme_is_passed() {
            let lexemes = vec![Lexeme { start: 0, symbols: String::from("|") }];
            evaluate_lexemes(lexemes);
        }
    }

    mod parse_tokens {
        use super::super::*;

        #[test]
        fn it_should_produce_the_correct_commands() {
            let tokens = vec![
                Token::from_parts(TokenKind::Command { value: '+' }, 0),
                Token::from_parts(TokenKind::Command { value: '+' }, 1),
                Token::from_parts(TokenKind::IntegerLiteral { value: 123 }, 2),
                Token::from_parts(TokenKind::Command { value: '-' }, 5),
                Token::from_parts(TokenKind::CurrentCellReference, 6),
            ];
            let commands = parse_tokens(tokens);

            assert_eq!(commands, vec![
                Command { symbol: '+', value: None, source_start: 0 },
                Command { symbol: '+', value: Some(Value::Number(123)), source_start: 1 },
                Command { symbol: '-', value: Some(Value::CurrentCell), source_start: 5 },
            ]);
        }
    }

    mod compile_to_intermediate {
        use super::super::*;

        #[test]
        fn it_should_ignore_invalid_characters() {
            let code = "+None of this should be considered*";
            let wrapped_instructions = compile_to_intermediate(code, false);
            let instructions: Vec<Instruction> = wrapped_instructions.iter().map(|instruction| instruction.instruction).collect();

            assert_eq!(instructions.len(), 2);

            assert_eq!(instructions[0], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(1) });
            assert_eq!(instructions[1], Instruction::ApplyOperatorToCell { operator: MathOperator::Multiplication, value: Value::Number(1) });
        }

        #[test]
        fn it_should_produce_the_correct_instruction_for_each_token() {
            let code = "[]+-*/<>@.,^";
            let wrapped_instructions = compile_to_intermediate(code, false);
            let instructions: Vec<Instruction> = wrapped_instructions.iter().map(|instruction| instruction.instruction).collect();

            assert_eq!(instructions.len(), 12);

            assert_eq!(instructions[0], Instruction::JumpToIf { position: 1, operator: EqualityOperator::Equal, match_value: 0 });
            assert_eq!(instructions[1], Instruction::JumpToIf { position: 0, operator: EqualityOperator::NotEqual, match_value: 0 });

            assert_eq!(instructions[2], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(1) });
            assert_eq!(instructions[3], Instruction::ApplyOperatorToCell { operator: MathOperator::Subtraction, value: Value::Number(1) });
            assert_eq!(instructions[4], Instruction::ApplyOperatorToCell { operator: MathOperator::Multiplication, value: Value::Number(1) });
            assert_eq!(instructions[5], Instruction::ApplyOperatorToCell { operator: MathOperator::Division, value: Value::Number(1) });
            assert_eq!(instructions[6], Instruction::ApplyOperatorToCellPtr { operator: CellMoveOperator::Left, value: Value::Number(1) });
            assert_eq!(instructions[7], Instruction::ApplyOperatorToCellPtr { operator: CellMoveOperator::Right, value: Value::Number(1) });
            assert_eq!(instructions[8], Instruction::ApplyOperatorToCellPtr { operator: CellMoveOperator::Set, value: Value::Number(1) });
            assert_eq!(instructions[9], Instruction::PrintOut);
            assert_eq!(instructions[10], Instruction::ReadIn);
        }

        #[test]
        fn it_should_properly_read_instruction_values_and_default_missing_ones_to_one() {
            let code = "++1+2+3+40+200";
            let wrapped_instructions = compile_to_intermediate(code, false);
            let instructions: Vec<Instruction> = wrapped_instructions.iter().map(|instruction| instruction.instruction).collect();

            assert_eq!(instructions.len(), 6);

            assert_eq!(instructions[0], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(1) });
            assert_eq!(instructions[1], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(1) });
            assert_eq!(instructions[2], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(2) });
            assert_eq!(instructions[3], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(3) });
            assert_eq!(instructions[4], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(40) });
            assert_eq!(instructions[5], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::Number(200) });
        }

        #[test]
        fn it_should_properly_add_insertion_values() {
            let code = "+V";
            let wrapped_instructions = compile_to_intermediate(code, false);
            let instructions: Vec<Instruction> = wrapped_instructions.iter().map(|instruction| instruction.instruction).collect();

            assert_eq!(instructions.len(), 1);
            assert_eq!(instructions[0], Instruction::ApplyOperatorToCell { operator: MathOperator::Addition, value: Value::CurrentCell });
        }

        #[test]
        #[should_panic]
        fn it_should_panic_on_mismatched_start_brace() {
            let code = "+[-";
            compile_to_intermediate(code, false);
        }

        #[test]
        #[should_panic]
        fn it_should_panic_on_mismatched_end_brace() {
            let code = "+]-";
            compile_to_intermediate(code, false);
        }
    }


}