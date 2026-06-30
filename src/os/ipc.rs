use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::models::{IPCMessage, IPCResponse};
use std::error::Error;
use std::path::PathBuf;

type StdError = dyn Error + Send + Sync;

pub fn get_ipc_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        crate::os::get_data_dir().join("daemon.sock")
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\static-memory")
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::*;
    use tokio::net::{UnixListener, UnixStream};
    use std::os::unix::fs::PermissionsExt;

    pub async fn listen() -> Result<UnixListener, Box<StdError>> {
        let path = get_ipc_path();
        let parent = path.parent().ok_or("Invalid path")?;
        std::fs::create_dir_all(parent)?;

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

pub async fn send_message<S>(stream: &mut S, msg: &IPCMessage) -> Result<(), Box<StdError>>
where S: AsyncWriteExt + Unpin {
    let payload = serde_json::to_vec(msg)?;
    let len = (payload.len() as u32).to_le_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&payload).await?;
    Ok(())
}

pub async fn receive_message<S>(stream: &mut S) -> Result<IPCMessage, Box<StdError>>
where S: AsyncReadExt + Unpin {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(serde_json::from_slice(&payload)?)
}

pub async fn send_response<S>(stream: &mut S, resp: &IPCResponse) -> Result<(), Box<StdError>>
where S: AsyncWriteExt + Unpin {
    let payload = serde_json::to_vec(resp)?;
    let len = (payload.len() as u32).to_le_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&payload).await?;
    Ok(())
}

pub async fn receive_response<S>(stream: &mut S) -> Result<IPCResponse, Box<StdError>>
where S: AsyncReadExt + Unpin {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(serde_json::from_slice(&payload)?)
}
