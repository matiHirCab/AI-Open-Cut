//! Optimal operation assignments, independent of font resolution and replay.

pub(super) struct Assignment {
    pub alternatives: Vec<Vec<Option<usize>>>,
    pub canonical: Vec<Option<usize>>,
}

pub(super) fn optimal_assignments(weights: &[Vec<Option<i64>>]) -> Assignment {
    let rows = weights.len();
    let old = weights.first().map_or(0, Vec::len);
    let size = rows + old;
    // Real rows can always use dummy columns; dummy rows complete the square.
    // Thus a forbidden edge can never be needed by an optimal assignment.
    let costs: Vec<Vec<i64>> = (0..size)
        .map(|row| {
            (0..size)
                .map(|column| {
                    if row < rows && column < old {
                        weights[row][column].map_or(1_000_000, |weight| -weight)
                    } else {
                        0
                    }
                })
                .collect()
        })
        .collect();
    let (mut owners, u, v) = minimum_assignment(&costs);
    let equality: Vec<Vec<usize>> = (0..size)
        .map(|row| {
            (0..size)
                .filter(|&column| costs[row][column] == u[row + 1] + v[column + 1])
                .collect()
        })
        .collect();

    // Collapse each matched column onto its owner row. A zero-cost edge can
    // occur in an optimum iff it is matched or belongs to an alternating cycle.
    // Keep these global alternatives before canonicalization locks any pairs.
    let mut reachable = vec![vec![false; size]; size];
    for (row, edges) in equality.iter().enumerate() {
        reachable[row][row] = true;
        for &column in edges {
            reachable[row][owners[column].unwrap()] = true;
        }
    }
    for via in 0..size {
        let destinations = reachable[via].clone();
        for row in &mut reachable {
            if row[via] {
                for (reachable, destination) in row.iter_mut().zip(&destinations) {
                    *reachable |= destination;
                }
            }
        }
    }
    let alternatives = (0..rows)
        .map(|row| {
            let mut allowed = vec![];
            for &column in &equality[row] {
                if reachable[owners[column].unwrap()][row] {
                    if column < old {
                        allowed.push(Some(column));
                    } else if allowed.last() != Some(&None) {
                        allowed.push(None);
                    }
                }
            }
            allowed
        })
        .collect();

    // Lexicographically first real pairs, with unmatched after every old row.
    // Reassignment stays within the equality graph, preserving the optimum.
    for (row, edges) in equality.iter().enumerate().take(rows) {
        for &column in edges {
            if owners[column].is_some_and(|owner| owner < row) {
                continue;
            }
            let current = owners.iter().position(|owner| *owner == Some(row)).unwrap();
            let mut candidate = owners.clone();
            candidate[current] = None;
            let displaced = candidate[column].replace(row);
            let mut seen: Vec<_> = candidate
                .iter()
                .map(|owner| owner.is_some_and(|owner| owner <= row))
                .collect();
            if displaced
                .is_none_or(|displaced| augment(displaced, &equality, &mut seen, &mut candidate))
            {
                owners = candidate;
                break;
            }
        }
    }
    let mut canonical = vec![None; rows];
    for (column, owner) in owners.into_iter().enumerate().take(old) {
        if let Some(row) = owner.filter(|&row| row < rows) {
            canonical[row] = Some(column);
        }
    }
    Assignment {
        alternatives,
        canonical,
    }
}

fn augment(
    row: usize,
    edges: &[Vec<usize>],
    seen: &mut [bool],
    owners: &mut [Option<usize>],
) -> bool {
    for &column in &edges[row] {
        if seen[column] {
            continue;
        }
        seen[column] = true;
        if owners[column].is_none_or(|owner| augment(owner, edges, seen, owners)) {
            owners[column] = Some(row);
            return true;
        }
    }
    false
}

// Hungarian shortest augmenting paths with integer dual potentials. The
// operation limit bounds the matrix to 200 x 200; this solver is cubic.
fn minimum_assignment(cost: &[Vec<i64>]) -> (Vec<Option<usize>>, Vec<i64>, Vec<i64>) {
    let size = cost.len();
    let mut u = vec![0; size + 1];
    let mut v = vec![0; size + 1];
    let mut owner = vec![0; size + 1];
    let mut previous = vec![0; size + 1];
    for row in 1..=size {
        owner[0] = row;
        let mut column = 0;
        let mut distance = vec![i64::MAX; size + 1];
        let mut seen = vec![false; size + 1];
        loop {
            seen[column] = true;
            let current = owner[column];
            let mut delta = i64::MAX;
            let mut next = 0;
            for target in 1..=size {
                if !seen[target] {
                    let reduced = cost[current - 1][target - 1] - u[current] - v[target];
                    if reduced < distance[target] {
                        distance[target] = reduced;
                        previous[target] = column;
                    }
                    if distance[target] < delta {
                        delta = distance[target];
                        next = target;
                    }
                }
            }
            for target in 0..=size {
                if seen[target] {
                    u[owner[target]] += delta;
                    v[target] -= delta;
                } else {
                    distance[target] -= delta;
                }
            }
            column = next;
            if owner[column] == 0 {
                break;
            }
        }
        loop {
            let next = previous[column];
            owner[column] = owner[next];
            column = next;
            if column == 0 {
                break;
            }
        }
    }
    (
        owner.into_iter().skip(1).map(|row| Some(row - 1)).collect(),
        u,
        v,
    )
}

