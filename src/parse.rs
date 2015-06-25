//! Functions for parsing TIS-100 assembly code into instructions.

use std::str::FromStr;
use std::fmt::{Display, Formatter, Error};
use std::collections::HashMap;
use core::*;
use core::Instruction::*;
use lex::{lex_program, Label, Line};

/// An error that can be returned while parsing a TIS-100 assembly program.
#[derive(Debug, PartialEq)]
pub enum ParseProgramError {
    InvalidLabel,
    UndefinedLabel(String),
    DuplicateLabel(String),
    InvalidOpcode(String),
    InvalidExpression(String),
    InvalidRegister(String),
    MissingOperand(String),
    TooManyOperands(String),
}

impl Display for ParseProgramError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            &InvalidLabel => f.write_str("Invalid label"),
            &UndefinedLabel(ref lbl) => f.write_fmt(format_args!("Undefined label: '{}'", lbl)),
            &DuplicateLabel(ref lbl) => f.write_fmt(format_args!("Label is already defined: '{}'", lbl)),
            &InvalidOpcode(ref op) => f.write_fmt(format_args!("Invalid opcode: '{}'", op)),
            &InvalidExpression(ref expr) => f.write_fmt(format_args!("Invalid expression: '{}'", expr)),
            &InvalidRegister(ref reg) => f.write_fmt(format_args!("Invalid register: '{}'", reg)),
            &MissingOperand(ref op) => f.write_fmt(format_args!("Missing operand: '{}'", op)),
            &TooManyOperands(ref ops) => f.write_fmt(format_args!("Too many operands: '{}'", ops)),
        }
    }
}

/// All errors discovered while parsing a TIS-100 assembly program.
pub type ProgramErrors = Vec<(usize, ParseProgramError)>;

use self::ParseProgramError::*;

/// A result that can be returned intermediate phases of the parsing process.
type ParseResult<T> = Result<T, ParseProgramError>;

/// Parse the program source code into a list of instructions. If one or more errors are
/// encountered during parsing, then the list of errors will be returned instead.
///
/// # Example
///
/// ```
/// use tis_100::core::Instruction::*;
/// use tis_100::core::Source::*;
/// use tis_100::core::Register::*;
/// use tis_100::core::IoRegister::*;
/// use tis_100::core::Port::*;
/// use tis_100::parse::parse_program;
///
/// let src = "MOV UP ACC\nADD 1\nMOV ACC DOWN\n";
/// let prog = parse_program(src).unwrap();
/// assert_eq!(prog[0], Mov(REG(IO(DIR(UP))), ACC));
/// assert_eq!(prog[1], Add(VAL(1)));
/// assert_eq!(prog[2], Mov(REG(ACC), IO(DIR(DOWN))));
/// ```
pub fn parse_program(src: &str) -> Result<Program, ProgramErrors> {
    // The basic parsing process is:
    // 1. Tokenize the source into labels, opcodes, and operands
    // 2. Create a mapping of labels to instruction indices
    // 3. Parse opcodes and operands line-by-line to generate instructions

    let mut label_map = HashMap::new();
    let mut instructions = Vec::new();
    let mut errors = Vec::new();

    let lines = lex_program(src);

    // Lable mapping pass
    for &Line(line_num, ref maybe_label, _) in lines.iter() {
        if let &Some(Label(ref name, index)) = maybe_label {
            if name.len() == 0 {
                errors.push((line_num, InvalidLabel));
            } else if let None = label_map.get(name) {
                label_map.insert(name.clone(), index as isize);
            } else {
                errors.push((line_num, DuplicateLabel(name.clone())));
            }
        }
    }

    // Instruction pass
    for &Line(line_num, _, ref lexemes) in lines.iter() {
        if lexemes.len() > 0 {
            match parse_instruction(&lexemes[0], &lexemes[1..], &label_map) {
                Ok(instruction) => instructions.push(instruction),
                Err(err) => errors.push((line_num, err)),
            }
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(instructions)
    }
}

/// Attempt to parse a single TIS-100 assembly instruction.
fn parse_instruction(opcode: &str, operands: &[String], labels: &HashMap<String, isize>) -> ParseResult<Instruction> {
    match str::parse::<Opcode>(opcode) {
        Ok(NOP) => parse_no_operands(Nop, operands),
        Ok(MOV) => parse_two_operands(Mov, opcode, operands),
        Ok(SWP) => parse_no_operands(Swp, operands),
        Ok(SAV) => parse_no_operands(Sav, operands),
        Ok(ADD) => parse_one_operand(Add, opcode, operands),
        Ok(SUB) => parse_one_operand(Sub, opcode, operands),
        Ok(NEG) => parse_no_operands(Neg, operands),
        Ok(JMP) => parse_jump(Jmp, opcode, operands, labels),
        Ok(JEZ) => parse_jump(Jez, opcode, operands, labels),
        Ok(JNZ) => parse_jump(Jnz, opcode, operands, labels),
        Ok(JGZ) => parse_jump(Jgz, opcode, operands, labels),
        Ok(JLZ) => parse_jump(Jlz, opcode, operands, labels),
        Ok(JRO) => parse_one_operand(Jro, opcode, operands),
        _ => Err(InvalidOpcode(opcode.to_string())),
    }
}

/// Attempt to resolve a jump label to an instruction index.
fn resolve_label<'a>(label: &str, labels: &'a HashMap<String, isize>) -> ParseResult<&'a isize> {
    labels.get(label).ok_or(UndefinedLabel(label.to_string()))
}

