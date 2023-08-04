#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, thiserror::Error)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Malformed LLVM module: {}", self.0)
    }
}
