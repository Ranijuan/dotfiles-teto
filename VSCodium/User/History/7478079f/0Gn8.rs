use std::env;
use std::fs;
use std::path::Path;

fn search_files(name: &str, dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(fname) = path.file_name() {
                if fname.to_string_lossy().contains(name) {
                    println!("{}", path.display());
                }
            }
            if path.is_dir() {
                search_files(name, &path);
            }
        }
    }
}

fn search_text(pattern: &str, dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(pattern) {
                            println!("{}:{}: {}", path.display(), i + 1, line.trim());
                        }
                    }
                }
            } else if path.is_dir() {
                search_text(pattern, &path);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Uso: teto-find [file|text] <patron> [directorio]");
        std::process::exit(1);
    }

    let mode = &args[1];
    let pattern = &args[2];
    let dir = if args.len() > 3 { args[3].clone() } else { "/".to_string() };
    let path = Path::new(&dir);

    match mode.as_str() {
        "file" => search_files(pattern, path),
        "text" => search_text(pattern, path),
        _ => eprintln!("Modo inválido: usa 'file' o 'text'"),
    }
}