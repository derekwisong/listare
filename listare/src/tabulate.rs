use std::{
    cmp::{max, min},
    error::Error,
};

pub trait CharacterLength {
    fn characters_long(&self) -> usize;
}

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

pub enum TabulateOrientation {
    Columns,
    Rows,
}

/// A tabulator for displaying data in columns
pub struct Tabulator<'a, T> {
    data: &'a [T],
    max_line_length: usize,
    orientation: TabulateOrientation,
}

impl<'a, T> Tabulator<'a, T> {
    fn get_column_config(&self) -> Result<ColumnConfiguration, ConfigError>
    where
        T: CharacterLength,
    {
        if self.data.is_empty() {
            return Err(ConfigError::EmptyData);
        }

        // Create a column configuration for each possible number of columns
        const MIN_COLUMN_WIDTH: usize = 3; // 1 char for name 2 separating white space
        let mut configs =
            init_column_configs(self.max_line_length, self.data.len(), MIN_COLUMN_WIDTH);

        // iterate over each file and determine the column widths for each configuration
        for (file_idx, entry) in self.data.iter().enumerate() {
            // for each configuration determine if the current file fits
            for config in configs.as_mut_slice() {
                if !config.valid {
                    continue;
                }

                let col_idx = match self.orientation {
                    TabulateOrientation::Rows => file_idx % config.num_columns,
                    TabulateOrientation::Columns => {
                        file_idx
                            / ((self.data.len() + config.num_columns - 1) / (config.num_columns))
                    }
                };
                // for horizontal use this instead:
                // let col_idx = file_idx % config.num_columns;
                // let col_idx = file_idx / ((self.data.len() + config.num_columns - 1) / (config.num_columns));
                let real_len = entry.characters_long()
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
                    config.valid = config.line_len < self.max_line_length;
                }
            }
        }

        // find the configuration with the largest number of columns that fits within the line length
        let position = configs.iter().rposition(|config| config.valid).unwrap_or(0);
        // TODO may panic when data empty (max columns will be 0, therefore configs will be empty)
        let config = configs.remove(position);
        Ok(config)
    }

    pub fn new(data: &'a [T], max_line_length: usize, orientation: TabulateOrientation) -> Self {
        Tabulator {
            data,
            max_line_length,
            orientation: orientation,
        }
    }
}

// implement Display for Tabulator
impl<'a, T> std::fmt::Display for Tabulator<'a, T>
where
    T: std::fmt::Display,
    T: CharacterLength,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let config = match self.get_column_config() {
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
                let idx = match self.orientation {
                    TabulateOrientation::Rows => row * config.num_columns + col,
                    TabulateOrientation::Columns => row + (col * rows),
                };
                //let idx = row + (col * rows);
                if idx < self.data.len() {
                    let entry = &self.data[idx];
                    write!(f, "{:width$}", entry, width = config.col_widths[col])?;
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
