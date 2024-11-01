use core::error;
use std::str::FromStr;

use scraper::{Html, Selector};

// TODO: usar wrap em vez de boxing
pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Extrai o o valor dum parágrafo filho do elemento `#box_result`.
/// Alguns desses parágrafos contém texto no formato `chave:valor`.
/// A função extrai somento o valor.
///
/// # Argumentos
/// * `body` - HTML da página do CA.
/// * `selector_do_pai` - Seletor do elemento pai. Ex.: `#box_result`
/// * `nome` - Nome da informação a ser extraída. Ex.: `natureza`, `validade` etc.
/// * `posicao` - Posição do referido filho.
pub fn nth_child<T: FromStr>(
    body: &Html,
    selector_do_pai: &str,
    nome: &str,
    posicao: u8,
) -> Result<T>
where
    <T as FromStr>::Err: std::error::Error,
    <T as FromStr>::Err: 'static,
{
    let nao_encontrada = Err(format!("não encontrado: {nome}").into());
    let selector_str = format!("{} > p:nth-child({})", selector_do_pai, posicao);
    let selector = match Selector::parse(&selector_str) {
        Ok(s) => s,
        Err(_) => return Err("Erro no parsing.".into()),
    };
    let child_str = match body.select(&selector).next() {
        Some(e) => e.text().collect::<String>(),
        None => return nao_encontrada,
    };
    if child_str.is_empty() {
        return nao_encontrada;
    }
    let child = match child_str.split(":").last() {
        Some(p) => p.trim().parse::<T>()?,
        None => return nao_encontrada,
    };
    Ok(child)
}
