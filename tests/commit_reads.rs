//! US2 integration tests: recent-commits list (count, ordering, fields).

mod common;

#[test]
fn caps_at_ten_newest_first_with_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = common::init_repo(tmp.path());

    for i in 0..15 {
        common::write_file(tmp.path(), &format!("f{i}.txt"), &i.to_string());
        common::commit_all(&repo, &format!("commit {i}"));
    }

    let opened = git_monitor::git::repo::open_repo(tmp.path()).unwrap();
    let snap = git_monitor::git::repo::read_snapshot(&opened).unwrap();

    assert_eq!(snap.commits.len(), 10, "should cap at 10");

    let newest = &snap.commits[0];
    assert_eq!(newest.summary, "commit 14", "newest first");
    assert!(!newest.short_hash.is_empty());
    assert_eq!(newest.author, "Test");
    assert!(!newest.relative_date.is_empty());
}

#[test]
fn fewer_than_ten_returns_all() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = common::init_repo(tmp.path());

    for i in 0..3 {
        common::write_file(tmp.path(), &format!("f{i}.txt"), &i.to_string());
        common::commit_all(&repo, &format!("commit {i}"));
    }

    let opened = git_monitor::git::repo::open_repo(tmp.path()).unwrap();
    let snap = git_monitor::git::repo::read_snapshot(&opened).unwrap();

    assert_eq!(snap.commits.len(), 3);
}
