/// The maximum number of characters per line.
const NUM_CHARS: usize = 18;

/// The maximum number of lines per program.
const NUM_LINES: usize = 16;

/// A label and the index of the instruction that it refers to.
#[derive(Debug, PartialEq)]
pub struct Label(pub String, pub usize);

/// A lexed source line, consisting of its line number, an optional label,
/// and one or more lexemes that form an instruction.
#[derive(Debug, PartialEq)]
pub struct Line(pub usize, pub Option<Label>, pub Vec<String>);

/// Split the source code into lines of labels and lexemes.
pub fn lex_program(src: &str) -> Vec<Line> {
    let mut next_op = 0;
    let mut lines = Vec::new();

    for (index, line) in src.lines().take(NUM_LINES).enumerate() {
        let (maybe_label, words) = lex_line(line);
        let label = if let Some(label) = maybe_label {
            Some(Label(label, next_op))
        } else {
            None
        };

        if words.len() > 0 {
            next_op += 1;
        }

        lines.push(Line(index, label, words));
    }

    lines
}

/// Lex a single line of source code.
fn lex_line(line: &str) -> (Option<String>, Vec<String>) {
    let mut label = None;
    let mut words = Vec::new();
    let mut word = String::new();

    for c in line.to_uppercase().chars().take(NUM_CHARS) {
        if is_comment_delimiter(c) {
            break;
        } else if is_whitespace(c) {
            if word.len() > 0 {
                words.push(word.clone());
                word.clear();
            }
        } else if label.is_some() || !is_label_delimiter(c) {
            word.push(c)
        } else {
            label = Some(word.clone());
            word.clear();
        }
    }

    if word.len() > 0 {
        words.push(word.clone());
    }

    (label, words)
}

/// Check if a character is whitespace.
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == ','
}

/// Check if a character is a comment delimiter.
fn is_comment_delimiter(c: char) -> bool {
    c == '#'
}

/// Check if a character is a label delimter.
fn is_label_delimiter(c: char) -> bool {
    c == ':'
}

#[test]
fn test_is_whitespace() {
    assert!(is_whitespace(' '));
    assert!(is_whitespace(','));
    assert!(!is_whitespace('1'));
    assert!(!is_whitespace('A'));
}

#[test]
fn test_is_comment_delimiter() {
    assert!(is_comment_delimiter('#'));
    assert!(!is_comment_delimiter('1'));
    assert!(!is_comment_delimiter('A'));
}

#[test]
fn test_is_label_delimiter() {
    assert!(is_label_delimiter(':'));
    assert!(!is_label_delimiter('1'));
    assert!(!is_label_delimiter('A'));
}

#[test]
fn test_lex_line() {
    let (lbl, lex) = lex_line("LABEL: MOV UP ACC # comment");
    assert_eq!(lbl, Some("LABEL".to_string()));
    assert_eq!(lex.len(), 3);
    assert_eq!(lex[0], "MOV");
    assert_eq!(lex[1], "UP");
    assert_eq!(lex[2], "ACC");

    let (lbl, lex) = lex_line("ADD 1");
    assert_eq!(lbl, None);
    assert_eq!(lex.len(), 2);
    assert_eq!(lex[0], "ADD");
    assert_eq!(lex[1], "1");

    let (lbl, lex) = lex_line(":ADD 1 2 3");
    assert_eq!(lbl, Some("".to_string()));
    assert_eq!(lex.len(), 4);
    assert_eq!(lex[0], "ADD");
    assert_eq!(lex[1], "1");
    assert_eq!(lex[2], "2");
    assert_eq!(lex[3], "3");

    let (lbl, lex) = lex_line(",,LABEL:,,ADD,1,,,,,");
    assert_eq!(lbl, Some("LABEL".to_string()));
    assert_eq!(lex.len(), 2);
    assert_eq!(lex[0], "ADD");
    assert_eq!(lex[1], "1");

    let (lbl, lex) = lex_line("# LABEL: MOV UP ACC");
    assert_eq!(lbl, None);
    assert_eq!(lex.len(), 0);

    let (lbl, lex) = lex_line("LABEL: MOV LEFT RIGHT");
    assert_eq!(lbl, Some("LABEL".to_string()));
    assert_eq!(lex.len(), 3);
    assert_eq!(lex[0], "MOV");
    assert_eq!(lex[1], "LEFT");
    assert_eq!(lex[2], "RI");
}

#[test]
fn test_lex_program() {
    let lines = lex_program("MOV UP ACC\nADD 1\nMOV ACC DOWN");
    assert_eq!(lines.len(), 3);

    let lines = lex_program("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n");
    assert_eq!(lines.len(), 16);

    let lines = lex_program("1:\n2:\n3: ADD 1\n4: ADD 1\n");
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].1, Some(Label("1".to_string(), 0)));
    assert_eq!(lines[1].1, Some(Label("2".to_string(), 0)));
    assert_eq!(lines[2].1, Some(Label("3".to_string(), 0)));
    assert_eq!(lines[3].1, Some(Label("4".to_string(), 1)));
}