/// Parse a jump opcode and label into a jump instruction.
fn parse_jump<F: Fn(isize) -> Instruction>(f: F, opcode: &str, operands: &[String], labels: &HashMap<String, isize>) -> ParseResult<Instruction> {
    if operands.len() < 1 {
        Err(MissingOperand(opcode.to_string()))
    } else if operands.len() == 1 {
        resolve_label(&operands[0], labels).map(|&i| f(i))
    } else {
        Err(TooManyOperands(operands[1..].connect(" ")))
    }
}

/// Parse an opcode into an instruction.
fn parse_no_operands(instruction: Instruction, operands: &[String]) -> ParseResult<Instruction> {
    if operands.len() == 0 {
        Ok(instruction)
    } else {
        Err(TooManyOperands(operands.connect(" ")))
    }
}

/// Parse an opcode and one operand into an instruction.
fn parse_one_operand<T: FromStr, F: Fn(T) -> Instruction>(f: F, opcode: &str, operands: &[String]) -> ParseResult<Instruction> {
    if operands.len() < 1 {
        Err(MissingOperand(opcode.to_string()))
    } else if operands.len() == 1 {
        match str::parse::<T>(&operands[0]) {
            Ok(op) => Ok(f(op)),
            Err(_) => Err(InvalidExpression(operands[0].clone())),
        }
    } else {
        Err(TooManyOperands(operands[1..].connect(" ")))
    }
}

/// Parse an opcode and two operands into an instruction.
fn parse_two_operands<T: FromStr, U: FromStr, F: Fn(T, U) -> Instruction>(f: F, opcode: &str, operands: &[String]) -> ParseResult<Instruction> {
    if operands.len() < 2 {
        Err(MissingOperand(opcode.to_string() + " " + &operands.connect(" ")))
    } else if operands.len() == 2 {
        match str::parse::<T>(&operands[0]) {
            Ok(op1) => match str::parse::<U>(&operands[1]) {
                Ok(op2) => Ok(f(op1, op2)),
                Err(_) => Err(InvalidRegister(operands[1].clone())),
            },
            Err(_) => Err(InvalidExpression(operands[0].clone())),
        }
    } else {
        Err(TooManyOperands(operands[2..].connect(" ")))
    }
}

/// The opcode component of a TIS-100 instruction.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    NOP,
    MOV,
    SWP,
    SAV,
    ADD,
    SUB,
    NEG,
    JMP,
    JEZ,
    JNZ,
    JGZ,
    JLZ,
    JRO,
}

use self::Opcode::*;

/// An error which can be returned when parsing an opcode.
#[derive(Debug, PartialEq)]
pub struct ParseOpcodeError;

impl FromStr for Opcode {
    type Err = ParseOpcodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NOP" => Ok(NOP),
            "MOV" => Ok(MOV),
            "SWP" => Ok(SWP),
            "SAV" => Ok(SAV),
            "ADD" => Ok(ADD),
            "SUB" => Ok(SUB),
            "NEG" => Ok(NEG),
            "JMP" => Ok(JMP),
            "JEZ" => Ok(JEZ),
            "JNZ" => Ok(JNZ),
            "JGZ" => Ok(JGZ),
            "JLZ" => Ok(JLZ),
            "JRO" => Ok(JRO),
            _ => Err(ParseOpcodeError),
        }
    }
}

#[test]
fn test_parse_opcode() {
    assert_eq!(str::parse::<Opcode>("NOP"), Ok(NOP));
    assert_eq!(str::parse::<Opcode>("MOV"), Ok(MOV));
    assert_eq!(str::parse::<Opcode>("SWP"), Ok(SWP));
    assert_eq!(str::parse::<Opcode>("SAV"), Ok(SAV));
    assert_eq!(str::parse::<Opcode>("ADD"), Ok(ADD));
    assert_eq!(str::parse::<Opcode>("SUB"), Ok(SUB));
    assert_eq!(str::parse::<Opcode>("NEG"), Ok(NEG));
    assert_eq!(str::parse::<Opcode>("JMP"), Ok(JMP));
    assert_eq!(str::parse::<Opcode>("JEZ"), Ok(JEZ));
    assert_eq!(str::parse::<Opcode>("JNZ"), Ok(JNZ));
    assert_eq!(str::parse::<Opcode>("JGZ"), Ok(JGZ));
    assert_eq!(str::parse::<Opcode>("JLZ"), Ok(JLZ));
    assert_eq!(str::parse::<Opcode>("JRO"), Ok(JRO));
    assert_eq!(str::parse::<Opcode>("nop"), Err(ParseOpcodeError));
    assert_eq!(str::parse::<Opcode>("bad"), Err(ParseOpcodeError));
}
