use pulldown_cmark::{html, Event, Parser};
use std::fs::{create_dir, read_to_string, remove_dir_all, File};
use std::io::prelude::*;
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

fn main() -> std::io::Result<()> {
    let options = Opt::from_args();

    // get content from options.input file
    let markdown_content = read_to_string(options.input).unwrap();

    let parser = Parser::new(&markdown_content).collect::<Vec<Event>>();
    let parsed_pages = parser.split(|event| *event == Event::Rule);

    // remove existing build directory
    remove_dir_all("./build")?;
    // create destination directory for pages
    create_dir("./build")?;

    for (index, page) in parsed_pages.enumerate() {
        let mut html_output = String::new();
        html::push_html(&mut html_output, page.to_vec().into_iter());

        let mut file = File::create(format!("./build/{}.html", index))?;
        file.write_all(&html_output.bytes().collect::<Vec<u8>>())?;
    }

    Ok(())
}
