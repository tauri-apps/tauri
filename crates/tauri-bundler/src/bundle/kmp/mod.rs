// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Knuth–Morris–Pratt algorithm
// based on https://github.com/howeih/rust_kmp
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn index_of(pattern: &[u8], target: &[u8]) -> Option<usize> {
  let failure_function = find_failure_function(pattern);

  let mut t_i: usize = 0;
  let mut p_i: usize = 0;
  let target_len = target.len();
  let mut result_idx = None;
  let pattern_len = pattern.len();

  while (t_i < target_len) && (p_i < pattern_len) {
    if target[t_i] == pattern[p_i] {
      if result_idx.is_none() {
        result_idx.replace(t_i);
      }
      t_i += 1;
      p_i += 1;
      if p_i >= pattern_len {
        return result_idx;
      }
    } else {
      if p_i == 0 {
        p_i = 0;
        t_i += 1;
      } else {
        p_i = failure_function[p_i - 1];
      }
      result_idx = None;
    }
  }
  None
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn find_failure_function(pattern: &[u8]) -> Vec<usize> {
  let mut i = 1;
  let mut j = 0;
  let pattern_length = pattern.len();
  let end_i = pattern_length - 1;
  let mut failure_function = vec![0usize; pattern_length];
  while i <= end_i {
    if pattern[i] == pattern[j] {
      failure_function[i] = j + 1;
      i += 1;
      j += 1;
    } else if j == 0 {
      failure_function[i] = 0;
      i += 1;
    } else {
      j = failure_function[j - 1];
    }
  }
  failure_function
}
