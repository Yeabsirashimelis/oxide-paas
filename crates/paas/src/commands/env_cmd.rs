use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn env_set(key_value: String) -> Result<()> {
    let parts: Vec<&str> = key_value.splitn(2, '=').collect();
    if parts.len() != 2 {
        eprintln!("Invalid format. Use KEY=VALUE");
        return Ok(());
    }
    let key = parts[0].trim().to_string();
    let value = parts[1].trim().to_string();

    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        eprintln!("No paas.toml found. Run `paas init` first.");
        return Ok(());
    }

    let content = fs::read_to_string(filename)?;
    let mut doc: toml::Value = toml::from_str(&content)?;

    if let Some(table) = doc.as_table_mut() {
        let env_table = table
            .entry("env")
            .or_insert(toml::Value::Table(toml::map::Map::new()));
        if let Some(env_map) = env_table.as_table_mut() {
            env_map.insert(key.clone(), toml::Value::String(value));
        }
    }

    fs::write(filename, toml::to_string_pretty(&doc)?)?;
    println!("Set env var: {}", key);
    Ok(())
}

pub fn env_list() -> Result<()> {
    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        eprintln!("No paas.toml found. Run `paas init` first.");
        return Ok(());
    }

    let content = fs::read_to_string(filename)?;
    let doc: toml::Value = toml::from_str(&content)?;

    if let Some(env_table) = doc.get("env").and_then(|e| e.as_table()) {
        if env_table.is_empty() {
            println!("No environment variables set.");
        } else {
            println!("Environment variables:");
            for (key, val) in env_table {
                println!("  {} = {}", key, val.as_str().unwrap_or(""));
            }
        }
    } else {
        println!("No environment variables set.");
    }
    Ok(())
}

pub fn env_remove(key: String) -> Result<()> {
    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        eprintln!("No paas.toml found. Run `paas init` first.");
        return Ok(());
    }

    let content = fs::read_to_string(filename)?;
    let mut doc: toml::Value = toml::from_str(&content)?;

    let mut removed = false;
    if let Some(table) = doc.as_table_mut() {
        if let Some(env_table) = table.get_mut("env").and_then(|e| e.as_table_mut()) {
            removed = env_table.remove(&key).is_some();
        }
    }

    if removed {
        fs::write(filename, toml::to_string_pretty(&doc)?)?;
        println!("Removed env var: {}", key);
    } else {
        eprintln!("Env var '{}' not found.", key);
    }
    Ok(())
}
