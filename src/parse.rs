use std::collections::HashMap;
use std::str::FromStr;
use core::{Opcode, Instruction, Program};
use core::Instruction::*;

/// An error which can be returned when parsing a TIS-100 program.
#[derive(Debug, PartialEq)]
pub enum ParseProgramError<'a> {
    InvalidSyntax(&'a str),
    UndefinedLabel(&'a str),
    DuplicateLabel(&'a str),
    InvalidOpcode(&'a str),
    InvalidExpression(&'a str),
    InvalidInstruction(&'a str, &'a str, &'a str),
}

use self::ParseProgramError::*;

/// The list of errors encountered while parsing the program source code.
pub type ErrorList<'a> = Vec<ParseProgramError<'a>>;

type ParseResult<'a, T> = Result<T, ParseProgramError<'a>>;

/// Parse the program source code into a list of instructions. If one or more errors are
/// encountered during parsing, then the list of errors will be returned instead.
///
/// Example:
///
/// ```
/// use tis_100::core::*;
/// use tis_100::core::Instruction::*;
/// use tis_100::core::Source::*;
/// use tis_100::core::Register::*;
/// use tis_100::core::IoRegister::*;
/// use tis_100::core::Port::*;
/// use tis_100::parse::parse_program;
///
/// let src = "MOV UP ACC\n
///            ADD 1\n
///            MOV ACC DOWN\n";
/// let prog = parse_program(src).unwrap();
/// assert_eq!(prog[0], Mov(Reg(Io(Dir(Up))), Acc));
/// assert_eq!(prog[1], Add(Val(1)));
/// assert_eq!(prog[2], Mov(Reg(Acc), Io(Dir(Down))));
/// ```
pub fn parse_program<'a>(source: &'a str) -> Result<Program, ErrorList<'a>> {
    // The basic parsing strategy is:
    // 1. Tokenize the input using a regex.
    // 2. Store the location of each label.
    // 3. Parse each instruction and resolve labels.

    let re = regex!(r"^,* *(?:(\w+):)?,* *(?:(\w+))?,* *(?:(\w+))?,* *(?:(\w+))?,* *(?:#.*)?$");

    let mut line_captures = Vec::new();
    let mut errors = Vec::new();
    let mut labels = HashMap::new();
    let mut instructions = Vec::new();

    // Label pass
    let mut next_idx = 0;
    for line in source.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(label) = captures.at(1) {
                if let None = labels.get(label) {
                    labels.insert(label, next_idx);
                } else {
                    errors.push(DuplicateLabel(label));
                }
            }

            // If we find an opcode, then increment the next index that the next label should
            // refer to.
            if let Some(_) = captures.at(2) {
                next_idx += 1;
            }

            line_captures.push(captures);
        } else {
            errors.push(InvalidSyntax(line));
        }
    }

    // Instruction pass
    for captures in line_captures.iter() {
        if let Some(opcode) = captures.at(2) {
            let operand1 = captures.at(3).unwrap_or("");
            let operand2 = captures.at(4).unwrap_or("");

            match parse_instruction(opcode, operand1, operand2, &labels) {
                Ok(instruction) => instructions.push(instruction),
                Err(error) => errors.push(error),
            };
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(instructions)
    }
}

/// Attempt to parse an instruction given an opcode, two operands, and a map of labels.
fn parse_instruction<'a>(opcode: &'a str, operand1: &'a str, operand2: &'a str, labels: &HashMap<&'a str, usize>) -> ParseResult<'a, Instruction> {
    let num_ops = count_operands(operand1, operand2);
    match str::parse::<Opcode>(opcode) {
        Ok(code) => {
            match code {
                Opcode::Nop if num_ops == 0 => Ok(Nop),
                Opcode::Mov if num_ops == 2 => parse_two_operands(operand1, operand2).and_then(|(op1, op2)| { Ok(Mov(op1, op2)) }),
                Opcode::Swp if num_ops == 0 => Ok(Swp),
                Opcode::Sav if num_ops == 0 => Ok(Sav),
                Opcode::Add if num_ops == 1 => parse_one_operand(operand1).and_then(|op| { Ok(Add(op)) }),
                Opcode::Sub if num_ops == 1 => parse_one_operand(operand1).and_then(|op| { Ok(Sub(op)) }),
                Opcode::Neg if num_ops == 0 => Ok(Neg),
                Opcode::Jmp if num_ops == 1 => resolve_jump_label(operand1, labels, Jmp),
                Opcode::Jez if num_ops == 1 => resolve_jump_label(operand1, labels, Jez),
                Opcode::Jnz if num_ops == 1 => resolve_jump_label(operand1, labels, Jnz),
                Opcode::Jgz if num_ops == 1 => resolve_jump_label(operand1, labels, Jgz),
                Opcode::Jlz if num_ops == 1 => resolve_jump_label(operand1, labels, Jlz),
                Opcode::Jro if num_ops == 1 => parse_one_operand(operand1).and_then(|op| { Ok(Jro(op)) }),
                _ => Err(InvalidInstruction(opcode, operand1, operand2)),
            }
        },
        Err(_) => Err(InvalidOpcode(opcode)),
    }
}

/// A helper function to count the number of operands.
fn count_operands(op1: &str, op2: &str) -> usize {
    let mut num_ops = 0;

    if op1.len() > 0 {
        num_ops += 1;
    }

    if op2.len() > 0 {
        num_ops += 1;
    }

    return num_ops;
}

/// Look up the index for a label and create a jump instruction if the label is found.
fn resolve_jump_label<'a, F: Fn(usize) -> Instruction>(label: &'a str, labels: &HashMap<&str, usize>, f: F) -> ParseResult<'a, Instruction> {
    labels.get(label)
        .ok_or(UndefinedLabel(label))
        .map(|&i| { f(i) })
}

