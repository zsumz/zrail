//! Cargo.lock Git URLs bind one manifest reference to one precise commit.

#[derive(Debug, Eq, PartialEq)]
enum GitReference {
    DefaultBranch,
    Branch(String),
    Tag(String),
    Revision(String),
}

struct LockedGitSource<'a> {
    repository: &'a str,
    reference: GitReference,
    precise: &'a str,
}

pub(super) fn matches_manifest_reference(
    candidate: &str,
    repository: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
) -> Result<bool, String> {
    let Some(locked) = parse_locked_source(candidate)? else {
        return Ok(false);
    };
    if locked.repository != repository {
        return Ok(false);
    }
    let expected = manifest_reference(branch, tag, revision)?;
    if locked.reference != expected {
        return Ok(false);
    }
    match expected {
        GitReference::Revision(revision) => precise_agrees(&revision, locked.precise),
        GitReference::DefaultBranch | GitReference::Branch(_) | GitReference::Tag(_) => Ok(true),
    }
}

fn parse_locked_source(source: &str) -> Result<Option<LockedGitSource<'_>>, String> {
    let Some(source) = source.strip_prefix("git+") else {
        return Ok(None);
    };
    let (address, precise) = source.rsplit_once('#').ok_or_else(|| {
        format!("Cargo.lock Git source {source:?} has no precise commit fragment")
    })?;
    if !valid_precise(precise) {
        return Err(format!(
            "Cargo.lock Git source {source:?} has invalid precise commit {precise:?}"
        ));
    }
    let (repository, query) = address
        .split_once('?')
        .map_or((address, None), |(repository, query)| {
            (repository, Some(query))
        });
    if repository.is_empty() {
        return Err("Cargo.lock Git source has an empty repository URL".into());
    }
    Ok(Some(LockedGitSource {
        repository,
        reference: parse_query(query)?,
        precise,
    }))
}

fn parse_query(query: Option<&str>) -> Result<GitReference, String> {
    let Some(query) = query else {
        return Ok(GitReference::DefaultBranch);
    };
    if query.is_empty() || query.contains('&') {
        return Err(format!(
            "Cargo.lock Git source query {query:?} does not identify exactly one reference"
        ));
    }
    let (key, value) = query.split_once('=').ok_or_else(|| {
        format!("Cargo.lock Git source query {query:?} is missing a reference value")
    })?;
    let key = percent_decode(key)?;
    let value = percent_decode(value)?;
    if value.is_empty() {
        return Err(format!(
            "Cargo.lock Git source query {query:?} has an empty reference"
        ));
    }
    match key.as_str() {
        "branch" => Ok(GitReference::Branch(value)),
        "tag" => Ok(GitReference::Tag(value)),
        "rev" => Ok(GitReference::Revision(value)),
        _ => Err(format!(
            "Cargo.lock Git source query {query:?} has unsupported reference kind {key:?}"
        )),
    }
}

fn manifest_reference(
    branch: Option<&str>,
    tag: Option<&str>,
    revision: Option<&str>,
) -> Result<GitReference, String> {
    match (branch, tag, revision) {
        (None, None, None) => Ok(GitReference::DefaultBranch),
        (Some(branch), None, None) => Ok(GitReference::Branch(branch.into())),
        (None, Some(tag), None) => Ok(GitReference::Tag(tag.into())),
        (None, None, Some(revision)) => Ok(GitReference::Revision(revision.into())),
        _ => Err("manifest Git branch, tag, and rev are not mutually exclusive".into()),
    }
}

fn precise_agrees(revision: &str, precise: &str) -> Result<bool, String> {
    if revision.is_empty() || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "manifest Git rev {revision:?} cannot prove correspondence to precise commit {precise:?}; use a commit hash"
        ));
    }
    Ok(revision.len() <= precise.len()
        && precise.as_bytes()[..revision.len()].eq_ignore_ascii_case(revision.as_bytes()))
}

fn valid_precise(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let high = hex(bytes[index + 1]);
                let low = hex(bytes[index + 2]);
                let (Some(high), Some(low)) = (high, low) else {
                    return Err(format!(
                        "Cargo.lock Git source has invalid escape in {value:?}"
                    ));
                };
                decoded.push((high << 4) | low);
                index += 3;
            }
            b'%' => {
                return Err(format!(
                    "Cargo.lock Git source has truncated escape in {value:?}"
                ));
            }
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded)
        .map_err(|_| format!("Cargo.lock Git source has non-UTF-8 query value {value:?}"))
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
