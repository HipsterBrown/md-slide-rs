use pulldown_cmark::{html, Event, Parser};
use std::fs::read_to_string;
use std::path::PathBuf;
use structopt::StructOpt;

/// md-slide CLI
#[derive(StructOpt, Debug)]
#[structopt(name = "md-slide")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// File to process
    #[structopt(name = "FILE", parse(from_os_str))]
    input: PathBuf,
}

fn main() {
    let options = Opt::from_args();

    // get content from options.input file
    let markdown_content = read_to_string(options.input).unwrap();

    let parser = Parser::new(&markdown_content).collect::<Vec<Event>>();
    let parsed_pages = parser.split(|event| *event == Event::Rule);

    for page in parsed_pages.clone() {
        let mut html_output = String::new();
        html::push_html(&mut html_output, page.to_vec().into_iter());

        println!("{:#?}", html_output);
    }
}
