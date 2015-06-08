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
pub enum PseudoPort {
    Any,
    Last,
}

/// An error which can be returned when parsing a psedo-port.
#[derive(Debug, PartialEq)]
pub struct ParsePseudoPortError;

impl FromStr for PseudoPort {
    type Err = ParsePseudoPortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ANY" => Ok(PseudoPort::Any),
            "LAST" => Ok(PseudoPort::Last),
            _ => Err(ParsePseudoPortError)
        }
    }
}

/// A TIS-100 port or pseudo-port.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum IoRegister {
    PortReg(Port),
    PseudoReg(PseudoPort),
}

/// An error which can be returned when parsing an IO register.
#[derive(Debug, PartialEq)]
pub struct ParseIoRegisterError;

impl FromStr for IoRegister {
    type Err = ParseIoRegisterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(port) = str::parse::<Port>(s) {
            Ok(IoRegister::PortReg(port))
        } else if let Ok(pseudo) = str::parse::<PseudoPort>(s) {
            Ok(IoRegister::PseudoReg(pseudo))
        } else {
            Err(ParseIoRegisterError)
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

/// The opcode component of a TIS-100 instruction.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    Nop,
    Mov,
    Swp,
    Sav,
    Add,
    Sub,
    Neg,
    Jmp,
    Jez,
    Jnz,
    Jgz,
    Jlz,
    Jro,
}

/// An error which can be returned when parsing an opcode.
#[derive(Debug, PartialEq)]
pub struct ParseOpcodeError;

impl FromStr for Opcode {
    type Err = ParseOpcodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NOP" => Ok(Opcode::Nop),
            "MOV" => Ok(Opcode::Mov),
            "SWP" => Ok(Opcode::Swp),
            "SAV" => Ok(Opcode::Sav),
            "ADD" => Ok(Opcode::Add),
            "SUB" => Ok(Opcode::Sub),
            "NEG" => Ok(Opcode::Neg),
            "JMP" => Ok(Opcode::Jmp),
            "JEZ" => Ok(Opcode::Jez),
            "JNZ" => Ok(Opcode::Jnz),
            "JGZ" => Ok(Opcode::Jgz),
            "JLZ" => Ok(Opcode::Jlz),
            "JRO" => Ok(Opcode::Jro),
            _ => Err(ParseOpcodeError),
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
fn test_parse_psedo_port() {
    assert_eq!(str::parse::<PseudoPort>("ANY"), Ok(PseudoPort::Any));
    assert_eq!(str::parse::<PseudoPort>("any"), Ok(PseudoPort::Any));
    assert_eq!(str::parse::<PseudoPort>("LAST"), Ok(PseudoPort::Last));
    assert_eq!(str::parse::<PseudoPort>("bad"), Err(ParsePseudoPortError));
}

#[test]
fn test_parse_io_register() {
    assert_eq!(str::parse::<IoRegister>("UP"), Ok(IoRegister::PortReg(Port::Up)));
    assert_eq!(str::parse::<IoRegister>("ANY"), Ok(IoRegister::PseudoReg(PseudoPort::Any)));
    assert_eq!(str::parse::<IoRegister>("bad"), Err(ParseIoRegisterError));
}

#[test]
fn test_parse_register() {
    assert_eq!(str::parse::<Register>("ACC"), Ok(Register::Acc));
    assert_eq!(str::parse::<Register>("acc"), Ok(Register::Acc));
    assert_eq!(str::parse::<Register>("NIl"), Ok(Register::Nil));
    assert_eq!(str::parse::<Register>("UP"), Ok(Register::Io(IoRegister::PortReg(Port::Up))));
    assert_eq!(str::parse::<Register>("bad"), Err(ParseRegisterError));
}

#[test]
fn test_parse_source() {
    assert_eq!(str::parse::<Source>("ACC"), Ok(Source::Reg(Register::Acc)));
    assert_eq!(str::parse::<Source>("1"), Ok(Source::Val(1)));
    assert_eq!(str::parse::<Source>("bad"), Err(ParseSourceError));
}

#[test]
fn test_parse_opcode() {
    assert_eq!(str::parse::<Opcode>("NOP"), Ok(Opcode::Nop));
    assert_eq!(str::parse::<Opcode>("nop"), Ok(Opcode::Nop));
    assert_eq!(str::parse::<Opcode>("MOV"), Ok(Opcode::Mov));
    assert_eq!(str::parse::<Opcode>("SWP"), Ok(Opcode::Swp));
    assert_eq!(str::parse::<Opcode>("SAV"), Ok(Opcode::Sav));
    assert_eq!(str::parse::<Opcode>("ADD"), Ok(Opcode::Add));
    assert_eq!(str::parse::<Opcode>("SUB"), Ok(Opcode::Sub));
    assert_eq!(str::parse::<Opcode>("NEG"), Ok(Opcode::Neg));
    assert_eq!(str::parse::<Opcode>("JMP"), Ok(Opcode::Jmp));
    assert_eq!(str::parse::<Opcode>("JEZ"), Ok(Opcode::Jez));
    assert_eq!(str::parse::<Opcode>("JNZ"), Ok(Opcode::Jnz));
    assert_eq!(str::parse::<Opcode>("JGZ"), Ok(Opcode::Jgz));
    assert_eq!(str::parse::<Opcode>("JLZ"), Ok(Opcode::Jlz));
    assert_eq!(str::parse::<Opcode>("JRO"), Ok(Opcode::Jro));
    assert_eq!(str::parse::<Opcode>("bad"), Err(ParseOpcodeError));
}
