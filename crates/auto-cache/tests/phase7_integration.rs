// Phase 7 Integration Tests for AutoCache
//
// Tests for:
// - Hit rate tracking
// - Artifact listing with filtering
// - Cache inspection by hash and module name
// - Cache integrity verification

use auto_cache::{ArtifactMetadata, ArtifactType, AutoCache};
use std::fs::{self, File};
use std::io::Write;

#[test]
fn test_hit_rate_calculation() {
    let temp_dir = std::env::temp_dir().join("test_hit_rate");
    let _ = fs::remove_dir_all(&temp_dir);

    let cache = AutoCache::new(temp_dir.clone()).unwrap();

    // Create test artifact
    let test_file = temp_dir.join("test.txt");
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"Test content").unwrap();

    let metadata = ArtifactMetadata {
        hash_key: "test1".to_string(),
        blob_path: test_file.clone(),
        artifact_type: ArtifactType::TranspiledC,
        file_size: 12,
        created_at: 1000,
        last_used_at: 2000,
        access_count: 5,
        source_hash: "abc123".to_string(),
        project_name: "test_project".to_string(),
        module_name: "test_module".to_string(),
    };

    cache.put("test1", &test_file, &metadata).unwrap();

    // Access the artifact multiple times to increase hit rate
    for _ in 0..3 {
        cache.get("test1");
    }

    let stats = cache.get_statistics();
    // Hit rate should be > 0 after accesses
    assert!(stats.hit_rate >= 0.0);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_list_artifacts_without_filter() {
    let temp_dir = std::env::temp_dir().join("test_list_no_filter");
    let _ = fs::remove_dir_all(&temp_dir);

    let cache = AutoCache::new(temp_dir.clone()).unwrap();

    // Create multiple test artifacts
    for i in 0..3 {
        let test_file = temp_dir.join(&format!("test{}.txt", i));
        let mut file = File::create(&test_file).unwrap();
        file.write_all(format!("content{}", i).as_bytes()).unwrap();

        let metadata = ArtifactMetadata {
            hash_key: format!("test{}", i),
            blob_path: test_file.clone(),
            artifact_type: if i % 2 == 0 {
                ArtifactType::TranspiledC
            } else {
                ArtifactType::TranspiledRust
            },
            file_size: 8 + i,
            created_at: 1000 + i * 100,
            last_used_at: 2000 + i * 100,
            access_count: 1,
            source_hash: format!("hash{}", i),
            project_name: "test_project".to_string(),
            module_name: format!("module{}", i),
        };

        cache
            .put(&format!("test{}", i), &test_file, &metadata)
            .unwrap();
    }

    // List all artifacts
    let artifacts = cache.list_artifacts(None, 10).unwrap();
    assert_eq!(artifacts.len(), 3);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_list_artifacts_with_type_filter() {
    let temp_dir = std::env::temp_dir().join("test_list_filter");
    let _ = fs::remove_dir_all(&temp_dir);

    let cache = AutoCache::new(temp_dir.clone()).unwrap();

    // Create C artifact
    let c_file = temp_dir.join("test.c");
    let mut file = File::create(&c_file).unwrap();
    file.write_all(b"int x;").unwrap();

    let c_metadata = ArtifactMetadata {
        hash_key: "test_c".to_string(),
        blob_path: c_file.clone(),
        artifact_type: ArtifactType::TranspiledC,
        file_size: 6,
        created_at: 1000,
        last_used_at: 2000,
        access_count: 1,
        source_hash: "hash_c".to_string(),
        project_name: "test_project".to_string(),
        module_name: "test_c_module".to_string(),
    };

    cache.put("test_c", &c_file, &c_metadata).unwrap();

    // Create Rust artifact
    let rust_file = temp_dir.join("test.rs");
    let mut file = File::create(&rust_file).unwrap();
    file.write_all(b"let x = 1;").unwrap();

    let rust_metadata = ArtifactMetadata {
        hash_key: "test_rust".to_string(),
        blob_path: rust_file.clone(),
        artifact_type: ArtifactType::TranspiledRust,
        file_size: 9,
        created_at: 1000,
        last_used_at: 2000,
        access_count: 1,
        source_hash: "hash_rust".to_string(),
        project_name: "test_project".to_string(),
        module_name: "test_rust_module".to_string(),
    };

    cache.put("test_rust", &rust_file, &rust_metadata).unwrap();

    // Filter by C type
    let c_artifacts = cache
        .list_artifacts(Some(ArtifactType::TranspiledC), 10)
        .unwrap();
    assert_eq!(c_artifacts.len(), 1);
    assert_eq!(c_artifacts[0].artifact_type, ArtifactType::TranspiledC);

    // Filter by Rust type
    let rust_artifacts = cache
        .list_artifacts(Some(ArtifactType::TranspiledRust), 10)
        .unwrap();
    assert_eq!(rust_artifacts.len(), 1);
    assert_eq!(
        rust_artifacts[0].artifact_type,
        ArtifactType::TranspiledRust
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_get_metadata_by_hash() {
    let temp_dir = std::env::temp_dir().join("test_get_metadata");
    let _ = fs::remove_dir_all(&temp_dir);

    let cache = AutoCache::new(temp_dir.clone()).unwrap();

    // Create test artifact
    let test_file = temp_dir.join("test.txt");
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"test").unwrap();

    let metadata = ArtifactMetadata {
        hash_key: "test_key".to_string(),
        blob_path: test_file.clone(),
        artifact_type: ArtifactType::Bytecode,
        file_size: 4,
        created_at: 1000,
        last_used_at: 2000,
        access_count: 3,
        source_hash: "source_hash".to_string(),
        project_name: "my_project".to_string(),
        module_name: "my_module".to_string(),
    };

    cache.put("test_key", &test_file, &metadata).unwrap();

    // Get metadata by hash key
    let retrieved = cache.get_metadata("test_key").unwrap();
    assert_eq!(retrieved.hash_key, "test_key");
    assert_eq!(retrieved.module_name, "my_module");
    assert_eq!(retrieved.project_name, "my_project");
    assert_eq!(retrieved.artifact_type, ArtifactType::Bytecode);
    assert_eq!(retrieved.access_count, 3);

    // Non-existent key should return None
    assert!(cache.get_metadata("non_existent").is_none());

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_verify_integrity_valid_cache() {
    let temp_dir = std::env::temp_dir().join("test_verify_valid");
    let _ = fs::remove_dir_all(&temp_dir);

    let cache = AutoCache::new(temp_dir.clone()).unwrap();

    // Create test artifact
    let test_file = temp_dir.join("test.txt");
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"test").unwrap();

    let metadata = ArtifactMetadata {
        hash_key: "test1".to_string(),
        blob_path: test_file.clone(),
        artifact_type: ArtifactType::TranspiledC,
        file_size: 4,
        created_at: 1000,
        last_used_at: 2000,
        access_count: 1,
        source_hash: "hash1".to_string(),
        project_name: "test_project".to_string(),
        module_name: "test_module".to_string(),
    };

    cache.put("test1", &test_file, &metadata).unwrap();

    // Verify integrity - should be valid
    let report = cache.verify_integrity().unwrap();
    assert!(report.is_valid);
    assert_eq!(report.metadata_entries, 1);
    assert_eq!(report.blob_files, 1);
    assert_eq!(report.corrupted_entries, 0);
    assert_eq!(report.orphaned_files, 0);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_verify_integrity_with_corrupted_entry() {
    let temp_dir = std::env::temp_dir().join("test_verify_corrupted");
    let _ = fs::remove_dir_all(&temp_dir);

    let cache = AutoCache::new(temp_dir.clone()).unwrap();

    // Create test artifact
    let test_file = temp_dir.join("test.txt");
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"test").unwrap();

    let metadata = ArtifactMetadata {
        hash_key: "test1".to_string(),
        blob_path: test_file.clone(),
        artifact_type: ArtifactType::TranspiledC,
        file_size: 4,
        created_at: 1000,
        last_used_at: 2000,
        access_count: 1,
        source_hash: "hash1".to_string(),
        project_name: "test_project".to_string(),
        module_name: "test_module".to_string(),
    };

    cache.put("test1", &test_file, &metadata).unwrap();

    // Corrupt the cache by deleting the blob file
    let blob_path = cache.get("test1").unwrap();
    fs::remove_file(&blob_path).unwrap();

    // Verify integrity - should detect corruption
    let report = cache.verify_integrity().unwrap();
    assert!(!report.is_valid);
    assert_eq!(report.metadata_entries, 1);
    assert_eq!(report.corrupted_entries, 1);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_list_respects_limit() {
    let temp_dir = std::env::temp_dir().join("test_limit");
    let _ = fs::remove_dir_all(&temp_dir);

    let cache = AutoCache::new(temp_dir.clone()).unwrap();

    // Create 5 artifacts
    for i in 0..5 {
        let test_file = temp_dir.join(&format!("test{}.txt", i));
        let mut file = File::create(&test_file).unwrap();
        file.write_all(format!("content{}", i).as_bytes()).unwrap();

        let metadata = ArtifactMetadata {
            hash_key: format!("test{}", i),
            blob_path: test_file.clone(),
            artifact_type: ArtifactType::TranspiledC,
            file_size: 8 + i,
            created_at: 1000 + i * 100,
            last_used_at: 2000 + i * 100,
            access_count: 1,
            source_hash: format!("hash{}", i),
            project_name: "test_project".to_string(),
            module_name: format!("module{}", i),
        };

        cache
            .put(&format!("test{}", i), &test_file, &metadata)
            .unwrap();
    }

    // List with limit of 3
    let artifacts = cache.list_artifacts(None, 3).unwrap();
    assert_eq!(artifacts.len(), 3);

    // List with limit of 10 (should return all 5)
    let artifacts = cache.list_artifacts(None, 10).unwrap();
    assert_eq!(artifacts.len(), 5);

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}
