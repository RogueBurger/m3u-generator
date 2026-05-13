use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn detect_m3u_name(file_paths: &[String]) -> String {
    let basenames: Vec<String> = file_paths
        .iter()
        .map(|p| {
            Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();

    if basenames.len() == 1 {
        return basenames[0].clone();
    }

    // Find longest common prefix character by character
    let mut prefix = basenames[0].clone();
    for basename in &basenames[1..] {
        let common_chars = prefix
            .chars()
            .zip(basename.chars())
            .take_while(|(a, b)| a == b)
            .count();
        // Convert char count to byte offset
        let byte_len = prefix
            .char_indices()
            .nth(common_chars)
            .map(|(i, _)| i)
            .unwrap_or(if common_chars >= prefix.chars().count() {
                prefix.len()
            } else {
                0
            });
        prefix.truncate(byte_len);
        if prefix.is_empty() {
            break;
        }
    }

    // Strip trailing whitespace, then trailing disk-related tokens, repeat until stable
    let mut name = prefix.trim_end().to_string();
    let disk_tokens = ["disk", "disc", "side", "cd"];
    loop {
        let lower = name.to_lowercase();
        let trimmed_lower = lower.trim_end();
        let mut stripped = false;
        for token in &disk_tokens {
            if trimmed_lower.ends_with(token) {
                let pos = trimmed_lower.len() - token.len();
                // Only strip if preceded by whitespace or separator (or start of string)
                let before = trimmed_lower[..pos].trim_end_matches(|c: char| {
                    c.is_whitespace() || c == '-' || c == '_'
                });
                name = name[..before.len()].to_string();
                stripped = true;
                break;
            }
        }
        if !stripped {
            break;
        }
    }

    let name = name.trim_end().to_string();
    if name.is_empty() {
        basenames[0].clone()
    } else {
        name
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum NaturalSegment {
    Alpha(String),
    Num(u64),
}

fn natural_key(s: &str) -> Vec<NaturalSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_digits = false;

    for c in s.chars() {
        let is_digit = c.is_ascii_digit();
        if is_digit != in_digits && !current.is_empty() {
            segments.push(if in_digits {
                NaturalSegment::Num(current.parse().unwrap_or(0))
            } else {
                NaturalSegment::Alpha(current.to_lowercase())
            });
            current.clear();
        }
        in_digits = is_digit;
        current.push(c);
    }
    if !current.is_empty() {
        segments.push(if in_digits {
            NaturalSegment::Num(current.parse().unwrap_or(0))
        } else {
            NaturalSegment::Alpha(current.to_lowercase())
        });
    }
    segments
}

#[tauri::command]
fn create_m3u(file_paths: Vec<String>) -> Result<String, String> {
    if file_paths.is_empty() {
        return Err("No files provided.".to_string());
    }

    // Validate: all files must be in the same parent directory
    let parent_dirs: HashSet<String> = file_paths
        .iter()
        .filter_map(|p| {
            Path::new(p)
                .parent()
                .and_then(|d| d.to_str())
                .map(String::from)
        })
        .collect();

    if parent_dirs.len() > 1 {
        return Err("All files must be in the same folder.".to_string());
    }

    let parent_dir = parent_dirs
        .into_iter()
        .next()
        .ok_or_else(|| "Could not determine parent directory.".to_string())?;

    let m3u_name = detect_m3u_name(&file_paths);
    let target_dir = format!("{}\\{}.m3u", parent_dir, m3u_name);

    if Path::new(&target_dir).exists() {
        return Err(format!("Folder already exists:\n\"{}\"", target_dir));
    }

    // Natural-sort by basename
    let mut sorted = file_paths.clone();
    sorted.sort_by(|a, b| {
        let a_name = Path::new(a)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let b_name = Path::new(b)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        natural_key(a_name).cmp(&natural_key(b_name))
    });

    fs::create_dir(&target_dir).map_err(|e| format!("Failed to create folder: {}", e))?;

    let mut basenames: Vec<String> = Vec::new();
    for src in &sorted {
        let basename = Path::new(src)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", src))?;
        let dest = format!("{}\\{}", target_dir, basename);
        fs::rename(src, &dest)
            .map_err(|e| format!("Failed to move \"{}\": {}", basename, e))?;
        basenames.push(basename.to_string());
    }

    let m3u_file = format!("{}\\{}.m3u", target_dir, m3u_name);
    let content = basenames.join("\n") + "\n";
    fs::write(&m3u_file, content).map_err(|e| format!("Failed to write .m3u file: {}", e))?;

    Ok(m3u_file)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![create_m3u])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
