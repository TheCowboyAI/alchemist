//! I/O-related events for file operations and data persistence
//!
//! These events handle:
//! - File loading and saving
//! - Import/export operations
//! - Auto-save functionality
//! - Backup and recovery
//! - Network operations (future)

use bevy::prelude::*;
use std::path::PathBuf;

/// Event for loading a JSON file
///
/// ## Producers
/// - File menu "Open" action
/// - Drag-and-drop handler
/// - Recent files menu
///
/// ## Consumers
/// - `file_loading_system` - Reads and parses file
/// - `ui_feedback_system` - Shows loading indicator
#[derive(Event)]
pub struct LoadJsonFileEvent {
    pub file_path: String,
}

/// Event for saving a JSON file
#[derive(Event)]
pub struct SaveJsonFileEvent {
    pub file_path: String,
}

/// Event for exporting graph
#[derive(Event)]
pub struct ExportGraphEvent {
    pub format: ExportFormat,
    pub file_path: String,
}

#[derive(Clone)]
pub enum ExportFormat {
    Json,
    Graphviz,
    Svg,
    Png,
}

/// Event for importing graph
#[derive(Event)]
pub struct ImportGraphEvent {
    pub format: ImportFormat,
    pub file_path: String,
}

#[derive(Clone)]
pub enum ImportFormat {
    Json,
    Graphviz,
    GraphML,
}

/// Event for auto-save trigger
#[derive(Event)]
pub struct AutoSaveEvent;

/// Event for file operation completion
#[derive(Event)]
pub struct FileOperationCompleteEvent {
    pub operation: FileOperation,
    pub success: bool,
    pub message: String,
}

#[derive(Clone)]
pub enum FileOperation {
    Load,
    Save,
    Export,
    Import,
    AutoSave,
}

/// Event for saving graph state
#[derive(Event)]
pub struct SaveGraphEvent {
    pub path: String,
}

/// Event for loading graph state
#[derive(Event)]
pub struct LoadGraphEvent {
    pub path: String,
}

/// Event for creating a backup
///
/// ## Producers
/// - Auto-backup timer
/// - Manual backup action
/// - Before risky operations
///
/// ## Consumers
/// - `backup_system` - Creates backup files
#[derive(Event)]
pub struct CreateBackupEvent {
    pub backup_type: BackupType,
}

#[derive(Clone)]
pub enum BackupType {
    Auto,
    Manual,
    PreOperation(String), // Description of operation
}

/// Event for file watching updates
///
/// ## Producers
/// - File system watcher
/// - External file modifications
///
/// ## Consumers
/// - `file_sync_system` - Handles external changes
#[derive(Event)]
pub struct FileChangedExternallyEvent {
    pub file_path: PathBuf,
    pub change_type: FileChangeType,
}

#[derive(Clone)]
pub enum FileChangeType {
    Modified,
    Deleted,
    Renamed(PathBuf), // New path
}

/// Event for batch file operations
///
/// ## Producers
/// - Project management UI
/// - Migration tools
///
/// ## Consumers
/// - `batch_io_system` - Processes multiple files
#[derive(Event)]
pub struct BatchFileOperationEvent {
    pub operations: Vec<FileOperation>,
}

/// Event for file validation
///
/// ## Producers
/// - Before file load
/// - Import validation
///
/// ## Consumers
/// - `file_validation_system` - Validates file format
#[derive(Event)]
pub struct ValidateFileEvent {
    pub file_path: PathBuf,
    pub validation_level: ValidationLevel,
}

#[derive(Clone, Copy)]
pub enum ValidationLevel {
    Quick,    // Just check file header/version
    Standard, // Check structure
    Deep,     // Full validation including references
}

/// Event for recent files update
///
/// ## Producers
/// - Successful file operations
/// - File menu system
///
/// ## Consumers
/// - `recent_files_system` - Updates recent files list
#[derive(Event)]
pub struct UpdateRecentFilesEvent {
    pub file_path: PathBuf,
    pub operation: RecentFileOperation,
}

#[derive(Clone)]
pub enum RecentFileOperation {
    Add,
    Remove,
    MoveToTop,
}

/// Event for project operations
///
/// ## Producers
/// - Project menu actions
/// - Project manager UI
///
/// ## Consumers
/// - `project_system` - Manages project files
#[derive(Event)]
pub struct ProjectOperationEvent {
    pub operation: ProjectOperation,
}

#[derive(Clone)]
pub enum ProjectOperation {
    Create { name: String, path: PathBuf },
    Open(PathBuf),
    Close,
    AddFile(PathBuf),
    RemoveFile(PathBuf),
}

/// Event for template operations
///
/// ## Producers
/// - New file dialog
/// - Template gallery
///
/// ## Consumers
/// - `template_system` - Creates from templates
#[derive(Event)]
pub struct LoadTemplateEvent {
    pub template_name: String,
    pub target_path: Option<PathBuf>,
}

/// Event for file recovery
///
/// ## Producers
/// - Crash recovery on startup
/// - Recovery menu action
///
/// ## Consumers
/// - `recovery_system` - Attempts file recovery
#[derive(Event)]
pub struct RecoverFileEvent {
    pub recovery_type: RecoveryType,
}

#[derive(Clone)]
pub enum RecoveryType {
    AutoSave,
    Backup(PathBuf),
    CrashRecovery,
}
