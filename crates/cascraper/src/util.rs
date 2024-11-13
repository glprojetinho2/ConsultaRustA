use core::error;

/// Extrai números de uma `String`.
pub fn extrair_numeros(a: String) -> Result<u64, Box<dyn error::Error>> {
    Ok(a.chars()
        .filter(|x| x.is_numeric())
        .collect::<String>()
        .parse::<u64>()?)
}

#[cfg(test)]
mod tests {
    use super::extrair_numeros;
    #[test]
    fn test_extrair_numeros() {
        assert_eq!(
            extrair_numeros("69.561.137/0001-11".to_string()).unwrap(),
            69561137000111
        );
    }
}
