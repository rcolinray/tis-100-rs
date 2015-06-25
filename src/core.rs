//! Basic types for parsing and interpreting TIS-100 assembly code.

use std::str::FromStr;

/// A TIS-100 port.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Port {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

use self::Port::*;

/// An error which can be returned when parsing a port.
#[derive(Debug, PartialEq)]
pub struct ParsePortError;

impl FromStr for Port {
    type Err = ParsePortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UP" => Ok(UP),
            "DOWN" => Ok(DOWN),
            "LEFT" => Ok(LEFT),
            "RIGHT" => Ok(RIGHT),
            _ => Err(ParsePortError)
        }
    }
}

/// Get the opposing direction for a given port.
///
/// # Example
///
/// ```
/// use tis_100::core::Port::*;
/// use tis_100::core::opposite_port;
///
/// assert_eq!(opposite_port(UP), DOWN);
/// assert_eq!(opposite_port(LEFT), RIGHT);
/// ```
pub fn opposite_port(port: Port) -> Port {
    match port {
        UP => DOWN,
        DOWN => UP,
        LEFT => RIGHT,
        RIGHT => LEFT,
    }
}

/// A TIS-100 port or pseudo-port.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IoRegister {
    DIR(Port),
    ANY, // All-caps so that we don't conflict with std::any::Any.
    LAST,
}

use self::IoRegister::*;

/// An error which can be returned when parsing an IO register.
#[derive(Debug, PartialEq)]
pub struct ParseIoRegisterError;

impl FromStr for IoRegister {
    type Err = ParseIoRegisterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ANY" => Ok(ANY),
            "LAST" => Ok(LAST),
            _ => if let Ok(port) = str::parse::<Port>(s) {
                Ok(DIR(port))
            } else {
                Err(ParseIoRegisterError)
            }
        }
    }
}

/// A TIS-100 register.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Register {
    ACC,
    NIL,
    IO(IoRegister),
}

use self::Register::*;

/// An error which can be returned when parsing a register.
#[derive(Debug, PartialEq)]
pub struct ParseRegisterError;

impl FromStr for Register {
    type Err = ParseRegisterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACC" => Ok(ACC),
            "NIL" => Ok(NIL),
            _ => {
                if let Ok(reg) = str::parse::<IoRegister>(s) {
                    Ok(IO(reg))
                } else {
                    Err(ParseRegisterError)
                }
            }
        }
    }
}

/// The source component of a TIS-100 instruction.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Source {
    VAL(isize),
    REG(Register),
}

use self::Source::*;

/// An error which can be returned when parsing a source.
#[derive(Debug, PartialEq)]
pub struct ParseSourceError;

impl FromStr for Source {
    type Err = ParseSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(val) = str::parse::<isize>(s) {
            Ok(VAL(val))
        } else if let Ok(register) = str::parse::<Register>(s) {
            Ok(REG(register))
        } else {
            Err(ParseSourceError)
        }
    }
}

/// A valid TIS-100 instruction.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Instruction {
    Nop,
    Mov(Source, Register),
    Swp,
    Sav,
    Add(Source),
    Sub(Source),
    Neg,
    Jmp(isize),
    Jez(isize),
    Jnz(isize),
    Jgz(isize),
    Jlz(isize),
    Jro(Source),
}

/// The list of instructions created by parsing the program source code. The
/// instructions can then be evaluated by a basic execution node.
pub type Program = Vec<Instruction>;

#[test]
fn test_parse_port() {
    assert_eq!(str::parse::<Port>("UP"), Ok(UP));
    assert_eq!(str::parse::<Port>("DOWN"), Ok(DOWN));
    assert_eq!(str::parse::<Port>("LEFT"), Ok(LEFT));
    assert_eq!(str::parse::<Port>("RIGHT"), Ok(RIGHT));
    assert_eq!(str::parse::<Port>("up"), Err(ParsePortError));
    assert_eq!(str::parse::<Port>("bad"), Err(ParsePortError));
}

#[test]
fn test_parse_io_register() {
    assert_eq!(str::parse::<IoRegister>("UP"), Ok(DIR(UP)));
    assert_eq!(str::parse::<IoRegister>("ANY"), Ok(ANY));
    assert_eq!(str::parse::<IoRegister>("LAST"), Ok(LAST));
    assert_eq!(str::parse::<IoRegister>("any"), Err(ParseIoRegisterError));
    assert_eq!(str::parse::<IoRegister>("bad"), Err(ParseIoRegisterError));
}

#[test]
fn test_parse_register() {
    assert_eq!(str::parse::<Register>("ACC"), Ok(ACC));
    assert_eq!(str::parse::<Register>("NIL"), Ok(NIL));
    assert_eq!(str::parse::<Register>("UP"), Ok(IO(DIR(UP))));
    assert_eq!(str::parse::<Register>("acc"), Err(ParseRegisterError));
    assert_eq!(str::parse::<Register>("bad"), Err(ParseRegisterError));
}

#[test]
fn test_parse_source() {
    assert_eq!(str::parse::<Source>("ACC"), Ok(REG(ACC)));
    assert_eq!(str::parse::<Source>("1"), Ok(VAL(1)));
    assert_eq!(str::parse::<Source>("bad"), Err(ParseSourceError));
}