#[cfg(test)]
mod matching_tests {
    use super::*;
    use std::collections::BTreeSet;

    fn enumerate(
        weights: &[Vec<Option<i64>>],
        chosen: &mut Vec<Option<usize>>,
        all: &mut Vec<Vec<Option<usize>>>,
    ) {
        if chosen.len() == weights.len() {
            all.push(chosen.clone());
            return;
        }
        let row = chosen.len();
        chosen.push(None);
        enumerate(weights, chosen, all);
        chosen.pop();
        for column in 0..weights[row].len() {
            if weights[row][column].is_some() && !chosen.contains(&Some(column)) {
                chosen.push(Some(column));
                enumerate(weights, chosen, all);
                chosen.pop();
            }
        }
    }

    fn optima(weights: &[Vec<Option<i64>>]) -> Vec<Vec<Option<usize>>> {
        let mut all = vec![];
        enumerate(weights, &mut vec![], &mut all);
        // Independent lexicographic objective, not the solver's scalar costs.
        let score = |assignment: &[Option<usize>]| {
            (
                assignment
                    .iter()
                    .enumerate()
                    .filter(|(row, column)| column.is_some_and(|c| weights[*row][c].unwrap() > 1))
                    .count(),
                assignment.iter().flatten().count(),
            )
        };
        let best = all.iter().map(|a| score(a)).max().unwrap();
        all.retain(|a| score(a) == best);
        all
    }

    // A small independent chronological model: each operation inherits only
    // when its selector agrees with the preceding operation. Retention of that
    // same binding is equivalent; explicit resolution always remains distinct.
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Outcome {
        Inherit(u8),
        Retain(u8),
        Resolve,
    }

    fn step(
        row: usize,
        old: Option<usize>,
        mode: usize,
        prefix: Option<(usize, u8)>,
    ) -> (Outcome, (usize, u8)) {
        let selector = if mode == 0 { 0 } else { row % 2 };
        let inherited = prefix
            .filter(|(previous, _)| *previous == selector)
            .map(|(_, binding)| binding);
        let outcome = match old {
            Some(old) if mode != 2 || old % 2 == selector => {
                let binding = if mode == 0 { 0 } else { old as u8 };
                if inherited == Some(binding) {
                    Outcome::Inherit(binding)
                } else {
                    Outcome::Retain(binding)
                }
            }
            Some(_) => Outcome::Resolve,
            None => inherited.map_or(Outcome::Resolve, Outcome::Inherit),
        };
        let binding = match outcome {
            Outcome::Inherit(b) | Outcome::Retain(b) => b,
            Outcome::Resolve => 0,
        };
        (outcome, (selector, binding))
    }

    #[test]
    fn weighted_alternatives_and_chronological_outcomes_match_exhaustive_assignments() {
        for rows in 1..=3 {
            for columns in 1..=3 {
                for graph in 0..3_usize.pow((rows * columns) as u32) {
                    let mut digits = graph;
                    let weights: Vec<Vec<_>> = (0..rows)
                        .map(|_| {
                            (0..columns)
                                .map(|_| {
                                    let kind = digits % 3;
                                    digits /= 3;
                                    match kind {
                                        0 => None,
                                        1 => Some(1),
                                        _ => Some(rows as i64 + 2),
                                    }
                                })
                                .collect()
                        })
                        .collect();
                    let all = optima(&weights);
                    let actual = optimal_assignments(&weights);
                    let first = all
                        .iter()
                        .min_by_key(|a| {
                            a.iter()
                                .map(|old| old.unwrap_or(columns))
                                .collect::<Vec<_>>()
                        })
                        .unwrap();
                    assert_eq!(&actual.canonical, first, "canonical graph {graph}");
                    for row in 0..rows {
                        let expected: BTreeSet<_> = all.iter().map(|a| a[row]).collect();
                        assert_eq!(
                            actual.alternatives[row]
                                .iter()
                                .copied()
                                .collect::<BTreeSet<_>>(),
                            expected
                        );
                    }
                    for mode in 0..3 {
                        let traces: Vec<_> = all
                            .iter()
                            .map(|assignment| {
                                let mut prefix = None;
                                assignment
                                    .iter()
                                    .enumerate()
                                    .map(|(row, &old)| {
                                        let (outcome, next) = step(row, old, mode, prefix);
                                        prefix = Some(next);
                                        outcome
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .collect();
                        let expected_ambiguity = traces.iter().any(|trace| trace != &traces[0]);
                        let mut prefix = None;
                        let mut ambiguous = false;
                        for row in 0..rows {
                            let (expected, next) = step(row, actual.canonical[row], mode, prefix);
                            if actual.alternatives[row]
                                .iter()
                                .any(|&old| step(row, old, mode, prefix).0 != expected)
                            {
                                ambiguous = true;
                                break;
                            }
                            prefix = Some(next);
                        }
                        assert_eq!(
                            ambiguous, expected_ambiguity,
                            "chronological graph {graph}, mode {mode}"
                        );
                    }
                }
            }
        }
    }
}
