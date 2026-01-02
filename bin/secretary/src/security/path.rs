use std::collections::HashSet;
use std::ffi::CString;
use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

static SENSITIVE_INODES: OnceLock<HashSet<u64>> = OnceLock::new();

const BLOCKED_COMPONENTS: &[&str] = &[
    ".env",
    ".git",
    ".ssh",
    ".aws",
    ".gnupg",
    ".gpg",
    ".netrc",
    ".npmrc",
    ".pypirc",
    ".docker",
    ".kube",
    ".config/gh",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    "credentials",
    "secrets",
];

const BLOCKED_EXTENSIONS: &[&str] = &[".pem", ".key", ".p12", ".pfx"];

pub fn init_sensitive_inodes() {
    SENSITIVE_INODES.get_or_init(|| {
        let mut inodes = HashSet::new();

        let sensitive_paths = [
            dirs::home_dir().map(|h| h.join(".ssh")),
            dirs::home_dir().map(|h| h.join(".gnupg")),
            dirs::home_dir().map(|h| h.join(".aws")),
            dirs::home_dir().map(|h| h.join(".netrc")),
            dirs::home_dir().map(|h| h.join(".npmrc")),
            dirs::home_dir().map(|h| h.join(".pypirc")),
            dirs::home_dir().map(|h| h.join(".docker")),
            dirs::home_dir().map(|h| h.join(".kube")),
            dirs::home_dir().map(|h| h.join(".config/gh")),
        ];

        for path_opt in sensitive_paths.iter().flatten() {
            if let Ok(metadata) = std::fs::metadata(path_opt) {
                use std::os::unix::fs::MetadataExt;
                inodes.insert(metadata.ino());
            }
        }

        if let Some(home) = dirs::home_dir() {
            if let Ok(entries) = std::fs::read_dir(home.join(".ssh")) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        use std::os::unix::fs::MetadataExt;
                        inodes.insert(metadata.ino());
                    }
                }
            }
        }

        inodes
    });
}

pub fn validate_and_open(path: &Path) -> Result<(File, PathBuf)> {
    let normalized = normalize_path(path)?;

    if has_parent_escape(path)? {
        return Err(Error::new(
            ErrorKind::PermissionDenied,
            "Path traversal detected: path escapes current directory",
        ));
    }

    check_blocked_components(&normalized)?;

    let (file, resolved_path) = open_path_safe(&normalized)?;

    if let Some(inodes) = SENSITIVE_INODES.get() {
        let metadata = file.metadata()?;
        use std::os::unix::fs::MetadataExt;
        if inodes.contains(&metadata.ino()) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Access denied: file matches sensitive inode (possible hardlink escape)",
            ));
        }
    }

    Ok((file, resolved_path))
}

fn open_path_safe(path: &Path) -> Result<(File, PathBuf)> {
    let components: Vec<_> = path.components().collect();

    if components.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "Empty path"));
    }

    let (start_fd, start_idx, mut resolved) = if let Component::RootDir = components[0] {
        let fd = openat_nofollow(libc::AT_FDCWD, Path::new("/"), true)?;
        (fd, 1, PathBuf::from("/"))
    } else {
        let cwd = std::env::current_dir()?;
        let fd = openat_nofollow(libc::AT_FDCWD, &cwd, true)?;
        (fd, 0, cwd)
    };

    let mut current_fd = start_fd;

    for (i, component) in components[start_idx..].iter().enumerate() {
        let is_last = i == components[start_idx..].len() - 1;

        match component {
            Component::Normal(name) => {
                let name_path = Path::new(name);

                check_blocked_components(name_path)?;

                let new_fd = openat_nofollow(current_fd, name_path, !is_last)?;

                if current_fd != start_fd || start_idx > 0 {
                    unsafe { libc::close(current_fd) };
                }

                current_fd = new_fd;
                resolved.push(name);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(Error::new(
                    ErrorKind::PermissionDenied,
                    "Parent directory traversal not allowed in path",
                ));
            }
            Component::RootDir | Component::Prefix(_) => {}
        }
    }

    let file = unsafe { File::from_raw_fd(current_fd) };

    Ok((file, resolved))
}

