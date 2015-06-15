/// The maximum number of characters per line.
const NUM_CHARS: usize = 19;

/// The maximum number of lines per program.
const NUM_LINES: usize = 16;

/// A label and the index of the instruction that it refers to.
#[derive(Debug, PartialEq)]
pub struct Label(pub String, pub usize);

/// A lexed source line, consisting of its line number, an optional label,
/// and one or more lexemes that form an instruction.
#[derive(Debug, PartialEq)]
pub struct Line(pub usize, pub Option<Label>, pub Vec<String>);

/// Parse the source code into lines of labels and lexemes.
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

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == ','
}

fn is_comment_delimiter(c: char) -> bool {
    c == '#'
}

fn is_label_delimiter(c: char) -> bool {
    c == ':'
}
