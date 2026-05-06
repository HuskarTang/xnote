use git2::{Cred, RemoteCallbacks};

pub fn safe_remote_label(url: &str) -> String {
    if let Some(protocol_end) = url.find("://") {
        let scheme = &url[..protocol_end + 3];
        let rest = &url[protocol_end + 3..];
        if let Some(at_index) = rest.find('@') {
            return format!("{}{}", scheme, &rest[at_index + 1..]);
        }
    }
    url.to_string()
}

pub fn expand_home(path: &str) -> std::path::PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    std::path::PathBuf::from(path)
}

pub fn callbacks(config: crate::config::GitSyncConfig) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        match config.auth_type.as_str() {
            "basic" => {
                if let (Some(username), Some(password)) = (&config.username, &config.password) {
                    Cred::userpass_plaintext(username, password)
                } else {
                    Cred::default()
                }
            }
            "ssh" => {
                let username = username_from_url.unwrap_or("git");
                if let Some(ssh_key_path) = &config.ssh_key_path {
                    if !ssh_key_path.is_empty() {
                        let expanded = expand_home(ssh_key_path);
                        return Cred::ssh_key(username, None, &expanded, None);
                    }
                }
                Cred::ssh_key_from_agent(username)
            }
            _ => Cred::default(),
        }
    });
    callbacks
}
