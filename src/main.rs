use clap::{builder::Styles, ArgAction, Parser};
use scraper::Html;

pub mod info;

#[must_use]
#[cfg(feature = "error-context")]
fn write_dynamic_context(
    error: &crate::error::Error,
    styled: &mut StyledStr,
    styles: &Styles,
) -> bool {
    use std::fmt::Write as _;
    let valid = styles.get_valid();
    let invalid = styles.get_invalid();
    let literal = styles.get_literal();

    match error.kind() {
        ErrorKind::ArgumentConflict => {
            let mut prior_arg = error.get(ContextKind::PriorArg);
            if let Some(ContextValue::String(invalid_arg)) = error.get(ContextKind::InvalidArg) {
                if Some(&ContextValue::String(invalid_arg.clone())) == prior_arg {
                    prior_arg = None;
                    let _ = write!(
                        styled,
                        "the argument '{invalid}{invalid_arg}{invalid:#}' cannot be used multiple times",
                    );
                } else {
                    let _ = write!(
                        styled,
                        "the argument '{invalid}{invalid_arg}{invalid:#}' cannot be used with",
                    );
                }
            } else if let Some(ContextValue::String(invalid_arg)) =
                error.get(ContextKind::InvalidSubcommand)
            {
                let _ = write!(
                    styled,
                    "the subcommand '{invalid}{invalid_arg}{invalid:#}' cannot be used with",
                );
            } else {
                styled.push_str(error.kind().as_str().unwrap());
            }

            if let Some(prior_arg) = prior_arg {
                match prior_arg {
                    ContextValue::Strings(values) => {
                        styled.push_str(":");
                        for v in values {
                            let _ = write!(styled, "\n{TAB}{invalid}{v}{invalid:#}",);
                        }
                    }
                    ContextValue::String(value) => {
                        let _ = write!(styled, " '{invalid}{value}{invalid:#}'",);
                    }
                    _ => {
                        styled.push_str(" one or more of the other specified arguments");
                    }
                }
            }

            true
        }
        ErrorKind::NoEquals => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                let _ = write!(
                    styled,
                    "equal sign is needed when assigning values to '{invalid}{invalid_arg}{invalid:#}'",
                );
                true
            } else {
                false
            }
        }
        ErrorKind::InvalidValue => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            let invalid_value = error.get(ContextKind::InvalidValue);
            if let (
                Some(ContextValue::String(invalid_arg)),
                Some(ContextValue::String(invalid_value)),
            ) = (invalid_arg, invalid_value)
            {
                if invalid_value.is_empty() {
                    let _ = write!(
                        styled,
                        "a value is required for '{invalid}{invalid_arg}{invalid:#}' but none was supplied",
                    );
                } else {
                    let _ = write!(
                        styled,
                        "invalid value '{invalid}{invalid_value}{invalid:#}' for '{literal}{invalid_arg}{literal:#}'",
                    );
                }

                let values = error.get(ContextKind::ValidValue);
                write_values_list("possible values", styled, valid, values);

                true
            } else {
                false
            }
        }
        ErrorKind::InvalidSubcommand => {
            let invalid_sub = error.get(ContextKind::InvalidSubcommand);
            if let Some(ContextValue::String(invalid_sub)) = invalid_sub {
                let _ = write!(
                    styled,
                    "unrecognized subcommand '{invalid}{invalid_sub}{invalid:#}'",
                );
                true
            } else {
                false
            }
        }
        ErrorKind::MissingRequiredArgument => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            if let Some(ContextValue::Strings(invalid_arg)) = invalid_arg {
                styled.push_str("the following required arguments were not provided:");
                for v in invalid_arg {
                    let _ = write!(styled, "\n{TAB}{valid}{v}{valid:#}",);
                }
                true
            } else {
                false
            }
        }
        ErrorKind::MissingSubcommand => {
            let invalid_sub = error.get(ContextKind::InvalidSubcommand);
            if let Some(ContextValue::String(invalid_sub)) = invalid_sub {
                let _ = write!(
                    styled,
                    "'{invalid}{invalid_sub}{invalid:#}' requires a subcommand but one was not provided",
                );
                let values = error.get(ContextKind::ValidSubcommand);
                write_values_list("subcommands", styled, valid, values);

                true
            } else {
                false
            }
        }
        ErrorKind::InvalidUtf8 => false,
        ErrorKind::TooManyValues => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            let invalid_value = error.get(ContextKind::InvalidValue);
            if let (
                Some(ContextValue::String(invalid_arg)),
                Some(ContextValue::String(invalid_value)),
            ) = (invalid_arg, invalid_value)
            {
                let _ = write!(
                    styled,
                    "unexpected value '{invalid}{invalid_value}{invalid:#}' for '{literal}{invalid_arg}{literal:#}' found; no more were expected",
                );
                true
            } else {
                false
            }
        }
        ErrorKind::TooFewValues => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            let actual_num_values = error.get(ContextKind::ActualNumValues);
            let min_values = error.get(ContextKind::MinValues);
            if let (
                Some(ContextValue::String(invalid_arg)),
                Some(ContextValue::Number(actual_num_values)),
                Some(ContextValue::Number(min_values)),
            ) = (invalid_arg, actual_num_values, min_values)
            {
                let were_provided = singular_or_plural(*actual_num_values as usize);
                let _ = write!(
                    styled,
                    "{valid}{min_values}{valid:#} values required by '{literal}{invalid_arg}{literal:#}'; only {invalid}{actual_num_values}{invalid:#}{were_provided}",
                );
                true
            } else {
                false
            }
        }
        ErrorKind::ValueValidation => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            let invalid_value = error.get(ContextKind::InvalidValue);
            if let (
                Some(ContextValue::String(invalid_arg)),
                Some(ContextValue::String(invalid_value)),
            ) = (invalid_arg, invalid_value)
            {
                let _ = write!(
                    styled,
                    "invalid value '{invalid}{invalid_value}{invalid:#}' for '{literal}{invalid_arg}{literal:#}'",
                );
                if let Some(source) = error.inner.source.as_deref() {
                    let _ = write!(styled, ": {source}");
                }
                true
            } else {
                false
            }
        }
        ErrorKind::WrongNumberOfValues => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            let actual_num_values = error.get(ContextKind::ActualNumValues);
            let num_values = error.get(ContextKind::ExpectedNumValues);
            if let (
                Some(ContextValue::String(invalid_arg)),
                Some(ContextValue::Number(actual_num_values)),
                Some(ContextValue::Number(num_values)),
            ) = (invalid_arg, actual_num_values, num_values)
            {
                let were_provided = singular_or_plural(*actual_num_values as usize);
                let _ = write!(
                    styled,
                    "{valid}{num_values}{valid:#} values required for '{literal}{invalid_arg}{literal:#}' but {invalid}{actual_num_values}{invalid:#}{were_provided}",
                );
                true
            } else {
                false
            }
        }
        ErrorKind::UnknownArgument => {
            let invalid_arg = error.get(ContextKind::InvalidArg);
            if let Some(ContextValue::String(invalid_arg)) = invalid_arg {
                let _ = write!(
                    styled,
                    "unexpected argument '{invalid}{invalid_arg}{invalid:#}' found",
                );
                true
            } else {
                false
            }
        }
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        | ErrorKind::DisplayVersion
        | ErrorKind::Io
        | ErrorKind::Format => false,
    }
}
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
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
    let args = Args::parse();
    for ca in args.cas {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://consultaca.com/".to_owned() + &ca.to_string())
            .send()
            .await;
        let body_txt = match resp {
            Ok(r) => match r.text().await {
                Ok(txt) => txt,
                Err(e) => panic!("{:#?}", e),
            },
            Err(e) => panic!("{}", e),
        };
        let body = Html::parse_document(&body_txt);
        let consulta = match info::CA::consultar(&body).await {
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
