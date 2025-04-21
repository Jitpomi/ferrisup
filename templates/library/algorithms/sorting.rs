//! Sorting algorithms implementation module.
//!
//! This module provides implementations of common sorting algorithms.

use num_traits::Number;
use rayon::prelude::*;
use std::cmp::Ordering;

/// Sort a slice using the quicksort algorithm.
pub fn quicksort<T: Ord + Copy>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }
    
    let pivot_index = partition(arr);
    
    let (left, right) = arr.split_at_mut(pivot_index);
    quicksort(left);
    quicksort(&mut right[1..]); // Skip the pivot
}

/// Partition a slice for quicksort.
fn partition<T: Ord + Copy>(arr: &mut [T]) -> usize {
    let pivot = arr[arr.len() - 1];
    let mut i = 0;
    
    for j in 0..arr.len() - 1 {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    
    arr.swap(i, arr.len() - 1);
    i
}

/// Sort a slice using the merge sort algorithm.
pub fn mergesort<T: Ord + Copy>(arr: &mut [T]) {
    if arr.len() <= 1 {
        return;
    }
    
    let mid = arr.len() / 2;
    let (left, right) = arr.split_at_mut(mid);
    
    mergesort(left);
    mergesort(right);
    
    // Merge the sorted halves
    let mut temp = Vec::with_capacity(arr.len());
    let mut left_idx = 0;
    let mut right_idx = 0;
    
    while left_idx < left.len() && right_idx < right.len() {
        if left[left_idx] <= right[right_idx] {
            temp.push(left[left_idx]);
            left_idx += 1;
        } else {
            temp.push(right[right_idx]);
            right_idx += 1;
        }
    }
    
    // Add remaining elements
    temp.extend_from_slice(&left[left_idx..]);
    temp.extend_from_slice(&right[right_idx..]);
    
    // Copy back to original array
    arr.copy_from_slice(&temp);
}

/// Sort a slice using parallel merge sort (using Rayon).
pub fn parallel_mergesort<T: Ord + Copy + Send + Sync>(arr: &mut [T]) {
    if arr.len() <= 1024 {  // Use sequential for small arrays
        mergesort(arr);
        return;
    }
    
    let mid = arr.len() / 2;
    let (left, right) = arr.split_at_mut(mid);
    
    rayon::join(
        || parallel_mergesort(left),
        || parallel_mergesort(right)
    );
    
    // Merge the sorted halves
    let mut temp = Vec::with_capacity(arr.len());
    let mut left_idx = 0;
    let mut right_idx = 0;
    
    while left_idx < left.len() && right_idx < right.len() {
        if left[left_idx] <= right[right_idx] {
            temp.push(left[left_idx]);
            left_idx += 1;
        } else {
            temp.push(right[right_idx]);
            right_idx += 1;
        }
    }
    
    // Add remaining elements
    temp.extend_from_slice(&left[left_idx..]);
    temp.extend_from_slice(&right[right_idx..]);
    
    // Copy back to original array
    arr.copy_from_slice(&temp);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quicksort() {
        let mut arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
        quicksort(&mut arr);
        assert_eq!(arr, [1, 1, 2, 3, 4, 5, 5, 6, 9]);
    }
    
    #[test]
    fn test_mergesort() {
        let mut arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
        mergesort(&mut arr);
        assert_eq!(arr, [1, 1, 2, 3, 4, 5, 5, 6, 9]);
    }
    
    #[test]
    fn test_parallel_mergesort() {
        let mut arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
        parallel_mergesort(&mut arr);
        assert_eq!(arr, [1, 1, 2, 3, 4, 5, 5, 6, 9]);
    }
}
