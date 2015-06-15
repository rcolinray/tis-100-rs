use parse::ProgramErrors;
use save::SaveErrors;

fn pretty_print_errors(node_num: usize, errors: &ProgramErrors) {
    for &(line_num, ref error) in errors.iter() {
        println!("node {}: line {}: {}\n", node_num, line_num, error);
    }
}

pub fn pretty_print_save_errors(save_errors: SaveErrors) {
    for (node_num, ref errors) in save_errors.iter() {
        pretty_print_errors(node_num, errors);
    }
}