/// Attempt to parse an operand from a string.
fn parse_one_operand<'a, T: FromStr>(operand: &'a str) -> ParseResult<'a, T> {
    match str::parse::<T>(operand) {
        Ok(op) => Ok(op),
        Err(_) => Err(InvalidExpression(operand)),
    }
}

/// Attempt to parse two operands from a string.
fn parse_two_operands<'a, T: FromStr, U: FromStr>(operand1: &'a str, operand2: &'a str) -> ParseResult<'a, (T, U)> {
    match str::parse::<T>(operand1) {
        Ok(op1) => {
            match str::parse::<U>(operand2) {
                Ok(op2) => Ok((op1, op2)),
                Err(_) => Err(InvalidExpression(operand2)),
            }
        },
        Err(_) => Err(InvalidExpression(operand1)),
    }
}

#[test]
fn test_parse_one_operand() {
    use core::{Register, Source};

    assert_eq!(parse_one_operand("ACC"), Ok(Source::Reg(Register::Acc)));
    assert_eq!(parse_one_operand::<Source>("bad"), Err(InvalidExpression("bad")));
}

#[test]
fn test_parse_two_operands() {
    use core::{Register, Source};

    assert_eq!(parse_two_operands("1", "ACC"), Ok((Source::Val(1), Source::Reg(Register::Acc))));
    assert_eq!(parse_two_operands::<Source, Register>("1", "bad"), Err(InvalidExpression("bad")));
    assert_eq!(parse_two_operands::<Source, Register>("bad", "bad"), Err(InvalidExpression("bad")));
}

#[test]
fn test_resolve_jump_label() {
    let mut labels = HashMap::new();
    labels.insert("good", 1);

    assert_eq!(resolve_jump_label("good", &labels, Jmp), Ok(Jmp(1)));
    assert_eq!(resolve_jump_label("bad", &labels, Jmp), Err(UndefinedLabel("bad")));
}

#[test]
fn test_count_operands() {
    assert_eq!(count_operands("", ""), 0);
    assert_eq!(count_operands("ACC", ""), 1);
    assert_eq!(count_operands("ACC", "BAK"), 2);
}

