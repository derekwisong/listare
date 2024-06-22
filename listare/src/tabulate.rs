use std::cmp::{min, max};

#[derive(Debug)]
struct ColumnConfiguration {
    num_columns: usize,
    col_widths: Vec<usize>, // the length of num_columns
    line_len: usize,
    valid: bool,
}

fn init_column_configs(max_cols: usize, min_col_width: usize) -> Vec<ColumnConfiguration> {
    let mut configs: Vec<ColumnConfiguration> = Vec::new();
    for num_columns in 1..=max_cols {
        let config = ColumnConfiguration {
            num_columns: num_columns,
            col_widths: vec![min_col_width; num_columns],
            line_len: num_columns * min_col_width,
            valid: true,
        };
        configs.push(config);
    }
    configs
}

fn get_line_length() -> usize {
    // get the environment variable COLUMNS
    // if it is not greater-than 0, return 80
    // otherwise, return the value of COLUMNS
    let default: usize = 80;
    if let Ok(val) = std::env::var("COLUMNS") {
        if let Ok(num) = val.parse::<usize>() {
            if num > 0 {
                return num;
            }
            else {
                eprintln!("COLUMNS must be greater than 0");
            
            }
        }
        else {
            eprintln!("Could not parse COLUMNS environment variable");
        }
    }
    else {
        eprintln!("Could not read COLUMNS environment variable");
    }
    default
}

fn get_column_config<T>(data: &[T]) -> ColumnConfiguration 
where T: std::fmt::Display,
{
    let line_length = get_line_length();
    let min_column_width: usize = 3; // 1 char for name 2 separating white space
    let max_columns = max(1, line_length / min_column_width);
    let max_columns = min(max_columns, data.len());
    
    // Create a column configuration for each possible number of columns
    let mut configs = init_column_configs(max_columns, min_column_width);
    
    // compute the column widths for each configuration
    for (file_idx, entry) in data.iter().enumerate() {
        let text = format!("{}", entry);
        for config in configs.as_mut_slice() {
            if !config.valid {
                continue;
            }
            // let col_idx = file_idx % config.num_columns;
            let col_idx = file_idx / ((data.len() + config.num_columns - 1) / (config.num_columns));
            let real_len = text.len() + (if col_idx == config.num_columns - 1 { 0 } else { 2 });
            if config.col_widths[col_idx] < real_len {
                config.line_len += real_len - config.col_widths[col_idx];
                config.col_widths[col_idx] = real_len;
                config.valid = config.line_len < line_length;
            }
        }
    }
    
    // find the configuration with the largest line length
    let position = configs.iter().rposition(|config| config.valid).unwrap_or(0);
    // TODO may panic when data empty (max columns will be 0, therefore configs will be empty)
    let config = configs.remove(position);
    config
}

pub fn tabulate<T>(data: &[T])
where
    T: std::fmt::Display,
{
    let config = get_column_config(data);
    let rows = (data.len() / config.num_columns) + if data.len() % config.num_columns != 0 { 1 } else { 0 };
    for row in 0..rows {
        for col in 0..config.num_columns {
            let idx = row + (col * rows);
            if idx < data.len() {
                let entry = &data[idx];
                let text = format!("{}", entry);
                print!("{:width$}", text, width = config.col_widths[col]);

            }
        }
        println!();
    }
}
