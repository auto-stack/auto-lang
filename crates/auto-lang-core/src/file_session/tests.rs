use super::{FileAccessError, FileMutation::*, FileSession, FileWritePermissions};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;

const WRITE_ALL: FileWritePermissions = FileWritePermissions {
    create: true,
    copy: true,
    rename: true,
    delete: true,
};

#[test]
fn authored_media_paths_use_the_source_directory_and_the_live_grant() {
    let fixture = tempfile::tempdir().unwrap();
    let source = fixture.path().join("src/front");
    let granted = fixture.path().join("tests/fixtures");
    let outside = fixture.path().join("tests/other");
    for directory in [&source, &granted, &outside] {
        fs::create_dir_all(directory).unwrap();
    }
    fs::write(granted.join("scenic.png"), b"fixture").unwrap();
    fs::write(outside.join("scenic.png"), b"outside").unwrap();

    let host = FileSession::for_media_source(&source).unwrap();
    let sample = Path::new("../../tests/fixtures/scenic.png");
    assert!(matches!(host.open_media(sample), Err(FileAccessError::NoReadGrant)));
    host.grant_read(&granted).unwrap();
    let viewer = host.fork_read().unwrap();
    assert!(!viewer.open_media(sample).unwrap().is_empty());
    assert!(!viewer.open_media(Path::new("../../tests/fixtures")).unwrap().is_empty());
    assert!(!viewer.open_media(Path::new("scenic.png")).unwrap().is_empty());
    assert!(matches!(
        viewer.open_media(Path::new("../../tests/other/scenic.png")),
        Err(FileAccessError::OutsideRoot)
    ));
    assert!(viewer.list(Path::new("../../tests/fixtures")).is_err());

    viewer.grant_read(&outside).unwrap();
    assert!(matches!(viewer.open_media(sample), Err(FileAccessError::OutsideRoot)));
    viewer.revoke();
    assert!(matches!(viewer.open_media(sample), Err(FileAccessError::NoReadGrant)));

    let ordinary = FileSession::default();
    ordinary.grant_read(&granted).unwrap();
    assert!(matches!(ordinary.open_media(sample), Err(FileAccessError::OutsideRoot)));
}

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

#[cfg(any(windows, unix))]
#[test]
fn deferred_file_read_is_bound_to_the_issuing_grant() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("root");
    let album = root.join("album");
    let supported = album.join("风景.png");
    let outside = fixture.path().join("outside");
    let outside_image = outside.join("风景.png");
    fs::create_dir_all(&album).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::write(&supported, b"selected image").unwrap();
    fs::write(&outside_image, b"different image").unwrap();
    let session = Arc::new(FileSession::default());

    assert!(matches!(
        session.read_ticket(&supported),
        Err(FileAccessError::NoReadGrant)
    ));
    session.grant_read(&root).unwrap();
    assert!(matches!(
        session.read_ticket(&outside_image),
        Err(FileAccessError::OutsideRoot)
    ));
    let ticket = session.read_ticket(&supported).unwrap();
    let ticket = std::thread::spawn(move || {
        assert_eq!(ticket.read(14).unwrap(), b"selected image");
        assert!(matches!(ticket.read(13), Err(FileAccessError::Io(error)) if error.kind() == ErrorKind::InvalidData));
        ticket
    })
    .join()
    .unwrap();

    let held_album = root.join("held");
    fs::rename(&album, &held_album).unwrap();
    make_directory_link(&outside, &album);
    assert!(ticket.read(64).is_err());
    #[cfg(windows)]
    fs::remove_dir(&album).unwrap();
    #[cfg(unix)]
    fs::remove_file(&album).unwrap();
    fs::rename(&held_album, &album).unwrap();
    assert_eq!(ticket.read(64).unwrap(), b"selected image");

    session.revoke();
    assert!(matches!(ticket.read(64), Err(FileAccessError::NoReadGrant)));
    session.grant_read(&root).unwrap();
    assert!(matches!(ticket.read(64), Err(FileAccessError::NoReadGrant)));
    let new_ticket = session.read_ticket(&supported).unwrap();
    assert_eq!(new_ticket.read(64).unwrap(), b"selected image");
    session.close();
    assert!(matches!(new_ticket.read(64), Err(FileAccessError::Closed)));
    drop(session);
    assert!(matches!(new_ticket.read(64), Err(FileAccessError::Closed)));
}

