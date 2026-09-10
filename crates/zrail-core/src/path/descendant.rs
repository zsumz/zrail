//! Bounded prefix reachability for proving that unvisited directories are irrelevant.

use super::{MAX_GLOB_PATTERN_BYTES, MAX_GLOB_PATTERN_SEGMENTS, matches_component, segments};

/// Whether a glob can match any strict descendant of a normalized directory.
///
/// Uses the same byte and `**` semantics as [`super::glob_matches`]. A true result
/// means the directory must be inspected or explicitly excluded before claiming
/// a complete selection. Oversized inputs return an error, never disjointness.
pub fn glob_can_match_descendant(pattern: &str, directory: &str) -> Result<bool, String> {
    let pattern = pattern.trim_matches('/');
    let directory = directory.trim_matches('/');
    if pattern.len() > MAX_GLOB_PATTERN_BYTES || directory.len() > MAX_GLOB_PATTERN_BYTES {
        return Err("glob prefix query exceeds the path-byte safety limit".into());
    }
    let pattern = segments(pattern);
    let directory = segments(directory);
    if pattern.len() > MAX_GLOB_PATTERN_SEGMENTS || directory.len() > MAX_GLOB_PATTERN_SEGMENTS {
        return Err("glob prefix query exceeds the path-segment safety limit".into());
    }
    let mut states = vec![false; pattern.len() + 1];
    states[0] = true;
    close(&pattern, &mut states);
    for component in directory {
        let mut next = vec![false; states.len()];
        for (index, segment) in pattern.iter().enumerate() {
            if states[index] {
                if *segment == "**" {
                    next[index] = true;
                } else if matches_component(segment, component) {
                    next[index + 1] = true;
                }
            }
        }
        close(&pattern, &mut next);
        states = next;
    }
    Ok(states[..pattern.len()].iter().any(|state| *state))
}

fn close(pattern: &[&str], states: &mut [bool]) {
    for (index, segment) in pattern.iter().enumerate() {
        if *segment == "**" && states[index] {
            states[index + 1] = true;
        }
    }
}

#[cfg(test)]
#[path = "descendant_test.rs"]
mod descendant_test;
