//! Comprehensive unit tests for bulk rename functionality.

use super::*;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(name: &str) -> PathBuf {
        PathBuf::from("/test").join(name)
    }

    // ============================================================================
    // Simple Replace Pattern Tests
    // ============================================================================

    mod simple_replace {
        use super::*;

        #[test]
        fn test_simple_replace_basic() {
            let pattern = RenamePattern::SimpleReplace {
                find: "old".to_string(),
                replace: "new".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("old_file.txt"),
                create_test_file("another_old.txt"),
            ];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "new_file.txt");
            assert_eq!(previews[1].new_name(), "another_new.txt");
        }

        #[test]
        fn test_simple_replace_multiple_occurrences() {
            let pattern = RenamePattern::SimpleReplace {
                find: "a".to_string(),
                replace: "b".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("banana.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "bbnbnb.txt");
        }

        #[test]
        fn test_simple_replace_empty_find() {
            let pattern = RenamePattern::SimpleReplace {
                find: "".to_string(),
                replace: "prefix".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("file.txt")];

            let previews = renamer.preview(&files);

            // Empty find should return original name
            assert_eq!(previews[0].new_name(), "file.txt");
        }

        #[test]
        fn test_simple_replace_empty_replace() {
            let pattern = RenamePattern::SimpleReplace {
                find: "remove".to_string(),
                replace: "".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("remove_this.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "_this.txt");
        }

        #[test]
        fn test_simple_replace_case_sensitive() {
            let pattern = RenamePattern::SimpleReplace {
                find: "test".to_string(),
                replace: "done".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("test_file.txt"),
                create_test_file("Test_file.txt"),
                create_test_file("TEST_FILE.TXT"),
            ];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "done_file.txt");
            assert_eq!(previews[1].new_name(), "Test_file.txt");
            assert_eq!(previews[2].new_name(), "TEST_FILE.TXT");
        }

        #[test]
        fn test_simple_replace_no_match() {
            let pattern = RenamePattern::SimpleReplace {
                find: "xyz".to_string(),
                replace: "abc".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("unchanged.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "unchanged.txt");
        }

        #[test]
        fn test_simple_replace_with_dots() {
            let pattern = RenamePattern::SimpleReplace {
                find: ".".to_string(),
                replace: "_".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("file.name.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "file_name_txt");
        }
    }

    // ============================================================================
    // Numbered Pattern Tests
    // ============================================================================

    mod numbered_pattern {
        use super::*;

        #[test]
        fn test_numbered_basic() {
            let pattern = RenamePattern::Numbered {
                template: "photo_{n}.jpg".to_string(),
                start: 1,
                pad_width: 0,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("img1.jpg"),
                create_test_file("img2.jpg"),
                create_test_file("img3.jpg"),
            ];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "photo_1.jpg");
            assert_eq!(previews[1].new_name(), "photo_2.jpg");
            assert_eq!(previews[2].new_name(), "photo_3.jpg");
        }

        #[test]
        fn test_numbered_with_padding() {
            let pattern = RenamePattern::Numbered {
                template: "file_{n}.txt".to_string(),
                start: 1,
                pad_width: 3,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("a.txt"),
                create_test_file("b.txt"),
                create_test_file("c.txt"),
                create_test_file("d.txt"),
            ];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "file_001.txt");
            assert_eq!(previews[1].new_name(), "file_002.txt");
            assert_eq!(previews[2].new_name(), "file_003.txt");
            assert_eq!(previews[3].new_name(), "file_004.txt");
        }

        #[test]
        fn test_numbered_custom_start() {
            let pattern = RenamePattern::Numbered {
                template: "chapter_{n}.md".to_string(),
                start: 10,
                pad_width: 2,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("chap1.md"),
                create_test_file("chap2.md"),
            ];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "chapter_10.md");
            assert_eq!(previews[1].new_name(), "chapter_11.md");
        }

        #[test]
        fn test_numbered_multiple_placeholders() {
            let pattern = RenamePattern::Numbered {
                template: "{n}_file_{n}.txt".to_string(),
                start: 1,
                pad_width: 0,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("f.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "1_file_1.txt");
        }

        #[test]
        fn test_numbered_large_numbers() {
            let pattern = RenamePattern::Numbered {
                template: "item_{n}.dat".to_string(),
                start: 9999,
                pad_width: 5,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("x.dat")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "item_09999.dat");
        }

        #[test]
        fn test_numbered_no_extension() {
            let pattern = RenamePattern::Numbered {
                template: "file_{n}".to_string(),
                start: 1,
                pad_width: 0,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("README")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "file_1");
        }
    }

    // ============================================================================
    // Regex Pattern Tests
    // ============================================================================

    mod regex_pattern {
        use super::*;

        #[test]
        fn test_regex_basic_substitution() {
            let pattern = RenamePattern::Regex {
                pattern: r"(.*)\.txt".to_string(),
                replacement: r"$1_backup.txt".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("document.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "document_backup.txt");
        }

        #[test]
        fn test_regex_multiple_groups() {
            let pattern = RenamePattern::Regex {
                pattern: r"(\d{4})-(\d{2})-(\d{2})".to_string(),
                replacement: r"$3-$2-$1".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("2024-01-15_photo.jpg")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "15-01-2024_photo.jpg");
        }

        #[test]
        fn test_regex_remove_pattern() {
            let pattern = RenamePattern::Regex {
                pattern: r"\[.*?\]".to_string(),
                replacement: "".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("file_[tag]_name.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "file__name.txt");
        }

        #[test]
        fn test_regex_case_insensitive() {
            let pattern = RenamePattern::Regex {
                pattern: r"(?i)photo".to_string(),
                replacement: "image".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("photo_1.jpg"),
                create_test_file("PHOTO_2.jpg"),
                create_test_file("Photo_3.jpg"),
            ];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "image_1.jpg");
            assert_eq!(previews[1].new_name(), "image_2.jpg");
            assert_eq!(previews[2].new_name(), "image_3.jpg");
        }

        #[test]
        fn test_regex_invalid_pattern() {
            let pattern = RenamePattern::Regex {
                pattern: r"(?P<unclosed".to_string(),
                replacement: "x".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("test.txt")];

            let previews = renamer.preview(&files);

            assert!(!previews[0].is_valid);
            assert!(previews[0].error.is_some());
        }

        #[test]
        fn test_regex_no_match() {
            let pattern = RenamePattern::Regex {
                pattern: r"\d+".to_string(),
                replacement: "NUM".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("nodigits.txt")];

            let previews = renamer.preview(&files);

            // No match means no change
            assert_eq!(previews[0].new_name(), "nodigits.txt");
        }
    }

    // ============================================================================
    // Case Transform Tests
    // ============================================================================

    mod case_transform {
        use super::*;

        #[test]
        fn test_case_uppercase_name_only() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::Uppercase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("lowercase.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "LOWERCASE.txt");
        }

        #[test]
        fn test_case_lowercase_name_only() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::Lowercase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("UPPERCASE.TXT")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "uppercase.TXT");
        }

        #[test]
        fn test_case_titlecase_name_only() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::TitleCase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("the quick brown fox.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "The Quick Brown Fox.txt");
        }

        #[test]
        fn test_case_sentence_name_only() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::SentenceCase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("HELLO WORLD.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "Hello world.txt");
        }

        #[test]
        fn test_case_toggle_name_only() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::ToggleCase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("HeLlO WoRlD.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "hElLo wOrLd.txt");
        }

        #[test]
        fn test_case_extension_only() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::Uppercase,
                scope: CaseScope::ExtensionOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("document.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "document.TXT");
        }

        #[test]
        fn test_case_entire_name() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::Uppercase,
                scope: CaseScope::EntireName,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("document.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "DOCUMENT.TXT");
        }

        #[test]
        fn test_case_no_extension() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::Uppercase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("README")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "README");
        }

        #[test]
        fn test_case_titlecase_single_word() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::TitleCase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("filename.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "Filename.txt");
        }

        #[test]
        fn test_case_toggle_numbers_unchanged() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::ToggleCase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("file123.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "FILE123.txt");
        }
    }

    // ============================================================================
    // Extension Manipulation Tests
    // ============================================================================

    mod extension_manipulation {
        use super::*;

        #[test]
        fn test_extension_add() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Add,
                new_extension: Some("txt".to_string()),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("README")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "README.txt");
        }

        #[test]
        fn test_extension_add_when_exists() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Add,
                new_extension: Some("txt".to_string()),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("document.txt")];

            let previews = renamer.preview(&files);

            // Should not add if extension exists
            assert_eq!(previews[0].new_name(), "document.txt");
        }

        #[test]
        fn test_extension_remove() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Remove,
                new_extension: None,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("document.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "document");
        }

        #[test]
        fn test_extension_remove_no_extension() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Remove,
                new_extension: None,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("README")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "README");
        }

        #[test]
        fn test_extension_replace() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Replace,
                new_extension: Some("md".to_string()),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("document.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "document.md");
        }

        #[test]
        fn test_extension_replace_no_extension() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Replace,
                new_extension: Some("txt".to_string()),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("README")];

            let previews = renamer.preview(&files);

            // Should add extension if none exists
            assert_eq!(previews[0].new_name(), "README.txt");
        }

        #[test]
        fn test_extension_keep() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Keep,
                new_extension: None,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("document.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "document.txt");
        }

        #[test]
        fn test_extension_multiple_dots() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Replace,
                new_extension: Some("tar".to_string()),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("archive.tar.gz")];

            let previews = renamer.preview(&files);

            // Only replaces the last extension
            assert_eq!(previews[0].new_name(), "archive.tar.tar");
        }

        #[test]
        fn test_extension_add_none_specified() {
            let pattern = RenamePattern::Extension {
                action: ExtensionAction::Add,
                new_extension: None,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("README")];

            let previews = renamer.preview(&files);

            // Should not add if no extension specified
            assert_eq!(previews[0].new_name(), "README");
        }
    }

    // ============================================================================
    // RenamePreview Tests
    // ============================================================================

    mod rename_preview {
        use super::*;

        #[test]
        fn test_preview_basic() {
            let preview = RenamePreview::new(
                PathBuf::from("/test/old.txt"),
                PathBuf::from("/test/new.txt"),
            );

            assert_eq!(preview.old_name(), "old.txt");
            assert_eq!(preview.new_name(), "new.txt");
            assert!(preview.is_valid);
            assert!(preview.error.is_none());
            assert!(preview.accepted);
        }

        #[test]
        fn test_preview_invalid_characters() {
            let preview = RenamePreview::new(
                PathBuf::from("/test/old.txt"),
                PathBuf::from("/test/file/name.txt"),
            );

            assert!(!preview.is_valid);
            assert!(preview.error.is_some());
        }

        #[test]
        fn test_preview_empty_name() {
            let preview = RenamePreview::new(
                PathBuf::from("/test/old.txt"),
                PathBuf::from("/test/"),
            );

            assert!(!preview.is_valid);
        }

        #[test]
        fn test_preview_same_name() {
            let preview = RenamePreview::new(
                PathBuf::from("/test/file.txt"),
                PathBuf::from("/test/file.txt"),
            );

            assert!(!preview.is_valid);
        }

        #[test]
        fn test_preview_special_characters() {
            let preview = RenamePreview::new(
                PathBuf::from("/test/old.txt"),
                PathBuf::from("/test/file:*.txt"),
            );

            assert!(!preview.is_valid);
        }

        #[test]
        fn test_preview_accepted_flag() {
            let mut preview = RenamePreview::new(
                PathBuf::from("/test/old.txt"),
                PathBuf::from("/test/new.txt"),
            );

            assert!(preview.accepted);

            preview.accepted = false;

            assert!(!preview.accepted);
        }
    }

    // ============================================================================
    // Edge Cases and Error Handling
    // ============================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_files_list() {
            let pattern = RenamePattern::SimpleReplace {
                find: "x".to_string(),
                replace: "y".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files: Vec<PathBuf> = vec![];

            let previews = renamer.preview(&files);

            assert!(previews.is_empty());
        }

        #[test]
        fn test_unicode_characters() {
            let pattern = RenamePattern::SimpleReplace {
                find: "日本語".to_string(),
                replace: "한국어".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("file_日本語.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "file_한국어.txt");
        }

        #[test]
        fn test_very_long_filename() {
            let long_name = "a".repeat(255);
            let pattern = RenamePattern::SimpleReplace {
                find: "a".to_string(),
                replace: "b".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![PathBuf::from(format!("/test/{}", long_name))];

            let previews = renamer.preview(&files);

            // Should handle long names
            assert_eq!(previews[0].new_name().len(), long_name.len());
        }

        #[test]
        fn test_leading_dot_filename() {
            let pattern = RenamePattern::SimpleReplace {
                find: "old".to_string(),
                replace: "new".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file(".oldconfig")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), ".newconfig");
        }

        #[test]
        fn test_only_extension_filename() {
            let pattern = RenamePattern::SimpleReplace {
                find: "txt".to_string(),
                replace: "md".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file(".txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), ".md");
        }

        #[test]
        fn test_consecutive_dots() {
            let pattern = RenamePattern::SimpleReplace {
                find: "old".to_string(),
                replace: "new".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("file..old..name.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "file..new..name.txt");
        }

        #[test]
        fn test_spaces_in_filename() {
            let pattern = RenamePattern::SimpleReplace {
                find: " ".to_string(),
                replace: "_".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("my file name.txt")];

            let previews = renamer.preview(&files);

            assert_eq!(previews[0].new_name(), "my_file_name.txt");
        }

        #[test]
        fn test_trailing_spaces() {
            let pattern = RenamePattern::Case {
                transform: CaseTransform::Uppercase,
                scope: CaseScope::NameOnly,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![create_test_file("file  .txt")];

            let previews = renamer.preview(&files);

            // Should preserve trailing spaces in name
            assert_eq!(previews[0].new_name(), "FILE  .txt");
        }
    }

    // ============================================================================
    // Pattern Combination Tests
    // ============================================================================

    mod pattern_combinations {
        use super::*;

        #[test]
        fn test_multiple_files_different_extensions() {
            let pattern = RenamePattern::SimpleReplace {
                find: "old".to_string(),
                replace: "new".to_string(),
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("old_file.txt"),
                create_test_file("old_doc.md"),
                create_test_file("old_image.png"),
            ];

            let previews = renamer.preview(&files);

            assert_eq!(previews.len(), 3);
            assert!(previews[0].new_name().ends_with(".txt"));
            assert!(previews[1].new_name().ends_with(".md"));
            assert!(previews[2].new_name().ends_with(".png"));
        }

        #[test]
        fn test_mixed_directories_and_files() {
            let pattern = RenamePattern::Numbered {
                template: "item_{n}".to_string(),
                start: 1,
                pad_width: 2,
            };
            let renamer = BulkRenamer::new(pattern, PathBuf::from("/test"));

            let files = vec![
                create_test_file("folder/"),
                create_test_file("file.txt"),
            ];

            let previews = renamer.preview(&files);

            // Pattern applies regardless of file type
            assert_eq!(previews.len(), 2);
        }
    }
}
