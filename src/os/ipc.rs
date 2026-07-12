use std::error::Error;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};

type StdError = dyn Error + Send + Sync;

pub fn get_ipc_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        crate::models::get_default_data_dir().join("daemon.sock")
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\static-memory")
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        PathBuf::from("daemon.sock")
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::*;
    use tokio::net::{UnixListener, UnixStream};
    use std::os::unix::fs::PermissionsExt;

    pub async fn listen() -> Result<UnixListener, Box<StdError>> {
        let path = get_ipc_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let listener = UnixListener::bind(&path)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        Ok(listener)
    }

    pub async fn connect() -> Result<UnixStream, Box<StdError>> {
        Ok(UnixStream::connect(get_ipc_path()).await?)
    }
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    use tokio::net::windows::named_pipe::{ServerOptions, ClientOptions, NamedPipeServer, NamedPipeClient};

    pub fn listen() -> Result<NamedPipeServer, Box<StdError>> {
        let path = get_ipc_path();
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(path)?;
        Ok(server)
    }

    pub async fn connect() -> Result<NamedPipeClient, Box<StdError>> {
        let client = ClientOptions::new().open(get_ipc_path())?;
        Ok(client)
    }
}

#[cfg(target_os = "linux")]
pub type ClientStream = tokio::net::UnixStream;

#[cfg(windows)]
pub type ClientStream = tokio::net::windows::named_pipe::NamedPipeClient;

#[cfg(not(any(target_os = "linux", windows)))]
pub type ClientStream = tokio::io::Empty;

pub async fn connect_with_retry(max_retries: u32, retry_delay: Duration) -> Result<ClientStream, Box<StdError>> {
    let mut retries = 0;
    loop {
        #[cfg(target_os = "linux")]
        let res = linux::connect().await;
        
        #[cfg(windows)]
        let res = windows::connect().await;

        #[cfg(not(any(target_os = "linux", windows)))]
        let res: Result<ClientStream, Box<StdError>> = Err("Unsupported OS".into());

        match res {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                if retries >= max_retries {
                    return Err(format!("Max retries reached: {}", e).into());
                }
                retries += 1;
                sleep(retry_delay).await;
            }
        }
    }
}

// Keep reading connection to prevent panics and handle disconnects
pub async fn send_message<S>(stream: &mut S, msg: &str) -> Result<(), Box<StdError>>
where S: AsyncWriteExt + Unpin {
    let payload = msg.as_bytes();
    let len = (payload.len() as u32).to_le_bytes();
    stream.write_all(&len).await?;
    stream.write_all(payload).await?;
    Ok(())
}

pub async fn receive_response<S>(stream: &mut S) -> Result<String, Box<StdError>>
where S: AsyncReadExt + Unpin {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(String::from_utf8(payload)?)
}
