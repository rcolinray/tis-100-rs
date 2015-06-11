use std::str::FromStr;

/// A TIS-100 port.
#[derive(Debug, PartialEq, Copy, Clone, Hash)]
pub enum Port {
    Up,
    Down,
    Left,
    Right,
}

/// An error which can be returned when parsing a port.
#[derive(Debug, PartialEq)]
pub struct ParsePortError;

impl FromStr for Port {
    type Err = ParsePortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "UP" => Ok(Port::Up),
            "DOWN" => Ok(Port::Down),
            "LEFT" => Ok(Port::Left),
            "RIGHT" => Ok(Port::Right),
            _ => Err(ParsePortError)
        }
    }
}

/// A TIS-100 port or pseudo-port.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IoRegister {
    Dir(Port),
    AnyPort,
    Last,
}

/// An error which can be returned when parsing an IO register.
#[derive(Debug, PartialEq)]
pub struct ParseIoRegisterError;

impl FromStr for IoRegister {
    type Err = ParseIoRegisterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ANY" => Ok(IoRegister::AnyPort),
            "LAST" => Ok(IoRegister::Last),
            _ => if let Ok(port) = str::parse::<Port>(s) {
                Ok(IoRegister::Dir(port))
            } else {
                Err(ParseIoRegisterError)
            }
        }
    }
}

/// A TIS-100 register.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Register {
    Acc,
    Nil,
    Io(IoRegister),
}

/// An error which can be returned when parsing a register.
#[derive(Debug, PartialEq)]
pub struct ParseRegisterError;

impl FromStr for Register {
    type Err = ParseRegisterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ACC" => Ok(Register::Acc),
            "NIL" => Ok(Register::Nil),
            _ => {
                if let Ok(reg) = str::parse::<IoRegister>(s) {
                    Ok(Register::Io(reg))
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
    Val(isize),
    Reg(Register),
}

/// An error which can be returned when parsing an source.
#[derive(Debug, PartialEq)]
pub struct ParseSourceError;

impl FromStr for Source {
    type Err = ParseSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(val) = str::parse::<isize>(s) {
            Ok(Source::Val(val))
        } else if let Ok(register) = str::parse::<Register>(s) {
            Ok(Source::Reg(register))
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
    Jmp(usize),
    Jez(usize),
    Jnz(usize),
    Jgz(usize),
    Jlz(usize),
    Jro(Source),
}

/// The list of instructions created by parsing the program source code. The
/// instructions can then be evaluated by a basic execution node.
pub type Program = Vec<Instruction>;

#[test]
fn test_parse_port() {
    assert_eq!(str::parse::<Port>("UP"), Ok(Port::Up));
    assert_eq!(str::parse::<Port>("up"), Ok(Port::Up));
    assert_eq!(str::parse::<Port>("DOWN"), Ok(Port::Down));
    assert_eq!(str::parse::<Port>("LEFT"), Ok(Port::Left));
    assert_eq!(str::parse::<Port>("RIGHT"), Ok(Port::Right));
    assert_eq!(str::parse::<Port>("bad"), Err(ParsePortError));
}

#[test]
fn test_parse_io_register() {
    assert_eq!(str::parse::<IoRegister>("UP"), Ok(IoRegister::Dir(Port::Up)));
    assert_eq!(str::parse::<IoRegister>("ANY"), Ok(IoRegister::AnyPort));
    assert_eq!(str::parse::<IoRegister>("any"), Ok(IoRegister::AnyPort));
    assert_eq!(str::parse::<IoRegister>("LAST"), Ok(IoRegister::Last));
    assert_eq!(str::parse::<IoRegister>("bad"), Err(ParseIoRegisterError));
}

#[test]
fn test_parse_register() {
    assert_eq!(str::parse::<Register>("ACC"), Ok(Register::Acc));
    assert_eq!(str::parse::<Register>("acc"), Ok(Register::Acc));
    assert_eq!(str::parse::<Register>("NIl"), Ok(Register::Nil));
    assert_eq!(str::parse::<Register>("UP"), Ok(Register::Io(IoRegister::Dir(Port::Up))));
    assert_eq!(str::parse::<Register>("bad"), Err(ParseRegisterError));
}

#[test]
fn test_parse_source() {
    assert_eq!(str::parse::<Source>("ACC"), Ok(Source::Reg(Register::Acc)));
    assert_eq!(str::parse::<Source>("1"), Ok(Source::Val(1)));
    assert_eq!(str::parse::<Source>("bad"), Err(ParseSourceError));
}
