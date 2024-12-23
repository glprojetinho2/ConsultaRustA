use cascraper::pagina;
use clap::{builder::Styles, ArgAction, Parser};
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Consulta CA's através do site https://consultaca.com/"
)]
#[command(help_template = format!("\
{{before-help}}{{about-with-newline}}
{}{}:{} {{usage}}
{{all-args}}{{after-help}}\
    ",
    Styles::default().get_usage().render(),
    "Uso",
    Styles::default().get_usage().render_reset()))]
#[command(next_help_heading = "Opções")]
#[command(disable_help_flag(true))]
#[command(disable_version_flag(true))]
struct Args {
    #[arg(required = true)]
    cas: Vec<u32>,
    #[arg(action = ArgAction::Help, short, long)]
    #[arg(help = "Mostra essa mensagem e sai.")]
    help: Option<bool>,

    #[arg(action = ArgAction::Version, short = 'V', long)]
    #[arg(help = "Mostra a versão e sai.")]
    version: Option<bool>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    for ca in args.cas {
        let client = reqwest::Client::new();
        let body = pagina(Some(client), ca).await;
        let consulta = match cascraper::CA::consultar(&body, ca).await {
            Ok(c) => c,
            Err(e) => panic!("{:#?}", e),
        };
        if consulta.ca == 0 {
            println!("CA {ca} não encontrado.");
            continue;
        }
        println!("{:#?}", consulta);
    }
}