#[test]
fn viewer_workers_read_original_and_rendition_through_the_same_grant() {
    use auto_media_core::image_pipeline::{
        media_http_response, parse_media_path, MediaAssetError, MediaReadSource, MediaRuntime,
        MAX_IMAGE_FILE_BYTES,
    };
    use std::time::{Duration, Instant};

    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("scenic.png");
    let original = include_bytes!("../../../auto-media-core/tests/fixtures/scenic-320x240.png");
    fs::write(&path, original).unwrap();
    let access = Arc::new(FileSession::default());
    access.grant_read(fixture.path()).unwrap();
    let ticket = access.read_ticket(&path).unwrap();
    let runtime = MediaRuntime::new(Duration::from_secs(2));
    let session =
        runtime.open_read_session(vec![MediaReadSource::new("scenic.png".into(), move || {
            ticket
                .read(MAX_IMAGE_FILE_BYTES)
                .map(Arc::from)
                .map_err(|error| MediaAssetError::Rejected(error.to_string()))
        })]);
    assert!(!session.is_empty());
    let wait_media = || {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let uri = runtime.session_uri(&session);
            let response = media_http_response(runtime.registry(), "GET", &uri, None);
            if !uri.is_empty() && response.status != 503 {
                break (uri, response);
            }
            assert!(Instant::now() < deadline, "media worker did not settle");
            std::thread::sleep(Duration::from_millis(5));
        }
    };

    let (original_uri, response) = wait_media();
    assert_eq!(runtime.session_names(&session), ["scenic.png"]);
    assert_eq!(response.status, 200);
    assert_eq!(response.body.as_deref(), Some(original.as_slice()));
    runtime.request_view(&session, 0, 64, 48, 90);
    let (rendition_uri, response) = wait_media();
    assert_ne!(original_uri, rendition_uri);
    assert_eq!(response.status, 200);
    assert_eq!(response.body.as_deref(), Some(original.as_slice()));
    let (asset, _) = parse_media_path(&rendition_uri).unwrap();
    let (width, height, _) = runtime.worker_pool().decoded_pixels(asset).unwrap();
    assert_eq!((width, height), (64, 48));

    access.revoke();
    runtime.request_view(&session, 0, 32, 24, 90);
    let (denied_uri, response) = wait_media();
    assert_ne!(rendition_uri, denied_uri);
    assert_eq!(response.status, 422);
    assert!(response.body.is_none());
    assert!(runtime.close_session(&session));
    assert!(runtime.session_uri(&session).is_empty());
    runtime.shutdown();
}

#[test]
fn private_media_pixels_follow_the_issuing_grant_and_current_ticket() {
    use auto_media_core::image_pipeline::{
        media_uri, parse_media_path, MediaAssetTicket, MediaSurfaceStatus,
    };
    use std::time::{Duration, Instant};

    let fixture = tempfile::tempdir().unwrap();
    fs::write(
        fixture.path().join("scenic.png"),
        include_bytes!("../../../auto-media-core/tests/fixtures/scenic-320x240.png"),
    ).unwrap();
    let owner = FileSession::default();
    owner.grant_read(fixture.path()).unwrap();
    let neighbor = owner.fork_read().unwrap();
    let session = owner.open_media(Path::new("scenic.png")).unwrap();
    let current_uri = || owner.with_media_runtime(|runtime| runtime.session_uri(&session)).unwrap();
    let wait_for_render = || {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let uri = current_uri();
            if let Some((_, width, height, rgba)) = owner.media_render(&uri) {
                assert_eq!(rgba.len(), width as usize * height as usize * 4);
                break (uri, width, height);
            }
            assert!(Instant::now() < deadline, "private media did not decode");
            std::thread::sleep(Duration::from_millis(5));
        }
    };

    let (uri, width, height) = wait_for_render();
    assert_eq!((width, height), (320, 240));
    assert!(matches!(owner.media_status(&uri), Some(MediaSurfaceStatus::Ready)));
    assert!(neighbor.media_render(&uri).is_none());
    let (id, revision) = parse_media_path(&uri).unwrap();
    let wrong = media_uri(MediaAssetTicket { id, revision: revision + 1 });
    assert!(owner.media_status(&wrong).is_none());

    owner.with_media_runtime(|runtime| runtime.request_view(&session, 0, 64, 48, 90)).unwrap();
    let (replacement, width, height) = wait_for_render();
    assert_ne!(replacement, uri);
    assert_eq!((width, height), (64, 48));
    assert!(owner.media_render(&uri).is_none());
    owner.revoke();
    assert!(owner.media_status(&replacement).is_none());
    assert!(owner.media_render(&replacement).is_none());
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
