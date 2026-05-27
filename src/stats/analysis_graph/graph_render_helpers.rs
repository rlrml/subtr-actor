use std::any::TypeId;
use std::collections::HashMap;

pub(super) fn ensure_external_render_node(
    labels: &mut Vec<(TypeId, Box<str>)>,
    external_node_ids: &mut HashMap<TypeId, usize>,
    dependency_type_id: TypeId,
    label: String,
) -> usize {
    if let Some(node_id) = external_node_ids.get(&dependency_type_id) {
        return *node_id;
    }

    let node_id = labels.len();
    labels.push((dependency_type_id, label.into_boxed_str()));
    external_node_ids.insert(dependency_type_id, node_id);
    node_id
}

pub(super) fn short_type_name(type_name: &str) -> String {
    let mut shortened = String::with_capacity(type_name.len());
    let mut token = String::new();

    for character in type_name.chars() {
        if character.is_alphanumeric() || matches!(character, '_' | ':') {
            token.push(character);
            continue;
        }

        if !token.is_empty() {
            shortened.push_str(token.rsplit("::").next().unwrap_or(&token));
            token.clear();
        }
        shortened.push(character);
    }

    if !token.is_empty() {
        shortened.push_str(token.rsplit("::").next().unwrap_or(&token));
    }

    shortened
}
