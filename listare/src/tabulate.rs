use std::{
    cmp::{max, min},
    error::Error,
};

#[derive(Debug)]
struct ColumnConfiguration {
    num_columns: usize,     // number of columns
    col_widths: Vec<usize>, // the lengths of each column
    line_len: usize,        // the total length of the line
    valid: bool,            // whether the configuration is valid
}

/// Create a vector of column configurations of increasing number of columns
/// Each configuration is initialized with the minimum column width
fn init_column_configs(
    max_line_length: usize,
    num_items: usize,
    min_col_width: usize,
) -> Vec<ColumnConfiguration> {
    let mut configs: Vec<ColumnConfiguration> = Vec::new();
    let max_columns = max(1, max_line_length / min_col_width);
    let max_columns = min(max_columns, num_items);
    for num_columns in 1..=max_columns {
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

#[derive(Debug)]
enum ConfigError {
    EmptyData,
}

impl Error for ConfigError {}
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::EmptyData => write!(f, "Data is empty"),
        }
    }
}

fn get_column_config<T>(
    data: &[T],
    max_line_length: usize,
) -> Result<ColumnConfiguration, ConfigError>
where
    T: std::fmt::Display,
{
    if data.is_empty() {
        return Err(ConfigError::EmptyData);
    }

    // Create a column configuration for each possible number of columns
    const MIN_COLUMN_WIDTH: usize = 3; // 1 char for name 2 separating white space
    let mut configs = init_column_configs(max_line_length, data.len(), MIN_COLUMN_WIDTH);

    // iterate over each file and determine the column widths for each configuration
    for (file_idx, entry) in data.iter().enumerate() {
        let text = format!("{}", entry);

        // for each configuration determine if the current file fits
        for config in configs.as_mut_slice() {
            if !config.valid {
                continue;
            }

            // for horizontal use this instead:
            // let col_idx = file_idx % config.num_columns;
            let col_idx = file_idx / ((data.len() + config.num_columns - 1) / (config.num_columns));
            let real_len = text.len()
                + (if col_idx == config.num_columns - 1 {
                    0
                } else {
                    2
                });

            // update the config if the column width is too small
            if config.col_widths[col_idx] < real_len {
                config.line_len += real_len - config.col_widths[col_idx];
                config.col_widths[col_idx] = real_len;
                // invalidate the configuration if the line length is too long
                config.valid = config.line_len < max_line_length;
            }
        }
    }

    // find the configuration with the largest number of columns that fits within the line length
    let position = configs.iter().rposition(|config| config.valid).unwrap_or(0);
    // TODO may panic when data empty (max columns will be 0, therefore configs will be empty)
    let config = configs.remove(position);
    Ok(config)
}

/// A tabulator for displaying data in columns
pub struct Tabulator<'a, T> {
    data: &'a [T],
    max_line_length: usize,
}

impl<'a, T> Tabulator<'a, T> {
    pub fn new(data: &'a [T], max_line_length: usize) -> Self {
        Tabulator {
            data,
            max_line_length,
        }
    }
}

// implement Display for Tabulator
impl<'a, T> std::fmt::Display for Tabulator<'a, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let config = match get_column_config(self.data, self.max_line_length) {
            Ok(config) => config,
            Err(e) => match e {
                ConfigError::EmptyData => {
                    return Ok(());
                }
            },
        };
        let rows = (self.data.len() / config.num_columns)
            + if self.data.len() % config.num_columns != 0 {
                1
            } else {
                0
            };
        for row in 0..rows {
            for col in 0..config.num_columns {
                let idx = row + (col * rows);
                if idx < self.data.len() {
                    let entry = &self.data[idx];
                    let text = format!("{}", entry);
                    write!(f, "{:width$}", text, width = config.col_widths[col])?;
                }
            }
            // if not the last row, print a newline
            if row < rows - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
