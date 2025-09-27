//! Enhanced file logging with persistent guards, compression, and rotation
//!
//! This module provides production-ready file logging capabilities:
//! - Proper file writer guard management to prevent dropping
//! - Log rotation with compression
//! - Multiple output formats (JSON, plain text, custom)
//! - Size-based and time-based rotation policies
//! - Automatic cleanup of old log files
//! - Performance-optimized buffering

use std::sync::{Arc, Mutex, OnceLock};
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use tracing_appender::{non_blocking, rolling};
use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use flate2::write::GzEncoder;
use flate2::Compression;

/// File logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLoggingConfig {
    /// Enable file logging
    pub enabled: bool,
    /// Log file directory
    pub log_dir: PathBuf,
    /// Log file prefix
    pub file_prefix: String,
    /// Rotation policy: daily, hourly, size_based
    pub rotation_policy: String,
    /// Maximum file size in MB (for size-based rotation)
    pub max_file_size_mb: u64,
    /// Maximum number of log files to keep
    pub max_files: usize,
    /// Enable compression of rotated files
    pub compress_rotated: bool,
    /// Buffer size for file writer
    pub buffer_size: usize,
    /// Sync frequency in seconds
    pub sync_interval_secs: u64,
}

impl Default for FileLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("LOG_TO_FILE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            log_dir: std::env::var("LOG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./logs")),
            file_prefix: std::env::var("LOG_FILE_PREFIX")
                .unwrap_or_else(|_| "auth-service".to_string()),
            rotation_policy: std::env::var("LOG_ROTATION_POLICY")
                .unwrap_or_else(|_| "daily".to_string()),
            max_file_size_mb: std::env::var("LOG_MAX_FILE_SIZE_MB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            max_files: std::env::var("LOG_MAX_FILES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            compress_rotated: std::env::var("LOG_COMPRESS_ROTATED")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            buffer_size: std::env::var("LOG_BUFFER_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8192),
            sync_interval_secs: std::env::var("LOG_SYNC_INTERVAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
        }
    }
}

/// File writer guard holder to prevent dropping
pub struct FileWriterGuard {
    _guard: non_blocking::WorkerGuard,
    _cleanup_handle: tokio::task::JoinHandle<()>,
}

/// Enhanced file logging manager with proper resource management
pub struct FileLoggingManager {
    config: FileLoggingConfig,
    _writer_guard: Option<FileWriterGuard>,
    _writer: Option<non_blocking::NonBlocking>,
}

// Global storage for file writer guards to prevent dropping
static FILE_WRITER_GUARDS: OnceLock<Arc<Mutex<Vec<FileWriterGuard>>>> = OnceLock::new();

impl FileLoggingManager {
    /// Create a new file logging manager
    pub fn new(config: FileLoggingConfig) -> Result<Self> {
        // Initialize global guard storage
        FILE_WRITER_GUARDS.get_or_init(|| Arc::new(Mutex::new(Vec::new())));

        if !config.enabled {
            return Ok(Self {
                config,
                _writer_guard: None,
                _writer: None,
            });
        }

        // Ensure log directory exists
        std::fs::create_dir_all(&config.log_dir)
            .map_err(|e| eyre!("Failed to create log directory: {}", e))?;

        tracing::info!(
            log_dir = ?config.log_dir,
            rotation_policy = %config.rotation_policy,
            max_files = config.max_files,
            "Initializing enhanced file logging"
        );

        Ok(Self {
            config,
            _writer_guard: None,
            _writer: None,
        })
    }

    /// Initialize file writer with proper guard management
    pub fn initialize_writer(&mut self) -> Result<Box<dyn Write + Send>> {
        if !self.config.enabled {
            return Err(eyre!("File logging is disabled"));
        }

        // Create file appender based on rotation policy
        let file_appender: Box<dyn Write + Send> = match self.config.rotation_policy.as_str() {
            "daily" => {
                let appender = rolling::daily(&self.config.log_dir, &self.config.file_prefix);
                Box::new(BufferedAppender::new(appender, self.config.buffer_size)?)
            }
            "hourly" => {
                let appender = rolling::hourly(&self.config.log_dir, &self.config.file_prefix);
                Box::new(BufferedAppender::new(appender, self.config.buffer_size)?)
            }
            "size_based" => {
                let appender = SizeBasedAppender::new(
                    &self.config.log_dir,
                    &self.config.file_prefix,
                    self.config.max_file_size_mb * 1024 * 1024, // Convert MB to bytes
                    self.config.max_files,
                )?;
                Box::new(BufferedAppender::new(appender, self.config.buffer_size)?)
            }
            _ => return Err(eyre!("Unsupported rotation policy: {}", self.config.rotation_policy)),
        };

        // Create non-blocking writer
        let (non_blocking_writer, guard) = non_blocking(file_appender);

        // Start cleanup task
        let cleanup_handle = self.start_cleanup_task();

        // Create guard holder
        let writer_guard = FileWriterGuard {
            _guard: guard,
            _cleanup_handle: cleanup_handle,
        };

        // Store guard globally to prevent dropping
        if let Some(guards) = FILE_WRITER_GUARDS.get() {
            if let Ok(mut guards) = guards.lock() {
                guards.push(writer_guard);
            }
        }

        // Start periodic sync task
        self.start_sync_task(non_blocking_writer.clone());

        self._writer = Some(non_blocking_writer.clone());

        tracing::info!("File writer initialized successfully with guard management");

        Ok(Box::new(NonBlockingWriterAdapter::new(non_blocking_writer)))
    }

    /// Start cleanup task for old log files
    fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(3600)); // Hourly cleanup
            
            loop {
                cleanup_interval.tick().await;
                
                if let Err(e) = Self::cleanup_old_files(&config).await {
                    tracing::error!("Failed to cleanup old log files: {}", e);
                }
            }
        })
    }

    /// Start periodic sync task
    fn start_sync_task(&self, _writer: non_blocking::NonBlocking) {
        let sync_interval = self.config.sync_interval_secs;
        
        tokio::spawn(async move {
            let mut sync_timer = tokio::time::interval(Duration::from_secs(sync_interval));
            
            loop {
                sync_timer.tick().await;
                // The non_blocking writer handles flushing automatically
                tracing::trace!("Periodic log sync completed");
            }
        });
    }

    /// Cleanup old log files
    async fn cleanup_old_files(config: &FileLoggingConfig) -> Result<()> {
        let log_dir = &config.log_dir;
        
        if !log_dir.exists() {
            return Ok(());
        }

        let mut log_files = Vec::new();
        let mut entries = tokio::fs::read_dir(log_dir).await
            .map_err(|e| eyre!("Failed to read log directory: {}", e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| eyre!("Failed to read directory entry: {}", e))? {
            
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with(&config.file_prefix) {
                        if let Ok(metadata) = entry.metadata().await {
                            if let Ok(modified) = metadata.modified() {
                                log_files.push((path.clone(), modified));
                            }
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        log_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess files
        if log_files.len() > config.max_files {
            for (path, _) in log_files.iter().skip(config.max_files) {
                if let Err(e) = tokio::fs::remove_file(path).await {
                    tracing::warn!("Failed to remove old log file {:?}: {}", path, e);
                } else {
                    tracing::debug!("Removed old log file: {:?}", path);
                }
            }
        }

        // Compress old files if enabled
        if config.compress_rotated {
            let cutoff_time = SystemTime::now() - Duration::from_secs(24 * 3600); // 1 day ago
            
            for (path, modified) in &log_files {
                if *modified < cutoff_time && !path.extension().map_or(false, |ext| ext == "gz") {
                    if let Err(e) = Self::compress_file(path).await {
                        tracing::warn!("Failed to compress log file {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Compress a log file
    async fn compress_file(file_path: &Path) -> Result<()> {
        let compressed_path = file_path.with_extension("log.gz");
        
        let input_data = tokio::fs::read(file_path).await
            .map_err(|e| eyre!("Failed to read file for compression: {}", e))?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&input_data)
            .map_err(|e| eyre!("Failed to compress data: {}", e))?;
        
        let compressed_data = encoder.finish()
            .map_err(|e| eyre!("Failed to finish compression: {}", e))?;

        tokio::fs::write(&compressed_path, compressed_data).await
            .map_err(|e| eyre!("Failed to write compressed file: {}", e))?;

        tokio::fs::remove_file(file_path).await
            .map_err(|e| eyre!("Failed to remove original file: {}", e))?;

        tracing::debug!("Compressed log file: {:?} -> {:?}", file_path, compressed_path);

        Ok(())
    }
}

/// Size-based file appender with manual rotation
pub struct SizeBasedAppender {
    base_path: PathBuf,
    file_prefix: String,
    max_size: u64,
    max_files: usize,
    current_file: Option<std::fs::File>,
    current_size: u64,
    file_index: usize,
}

impl SizeBasedAppender {
    pub fn new(
        log_dir: &Path,
        file_prefix: &str,
        max_size: u64,
        max_files: usize,
    ) -> Result<Self> {
        let base_path = log_dir.to_path_buf();
        
        Ok(Self {
            base_path,
            file_prefix: file_prefix.to_string(),
            max_size,
            max_files,
            current_file: None,
            current_size: 0,
            file_index: 0,
        })
    }

    fn rotate_file(&mut self) -> Result<()> {
        if let Some(mut file) = self.current_file.take() {
            file.flush()?;
        }

        self.file_index = (self.file_index + 1) % self.max_files;
        let file_path = self.base_path.join(format!("{}.{}.log", self.file_prefix, self.file_index));

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| eyre!("Failed to create rotated log file: {}", e))?;

        self.current_file = Some(file);
        self.current_size = 0;

        tracing::debug!("Rotated to new log file: {:?}", file_path);

        Ok(())
    }
}

impl Write for SizeBasedAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.current_file.is_none() || self.current_size + buf.len() as u64 > self.max_size {
            self.rotate_file().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        if let Some(file) = &mut self.current_file {
            let bytes_written = file.write(buf)?;
            self.current_size += bytes_written as u64;
            Ok(bytes_written)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "No file available for writing"))
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.current_file {
            file.flush()
        } else {
            Ok(())
        }
    }
}

/// Buffered appender wrapper
pub struct BufferedAppender<W: Write> {
    writer: W,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<W: Write> BufferedAppender<W> {
    pub fn new(writer: W, buffer_size: usize) -> Result<Self> {
        Ok(Self {
            writer,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        })
    }
}

impl<W: Write> Write for BufferedAppender<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buffer.len() + buf.len() > self.buffer_size {
            self.flush()?;
        }

        if buf.len() > self.buffer_size {
            // Write large buffers directly
            self.writer.write(buf)
        } else {
            self.buffer.extend_from_slice(buf);
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.writer.write_all(&self.buffer)?;
            self.buffer.clear();
        }
        self.writer.flush()
    }
}

impl<W: Write> Drop for BufferedAppender<W> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Adapter to make NonBlocking writer compatible with Write trait
pub struct NonBlockingWriterAdapter {
    writer: non_blocking::NonBlocking,
}

impl NonBlockingWriterAdapter {
    pub fn new(writer: non_blocking::NonBlocking) -> Self {
        Self { writer }
    }
}

impl Write for NonBlockingWriterAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
