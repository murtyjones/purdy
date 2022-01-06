// FIXME: These conversions could be dangerous with pathological values, since usize -> f64 cannot be perfectly converted
pub fn invert_index(i: usize, arr_length: usize) -> usize {
    let result = 1.0 - (i as f64 / arr_length as f64);
    let result = ((result as f64) * (arr_length - 1) as f64).floor();
    return result as usize;
}

#[test]
fn test_invert_index() {
    let arr = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'/*7*/];

    assert_eq!(invert_index(0, arr.len()), 7);
    assert_eq!(invert_index(1, arr.len()), 6);
    assert_eq!(invert_index(2, arr.len()), 5);
    assert_eq!(invert_index(3, arr.len()), 4);
    assert_eq!(invert_index(4, arr.len()), 3);
    assert_eq!(invert_index(5, arr.len()), 2);
    assert_eq!(invert_index(6, arr.len()), 1);
    assert_eq!(invert_index(7, arr.len()), 0);
}