use clap::Parser;
use librsdu::{traverse_directory, FileInfo};
use ncurses::*;
use std::fs;
use std::path::PathBuf;

/// Command-line arguments parser.
#[derive(Parser)]
#[command(name = "rsdu", about = "A Rust-based ncdu replacement")]
struct Cli {
    #[arg(help = "Directory to scan")]
    directory: String,
}

/// Holds the application state for navigation.
struct AppState {
    stack: Vec<FileInfo>,
    selected_index: usize,
}

fn main() {
    // Parse command-line arguments.
    let args = Cli::parse();

    // Resolve the absolute path.
    let root_path = match fs::canonicalize(&args.directory) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error resolving path '{}': {}", args.directory, e);
            std::process::exit(1);
        }
    };

    // Traverse the directory and build the file tree.
    let root_info = match traverse_directory(&root_path) {
        Ok(info) => info,
        Err(e) => {
            eprintln!(
                "Error traversing directory '{}': {}",
                root_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    let total_size = root_info.size;
    let total_items = root_info.items;

    let mut app_state = AppState {
        stack: vec![root_info],
        selected_index: 0,
    };

    // Initialize ncurses.
    initscr();
    keypad(stdscr(), true);
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    loop {
        // Clear the screen and get the current directory info.
        clear();
        let current_dir = app_state.stack.last().unwrap();
        let entries = current_dir.children.as_deref().unwrap_or(&[]);

        // Get the window size.
        let (max_y, max_x) = {
            let mut y = 0;
            let mut x = 0;
            getmaxyx(stdscr(), &mut y, &mut x);
            (y, x)
        };

        // Display the header line with the current directory path.
        let header = format!(
            "--- {} {}",
            current_dir.path.display(),
            "-".repeat((max_x as usize).saturating_sub(current_dir.path.display().to_string().len() + 4))
        );
        mvprintw(0, 0, &header);

        // Find the maximum size among entries for bar graph scaling.
        let max_entry_size = entries.iter().map(|e| e.size).max().unwrap_or(1);

        // Display the list of files and directories.
        for (i, entry) in entries.iter().enumerate() {
            if i >= (max_y as usize - 4) {
                break; // Avoid writing beyond the screen
            }

            let y_pos = i as i32 + 1;
            if i == app_state.selected_index {
                attron(A_REVERSE());
            }

            let size_str = human_readable_size(entry.size);
            let bar = generate_bar(entry.size, max_entry_size, 30); // 30 characters wide bar

            let name = entry
                .path
                .file_name()
                .unwrap_or_else(|| entry.path.as_os_str())
                .to_string_lossy();

            mvprintw(
                y_pos,
                0,
                &format!("{:>10} [{}] {}", size_str, bar, name),
            );

            if i == app_state.selected_index {
                attroff(A_REVERSE());
            }
        }

        // Display the footer with total disk usage, apparent size, and items.
        let footer_y = (max_y - 2) as i32;
        let total_size_str = human_readable_size(total_size);
        mvprintw(
            footer_y,
            0,
            &format!(
                "*Total disk usage: {:>10}   Apparent size: {:>10}   Items: {}",
                total_size_str, total_size_str, total_items
            ),
        );

        // Display instructions.
        mvprintw(
            (max_y - 1) as i32,
            0,
            "Press 'q' to quit. Use arrow keys to navigate. Enter to open directory. Backspace to go back.",
        );

        refresh();

        // Handle user input.
        let ch = getch();
        match ch {
            KEY_UP => {
                if app_state.selected_index > 0 {
                    app_state.selected_index -= 1;
                }
            }
            KEY_DOWN => {
                if app_state.selected_index + 1 < entries.len() {
                    app_state.selected_index += 1;
                }
            }
            10 => {
                // Enter key to navigate into a directory.
                let selected_entry = &entries[app_state.selected_index];
                if selected_entry.is_dir {
                    app_state.stack.push(selected_entry.clone());
                    app_state.selected_index = 0;
                }
            }
            ch if ch == 'q' as i32 => {
                // Quit the application.
                break;
            }
            KEY_BACKSPACE | 127 | 8 => {
                // Backspace to go up one directory.
                if app_state.stack.len() > 1 {
                    app_state.stack.pop();
                    app_state.selected_index = 0;
                }
            }
            _ => {}
        }
    }

    // End ncurses mode.
    endwin();
}

fn generate_bar(size: u64, max_size: u64, bar_width: usize) -> String {
    let ratio = size as f64 / max_size as f64;
    let filled_length = (ratio * bar_width as f64).round() as usize;
    let bar = "#".repeat(filled_length);
    let empty = " ".repeat(bar_width - filled_length);
    format!("{}{}", bar, empty)
}

fn human_readable_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let size_f = size as f64;

    if size_f >= TB {
        format!("{:.1} TiB", size_f / TB)
    } else if size_f >= GB {
        format!("{:.1} GiB", size_f / GB)
    } else if size_f >= MB {
        format!("{:.1} MiB", size_f / MB)
    } else if size_f >= KB {
        format!("{:.1} KiB", size_f / KB)
    } else {
        format!("{} B", size)
    }
}
