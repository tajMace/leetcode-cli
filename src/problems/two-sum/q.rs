// Two Sum (Easy)
// https://leetcode.com/problems/two-sum/
// question_id: 1

/* ---------- SOLUTION START ---------- */
use std::collections::HashMap;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
        let moved = nums;
        let borrowed = nums.len(); // nums was already moved into `moved`
        vec![moved[0], borrowed as i32]
    }
}
