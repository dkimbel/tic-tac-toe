use std::collections::HashMap;

use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let game = Game::new(3)?;
    println!("{}", game.render_board());
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Coordinates {
    column: char,
    row: u8,
}

#[derive(Debug)]
struct Tile<'a> {
    occupation_state: TileOccupationState<'a>,
    display_state: TileDisplayState,
}

#[derive(Debug)]
enum TileOccupationState<'a> {
    Empty,
    Occupied(&'a Player),
}

// lets us display all the tiles involved in a win in green
#[derive(Debug)]
enum TileDisplayState {
    Normal,
    Victory,
}

#[derive(Debug)]
struct BoardState<'a> {
    tiles: HashMap<Coordinates, Tile<'a>>,
}

#[derive(Debug)]
struct Player {
    number: u8,
    mark: char,
}

#[derive(Debug)]
struct Game<'a> {
    pub players: Vec<Player>,
    board_state: BoardState<'a>,
    grid_dimensions: u8,
    turn_number: u32,
    winner: Option<&'a Player>,
}

impl Game<'_> {
    const MIN_NUM_ROWS_OR_COLUMNS: u8 = 3;
    // NOTE: be sure to also add more entries to COLUMN_HEADERS if increasing max num cols
    const MAX_NUM_ROWS_OR_COLUMNS: u8 = 8;
    const COLUMN_HEADERS: [char; 8] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];

    fn get_tile_from_indices(&self, row_index: usize, column_index: usize) -> Option<&Tile> {
        let row = row_index as u8 + 1;
        let column = Self::COLUMN_HEADERS[column_index];
        let coords = Coordinates { row, column };
        self.board_state.tiles.get(&coords)
    }

    fn render_board(&self) -> String {
        let mut rendered_grid = String::new();
        // NOTE: we operate on the assumption that the board is a square -- its
        // number of rows and columns are equal, and every one of them contains
        // the same number of items
        for row_index in 0..self.grid_dimensions as usize {
            let mut rendered_row = String::new();
            for column_index in 0..self.grid_dimensions as usize {
                let tile = self.get_tile_from_indices(row_index, column_index).unwrap();
                use TileOccupationState::*;
                let tile_mark = match tile.occupation_state {
                    Empty => ' ',
                    Occupied(player) => player.mark,
                };
                let cell_ending = if column_index + 1 < self.grid_dimensions as usize {
                    '|'
                } else {
                    '\n'
                };
                let cell = &format!(" {} {}", tile_mark, cell_ending);
                rendered_row.push_str(cell);
            }
            rendered_grid.push_str(&rendered_row);
            // if the row we just added wasn't the last one...
            if row_index + 1 < self.grid_dimensions as usize {
                // ... then build and add a divider row. We subtract 1 from the length to account
                // for the newline at the end.
                let divider_row = format!("{}\n", "-".repeat(rendered_row.len() - 1));
                rendered_grid.push_str(&divider_row);
            }
        }
        rendered_grid
    }

    fn new(num_rows_or_columns: u8) -> Result<Self> {
        if num_rows_or_columns < Self::MIN_NUM_ROWS_OR_COLUMNS
            || num_rows_or_columns > Self::MAX_NUM_ROWS_OR_COLUMNS
        {
            return Err(anyhow!(
                "Number of rows/columns on game board must be between {} and {}",
                Self::MIN_NUM_ROWS_OR_COLUMNS,
                Self::MAX_NUM_ROWS_OR_COLUMNS
            ));
        }

        let players = vec![
            Player {
                number: 1,
                mark: 'X',
            },
            Player {
                number: 2,
                mark: 'O',
            },
        ];
        let mut tiles = HashMap::new();
        for row_index in 0..num_rows_or_columns {
            for column_index in 0..num_rows_or_columns {
                tiles.insert(
                    Coordinates {
                        column: Self::COLUMN_HEADERS[column_index as usize],
                        row: row_index + 1, // convert 0-indexed to user-facing 1-indexed
                    },
                    Tile {
                        occupation_state: TileOccupationState::Empty,
                        display_state: TileDisplayState::Normal,
                    },
                );
            }
        }
        Ok(Self {
            players,
            board_state: BoardState { tiles },
            grid_dimensions: num_rows_or_columns,
            turn_number: 1,
            winner: None,
        })
    }
}

// add helper fn for getting current turn player (use turn number -1 to index into players vec,
//   protecting against overflow from zero to neg 1)
// add a `render_turn`
// accept user input for coords
// actually update board state every turn
// actually check for victory at the end of every turn, print winner, stop game

// be very friendly in stripping whitespace and accepting lowercase chars for coords
// use official turn-building logic even during init?
// add ability to render a game board, incl coords
// refactor out to multiple modules/files
// refactor to smaller functions, esp with more `new()` functions
// add test coverage
// remove many Debug derives
// can I clean up 'cell' logic using map + join, so I join with '|'?
// try to always use some kind of safe cast and index lookup
// sprinkle in `+` in rows to match up with vertical lines
// print row and column headers
// maybe clean up logic for how we look up tiles -- by indices always? by 'coords' always?
// maybe clean up how we build coords, so it's more foolproof about adding 1 to convert 0-indexed to 1
// allow min_rows_cols all the way down to 1?
// is it possible to fully avoid use of unwrap?
// somehow make get_tile_from_indices less dangerous, by letting it have named parmams via some kind of struct?
// generally do less casting
// can I make COLUMN_HEADERS a vec and fill it up automatically?
