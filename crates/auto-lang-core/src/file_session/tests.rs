use super::{FileAccessError, FileMutation::*, FileSession, FileWritePermissions};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

const WRITE_ALL: FileWritePermissions = FileWritePermissions {
    create: true,
    copy: true,
    rename: true,
    delete: true,
};

#[cfg(any(windows, unix))]
#[test]
fn file_open_target_requires_a_live_grant_and_a_regular_entry() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("root");
    let outside = fixture.path().join("outside");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&outside).unwrap();
    let document = root.join("资料.txt");
    let private = outside.join("private.txt");
    fs::write(&document, b"authorized").unwrap();
    fs::write(&private, b"private").unwrap();
    let link = root.join("linked");
    make_directory_link(&outside, &link);

    let session = FileSession::default();
    let unexpected_launch = |_: &Path| -> std::io::Result<()> { panic!("unauthorized launch") };
    assert!(matches!(
        session.with_open_target(&document, unexpected_launch),
        Err(FileAccessError::NoReadGrant)
    ));
    session.grant_read(&root).unwrap();
    for denied in [&private, &root.join("../outside/private.txt"), &link.join("private.txt"), &root] {
        assert!(session.with_open_target(denied, unexpected_launch).is_err());
    }
    let content = session.with_open_target(&document, |path| fs::read(path)).unwrap();
    assert_eq!(content, b"authorized");
    session.revoke();
    assert!(session.with_open_target(&document, unexpected_launch).is_err());
    session.grant_read(&root).unwrap();
    session.close();
    assert!(matches!(
        session.with_open_target(&document, unexpected_launch),
        Err(FileAccessError::Closed)
    ));

    #[cfg(windows)]
    fs::remove_dir(&link).unwrap();
    #[cfg(unix)]
    fs::remove_file(&link).unwrap();
}

#[test]
fn file_mutations_follow_the_live_grant_and_the_requested_operation() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("root");
    let outside = fixture.path().join("outside");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&outside).unwrap();
    let inside_file = root.join("目标.txt");
    let copied = root.join("copy.txt");
    let outside_file = outside.join("data.txt");
    fs::write(&outside_file, b"private").unwrap();

    let session = FileSession::default();
    assert!(matches!(
        session.mutate(CreateFile(&inside_file)),
        Err(FileAccessError::NoReadGrant)
    ));
    session.grant_read(&root).unwrap();
    let full = [
        CreateFile(&inside_file),
        CreateDirectory(&inside_file),
        Copy {
            from: &outside_file,
            to: &copied,
        },
        Rename {
            from: &inside_file,
            to: &copied,
        },
        DeleteFile(&inside_file),
        DeleteEmptyDirectory(&root),
    ];
    for command in full {
        assert!(
            matches!(
                session.mutate(command),
                Err(FileAccessError::OperationDenied)
            ),
            "{command:?}"
        );
    }
    assert!(!inside_file.exists());
    session
        .grant(
            &root,
            FileWritePermissions {
                create: true,
                ..Default::default()
            },
        )
        .unwrap();
    session.mutate(CreateFile(&inside_file)).unwrap();
    assert!(matches!(
        session.mutate(Rename {
            from: &inside_file,
            to: &copied
        }),
        Err(FileAccessError::OperationDenied)
    ));
    assert!(inside_file.exists() && !copied.exists());
    session.grant(&root, WRITE_ALL).unwrap();
    for command in [
        Copy {
            from: &outside_file,
            to: &copied,
        },
        Copy {
            from: &inside_file,
            to: &outside_file,
        },
        Rename {
            from: &inside_file,
            to: &outside_file,
        },
        Rename {
            from: &outside_file,
            to: &copied,
        },
        DeleteFile(&outside_file),
        DeleteEmptyDirectory(&root),
        CreateDirectory(&root.join("../outside/new")),
    ] {
        assert!(
            matches!(session.mutate(command), Err(FileAccessError::OutsideRoot)),
            "{command:?}"
        );
    }
    assert_eq!(fs::read(&outside_file).unwrap(), b"private");
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
    assert!(inside_file.exists() && !copied.exists());

    session.revoke();
    assert!(matches!(
        session.mutate(DeleteFile(&inside_file)),
        Err(FileAccessError::NoReadGrant)
    ));
    assert!(inside_file.exists());
    session.grant(&root, WRITE_ALL).unwrap();
    session.mutate(DeleteFile(&inside_file)).unwrap();
    session.close();
    assert!(matches!(
        session.mutate(CreateFile(&inside_file)),
        Err(FileAccessError::Closed)
    ));
    assert!(matches!(
        session.grant(&root, WRITE_ALL),
        Err(FileAccessError::Closed)
    ));
    assert!(!inside_file.exists());
}

