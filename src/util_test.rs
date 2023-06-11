use super::*;

#[test]
fn test_find_update_in_direction() {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let current_index = 4; // Starting search from number 5
    let predicate = |&x: &i32| if x % 2 == 0 { Some(x) } else { None }; // Looking for the first even number

    // Test forward search.
    let result_forward =
        util::find_in_direction(&items, current_index, SearchDirection::Forward, predicate);
    // Check that the result is as expected.
    assert_eq!(result_forward, Some((5, 6))); // First even number after index 4 is 6 at index 5

    // Test backward search.
    let result_backward =
        util::find_in_direction(&items, current_index, SearchDirection::Backward, predicate);
    // Check that the result is as expected.
    assert_eq!(result_backward, Some((3, 4))); // First even number before index 4 is 4 at index 3
}
