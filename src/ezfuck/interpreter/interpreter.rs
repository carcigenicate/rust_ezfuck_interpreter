use std::cmp::min;
use std::io;
use std::io::{BufRead, Read, Write};
use crate::ezfuck::parser::parser::{Instruction, EqualityOperator, MathOperator, Value, Direction, compile_to_intermediate, CellMoveOperator, WrappedInstruction};
use crate::ezfuck::repl::cell_repr::{produce_cells_repr};

#[derive(Clone, Debug)]
pub struct ExecutionState {
    pub cells: Vec<u8>,
    pub cell_ptr: usize,
    pub instruction_ptr: usize,
    pub is_debugging: bool,
}

impl ExecutionState {
    pub fn new() -> ExecutionState {
        return ExecutionState {
            cell_ptr: 0,
            instruction_ptr: 0,
            cells: vec![0],
            is_debugging: false,
        };
    }

    pub fn set_instruction_pointer(self: &mut Self, ptr: usize) {
        self.instruction_ptr = ptr;
    }

    pub fn get_current_cell(self: &Self) -> u8 {
        return self.cells[self.cell_ptr];
    }

    pub fn set_current_cell(self: &mut Self, new_value: u8) -> () {
        self.cells[self.cell_ptr] = new_value;
    }

    pub fn set_cell_pointer(self: &mut Self, ptr: usize) {
        self.ensure_cell(ptr);
        self.cell_ptr = ptr;
    }

    fn ensure_cell(self: &mut Self, ptr: usize) -> () {
        let needed = (ptr as isize) - (self.cells.len() as isize) + 1;
        if needed > 0 {
            for _ in 0..needed {
                self.cells.push(0);
            }
        }
    }
}

fn apply_math_operator(current_cell_value: u8, operator: MathOperator, value: u8) -> u8 {
    return match operator {
        MathOperator::Addition => current_cell_value.wrapping_add(value),
        MathOperator::Subtraction => current_cell_value.wrapping_sub(value),
        MathOperator::Multiplication => current_cell_value.wrapping_mul(value),
        MathOperator::Division => current_cell_value.wrapping_div(value),
    }
}

fn apply_cell_ptr_operator(current_cell_ptr: usize, operator: CellMoveOperator, value: usize) -> usize {
    return match operator {
        CellMoveOperator::Left => current_cell_ptr.checked_sub(value).expect("Cell Pointer Underflowed"),
        CellMoveOperator::Right => current_cell_ptr.checked_add(value).expect("Cell Pointer Overflowed"),
        CellMoveOperator::Set => value,
    }
}

fn add_cell_ptr_value(current_cell_ptr: usize, ptr_offset: isize) -> usize {
    return match current_cell_ptr.checked_add_signed(ptr_offset) {
        Some(added) => added,
        None => panic!("Cell Pointer Became Negative!")
    };
}

fn print_value<W: Write>(out_stream: &mut W, cell: u8) {
    write!(out_stream, "{}", char::from(cell)).unwrap();
    io::stdout().flush().unwrap();
}

fn read_value<R: BufRead>(in_stream: &mut R) -> u8 {
    let mut input = [0; 1];
    in_stream.read_exact(&mut input).expect("Reading byte from stdin");
    return input[0];
}

