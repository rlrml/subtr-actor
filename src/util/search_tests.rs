use super::*;

#[test]
fn find_in_direction_finds_first_match_after_current_index() {
    let items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let current_index = 4;
    let predicate = |&x: &i32| if x % 2 == 0 { Some(x) } else { None };

    let forward = find_in_direction(&items, current_index, SearchDirection::Forward, predicate);
    let backward = find_in_direction(&items, current_index, SearchDirection::Backward, predicate);

    assert_eq!(forward, Some((5, 6)));
    assert_eq!(backward, Some((3, 4)));
}

#[test]
fn find_in_direction_forward_handles_out_of_range_current_index() {
    let items = [1, 2, 3];

    let result = find_in_direction(&items, usize::MAX, SearchDirection::Forward, |item| {
        Some(*item)
    });

    assert_eq!(result, None);
}

#[test]
fn find_in_direction_backward_handles_out_of_range_current_index() {
    let items = [1, 2, 3];

    let result = find_in_direction(&items, usize::MAX, SearchDirection::Backward, |item| {
        Some(*item)
    });

    assert_eq!(result, Some((2, 3)));
}

#[test]
fn find_in_direction_remains_exclusive_of_current_index() {
    let items = [1, 2, 3];

    let forward = find_in_direction(&items, 1, SearchDirection::Forward, |item| Some(*item));
    let backward = find_in_direction(&items, 1, SearchDirection::Backward, |item| Some(*item));

    assert_eq!(forward, Some((2, 3)));
    assert_eq!(backward, Some((0, 1)));
}
