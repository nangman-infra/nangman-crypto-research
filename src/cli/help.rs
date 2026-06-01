pub fn print_help() {
    println!("{}", help_text());
}

pub(crate) fn help_text() -> &'static str {
    include_str!("help.txt").trim_end_matches('\n')
}
