use crate::prelude::*;
use std::collections::HashMap;

pub(crate) fn validate_unique_inferred_variables(
    mut state: CompilationIntermediate,
) -> CompilationIntermediate {
    // TODO: I am pretty sure this is made redundant by the v3 type system, will need to check if it's still needed.
    // determining if there are any duplicate inferred variables
    // at this point this shouldn't happen, but if it has we need to error out now

    let duplicate_inferred_vars = state
        .known_variable_declarations
        .iter()
        .filter(|&d| d.is_variable())
        .fold(HashMap::<&str, Vec<&Declaration>>::new(), |mut acc, d| {
            acc.entry(&d.name).or_default().push(d);
            acc
        })
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .collect::<Vec<(&str, Vec<&Declaration>)>>();

    for (key, group) in duplicate_inferred_vars {
        let group_message = group
            .iter()
            .map(|d| {
                format!(
                    "\"{}\" on line {}",
                    d.source_file_name,
                    d.source_file_line().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        state.diagnostics.push(
            Diagnostic::from_message(format!(
                "\"{key}\" has had its default value inferred in multiple places: {group_message}"
            ))
            .with_severity(DiagnosticSeverity::Error),
        );
    }

    // Note from da39c71 -> 2.5.0 migration
    // The line below don't do anything therefore no Rust code has been migrated

    // removing all the duplicate keys
    // var duplicateKeys = duplicateInferredVars.Select(g => g.Key);
    // knownVariableDeclarations.Where(d => duplicateKeys.Contains(d.Name));

    state
}
