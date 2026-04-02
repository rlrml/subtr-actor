use serde::Serialize;

/// Enum to define the direction of searching within a collection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Searches for an item in a slice in a specified direction and returns the
/// first item that matches the provided predicate.
///
/// # Arguments
///
/// * `items` - The list of items to search.
/// * `current_index` - The index to start the search from.
/// * `direction` - The direction to search in.
/// * `predicate` - A function that takes an item and returns an [`Option<R>`].
///   When this function returns `Some(R)`, the item is considered a match.
///
/// # Returns
///
/// Returns a tuple of the index and the result `R` of the predicate for the first item that matches.
pub fn find_in_direction<T, F, R>(
    items: &[T],
    current_index: usize,
    direction: SearchDirection,
    predicate: F,
) -> Option<(usize, R)>
where
    F: Fn(&T) -> Option<R>,
{
    match direction {
        SearchDirection::Forward => items
            .iter()
            .enumerate()
            .skip(current_index + 1)
            .find_map(|(i, item)| predicate(item).map(|res| (i, res))),
        SearchDirection::Backward => items[..current_index]
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, item)| predicate(item).map(|res| (i, res))),
    }
}
