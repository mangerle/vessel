use crate::connection::get_docker_client;
use crate::error::AppResult;
use bollard::container::ListContainersOptions;
use std::collections::HashMap;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::io::AsyncBufReadExt;
use super::ComposeProject;

/// 获取 Compose 项目列表
#[tauri::command]
pub async fn list_compose_projects() -> AppResult<Vec<ComposeProject>> {
    let docker = get_docker_client().await?;
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    struct ProjectData {
        total: usize,
        running: usize,
        working_dir: Option<String>,
        config_file: Option<String>,
    }

    let mut projects_map: HashMap<String, ProjectData> =
        HashMap::new();

    for container in containers {
        if let Some(labels) = container.labels {
            if let Some(project_name) = labels.get("com.docker.compose.project") {
                let data = projects_map
                    .entry(project_name.clone())
                    .or_insert(ProjectData {
                        total: 0,
                        running: 0,
                        working_dir: labels
                            .get("com.docker.compose.project.working_dir")
                            .cloned(),
                        config_file: labels
                            .get("com.docker.compose.project.config_files")
                            .cloned(),
                    });

                data.total += 1;
                if container.state.as_deref() == Some("running") {
                    data.running += 1;
                }
            }
        }
    }

    let projects = projects_map
        .into_iter()
        .map(|(name, data)| ComposeProject {
            name,
            container_count: data.total,
            running_count: data.running,
            status: if data.running > 0 {
                "running".to_string()
            } else {
                "exited".to_string()
            },
            working_dir: data.working_dir,
            config_file: data.config_file,
        })
        .collect();

    Ok(projects)
}

/// 读取 Compose 配置文件内容
#[tauri::command]
pub async fn read_compose_file(
    path: String,
    mode: String,
    distro: Option<String>,
) -> AppResult<String> {
    if mode == "wsl" {
        let mut cmd = tokio::process::Command::new("wsl");
        if let Some(d) = distro {
            if !d.is_empty() {
                cmd.args(["-d", &d]);
            }
        }
        cmd.args(["-u", "root", "--", "cat", &path]);
        
        #[cfg(windows)]
        cmd.creation_flags(0x08000000);

        let out = cmd.output().await?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string().into())
        }
    } else {
        Ok(tokio::fs::read_to_string(path).await?)
    }
}

/// 写入 Compose 配置文件内容
#[tauri::command]
pub async fn write_compose_file(
    path: String, 
    content: String,
    mode: String,
    distro: Option<String>,
) -> AppResult<()> {
    if mode == "wsl" {
        let mut cmd = tokio::process::Command::new("wsl");
        if let Some(d) = distro {
            if !d.is_empty() {
                cmd.args(["-d", &d]);
            }
        }
        let shell_cmd = format!("cat << 'EOF' > \"{}\"\n{}\nEOF", path, content);
        cmd.args(["-u", "root", "--", "sh", "-c", &shell_cmd]);

        #[cfg(windows)]
        cmd.creation_flags(0x08000000);

        let out = cmd.output().await?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string().into())
        }
    } else {
        Ok(tokio::fs::write(path, content).await?)
    }
}

/// 执行 Compose 命令并实时流式传输输出
#[tauri::command]
pub async fn run_compose_command(
    app: AppHandle,
    project_dir: String,
    args: Vec<String>,
    mode: String,
    distro: Option<String>,
) -> AppResult<()> {
    let mut cmd = if mode == "wsl" {
        let mut c = tokio::process::Command::new("wsl");
        if let Some(d) = distro {
            if !d.is_empty() {
                c.args(["-d", &d]);
            }
        }
        let args_str = args.join(" ");
        c.args(["sh", "-c", &format!("cd \"{}\" && docker compose {}", project_dir, args_str)]);
        c
    } else {
        let mut c = tokio::process::Command::new("docker");
        c.arg("compose")
            .args(args)
            .current_dir(project_dir);
        c
    };

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone.emit("compose-cmd-output", line);
        }
    });

    let app_clone_err = app.clone();
    tauri::async_runtime::spawn(async move {
        let reader = tokio::io::BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = app_clone_err.emit("compose-cmd-output", line);
        }
    });

    let app_clone_finish = app.clone();
    tauri::async_runtime::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    let _ = app_clone_finish.emit("compose-cmd-finished", ());
                } else {
                    let _ = app_clone_finish.emit("compose-cmd-error", format!("Process exited with status: {}", status));
                }
            }
            Err(e) => {
                let _ = app_clone_finish.emit("compose-cmd-error", format!("Failed to wait for process: {}", e));
            }
        }
    });

    Ok(())
}
