use pulldown_cmark::{html, Event, Parser};
use std::fs::{create_dir, read_to_string, remove_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tera::{Context, Tera};
use async_std::task;

#[derive(Clone)]
struct StaticFile {
    root: PathBuf,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// serve presentation
    Serve {
        #[structopt(default_value = "./build", parse(from_os_str))]
        directory: PathBuf,
    },
}

/// md-slide CLI
#[derive(StructOpt, Debug)]
#[structopt(name = "md-slide")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Serve generated slides
    #[structopt(subcommand)]
    command: Option<Command>,

    /// File to process
    #[structopt(name = "FILE", parse(from_os_str))]
    input: Option<PathBuf>,
}

static DEFAULT_SLIDE_TEMPLATE: &'static str = "<html lang=\"en\">
  <head>
    <meta charset=\"utf-8\" />
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
    <title>Slide {{ index }}</title>
  </head>
  <body>
    {{ content }}
  </body>
</html>";

// async fn handle_path(ctx: tide::Request<StaticFile>) -> Result<(), std::io::Error> {
//     let path = ctx.uri().path();
//     println!("{:?}", path);
//     Ok(())
// }

fn main() -> Result<(), std::io::Error> {
    let options = Opt::from_args();

    match options.command {
        Some(cmd) => {
            let root_dir = match cmd {
                Command::Serve { directory } => directory,
            };
            println!("Serving presenation slides");
            task::block_on(async {
                let mut server = tide::with_state(StaticFile { root: root_dir });
                server.at("/*path").get(|req: tide::Request<StaticFile>| async move {
                    let path = req.uri().path();
                    println!("file path: {}", path);
                    format!("root directory: {:?}", &req.state().root)
                });
                server.listen("0.0.0.0:8080").await?;
                Ok(())
            })
        }
        None => {
            // get content from options.input file
            let markdown_content = read_to_string(options.input.unwrap()).unwrap();

            let parser = Parser::new(&markdown_content).collect::<Vec<Event>>();
            let parsed_pages = parser.split(|event| *event == Event::Rule);

            // remove existing build directory
            remove_dir_all("./build")?;
            // create destination directory for pages
            create_dir("./build")?;

            let mut tera = Tera::default();
            tera.autoescape_on(vec![]);
            tera.add_raw_template("slide.html", DEFAULT_SLIDE_TEMPLATE)
                .unwrap();

            for (index, page) in parsed_pages.enumerate() {
                let mut html_output = String::new();
                html::push_html(&mut html_output, page.to_vec().into_iter());

                let mut context = Context::new();
                context.insert("index", &index);
                context.insert("content", &html_output);

                let slide_html = tera.render("slide.html", &context).unwrap();

                let mut file = File::create(format!("./build/{}.html", index))?;
                file.write_all(&slide_html.bytes().collect::<Vec<u8>>())?;
            }

            println!("Slides created!");

            Ok(())
        }
    }
}