#[test]
fn mutations_keep_contents_and_never_replace_an_existing_entry() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let source = root.join("源");
    let child = source.join("nested");
    let target = root.join("copied");
    let renamed = root.join("重命名");
    let moved = source.join("moved.txt");
    let session = FileSession::default();
    session.grant(root, WRITE_ALL).unwrap();

    session.mutate(CreateDirectory(&source)).unwrap();
    session.mutate(CreateDirectory(&child)).unwrap();
    let original = child.join("item.txt");
    session.mutate(CreateFile(&original)).unwrap();
    assert_eq!(fs::metadata(&original).unwrap().len(), 0);
    fs::write(&original, b"real file contents").unwrap();
    assert!(
        matches!(session.mutate(CreateFile(&original)), Err(FileAccessError::Io(error)) if error.kind() == ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(&original).unwrap(), b"real file contents");

    assert!(session
        .mutate(Copy {
            from: &source,
            to: &child.join("recursive")
        })
        .is_err());
    assert!(!child.join("recursive").exists());
    session
        .mutate(Copy {
            from: &source,
            to: &target,
        })
        .unwrap();
    let copied_file = target.join("nested/item.txt");
    assert_eq!(fs::read(&copied_file).unwrap(), b"real file contents");
    fs::write(&copied_file, b"different data").unwrap();
    assert!(session
        .mutate(Copy {
            from: &original,
            to: &copied_file
        })
        .is_err());
    assert!(session
        .mutate(Rename {
            from: &original,
            to: &copied_file
        })
        .is_err());
    assert_eq!(fs::read(&copied_file).unwrap(), b"different data");
    assert_eq!(fs::read(&original).unwrap(), b"real file contents");
    assert!(!fs::read_dir(root).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".auto-copy-")));

    session
        .mutate(Rename {
            from: &target,
            to: &renamed,
        })
        .unwrap();
    assert!(!target.exists());
    let renamed_file = renamed.join("nested/item.txt");
    assert_eq!(fs::read(&renamed_file).unwrap(), b"different data");
    session
        .mutate(Rename {
            from: &renamed_file,
            to: &moved,
        })
        .unwrap();
    assert!(!renamed_file.exists());
    assert_eq!(fs::read(&moved).unwrap(), b"different data");

    assert!(session.mutate(DeleteEmptyDirectory(&source)).is_err());
    assert!(original.exists() && moved.exists());
    session.mutate(DeleteFile(&moved)).unwrap();
    session
        .mutate(DeleteEmptyDirectory(&renamed.join("nested")))
        .unwrap();
    session.mutate(DeleteEmptyDirectory(&renamed)).unwrap();
    assert!(!renamed.exists() && !moved.exists() && original.exists());
}

#[cfg(any(windows, unix))]
#[test]
fn file_mutations_refuse_a_linked_parent_and_discard_a_partial_copy() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("grant");
    let outside = fixture.path().join("outside");
    let branch = root.join("branch");
    fs::create_dir_all(&branch).unwrap();
    fs::create_dir(&outside).unwrap();
    let fresh = branch.join("copied-before-link.txt");
    fs::write(&fresh, b"inner").unwrap();
    let mut permissions = fs::metadata(&fresh).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&fresh, permissions).unwrap();
    let outside_file = outside.join("protected.txt");
    fs::write(&outside_file, b"outside").unwrap();
    let link = branch.join("junction");
    make_directory_link(&outside, &link);

    let session = FileSession::default();
    session.grant(&root, WRITE_ALL).unwrap();
    let link_target = link.join("protected.txt");
    let copy_target = root.join("published");
    for command in [
        Copy {
            from: &link_target,
            to: &copy_target,
        },
        Copy {
            from: &fresh,
            to: &link.join("new"),
        },
        Rename {
            from: &link_target,
            to: &copy_target,
        },
        Rename {
            from: &fresh,
            to: &link.join("new"),
        },
        DeleteFile(&link_target),
        CreateFile(&link.join("new")),
        Copy {
            from: &branch,
            to: &copy_target,
        },
    ] {
        assert!(session.mutate(command).is_err(), "{command:?}");
    }
    assert!(!copy_target.exists());
    assert_eq!(fs::read(&outside_file).unwrap(), b"outside");
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
    assert_eq!(fs::read(&fresh).unwrap(), b"inner");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    session
        .mutate(Copy {
            from: &fresh,
            to: &copy_target,
        })
        .unwrap();
    assert_eq!(fs::read(&copy_target).unwrap(), b"inner");
    assert!(fs::metadata(&copy_target).unwrap().permissions().readonly());
}

#[cfg(unix)]
fn make_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(windows)]
fn make_directory_link(target: &Path, link: &Path) {
    let result = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile", "-NonInteractive", "-Command",
            "New-Item -ItemType Junction -Path $env:AUTO_TEST_FILE_LINK -Target $env:AUTO_TEST_FILE_TARGET -ErrorAction Stop | Out-Null",
        ])
        .env("AUTO_TEST_FILE_LINK", link)
        .env("AUTO_TEST_FILE_TARGET", target)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
