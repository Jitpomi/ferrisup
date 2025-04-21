//! Search algorithms implementation module.
//!
//! This module provides implementations of common search algorithms.

use std::cmp::Ordering;

/// Perform a binary search on a sorted slice.
///
/// Returns `Some(index)` if the value is found, or `None` if it is not.
pub fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    if arr.is_empty() {
        return None;
    }
    
    let mut low = 0;
    let mut high = arr.len() - 1;
    
    while low <= high {
        let mid = low + (high - low) / 2;
        
        match arr[mid].cmp(target) {
            Ordering::Equal => return Some(mid),
            Ordering::Greater => {
                if mid == 0 {
                    return None;
                }
                high = mid - 1;
            },
            Ordering::Less => {
                low = mid + 1;
            }
        }
    }
    
    None
}

/// Perform a linear search on a slice.
///
/// Returns `Some(index)` if the value is found, or `None` if it is not.
pub fn linear_search<T: PartialEq>(arr: &[T], target: &T) -> Option<usize> {
    for (i, item) in arr.iter().enumerate() {
        if item == target {
            return Some(i);
        }
    }
    
    None
}

/// Find the maximum value in a slice.
///
/// Returns `None` if the slice is empty.
pub fn find_max<T: Ord>(arr: &[T]) -> Option<&T> {
    if arr.is_empty() {
        return None;
    }
    
    let mut max = &arr[0];
    
    for item in arr.iter().skip(1) {
        if item > max {
            max = item;
        }
    }
    
    Some(max)
}

/// Find the minimum value in a slice.
///
/// Returns `None` if the slice is empty.
pub fn find_min<T: Ord>(arr: &[T]) -> Option<&T> {
    if arr.is_empty() {
        return None;
    }
    
    let mut min = &arr[0];
    
    for item in arr.iter().skip(1) {
        if item < min {
            min = item;
        }
    }
    
    Some(min)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binary_search() {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        
        assert_eq!(binary_search(&arr, &5), Some(4));
        assert_eq!(binary_search(&arr, &10), None);
    }
    
    #[test]
    fn test_linear_search() {
        let arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
        
        assert_eq!(linear_search(&arr, &9), Some(5));
        assert_eq!(linear_search(&arr, &7), None);
    }
    
    #[test]
    fn test_find_max() {
        let arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
        
        assert_eq!(find_max(&arr), Some(&9));
        assert_eq!(find_max(&[] as &[i32]), None);
    }
    
    #[test]
    fn test_find_min() {
        let arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
        
        assert_eq!(find_min(&arr), Some(&1));
        assert_eq!(find_min(&[] as &[i32]), None);
    }
}
