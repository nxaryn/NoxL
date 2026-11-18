//! Archive operations: create, extract, list.

use std::path::Path;

use anyhow::Result;

/// An archive manager.
pub struct ArchiveManager {
    pub current_archive: Option<String>,
    pub entries: Vec<String>,
}

impl ArchiveManager {
    pub fn new() -> Self {
        Self {
            current_archive: None,
            entries: Vec::new(),
        }
    }

    /// Open an archive and list its contents.
    pub fn open(&mut self, path: &str) -> Result<()> {
        self.current_archive = Some(path.to_string());
        self.entries.clear();

        if path.ends_with(".zip") {
            self.list_zip(path)?;
        } else if path.ends_with(".tar") {
            self.list_tar(path)?;
        } else if path.ends_with(".tar.gz") || path.ends_with(".tgz") {
            self.list_tar_gz(path)?;
        } else if path.ends_with(".gz") {
            self.list_gz(path)?;
        }

        Ok(())
    }

    fn list_zip(&mut self, path: &str) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let mut zip = zip::ZipArchive::new(file)?;
        for i in 0..zip.len() {
            let file = zip.by_index(i)?;
            self.entries.push(file.name().to_string());
        }
        Ok(())
    }

    fn list_tar(&mut self, path: &str) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let mut tar = tar::Archive::new(file);
        for entry in tar.entries()? {
            let entry = entry?;
            self.entries.push(entry.path()?.to_string_lossy().to_string());
        }
        Ok(())
    }

    fn list_tar_gz(&mut self, path: &str) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut tar = tar::Archive::new(decoder);
        for entry in tar.entries()? {
            let entry = entry?;
            self.entries.push(entry.path()?.to_string_lossy().to_string());
        }
        Ok(())
    }

    fn list_gz(&mut self, path: &str) -> Result<()> {
        self.entries.push(path.to_string());
        Ok(())
    }

    /// Extract the current archive to a directory.
    pub fn extract(&self, to: &str) -> Result<()> {
        let path = self.current_archive.as_ref()
            .ok_or_else(|| anyhow::anyhow!("no archive open"))?;

        if path.ends_with(".zip") {
            self.extract_zip(path, to)?;
        } else if path.ends_with(".tar") {
            self.extract_tar(path, to)?;
        } else if path.ends_with(".tar.gz") || path.ends_with(".tgz") {
            self.extract_tar_gz(path, to)?;
        }

        Ok(())
    }

    fn extract_zip(&self, path: &str, to: &str) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let mut zip = zip::ZipArchive::new(file)?;
        zip.extract(to)?;
        Ok(())
    }

    fn extract_tar(&self, path: &str, to: &str) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let mut tar = tar::Archive::new(file);
        tar.unpack(to)?;
        Ok(())
    }

    fn extract_tar_gz(&self, path: &str, to: &str) -> Result<()> {
        let file = std::fs::File::open(path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut tar = tar::Archive::new(decoder);
        tar.unpack(to)?;
        Ok(())
    }

    /// Create a new archive from a directory.
    pub fn create(&self, archive_path: &str, source: &str) -> Result<()> {
        if archive_path.ends_with(".zip") {
            self.create_zip(archive_path, source)?;
        } else if archive_path.ends_with(".tar") {
            self.create_tar(archive_path, source)?;
        } else if archive_path.ends_with(".tar.gz") || archive_path.ends_with(".tgz") {
            self.create_tar_gz(archive_path, source)?;
        }
        Ok(())
    }

    fn create_zip(&self, archive_path: &str, source: &str) -> Result<()> {
        let file = std::fs::File::create(archive_path)?;
        let mut zip = zip::ZipWriter::new(file);
        for entry in walkdir::WalkDir::new(source) {
            let entry = entry?;
            let path = entry.path();
            let name = path.strip_prefix(source)?;
            if path.is_file() {
                zip.start_file(name.to_string_lossy(), zip::write::FileOptions::default())?;
                std::io::copy(&mut std::fs::File::open(path)?, &mut zip)?;
            }
        }
        zip.finish()?;
        Ok(())
    }

    fn create_tar(&self, archive_path: &str, source: &str) -> Result<()> {
        let file = std::fs::File::create(archive_path)?;
        let mut tar = tar::Builder::new(file);
        for entry in walkdir::WalkDir::new(source) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                tar.append_file(path, &mut std::fs::File::open(path)?)?;
            }
        }
        tar.finish()?;
        Ok(())
    }

    fn create_tar_gz(&self, archive_path: &str, source: &str) -> Result<()> {
        let file = std::fs::File::create(archive_path)?;
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(encoder);
        for entry in walkdir::WalkDir::new(source) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                tar.append_file(path, &mut std::fs::File::open(path)?)?;
            }
        }
        tar.finish()?;
        Ok(())
    }
}
