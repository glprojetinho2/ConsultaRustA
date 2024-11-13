/// Erros que podem ocorrer ao chamar a API.
/// E0001: ca deve ser um u32 ( e não '{}').
/// E0002: ca {} nao encontrado.
/// E0003: ca deve ser especificado.
#[macro_export]
macro_rules! erro {
    (1, $ca:expr) => {
        format!("E0001: ca deve ser um u32 ( e não '{}').", $ca)
    };
    (2, $ca:expr) => {
        format!("E0002: ca {} nao encontrado.", $ca)
    };
    (3) => {
        "E0003: ca deve ser especificado."
    };
}
