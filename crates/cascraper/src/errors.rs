use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum CAError {
    /// Ocorre quando não se consegue encontrar a página
    /// do CA. Este erro contém o CA não encontrado.
    #[error("CA {0} não encontrado.")]
    NaoEncontrado(u32),
}
