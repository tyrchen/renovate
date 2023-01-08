mod macros;
mod node;
pub mod parsec;

pub use node::{
    node_enum_to_string, node_to_embed_constraint, node_to_string, type_name_to_string,
};

#[allow(dead_code)]
pub fn drain_where<T, Pred: Fn(&T) -> bool>(source: Vec<T>, pred: Pred) -> (Vec<T>, Vec<T>) {
    let mut orig: Vec<T> = Vec::with_capacity(source.len());
    let mut drained: Vec<T> = Vec::new();

    for v in source.into_iter() {
        if pred(&v) {
            drained.push(v);
        } else {
            orig.push(v);
        }
    }
    (orig, drained)
}
