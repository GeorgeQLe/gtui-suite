use anyhow::Result;
use bollard::container::{
    ListContainersOptions, StartContainerOptions, StopContainerOptions,
    RemoveContainerOptions, LogsOptions, RestartContainerOptions,
};
use bollard::image::ListImagesOptions;
use bollard::network::ListNetworksOptions;
use bollard::volume::ListVolumesOptions;
use bollard::Docker;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use futures_util::StreamExt;

use crate::container::{Container, ContainerState, Image, Mount, Network, PortBinding, Volume};

/// Runtime type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    Docker,
    Podman,
}

impl Runtime {
    pub fn label(&self) -> &'static str {
        match self {
            Runtime::Docker => "Docker",
            Runtime::Podman => "Podman",
        }
    }
}

/// Docker client wrapper
pub struct DockerClient {
    docker: Docker,
    pub runtime: Runtime,
}

impl DockerClient {
    /// Connect to Docker or Podman
    pub fn connect(runtime: Runtime) -> Result<Self> {
        let docker = match runtime {
            Runtime::Docker => Docker::connect_with_socket_defaults()?,
            Runtime::Podman => {
                // Try Podman socket location
                if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
                    let socket = format!("{}/podman/podman.sock", xdg);
                    Docker::connect_with_unix(&socket, 120, bollard::API_DEFAULT_VERSION)?
                } else {
                    Docker::connect_with_socket_defaults()?
                }
            }
        };

        Ok(Self { docker, runtime })
    }

    /// Detect available runtime
    pub async fn detect_runtime() -> Option<Runtime> {
        // Try Docker first
        if let Ok(client) = Docker::connect_with_socket_defaults() {
            if client.ping().await.is_ok() {
                return Some(Runtime::Docker);
            }
        }

        // Try Podman
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            let socket = format!("{}/podman/podman.sock", xdg);
            if let Ok(client) = Docker::connect_with_unix(&socket, 120, bollard::API_DEFAULT_VERSION) {
                if client.ping().await.is_ok() {
                    return Some(Runtime::Podman);
                }
            }
        }

        None
    }

    /// List containers
    pub async fn list_containers(&self, all: bool) -> Result<Vec<Container>> {
        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;

        Ok(containers.into_iter().map(|c| {
            let ports = c.ports.unwrap_or_default().into_iter().map(|p| {
                PortBinding {
                    host_ip: p.ip,
                    host_port: p.public_port,
                    container_port: p.private_port,
                    protocol: p.typ.map(|t| format!("{:?}", t).to_lowercase()).unwrap_or_else(|| "tcp".to_string()),
                }
            }).collect();

            let mounts = c.mounts.unwrap_or_default().into_iter().map(|m| {
                Mount {
                    source: m.source.unwrap_or_default(),
                    destination: m.destination.unwrap_or_default(),
                    mode: m.mode.unwrap_or_default(),
                    mount_type: m.typ.map(|t| format!("{:?}", t)).unwrap_or_default(),
                }
            }).collect();

            let id = c.id.unwrap_or_default();
            let short_id = id.chars().take(12).collect();

            Container {
                id: id.clone(),
                short_id,
                names: c.names.unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                command: c.command.unwrap_or_default(),
                created: c.created.map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_default()),
                state: c.state.map(|s| ContainerState::from_string(&s)).unwrap_or(ContainerState::Created),
                status: c.status.unwrap_or_default(),
                ports,
                mounts,
            }
        }).collect())
    }

    /// Start a container
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.docker.start_container(id, None::<StartContainerOptions<String>>).await?;
        Ok(())
    }

    /// Stop a container
    pub async fn stop_container(&self, id: &str) -> Result<()> {
        let options = StopContainerOptions { t: 10 };
        self.docker.stop_container(id, Some(options)).await?;
        Ok(())
    }

    /// Restart a container
    pub async fn restart_container(&self, id: &str) -> Result<()> {
        let options = RestartContainerOptions { t: 10 };
        self.docker.restart_container(id, Some(options)).await?;
        Ok(())
    }

    /// Remove a container
    pub async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        self.docker.remove_container(id, Some(options)).await?;
        Ok(())
    }

    /// Get container logs
    pub async fn get_logs(&self, id: &str, tail: usize, tx: mpsc::Sender<String>) -> Result<()> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            follow: true,
            timestamps: true,
            ..Default::default()
        };

        let mut stream = self.docker.logs(id, Some(options));

        while let Some(result) = stream.next().await {
            match result {
                Ok(log) => {
                    let line: String = format!("{}", log);
                    if tx.send(line).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    /// List images
    pub async fn list_images(&self) -> Result<Vec<Image>> {
        let options = ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        };

        let images = self.docker.list_images(Some(options)).await?;

        Ok(images.into_iter().map(|i| {
            let id = i.id.clone();
            let short_id = id.trim_start_matches("sha256:").chars().take(12).collect();

            Image {
                id,
                short_id,
                repo_tags: i.repo_tags,
                repo_digests: i.repo_digests,
                created: Some(DateTime::from_timestamp(i.created, 0).unwrap_or_default()),
                size: i.size as u64,
                containers: i.containers,
            }
        }).collect())
    }

    /// Remove an image
    pub async fn remove_image(&self, id: &str, force: bool) -> Result<()> {
        let options = bollard::image::RemoveImageOptions {
            force,
            ..Default::default()
        };
        self.docker.remove_image(id, Some(options), None).await?;
        Ok(())
    }

    /// List volumes
    pub async fn list_volumes(&self) -> Result<Vec<Volume>> {
        let options = ListVolumesOptions::<String> {
            ..Default::default()
        };

        let response = self.docker.list_volumes(Some(options)).await?;
        let volumes = response.volumes.unwrap_or_default();

        Ok(volumes.into_iter().map(|v| {
            let labels = v.labels
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect();

            Volume {
                name: v.name,
                driver: v.driver,
                mountpoint: v.mountpoint,
                created: v.created_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                labels,
            }
        }).collect())
    }

    /// List networks
    pub async fn list_networks(&self) -> Result<Vec<Network>> {
        let options = ListNetworksOptions::<String> {
            ..Default::default()
        };

        let networks = self.docker.list_networks(Some(options)).await?;

        Ok(networks.into_iter().map(|n| {
            let id = n.id.unwrap_or_default();
            let short_id = id.chars().take(12).collect();
            let containers = n.containers.map(|c| c.keys().cloned().collect()).unwrap_or_default();

            Network {
                id,
                short_id,
                name: n.name.unwrap_or_default(),
                driver: n.driver.unwrap_or_default(),
                scope: n.scope.unwrap_or_default(),
                containers,
            }
        }).collect())
    }
}
