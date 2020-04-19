use pulldown_cmark::{html, Event, Parser};
use std::fs::{create_dir, read_to_string, remove_dir_all, File};
use std::io::prelude::*;
use std::path::{Component, Path, PathBuf};
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
    <style>
        html {
            font-family: Helvetica, sans-serif;
            font-size: 22px;
            height: 100%;
        }
        body {
            height: 100%;
            padding: 2rem;
            position: relative;
        }
        h1 {
            font-size: 4rem;
        }
        h2 {
            font-size: 3rem;
        }
        h3 {
            font-size: 2rem;
        }
        h4 {
            font-size: 1rem;
        }
        h5 {
            font-size: 0.75rem;
        }
        p, ul, ol {
            font-size: 1.5rem;
        }
        nav {
            bottom: 5vw;
            display: flex;
            justify-content: space-between;
            left: 0;
            padding: 0 2rem;
            position: absolute;
            width: 100%;
        }
        nav a {
            flex: 1 1 auto;
            text-decoration: none;
        }
        ul {
            padding: 0 1rem;
        }
        ol {
            padding: 0 1.5rem;
        }
        blockquote {
            border-left: 4px solid black;
            color: rgba(0, 0, 0, 0.6);
            margin: 0;
            padding-left: 1rem;
        }
        img {
            display: block;
            height: auto;
            margin: 0 auto;
            max-width: 600px;
            width: 80vw;
        }
        .ta-right {
            text-align: right;
        }
    </style>
  </head>
  <body>
    {{ content }}

    <nav>
        {% if index > 0 %}
            <a href=\"/{{ index - 1 }}.html\"><= Previous</a>
        {% endif %}
        {% if index + 1 < num_of_slides %}
            <a class=\"ta-right\" href=\"/{{ index + 1 }}.html\">Next =></a>
        {% endif %}
    </nav>
  </body>
</html>";

fn get_path(root: &Path, file_path: &str) -> PathBuf {
    let relative_path = Path::new(file_path)
        .components()
        .fold(PathBuf::new(), |mut result, path| {
            match path {
                Component::Normal(x) => result.push({
                    let s = x.to_str().unwrap_or("");
                    &*percent_encoding::percent_decode(s.as_bytes()).decode_utf8_lossy()
                }),
                Component::ParentDir => {
                    result.pop();
                },
                _ => (),
            }
            result
        });
    root.join(relative_path)
}

async fn serve_static_files(request: tide::Request<StaticFile>) -> tide::Result {
    let actual_path: String = request.param("path").unwrap();
    let state = request.state();
    let response = task::block_on(async move {
        let path = get_path(&state.root, &actual_path);
        let meta = async_std::fs::metadata(&path).await.ok();

        // If the file doesn't exist, then bail out.
        if meta.is_none() {
            return Ok(tide::Response::new(404)
                .set_mime(mime::TEXT_HTML)
                .body_string(format!("Couldn't locate requested file {:?}", actual_path)));
        }

        let meta = meta.unwrap();
        let size = format!("{}", meta.len());
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        let file = async_std::fs::File::open(PathBuf::from(&path)).await.unwrap();
        let reader = async_std::io::BufReader::new(file);
        Ok(tide::Response::new(200).body(reader).set_header("Content-Length", size).set_mime(mime))
    });
    response
}

fn main() -> Result<(), std::io::Error> {
    let options = Opt::from_args();

    match options.command {
        Some(cmd) => {
            let root_dir = match cmd {
                Command::Serve { directory } => directory,
            };
            task::block_on(async {
                let mut server = tide::with_state(StaticFile { root: root_dir });
                server.at("/*path").get(|request| async { serve_static_files(request).await.unwrap() });

                println!("Serving presenation slides at 0.0.0.0:8080/0.html");
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
            if Path::new("./build").exists() {
                remove_dir_all("./build")?;
            }
            // create destination directory for pages
            create_dir("./build")?;

            let mut tera = Tera::default();
            tera.autoescape_on(vec![]);
            tera.add_raw_template("slide.html", DEFAULT_SLIDE_TEMPLATE)
                .unwrap();

            let num_of_slides = parsed_pages.clone().count();
            for (index, page) in parsed_pages.enumerate() {
                let mut html_output = String::new();
                html::push_html(&mut html_output, page.to_vec().into_iter());

                let mut context = Context::new();
                context.insert("index", &index);
                context.insert("content", &html_output);
                context.insert("num_of_slides", &num_of_slides);

                let slide_html = tera.render("slide.html", &context).unwrap();

                let mut file = File::create(format!("./build/{}.html", index))?;
                file.write_all(&slide_html.bytes().collect::<Vec<u8>>())?;
            }

            println!("Slides created!");

            Ok(())
        }
    }
}
