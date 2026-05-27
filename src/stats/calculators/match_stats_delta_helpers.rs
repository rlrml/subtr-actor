pub(super) fn optional_delta<T: Copy + PartialEq>(
    current: Option<T>,
    previous: Option<T>,
) -> Option<T> {
    if current == previous {
        None
    } else {
        current
    }
}

pub(super) fn sample_delta<T: Copy + PartialEq>(current: &[T], previous: &[T]) -> Vec<T> {
    let mut unmatched_previous = previous.to_vec();
    let mut delta = Vec::new();
    for value in current {
        if let Some(index) = unmatched_previous
            .iter()
            .position(|previous_value| previous_value == value)
        {
            unmatched_previous.remove(index);
        } else {
            delta.push(*value);
        }
    }
    delta
}
