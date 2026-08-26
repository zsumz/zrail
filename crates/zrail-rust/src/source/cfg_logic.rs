//! Bounded propositional proofs recognize cfg partitions without guessing target values.

use super::CfgPredicate;

const MAX_PROOF_ATOMS: usize = 10;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Atom {
    Test,
    Feature(String),
    Opaque(String),
}

impl CfgPredicate {
    pub(crate) fn implies(&self, required: &Self) -> bool {
        if matches!(self, Self::False) || matches!(required, Self::True) || self == required {
            return true;
        }
        if structurally_implies(self, required) {
            return true;
        }
        let contradiction = Self::all(vec![self.clone(), Self::not(required.clone())]);
        contradiction.is_satisfiable() == Some(false)
    }

    pub(crate) fn is_satisfiable(&self) -> Option<bool> {
        let mut atoms = Vec::new();
        collect_atoms(self, &mut atoms);
        atoms.sort();
        atoms.dedup();
        if atoms.len() > MAX_PROOF_ATOMS {
            return None;
        }
        Some(
            (0..(1_usize << atoms.len()))
                .filter(|values| valid_assignment(&atoms, *values))
                .any(|values| evaluate(self, &atoms, values)),
        )
    }
}

fn structurally_implies(left: &CfgPredicate, right: &CfgPredicate) -> bool {
    match (left, right) {
        (_, CfgPredicate::All(values)) => {
            values.iter().all(|value| structurally_implies(left, value))
        }
        (_, CfgPredicate::Any(values)) => {
            values.iter().any(|value| structurally_implies(left, value))
        }
        (CfgPredicate::All(values), _) => values
            .iter()
            .any(|value| structurally_implies(value, right)),
        (CfgPredicate::Any(values), _) => values
            .iter()
            .all(|value| structurally_implies(value, right)),
        _ => false,
    }
}

fn collect_atoms(predicate: &CfgPredicate, atoms: &mut Vec<Atom>) {
    match predicate {
        CfgPredicate::Test => atoms.push(Atom::Test),
        CfgPredicate::Feature(feature) => atoms.push(Atom::Feature(feature.clone())),
        CfgPredicate::Opaque(value) => atoms.push(Atom::Opaque(value.clone())),
        CfgPredicate::Not(value) => collect_atoms(value, atoms),
        CfgPredicate::All(values) | CfgPredicate::Any(values) => {
            for value in values {
                collect_atoms(value, atoms);
            }
        }
        CfgPredicate::True | CfgPredicate::False => {}
    }
}

fn evaluate(predicate: &CfgPredicate, atoms: &[Atom], values: usize) -> bool {
    match predicate {
        CfgPredicate::True => true,
        CfgPredicate::False => false,
        CfgPredicate::Test => atom_value(atoms, &Atom::Test, values),
        CfgPredicate::Feature(feature) => {
            atom_value(atoms, &Atom::Feature(feature.clone()), values)
        }
        CfgPredicate::Opaque(value) => atom_value(atoms, &Atom::Opaque(value.clone()), values),
        CfgPredicate::Not(value) => !evaluate(value, atoms, values),
        CfgPredicate::All(items) => items.iter().all(|item| evaluate(item, atoms, values)),
        CfgPredicate::Any(items) => items.iter().any(|item| evaluate(item, atoms, values)),
    }
}

fn atom_value(atoms: &[Atom], atom: &Atom, values: usize) -> bool {
    atoms
        .binary_search(atom)
        .is_ok_and(|index| values & (1_usize << index) != 0)
}

fn valid_assignment(atoms: &[Atom], values: usize) -> bool {
    atoms.iter().enumerate().all(|(left_index, left)| {
        atoms[left_index + 1..].iter().all(|right| {
            !atom_value(atoms, left, values)
                || !atom_value(atoms, right, values)
                || !exclusive_target_values(left, right)
        })
    })
}

fn exclusive_target_values(left: &Atom, right: &Atom) -> bool {
    let (Atom::Opaque(left), Atom::Opaque(right)) = (left, right) else {
        return false;
    };
    let (Some((left_key, left_value)), Some((right_key, right_value))) =
        (left.split_once('='), right.split_once('='))
    else {
        return false;
    };
    left_key == right_key
        && left_value != right_value
        && matches!(
            left_key,
            "panic"
                | "target_abi"
                | "target_arch"
                | "target_endian"
                | "target_env"
                | "target_os"
                | "target_pointer_width"
                | "target_vendor"
        )
}
