use super::*;

#[test]
fn find_in_direction_finds_first_match_each_way() {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let current_index = 4;
    let predicate = |&x: &i32| if x % 2 == 0 { Some(x) } else { None };

    let result_forward =
        find_in_direction(&items, current_index, SearchDirection::Forward, predicate);
    assert_eq!(result_forward, Some((5, 6)));

    let result_backward =
        find_in_direction(&items, current_index, SearchDirection::Backward, predicate);
    assert_eq!(result_backward, Some((3, 4)));
}
