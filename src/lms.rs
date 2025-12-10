use std::process::Command;
use std::time::Duration;
use reqwest::Client;
use tokio::time::sleep;

fn run_command(cmd: &str, args: &[&str]) -> Result<std::process::ExitStatus, Box<dyn std::error::Error>> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("Failed to execute command '{}': {}", cmd, e))?;
    Ok(status)
}

/// Checks if LM Studio server is running and starts it if necessary.
async fn start_lm_studio(model_name: &str, api_base: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = Client::new();
    let max_retries = 3;
    let retry_delay = Duration::from_secs(5);

    for attempt in 1..=max_retries {
        match client.get(format!("{}/models", api_base)).send().await {
            Ok(response) if response.status().is_success() => {
                println!("LM Studio server is already running.");
                ensure_model_loaded(model_name)?;
                return Ok(false);
            }
            _ => {
                println!("LM Studio server is not running. Starting it (attempt {}/{})...", attempt, max_retries);
                let (cmd, args) = if cfg!(windows) {
                    (
                        "cmd",
                        vec!["/C", "%USERPROFILE%\\.lmstudio\\bin\\lms.exe", "server", "start"],
                    )
                } else {
                    (
                        "sh",
                        vec!["-c", "~/.lmstudio/bin/lms server start"],
                    )
                };

                let status = run_command(cmd, &args)?;
                if !status.success() {
                    return Err(format!("Failed to start LM Studio server: exit code {:?}", status.code()).into());
                }

                sleep(retry_delay).await;

                match client.get(format!("{}/models", api_base)).send().await {
                    Ok(response) if response.status().is_success() => {
                        println!("LM Studio server started successfully.");
                        ensure_model_loaded(model_name)?;
                        return Ok(true);
                    }
                    _ if attempt == max_retries => {
                        return Err("LM Studio server failed to start or is unreachable after retries.".into());
                    }
                    _ => {
                        println!("Server not yet ready, retrying...");
                    }
                }
            }
        }
    }
    Ok(false)
}

/// Checks if the specified model is loaded using lms ps, and loads it if not.
fn ensure_model_loaded(model_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (cmd, args) = if cfg!(windows) {
        (
            "cmd",
            vec!["/C", "%USERPROFILE%\\.lmstudio\\bin\\lms.exe", "ps"],
        )
    } else {
        (
            "sh",
            vec!["-c", "~/.lmstudio/bin/lms ps"],
        )
    };

    let output = Command::new(cmd)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute lms ps command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Failed to check loaded models: exit code {:?}", output.status.code()).into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    cleanup_duplicate_models(model_name, &output_str)?;

    if output_str.contains(model_name) {
        println!("\x1b[32mModel {} is already loaded.\x1b[0m", model_name);
        return Ok(());
    }

    println!("Loading model {}...", model_name);
    let formatted_cmd;
    let (cmd, args) = if cfg!(windows) {
        (
            "cmd",
            vec!["/C", "%USERPROFILE%\\.lmstudio\\bin\\lms.exe", "load", model_name],
        )
    } else {
        formatted_cmd = format!("~/.lmstudio/bin/lms load {}", model_name);
        (
            "sh",
            vec!["-c", &formatted_cmd],
        )
    };

    let status = run_command(cmd, &args)?;
    if !status.success() {
        return Err(format!("Failed to load model {}: exit code {:?}", model_name, status.code()).into());
    }

    println!("\x1b[32mModel {} loaded successfully.\x1b[0m", model_name);
    Ok(())
}

/// Unloads duplicate instances of the model to free resources.
fn cleanup_duplicate_models(model_name: &str, ps_output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let duplicates: Vec<String> = ps_output
        .lines()
        .filter(|line| line.contains(model_name) && line.contains(":"))
        .map(|line| line.trim().to_string())
        .collect();

    for duplicate in duplicates {
        if !duplicate.starts_with(model_name) {
            continue; // Skip partial matches
        }
        println!("Unloading duplicate model instance {}...", duplicate);
        let formatted_cmd;
        let (cmd, args) = if cfg!(windows) {
            (
                "cmd",
                vec!["/C", "%USERPROFILE%\\.lmstudio\\bin\\lms.exe", "unload", &duplicate],
            )
        } else {
            formatted_cmd = format!("~/.lmstudio/bin/lms unload {}", duplicate);
            (
                "sh",
                vec!["-c", &formatted_cmd],
            )
        };

        let status = run_command(cmd, &args)?;
        if status.success() {
            println!("\x1b[32mDuplicate model {} unloaded successfully.\x1b[0m", duplicate);
        } else {
            eprintln!("\x1b[31mFailed to unload duplicate model {}: exit code {:?}\x1b[0m", duplicate, status.code());
        }
    }
    Ok(())
}

/// Unloads the specified model using lms unload.
fn unload_model(model_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Unloading model {}...", model_name);
    let formatted_cmd;
    let (cmd, args) = if cfg!(windows) {
        (
            "cmd",
            vec!["/C", "%USERPROFILE%\\.lmstudio\\bin\\lms.exe", "unload", model_name],
        )
    } else {
        formatted_cmd = format!("~/.lmstudio/bin/lms unload {}", model_name);
        (
            "sh",
            vec!["-c", &formatted_cmd],
        )
    };

    let status = run_command(cmd, &args)?;
    if !status.success() {
        return Err(format!("Failed to unload model {}: exit code {:?}", model_name, status.code()).into());
    }

    println!("\x1b[32mModel {} unloaded successfully.\x1b[0m", model_name);
    Ok(())
}

/// Stops the LM Studio server if it was started by the program.
fn stop_lm_studio(started_server: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !started_server {
        return Ok(());
    }

    println!("Stopping LM Studio server...");
    let (cmd, args) = if cfg!(windows) {
        (
            "cmd",
            vec!["/C", "%USERPROFILE%\\.lmstudio\\bin\\lms.exe", "server", "stop"],
        )
    } else {
        (
            "sh",
            vec!["-c", "~/.lmstudio/bin/lms server stop"],
        )
    };

    let status = run_command(cmd, &args)?;
    if status.success() {
        println!("\x1b[32mLM Studio server stopped successfully.\x1b[0m");
    } else {
        eprintln!("\x1b[31mFailed to stop LM Studio server: exit code {:?}\x1b[0m", status.code());
    }
    Ok(())
}