fn openat_nofollow(dir_fd: RawFd, path: &Path, is_dir: bool) -> Result<RawFd> {
    let path_cstr = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        Error::new(ErrorKind::InvalidInput, "Path contains null byte")
    })?;

    let mut flags = libc::O_NOFOLLOW | libc::O_CLOEXEC;

    if is_dir {
        flags |= libc::O_RDONLY | libc::O_DIRECTORY;
    } else {
        flags |= libc::O_RDONLY;
    }

    let fd = unsafe { libc::openat(dir_fd, path_cstr.as_ptr(), flags) };

    if fd < 0 {
        let err = Error::last_os_error();
        if err.raw_os_error() == Some(libc::ELOOP) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                "Symbolic link detected: symlinks are not allowed",
            ));
        }
        return Err(err);
    }

    Ok(fd)
}

fn check_blocked_components(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    for blocked in BLOCKED_COMPONENTS {
        if path_str.contains(blocked) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                format!("Access denied: blocked path component '{}'", blocked),
            ));
        }
    }

    for ext in BLOCKED_EXTENSIONS {
        if path_str.ends_with(ext) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                format!("Access denied: blocked file extension '{}'", ext),
            ));
        }
    }

    Ok(())
}

fn normalize_path(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(p) => normalized.push(p.as_os_str()),
            Component::RootDir => normalized.push("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            Component::Normal(c) => normalized.push(c),
        }
    }

    if normalized.as_os_str().is_empty() {
        normalized.push(".");
    }

    Ok(normalized)
}

fn has_parent_escape(path: &Path) -> Result<bool> {
    let cwd = std::env::current_dir()?;

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    let mut depth: i32 = 0;
    let mut in_cwd = true;

    for component in absolute.components() {
        match component {
            Component::RootDir => {
                depth = 0;
                in_cwd = false;
            }
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 && in_cwd {
                    return Ok(true);
                }
            }
            Component::Normal(_) => {
                depth += 1;
            }
            _ => {}
        }
    }

    let normalized = normalize_path(&absolute)?;
    if !normalized.starts_with(&cwd) && !path.is_absolute() {
        return Ok(true);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_init_sensitive_inodes() {
        init_sensitive_inodes();
        assert!(SENSITIVE_INODES.get().is_some());
    }

    #[test]
    fn test_blocked_components() {
        assert!(check_blocked_components(Path::new(".git")).is_err());
        assert!(check_blocked_components(Path::new(".ssh")).is_err());
        assert!(check_blocked_components(Path::new(".env")).is_err());
        assert!(check_blocked_components(Path::new("normal_file.txt")).is_ok());
    }

    #[test]
    fn test_blocked_extensions() {
        assert!(check_blocked_components(Path::new("key.pem")).is_err());
        assert!(check_blocked_components(Path::new("cert.key")).is_err());
        assert!(check_blocked_components(Path::new("file.txt")).is_ok());
    }

    #[test]
    fn test_normalize_path() {
        let normalized = normalize_path(Path::new("./foo/../bar")).unwrap();
        assert_eq!(normalized, PathBuf::from("bar"));

        let normalized = normalize_path(Path::new("/foo/./bar")).unwrap();
        assert_eq!(normalized, PathBuf::from("/foo/bar"));
    }

    #[test]
    fn test_validate_and_open_regular_file() {
        let temp = TempDir::new().unwrap();
        let canonical_temp = temp.path().canonicalize().unwrap();
        let file_path = canonical_temp.join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        init_sensitive_inodes();

        let result = validate_and_open(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_and_open_blocked_path() {
        let temp = TempDir::new().unwrap();
        let canonical_temp = temp.path().canonicalize().unwrap();
        let git_path = canonical_temp.join(".git");
        fs::create_dir(&git_path).unwrap();
        let file_path = git_path.join("config");
        fs::write(&file_path, "test").unwrap();

        init_sensitive_inodes();

        let result = validate_and_open(&file_path);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_blocked() {
        let temp = TempDir::new().unwrap();
        let canonical_temp = temp.path().canonicalize().unwrap();
        let real_file = canonical_temp.join("real.txt");
        fs::write(&real_file, "content").unwrap();

        let symlink_path = canonical_temp.join("link.txt");
        std::os::unix::fs::symlink(&real_file, &symlink_path).unwrap();

        init_sensitive_inodes();

        let result = validate_and_open(&symlink_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parent_escape_detection() {
        assert!(has_parent_escape(Path::new("../../../etc/passwd")).unwrap());
        assert!(!has_parent_escape(Path::new("./subdir/file.txt")).unwrap());
    }
}
