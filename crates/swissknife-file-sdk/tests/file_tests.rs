#[cfg(feature = "sftp")]
mod sftp_tests {
    use swissknife_file_sdk::sftp::{
        SftpClient, ListOptions, UploadOptions, DownloadOptions,
    };

    #[test]
    fn test_sftp_client_creation() {
        let client = SftpClient::new(
            "https://sftp-gateway.example.com",
            "api-key-123",
        );
        assert!(true);
    }

    #[test]
    fn test_list_options() {
        let options = ListOptions {
            show_hidden: Some(true),
            recursive: Some(false),
            limit: Some(100),
        };

        assert_eq!(options.show_hidden, Some(true));
        assert_eq!(options.recursive, Some(false));
        assert_eq!(options.limit, Some(100));
    }

    #[test]
    fn test_upload_options() {
        let options = UploadOptions {
            overwrite: Some(true),
            create_dirs: Some(true),
            permissions: Some("0644".to_string()),
        };

        assert_eq!(options.overwrite, Some(true));
        assert_eq!(options.create_dirs, Some(true));
        assert_eq!(options.permissions, Some("0644".to_string()));
    }

    #[test]
    fn test_download_options() {
        let options = DownloadOptions {
            preserve_permissions: Some(true),
            follow_symlinks: Some(false),
        };

        assert_eq!(options.preserve_permissions, Some(true));
        assert_eq!(options.follow_symlinks, Some(false));
    }

    #[test]
    fn test_options_defaults() {
        let list_opts = ListOptions::default();
        let upload_opts = UploadOptions::default();
        let download_opts = DownloadOptions::default();

        assert!(list_opts.show_hidden.is_none());
        assert!(upload_opts.overwrite.is_none());
        assert!(download_opts.preserve_permissions.is_none());
    }
}

#[cfg(feature = "ssh")]
mod ssh_tests {
    use swissknife_file_sdk::ssh::{
        SshClient, ExecOptions, TunnelConfig, TunnelDirection,
    };

    #[test]
    fn test_ssh_client_creation() {
        let client = SshClient::new(
            "https://ssh-gateway.example.com",
            "api-key-456",
        );
        assert!(true);
    }

    #[test]
    fn test_exec_options() {
        let options = ExecOptions {
            timeout: Some(30),
            env: Some(vec![
                ("PATH".to_string(), "/usr/bin".to_string()),
                ("HOME".to_string(), "/home/user".to_string()),
            ]),
            working_dir: Some("/home/user/project".to_string()),
            pty: Some(false),
        };

        assert_eq!(options.timeout, Some(30));
        assert!(options.env.is_some());
        assert_eq!(options.working_dir, Some("/home/user/project".to_string()));
        assert_eq!(options.pty, Some(false));
    }

    #[test]
    fn test_tunnel_config_local() {
        let config = TunnelConfig {
            direction: TunnelDirection::Local,
            local_port: 8080,
            remote_host: "localhost".to_string(),
            remote_port: 80,
        };

        assert!(matches!(config.direction, TunnelDirection::Local));
        assert_eq!(config.local_port, 8080);
        assert_eq!(config.remote_host, "localhost");
        assert_eq!(config.remote_port, 80);
    }

    #[test]
    fn test_tunnel_config_remote() {
        let config = TunnelConfig {
            direction: TunnelDirection::Remote,
            local_port: 3000,
            remote_host: "0.0.0.0".to_string(),
            remote_port: 3000,
        };

        assert!(matches!(config.direction, TunnelDirection::Remote));
    }

    #[test]
    fn test_tunnel_direction_variants() {
        let local = TunnelDirection::Local;
        let remote = TunnelDirection::Remote;

        assert!(matches!(local, TunnelDirection::Local));
        assert!(matches!(remote, TunnelDirection::Remote));
    }

    #[test]
    fn test_exec_options_defaults() {
        let options = ExecOptions::default();

        assert!(options.timeout.is_none());
        assert!(options.env.is_none());
        assert!(options.working_dir.is_none());
        assert!(options.pty.is_none());
    }
}

mod error_tests {
    use swissknife_file_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Connection refused".to_string(),
            code: Some("ECONNREFUSED".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Connection refused"));
    }

    #[test]
    fn test_file_not_found_error() {
        let error = Error::FileNotFound("/path/to/file.txt".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("/path/to/file.txt"));
    }

    #[test]
    fn test_permission_denied_error() {
        let error = Error::PermissionDenied("/restricted/file".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("/restricted/file"));
    }

    #[test]
    fn test_connection_error() {
        let error = Error::Connection("Host unreachable".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Host unreachable"));
    }

    #[test]
    fn test_auth_error() {
        let error = Error::Auth("Invalid SSH key".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid SSH key"));
    }
}
