//! Comprehensive unit tests for history/undo-redo functionality.

use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_path(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    // ============================================================================
    // HistoryRecord Tests
    // ============================================================================

    mod history_record {
        use super::*;

        #[test]
        fn test_history_record_creation() {
            let operation = Operation::Copy {
                source: create_test_path("/src/file.txt"),
                destination: create_test_path("/dst/file.txt"),
            };

            let record = HistoryRecord::new(operation.clone());

            assert_eq!(record.operation, operation);
        }

        #[test]
        fn test_history_record_timestamp() {
            let before = chrono::Utc::now();
            let operation = Operation::CreateDir {
                path: create_test_path("/test/dir"),
            };

            let record = HistoryRecord::new(operation);
            let after = chrono::Utc::now();

            assert!(record.timestamp >= before);
            assert!(record.timestamp <= after);
        }
    }

    // ============================================================================
    // Operation Description Tests
    // ============================================================================

    mod operation_description {
        use super::*;

        #[test]
        fn test_description_copy() {
            let op = Operation::Copy {
                source: create_test_path("/src/file.txt"),
                destination: create_test_path("/dst/file.txt"),
            };

            let desc = op.description();

            assert!(desc.contains("Copy"));
            assert!(desc.contains("file.txt"));
        }

        #[test]
        fn test_description_move() {
            let op = Operation::Move {
                original_path: create_test_path("/old/file.txt"),
                new_path: create_test_path("/new/file.txt"),
            };

            let desc = op.description();

            assert!(desc.contains("Move"));
        }

        #[test]
        fn test_description_delete() {
            let op = Operation::Delete {
                path: create_test_path("/test/file.txt"),
                was_directory: false,
                backup_path: None,
            };

            let desc = op.description();

            assert!(desc.contains("Delete"));
        }

        #[test]
        fn test_description_create_dir() {
            let op = Operation::CreateDir {
                path: create_test_path("/test/newdir"),
            };

            let desc = op.description();

            assert!(desc.contains("CreateDir"));
        }

        #[test]
        fn test_description_rename() {
            let op = Operation::Rename {
                original_path: create_test_path("/test/old.txt"),
                new_path: create_test_path("/test/new.txt"),
            };

            let desc = op.description();

            assert!(desc.contains("Rename"));
        }
    }

    // ============================================================================
    // HistoryManager Initialization Tests
    // ============================================================================

    mod initialization {
        use super::*;

        #[test]
        fn test_new_manager() {
            let manager = HistoryManager::new().unwrap();

            assert!(!manager.can_undo());
            assert!(!manager.can_redo());
            assert_eq!(manager.undo_count(), 0);
            assert_eq!(manager.redo_count(), 0);
        }

        #[test]
        fn test_default_manager() {
            let manager = HistoryManager::default();

            assert!(!manager.can_undo());
            assert!(!manager.can_redo());
        }

        #[test]
        fn test_backup_dir_created() {
            let manager = HistoryManager::new().unwrap();

            assert!(manager.backup_dir.exists());
        }

        #[test]
        fn test_clear_empty_manager() {
            let mut manager = HistoryManager::new().unwrap();

            manager.clear();

            assert!(!manager.can_undo());
            assert!(!manager.can_redo());
        }
    }

    // ============================================================================
    // Recording Operations Tests
    // ============================================================================

    mod recording {
        use super::*;

        #[test]
        fn test_record_copy() {
            let mut manager = HistoryManager::new().unwrap();

            let op = Operation::Copy {
                source: create_test_path("/src/file.txt"),
                destination: create_test_path("/dst/file.txt"),
            };

            manager.record(op.clone());

            assert!(manager.can_undo());
            assert!(!manager.can_redo());
            assert_eq!(manager.undo_count(), 1);
        }

        #[test]
        fn test_record_multiple_operations() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir3"),
            });

            assert_eq!(manager.undo_count(), 3);
            assert!(!manager.can_redo());
        }

        #[test]
        fn test_record_clears_redo_stack() {
            let mut manager = HistoryManager::new().unwrap();

            // Record and undo
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            let _ = manager.undo();

            assert!(manager.can_redo());

            // Record new operation
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });

            // Redo should be cleared
            assert!(!manager.can_redo());
            assert_eq!(manager.undo_count(), 1);
        }

        #[test]
        fn test_record_max_history_size() {
            let mut manager = HistoryManager::new().unwrap();
            manager.max_history_size = 5;

            for i in 0..10 {
                manager.record(Operation::CreateDir {
                    path: create_test_path(&format!("/test/dir{}", i)),
                });
            }

            // Should only keep max_history_size operations
            assert_eq!(manager.undo_count(), 5);
        }
    }

    // ============================================================================
    // Peek Operations Tests
    // ============================================================================

    mod peek {
        use super::*;

        #[test]
        fn test_peek_undo_empty() {
            let manager = HistoryManager::new().unwrap();

            assert!(manager.peek_undo().is_none());
        }

        #[test]
        fn test_peek_undo_single() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir"),
            });

            let desc = manager.peek_undo().unwrap();

            assert!(desc.contains("CreateDir"));
        }

        #[test]
        fn test_peek_undo_multiple() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });

            // Should peek the most recent
            let desc = manager.peek_undo().unwrap();
            assert!(desc.contains("dir2"));
        }

        #[test]
        fn test_peek_redo_empty() {
            let manager = HistoryManager::new().unwrap();

            assert!(manager.peek_redo().is_none());
        }

        #[test]
        fn test_peek_redo_after_undo() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir"),
            });
            let _ = manager.undo();

            assert!(manager.peek_redo().is_some());
        }
    }

    // ============================================================================
    // Undo/Redo Stack Management Tests
    // ============================================================================

    mod stack_management {
        use super::*;

        #[test]
        fn test_undo_empty_stack() {
            let mut manager = HistoryManager::new().unwrap();

            let result = manager.undo();

            assert!(result.is_err());
        }

        #[test]
        fn test_redo_empty_stack() {
            let mut manager = HistoryManager::new().unwrap();

            let result = manager.redo();

            assert!(result.is_err());
        }

        #[test]
        fn test_undo_single_operation() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir"),
            });

            assert!(manager.can_undo());

            let op = manager.undo().unwrap();

            assert!(!manager.can_undo());
            assert!(manager.can_redo());
            assert!(matches!(op, Operation::CreateDir { .. }));
        }

        #[test]
        fn test_undo_multiple_operations() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });

            assert_eq!(manager.undo_count(), 2);

            let _ = manager.undo();
            assert_eq!(manager.undo_count(), 1);

            let _ = manager.undo();
            assert_eq!(manager.undo_count(), 0);
        }

        #[test]
        fn test_redo_after_undo() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir"),
            });

            let _ = manager.undo();
            assert!(manager.can_redo());

            let op = manager.redo().unwrap();

            assert!(manager.can_undo());
            assert!(!manager.can_redo());
            assert!(matches!(op, Operation::CreateDir { .. }));
        }

        #[test]
        fn test_multiple_undo_redo_cycles() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir"),
            });

            // Undo
            let _ = manager.undo();
            assert!(!manager.can_undo());
            assert!(manager.can_redo());

            // Redo
            let _ = manager.redo();
            assert!(manager.can_undo());
            assert!(!manager.can_redo());

            // Undo again
            let _ = manager.undo();
            assert!(!manager.can_undo());
            assert!(manager.can_redo());

            // Redo again
            let _ = manager.redo();
            assert!(manager.can_undo());
            assert!(!manager.can_redo());
        }

        #[test]
        fn test_undo_redo_order_lifo() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir3"),
            });

            // Should undo in reverse order
            if let Operation::CreateDir { path } = manager.undo().unwrap() {
                assert!(path.to_string_lossy().contains("dir3"));
            }

            if let Operation::CreateDir { path } = manager.undo().unwrap() {
                assert!(path.to_string_lossy().contains("dir2"));
            }

            if let Operation::CreateDir { path } = manager.undo().unwrap() {
                assert!(path.to_string_lossy().contains("dir1"));
            }
        }
    }

    // ============================================================================
    // Backup Creation Tests
    // ============================================================================

    mod backup_tests {
        use super::*;
        use std::fs::write;
        use std::io::Write;

        #[test]
        fn test_create_backup_file() {
            let temp_dir = TempDir::new().unwrap();
            let manager = HistoryManager::new().unwrap();
            let test_file = temp_dir.path().join("test.txt");

            write(&test_file, b"test content").unwrap();

            let backup = manager.create_backup(&test_file).unwrap();

            assert!(backup.exists());
            assert!(backup.starts_with(manager.backup_dir));
        }

        #[test]
        fn test_create_backup_directory() {
            let temp_dir = TempDir::new().unwrap();
            let manager = HistoryManager::new().unwrap();
            let test_dir = temp_dir.path().join("test_dir");

            std::fs::create_dir(&test_dir).unwrap();
            write(test_dir.join("file.txt"), b"content").unwrap();

            let backup = manager.create_backup(&test_dir).unwrap();

            assert!(backup.exists());
            assert!(backup.is_dir());
        }

        #[test]
        fn test_backup_unique_names() {
            let temp_dir = TempDir::new().unwrap();
            let manager = HistoryManager::new().unwrap();
            let test_file1 = temp_dir.path().join("test1.txt");
            let test_file2 = temp_dir.path().join("test2.txt");

            write(&test_file1, b"content1").unwrap();
            write(&test_file2, b"content2").unwrap();

            let backup1 = manager.create_backup(&test_file1).unwrap();
            let backup2 = manager.create_backup(&test_file2).unwrap();

            // Backups should have unique names
            assert_ne!(backup1, backup2);
        }

        #[test]
        fn test_backup_nonexistent_file() {
            let temp_dir = TempDir::new().unwrap();
            let manager = HistoryManager::new().unwrap();
            let test_file = temp_dir.path().join("nonexistent.txt");

            let result = manager.create_backup(&test_file);

            assert!(result.is_err());
        }
    }

    // ============================================================================
    // Backup Cleanup Tests
    // ============================================================================

    mod cleanup_tests {
        use super::*;
        use std::fs::write;
        use std::time::Duration;

        #[test]
        fn test_cleanup_old_backups() {
            let manager = HistoryManager::new().unwrap();

            // This test is limited as we can't easily create old backups
            // Just verify the method runs without error
            let result = manager.cleanup_old_backups(24);

            assert!(result.is_ok());
        }

        #[test]
        fn test_cleanup_recent_backups() {
            let temp_dir = TempDir::new().unwrap();
            let manager = HistoryManager::new().unwrap();
            let test_file = temp_dir.path().join("test.txt");

            write(&test_file, b"content").unwrap();

            let _backup = manager.create_backup(&test_file).unwrap();

            // Cleanup with short timeframe - shouldn't remove anything
            let result = manager.cleanup_old_backups(0);

            assert!(result.is_ok());

            // Backup dir should still exist
            assert!(manager.backup_dir.exists());
        }
    }

    // ============================================================================
    // Clear History Tests
    // ============================================================================

    mod clear_tests {
        use super::*;

        #[test]
        fn test_clear_undo_stack() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });

            assert_eq!(manager.undo_count(), 2);

            manager.clear();

            assert_eq!(manager.undo_count(), 0);
        }

        #[test]
        fn test_clear_redo_stack() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir"),
            });
            let _ = manager.undo();

            assert!(manager.can_redo());

            manager.clear();

            assert!(!manager.can_redo());
        }

        #[test]
        fn test_clear_both_stacks() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            let _ = manager.undo();
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });

            manager.clear();

            assert!(!manager.can_undo());
            assert!(!manager.can_redo());
        }
    }

    // ============================================================================
    // Operation Type Tests
    // ============================================================================

    mod operation_types {
        use super::*;

        #[test]
        fn test_all_operation_types() {
            let operations = vec![
                Operation::Copy {
                    source: create_test_path("/src/file.txt"),
                    destination: create_test_path("/dst/file.txt"),
                },
                Operation::Move {
                    original_path: create_test_path("/old/file.txt"),
                    new_path: create_test_path("/new/file.txt"),
                },
                Operation::Delete {
                    path: create_test_path("/test/file.txt"),
                    was_directory: false,
                    backup_path: None,
                },
                Operation::CreateDir {
                    path: create_test_path("/test/dir"),
                },
                Operation::Rename {
                    original_path: create_test_path("/test/old.txt"),
                    new_path: create_test_path("/test/new.txt"),
                },
            ];

            for op in operations {
                // Should be able to create description for all types
                let desc = op.description();
                assert!(!desc.is_empty());

                // Should be serializable
                let serialized = serde_json::to_string(&op).unwrap();
                assert!(!serialized.is_empty());
            }
        }
    }

    // ============================================================================
    // Edge Cases and Error Handling
    // ============================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_undo_with_backup_path() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::Delete {
                path: create_test_path("/test/file.txt"),
                was_directory: false,
                backup_path: Some(create_test_path("/backup/file.txt")),
            });

            assert!(manager.can_undo());

            let op = manager.undo().unwrap();

            if let Operation::Delete { backup_path, .. } = op {
                assert!(backup_path.is_some());
            } else {
                panic!("Expected Delete operation");
            }
        }

        #[test]
        fn test_undo_directory_delete() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::Delete {
                path: create_test_path("/test/dir"),
                was_directory: true,
                backup_path: None,
            });

            assert!(manager.can_undo());
        }

        #[test]
        fn test_concurrent_undo_operations() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });

            let _ = manager.undo();
            assert_eq!(manager.undo_count(), 1);

            let _ = manager.undo();
            assert_eq!(manager.undo_count(), 0);

            // Should handle undoing empty stack gracefully
            let result = manager.undo();
            assert!(result.is_err());
        }

        #[test]
        fn test_record_after_undo_clears_redo() {
            let mut manager = HistoryManager::new().unwrap();

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir1"),
            });
            let _ = manager.undo();

            assert!(manager.can_redo());

            manager.record(Operation::CreateDir {
                path: create_test_path("/test/dir2"),
            });

            assert!(!manager.can_redo());
        }

        #[test]
        fn test_long_operation_chain() {
            let mut manager = HistoryManager::new().unwrap();

            for i in 0..50 {
                manager.record(Operation::CreateDir {
                    path: create_test_path(&format!("/test/dir{}", i)),
                });
            }

            assert_eq!(manager.undo_count(), 50);

            // Undo all
            for _ in 0..50 {
                assert!(manager.can_undo());
                let _ = manager.undo();
            }

            assert!(!manager.can_undo());
            assert_eq!(manager.redo_count(), 50);
        }
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    mod serialization {
        use super::*;

        #[test]
        fn test_serialize_operation() {
            let op = Operation::Copy {
                source: create_test_path("/src/file.txt"),
                destination: create_test_path("/dst/file.txt"),
            };

            let serialized = serde_json::to_string(&op).unwrap();
            let deserialized: Operation = serde_json::from_str(&serialized).unwrap();

            assert!(matches!(deserialized, Operation::Copy { .. }));
        }

        #[test]
        fn test_serialize_all_operations() {
            let operations = vec![
                Operation::Copy {
                    source: create_test_path("/src/file.txt"),
                    destination: create_test_path("/dst/file.txt"),
                },
                Operation::Move {
                    original_path: create_test_path("/old/file.txt"),
                    new_path: create_test_path("/new/file.txt"),
                },
                Operation::Delete {
                    path: create_test_path("/test/file.txt"),
                    was_directory: false,
                    backup_path: Some(create_test_path("/backup/file.txt")),
                },
                Operation::CreateDir {
                    path: create_test_path("/test/dir"),
                },
                Operation::Rename {
                    original_path: create_test_path("/test/old.txt"),
                    new_path: create_test_path("/test/new.txt"),
                },
            ];

            for op in operations {
                let serialized = serde_json::to_string(&op).unwrap();
                let deserialized: Operation = serde_json::from_str(&serialized).unwrap();

                // Round-trip should preserve operation type
                match (&op, &deserialized) {
                    (Operation::Copy { .. }, Operation::Copy { .. }) => {}
                    (Operation::Move { .. }, Operation::Move { .. }) => {}
                    (Operation::Delete { .. }, Operation::Delete { .. }) => {}
                    (Operation::CreateDir { .. }, Operation::CreateDir { .. }) => {}
                    (Operation::Rename { .. }, Operation::Rename { .. }) => {}
                    _ => panic!("Operation type mismatch after serialization"),
                }
            }
        }
    }
}
