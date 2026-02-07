mod bulk_rename;
mod history;
mod navigator;
mod operations;
mod search;
mod windows;

pub use bulk_rename::{BulkRenamer, CaseScope, CaseTransform, ExtensionAction, RenamePattern, RenamePreview};
pub use history::{HistoryManager, HistoryRecord, Operation};
pub use navigator::Navigator;
pub use operations::{Clipboard, ClipboardOp, perform_copy, perform_delete, perform_mkdir, perform_move, perform_rename};
pub use search::{collect_files_in_directory, search_files_sync, ContentSearcher, SearchConfig, SearchResult, SearchProgress};
pub use windows::{DriveInfo, WindowsDrives};

// Test modules are included in each respective module file