pub fn interpret_instruction<R: BufRead, W: Write>(wrapped_instruction: WrappedInstruction, state: &mut ExecutionState, in_stream: &mut R, out_stream: &mut W, allow_debugging: bool) -> () {
    let WrappedInstruction { instruction, source_start } = wrapped_instruction;

    match instruction {
        Instruction::ApplyOperatorToCell { operator, value } => {
            let actual_value = value.determine_value(state.get_current_cell());
            let new_cell_value = apply_math_operator(state.get_current_cell(), operator, actual_value);
            state.set_current_cell(new_cell_value);
        }

        Instruction::ApplyOperatorToCellPtr { operator, value } => {
            let actual_value = value.determine_value(state.get_current_cell());
            let new_cell_ptr = apply_cell_ptr_operator(state.cell_ptr, operator, actual_value as usize);
            state.set_cell_pointer(new_cell_ptr);
        }

        Instruction::JumpToIf { position, operator, match_value } => {
            match operator {
                EqualityOperator::Equal => {
                    if state.get_current_cell() == match_value {
                        state.set_instruction_pointer(position);
                    }
                },
                EqualityOperator::NotEqual => {
                    if state.get_current_cell() != match_value {
                        state.set_instruction_pointer(position);
                    }
                }
            }
        }

        Instruction::PrintOut => {
            print_value(out_stream, state.get_current_cell());
        }

        Instruction::ReadIn => {
            let input = read_value(in_stream);
            state.set_current_cell(input);
        }

        Instruction::SetCell { value } => {
            let actual_value = value.determine_value(state.get_current_cell());
            state.set_current_cell(actual_value);
        }
        Instruction::Breakpoint => {
            if allow_debugging {
                state.is_debugging = true;
            }
        }
    }
}

pub fn interpret<R: BufRead, W: Write>(instructions: &Vec<WrappedInstruction>, state: &mut ExecutionState, in_stream: &mut R, out_stream: &mut W, allow_debugging: bool) -> () {
    while state.instruction_ptr < instructions.len() {
        if state.is_debugging {
            start_debugger(&instructions, state, in_stream, out_stream);
        } else {
            let current_instruction = instructions[state.instruction_ptr];
            interpret_instruction(current_instruction, state, in_stream, out_stream, allow_debugging);
        }

        state.instruction_ptr += 1;
    }
}

fn produce_instructions_repr(instructions: &Vec<WrappedInstruction>, instruction_ptr: usize, show_n_around: usize) -> String {
    let start_bound = instruction_ptr.checked_sub(show_n_around).unwrap_or(0);
    let end_bound = min(instruction_ptr + show_n_around, instructions.len() - 1);
    let relevant_instructions = &instructions[start_bound..=end_bound];

    let instruction_ptr_places = (instructions.len().ilog10() + 1) as usize;

    let mut repr = String::new();
    for (i, instruction) in relevant_instructions.into_iter().enumerate() {
        let current_instruction_ptr = start_bound + i;
        let marker = if instruction_ptr == current_instruction_ptr { "> " } else { "  " };
        repr.push_str(format!("{current_instruction_ptr:0instruction_ptr_places$} {marker}{:?}\n", instruction).as_str());
    }

    return repr;
}

fn start_debugger<R: BufRead, W: Write>(instructions: &Vec<WrappedInstruction>, state: &mut ExecutionState, in_stream: &mut R, out_stream: &mut W) -> () {
    writeln!(out_stream, "").unwrap();
    let cells_repr = produce_cells_repr(&state.cells, state.cell_ptr);
    out_stream.write(cells_repr.as_bytes()).unwrap();
    out_stream.flush().unwrap();

    let instructions_repr = produce_instructions_repr(instructions, state.instruction_ptr, 3);
    out_stream.write(instructions_repr.as_bytes()).unwrap();

    out_stream.write(b"EZ> ").unwrap();
    out_stream.flush().unwrap();

    let mut input_buffer: String = String::new();
    in_stream.read_line(&mut input_buffer).unwrap();

    if input_buffer.starts_with("!") {
        state.is_debugging = false;
    } else if input_buffer.is_empty() == false {
        let dbg_instructions = compile_to_intermediate(&input_buffer, false);

        let current_instruction_ptr = state.instruction_ptr;
        state.instruction_ptr = 0;

        let current_cell_ptr = state.cell_ptr;

        interpret(&dbg_instructions, state, in_stream, out_stream, false);

        state.cell_ptr = current_cell_ptr;
        state.instruction_ptr = current_instruction_ptr;

        out_stream.write(b"\n").unwrap();
    }

    match instructions.get(state.instruction_ptr) {
        Some(instruction) => {
            interpret_instruction(*instruction, state, in_stream, out_stream, false);
        }
        None => {
            // TODO: Is this even possible? When entering debugging mode on the last instruction?
        }
    }

    writeln!(out_stream, "").unwrap();
}