#[test]
fn test_parse_instruction() {
    use core::Source::*;
    use core::Register::*;

    let mut labels = HashMap::new();
    labels.insert("TEST", 0);

    assert_eq!(parse_instruction("BAD", "", "", &labels), Err(InvalidOpcode("BAD")));

    assert_eq!(parse_instruction("NOP", "", "", &labels), Ok(Nop));
    assert_eq!(parse_instruction("NOP", "bad", "", &labels), Err(InvalidInstruction("NOP", "bad", "")));

    assert_eq!(parse_instruction("MOV", "1", "ACC", &labels), Ok(Mov(Val(1), Acc)));
    assert_eq!(parse_instruction("MOV", "", "", &labels), Err(InvalidInstruction("MOV", "", "")));

    assert_eq!(parse_instruction("SWP", "", "", &labels), Ok(Swp));
    assert_eq!(parse_instruction("SWP", "bad", "", &labels), Err(InvalidInstruction("SWP", "bad", "")));

    assert_eq!(parse_instruction("SAV", "", "", &labels), Ok(Sav));
    assert_eq!(parse_instruction("SAV", "bad", "", &labels), Err(InvalidInstruction("SAV", "bad", "")));

    assert_eq!(parse_instruction("ADD", "1", "", &labels), Ok(Add(Val(1))));
    assert_eq!(parse_instruction("ADD", "", "", &labels), Err(InvalidInstruction("ADD", "", "")));

    assert_eq!(parse_instruction("SUB", "1", "", &labels), Ok(Sub(Val(1))));
    assert_eq!(parse_instruction("SUB", "", "", &labels), Err(InvalidInstruction("SUB", "", "")));

    assert_eq!(parse_instruction("NEG", "", "", &labels), Ok(Neg));
    assert_eq!(parse_instruction("NEG", "bad", "", &labels), Err(InvalidInstruction("NEG", "bad", "")));

    assert_eq!(parse_instruction("JMP", "TEST", "", &labels), Ok(Jmp(0)));
    assert_eq!(parse_instruction("JMP", "", "", &labels), Err(InvalidInstruction("JMP", "", "")));
    assert_eq!(parse_instruction("JMP", "BAD", "", &labels), Err(UndefinedLabel("BAD")));

    assert_eq!(parse_instruction("JEZ", "TEST", "", &labels), Ok(Jez(0)));
    assert_eq!(parse_instruction("JEZ", "", "", &labels), Err(InvalidInstruction("JEZ", "", "")));
    assert_eq!(parse_instruction("JEZ", "BAD", "", &labels), Err(UndefinedLabel("BAD")));

    assert_eq!(parse_instruction("JNZ", "TEST", "", &labels), Ok(Jnz(0)));
    assert_eq!(parse_instruction("JNZ", "", "", &labels), Err(InvalidInstruction("JNZ", "", "")));
    assert_eq!(parse_instruction("JNZ", "BAD", "", &labels), Err(UndefinedLabel("BAD")));

    assert_eq!(parse_instruction("JGZ", "TEST", "", &labels), Ok(Jgz(0)));
    assert_eq!(parse_instruction("JGZ", "", "", &labels), Err(InvalidInstruction("JGZ", "", "")));
    assert_eq!(parse_instruction("JGZ", "BAD", "", &labels), Err(UndefinedLabel("BAD")));

    assert_eq!(parse_instruction("JLZ", "TEST", "", &labels), Ok(Jlz(0)));
    assert_eq!(parse_instruction("JLZ", "", "", &labels), Err(InvalidInstruction("JLZ", "", "")));
    assert_eq!(parse_instruction("JLZ", "BAD", "", &labels), Err(UndefinedLabel("BAD")));

    assert_eq!(parse_instruction("JRO", "-1", "", &labels), Ok(Jro(Val(-1))));
    assert_eq!(parse_instruction("JRO", "ACC", "", &labels), Ok(Jro(Reg(Acc))));
    assert_eq!(parse_instruction("JRO", "", "", &labels), Err(InvalidInstruction("JRO", "", "")));
}
