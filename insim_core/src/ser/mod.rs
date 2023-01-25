/// Serialization and Deserialization, or Encodable and Decodable as we're calling to avoid
/// overlapping with Serde.
pub mod decode;
pub mod encode;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Limit {
    Count(usize),
    Bytes(usize),
}

pub fn limit_to_bytes(limit: Option<Limit>, type_size: usize) -> usize {
    match limit {
        Some(Limit::Count(i)) => i * type_size,
        Some(Limit::Bytes(i)) => i,
        None => type_size,
    }
}
