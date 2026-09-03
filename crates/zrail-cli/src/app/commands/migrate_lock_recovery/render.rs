//! Stable human rendering for migration-base recovery evidence.

use super::RecoveryContext;

pub(super) fn discovery(context: &RecoveryContext, lock_digest: &str) -> String {
    let mut text = String::from("Migration base discovery\n\n");
    push_context(&mut text, context, lock_digest);
    push_candidates(&mut text, &context.candidates);
    if context.candidates.is_empty() {
        text.push_str("\nNo matching revision was found in reachable Git history.\n");
    } else {
        text.push_str(
            "\nNo revision was selected automatically. Run migration with:\n\n  \
             zrail migrate-lock --base <revision> --output <report-path>\n",
        );
    }
    text
}

pub(super) fn mismatch(
    context: &RecoveryContext,
    lock_digest: &str,
    selected_digest: &str,
) -> String {
    let mut text = if context.current_digest == lock_digest {
        String::from(
            "The selected migration base differs from the contract that produced this lock.\n\n",
        )
    } else {
        String::from("The current contract differs from the contract that produced this lock.\n\n")
    };
    push_context(&mut text, context, lock_digest);
    text.push_str("Selected base contract digest: ");
    text.push_str(selected_digest);
    text.push('\n');
    push_candidates(&mut text, &context.candidates);
    text.push_str(
        "\nRun migration from a listed revision containing the lock's contract:\n\n  \
         zrail migrate-lock --base <revision> --output <report-path>\n\n\
         Rediscover candidates with:\n\n  zrail migrate-lock --discover-base\n",
    );
    text
}

fn push_context(text: &mut String, context: &RecoveryContext, lock_digest: &str) {
    text.push_str("Lock contract digest: ");
    text.push_str(lock_digest);
    text.push_str("\nCurrent contract digest: ");
    text.push_str(&context.current_digest);
    text.push('\n');
    match &context.head_digest {
        Ok(head) => {
            text.push_str("HEAD contract digest: ");
            text.push_str(head);
            text.push('\n');
            let contribution = if head == &context.current_digest {
                "no"
            } else if head == lock_digest {
                "yes; HEAD still matches the lock"
            } else {
                "no; the mismatch already exists at HEAD"
            };
            text.push_str("Local uncommitted contract edits contributed: ");
            text.push_str(contribution);
            text.push('\n');
        }
        Err(error) => {
            text.push_str("HEAD contract digest: unavailable (");
            text.push_str(error);
            text.push_str(")\nLocal uncommitted contract edits contributed: unknown\n");
        }
    }
}

fn push_candidates(text: &mut String, candidates: &[String]) {
    text.push_str("Candidate matching revisions:");
    if candidates.is_empty() {
        text.push_str(" none\n");
        return;
    }
    text.push('\n');
    for revision in candidates {
        text.push_str("  ");
        text.push_str(revision);
        text.push('\n');
    }
}