pub fn interpret_with_std_io(instructions: &Vec<WrappedInstruction>, allow_debugging: bool) -> () {
    let stdin = io::stdin();
    let mut input = stdin.lock();

    let mut stdout = io::stdout();

    let mut state = ExecutionState::new();

    interpret(instructions, &mut state, &mut input, &mut stdout, allow_debugging);
}

#[cfg(test)]
mod tests {
    use crate::ezfuck::parser::parser::{compile_to_intermediate, WrappedInstruction};
    use super::*;

    fn interpret_and_collect_output(instructions: &Vec<WrappedInstruction>, state: &mut ExecutionState, input: &[u8]) -> String {
        let mut input = &input[..];
        let mut output = vec![];

        interpret(&instructions, state, &mut input, &mut output, false);

        let output_string = String::from_utf8(output).unwrap();
        return output_string;
    }

    fn interpret_unwrapped_instruction_and_collect_output(instruction: Instruction, state: &mut ExecutionState, input: &[u8]) -> String {
        let mut input = &input[..];
        let mut output = vec![];

        let wrapped_instruction = WrappedInstruction { instruction: instruction, source_start: 0 };


        interpret_instruction(wrapped_instruction, state, &mut input, &mut output, false);

        let output_string = String::from_utf8(output).unwrap();
        return output_string;
    }

