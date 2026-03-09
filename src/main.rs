use clap::Parser;
use serde_json::{ Value };
use std::fs;
use std::fmt;
use std::process;
use colored::*;

/// Simple program to print json file to table in cli
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// List of fields to be shown in a table with exact order
    #[arg(short, long, value_delimiter = ',')]
    fields: Vec<String>,

    #[arg(short, long)]
    input: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let args = Args::parse();
    if args.fields.is_empty() {
        return Err(AppError::NoFields);
    }

    let data = fs::read_to_string(&args.input).map_err(|source| AppError::ReadInput {
        path: args.input.clone(),
        source,
    })?;
    let v: Value = serde_json::from_str(&data).map_err(AppError::InvalidJson)?;

    if !v.is_array() {
        return Err(AppError::ExpectedJsonArray);
    }

    let (table, cell_size) = build_table(&v, &args.fields);

    let tl = '╭'; // Top Left
    let tr = '╮'; // Top Right
    let bl = '╰'; // Bottom Left
    let br = '╯'; // Bottom Right

    let horizontal = '─'; // Horizontal line
    let vertical = '│'; // Vertical line

    let thj = '┬'; // Top Horizontal Junction (T-down)
    let bhj = '┴'; // Bottom Horizontal Junction (T-up)
    let lvj = '├'; // Left Vertical Junction (T-right)
    let rvj = '┤'; // Right Vertical Junction (T-left)

    let cross = '┼'; // Center Intersection (The "plus")

    print_horizontal_line(&cell_size, tl, thj, tr, horizontal);
    print_colored_row(&args.fields, &cell_size, vertical);
    print_horizontal_line(&cell_size, lvj, cross, rvj, horizontal);

    for row in &table {
        print_colored_row(row, &cell_size, vertical);
    }

    print_horizontal_line(&cell_size, bl, bhj, br, horizontal);

    Ok(())
}

#[derive(Debug)]
enum AppError {
    NoFields,
    ReadInput {
        path: String,
        source: std::io::Error,
    },
    InvalidJson(serde_json::Error),
    ExpectedJsonArray,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NoFields => { write!(f, "no fields provided (use -f field1,field2)") }
            AppError::ReadInput { path, source } => {
                write!(f, "unable to read input file '{}': {}", path, source)
            }
            AppError::InvalidJson(source) => {
                write!(f, "input JSON is not well-formatted: {}", source)
            }
            AppError::ExpectedJsonArray => { write!(f, "input JSON must be an array of objects") }
        }
    }
}

fn build_table(data: &Value, fields: &[String]) -> (Vec<Vec<String>>, Vec<usize>) {
    let mut table: Vec<Vec<String>> = Vec::new();
    let mut cell_size: Vec<usize> = fields
        .iter()
        .map(|field| field.len() + 2)
        .collect();

    if let Some(items) = data.as_array() {
        for item in items {
            let mut row: Vec<String> = Vec::with_capacity(fields.len());
            for (index, key) in fields.iter().enumerate() {
                let cell = get_value_by_path(item, key)
                    .map(value_to_string)
                    .unwrap_or_else(|| String::from("-"));

                cell_size[index] = cell_size[index].max(cell.len() + 2);
                row.push(cell);
            }
            table.push(row);
        }
    }

    (table, cell_size)
}

fn print_horizontal_line(
    cell_size: &[usize],
    left: char,
    junction: char,
    right: char,
    horizontal: char
) {
    for (index, width) in cell_size.iter().enumerate() {
        if index == 0 {
            print!("{}", left);
        }

        let end = if index == cell_size.len() - 1 { right } else { junction };
        print!("{}{}", pad_end("", *width, horizontal), end);
    }
    println!();
}

fn print_colored_row(cells: &[String], cell_size: &[usize], vertical: char) {
    for (index, cell) in cells.iter().enumerate() {
        let color = get_color_by_index(index, cells.len());
        if index == 0 {
            print!("{}", vertical);
        }

        let value = format!(" {} ", cell);
        print!("{}{}", pad_end(&value, cell_size[index], ' ').color(color), vertical);
    }
    println!();
}

fn value_to_string(value: &Value) -> String {
    match value.as_str() {
        Some(s) => s.to_string(),
        None => value.to_string(),
    }
}

fn get_color_by_index(index: usize, total: usize) -> Color {
    let gradient_colors = [
        Color::Red,
        Color::Yellow,
        Color::Green,
        Color::Cyan,
        Color::Blue,
        Color::Magenta,
    ];
    let color_index = (index * gradient_colors.len()) / total;
    gradient_colors[color_index % gradient_colors.len()]
}

fn pad_end(s: &str, width: usize, pad_char: char) -> String {
    let padding = width.saturating_sub(s.len());
    format!("{}{}", s, pad_char.to_string().repeat(padding))
}

fn get_value_by_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}