    #[test]
    fn it_should_add_to_the_current_cell() {
        let instruction = Instruction::ApplyOperatorToCell {
            operator: MathOperator::Addition,
            value: Value::Number(5),
        };

        let mut state = ExecutionState::new();
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cells, vec![5]);
    }

    #[test]
    fn it_should_subtract_from_the_current_cell() {
        let instruction = Instruction::ApplyOperatorToCell {
            operator: MathOperator::Subtraction,
            value: Value::Number(5),
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(20);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cells, vec![15]);
    }

    #[test]
    fn it_should_multiply_the_current_cell() {
        let instruction = Instruction::ApplyOperatorToCell {
            operator: MathOperator::Multiplication,
            value: Value::Number(5),
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(10);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cells, vec![50]);
    }

    #[test]
    fn it_should_divide_the_current_cell() {
        let instruction = Instruction::ApplyOperatorToCell {
            operator: MathOperator::Division,
            value: Value::Number(5),
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(50);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cells, vec![10]);
    }

    #[test]
    fn it_should_jump_the_instruction_pointer_if_equal() {
        let instruction = Instruction::JumpToIf {
            operator: EqualityOperator::Equal,
            match_value: 10,
            position: 5,
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(10);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.instruction_ptr, 5);
    }

    #[test]
    fn it_should_not_jump_the_instruction_pointer_if_not_equal() {
        let instruction = Instruction::JumpToIf {
            operator: EqualityOperator::Equal,
            match_value: 100,
            position: 5,
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(10);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.instruction_ptr, 0);
    }

    #[test]
    fn it_should_jump_the_instruction_pointer_if_not_equal() {
        let instruction = Instruction::JumpToIf {
            operator: EqualityOperator::NotEqual,
            match_value: 10,
            position: 5,
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(5);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.instruction_ptr, 5);
    }

    #[test]
    fn it_should_move_the_cell_pointer_to_the_left() {
        let instruction = Instruction::ApplyOperatorToCellPtr {
            operator: CellMoveOperator::Left,
            value: Value::Number(5),
        };

        let mut state = ExecutionState::new();
        state.set_cell_pointer(20);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cell_ptr, 15);
    }

    #[test]
    fn it_should_move_the_cell_pointer_to_the_right() {
        let instruction = Instruction::ApplyOperatorToCellPtr {
            operator: CellMoveOperator::Right,
            value: Value::Number(5),
        };

        let mut state = ExecutionState::new();
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cell_ptr, 5);
    }

    #[test]
    fn it_should_set_the_cell_pointer_directly() {
        let instruction = Instruction::ApplyOperatorToCellPtr {
            operator: CellMoveOperator::Set,
            value: Value::Number(10),
        };

        let mut state = ExecutionState::new();
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cell_ptr, 10);
    }

    #[test]
    fn it_should_set_the_current_cell() {
        let instruction = Instruction::SetCell {
            value: Value::Number(5),
        };

        let mut state = ExecutionState::new();
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.cells, vec![5]);
    }

    #[test]
    fn it_should_not_jump_the_instruction_pointer_if_equal() {
        let instruction = Instruction::JumpToIf {
            operator: EqualityOperator::NotEqual,
            match_value: 10,
            position: 5,
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(10);
        interpret_unwrapped_instruction_and_collect_output(instruction, &mut state, b"");
        assert_eq!(state.instruction_ptr, 0);
    }

    #[test]
    fn it_should_print_hello_world() {
        // TODO: Find a more isolated, clean way of doing this test without relying on the parser
        let code = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let instructions = compile_to_intermediate(code, false);

        let mut state = ExecutionState::new();
        let output_string = interpret_and_collect_output(&instructions, &mut state, b"");
        assert_eq!(output_string, "Hello World!\n");
    }

    #[test]
    fn it_should_print_hello_world_using_values() {
        let code = "+8[>+4[>+2>+3>+3>+<4-]>+>+>->2+[<]<-]>2.>-3.+7..+3.>2.<-.<.+3.-6.-8.>2+.>+2.";
        let instructions = compile_to_intermediate(code, false);

        let mut state = ExecutionState::new();
        let output_string = interpret_and_collect_output(&instructions, &mut state, b"");
        assert_eq!(output_string, "Hello World!\n");
    }

    #[test]
    fn it_should_set_cell_value_using_extraction() {
        let code = "^65 .";
        let instructions = compile_to_intermediate(code, false);

        let mut state = ExecutionState::new();
        let output_string = interpret_and_collect_output(&instructions, &mut state, b"");
        assert_eq!(output_string, "A");
    }

/*    #[test]
    fn it_should_get_cell_value_using_insertion() {
        let code = "^2 >V ";  // TODO: How to test?
        let instructions = compile_to_intermediate(code);

        let output_string = interpret_and_collect_output(&instructions, b"");
        assert_eq!(output_string, "A");
    }*/

    #[test]
    fn it_should_properly_parse_concurrent_insertions() {
        let code = "^^65 .";
        let instructions = compile_to_intermediate(code, false);

        let mut state = ExecutionState::new();
        let output_string = interpret_and_collect_output(&instructions, &mut state, b"");
        assert_eq!(output_string, "A");
    }

    #[test]
    fn it_should_wrap_cell_values_properly_on_increment() {
        let increment = Instruction::ApplyOperatorToCell {
            operator: MathOperator::Addition,
            value: Value::Number(2)
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(255);
        interpret_unwrapped_instruction_and_collect_output(increment, &mut state, b"");
        assert_eq!(state.get_current_cell(), 1);
    }

    #[test]
    fn it_should_wrap_cell_values_properly_on_decrement() {
        let decrement = Instruction::ApplyOperatorToCell {
            operator: MathOperator::Subtraction,
            value: Value::Number(2)
        };

        let mut state = ExecutionState::new();
        state.set_current_cell(0);
        interpret_unwrapped_instruction_and_collect_output(decrement, &mut state, b"");
        assert_eq!(state.get_current_cell(), 254);
    }
}