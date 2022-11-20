use std::collections::HashMap;

use anyhow::{anyhow, Result};

fn main() {
    let game = Game::new(3).unwrap();
    println!("{:?}", game);
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
    turn_number: u32,
    winner: Option<&'a Player>,
}

impl Game<'_> {
    const MIN_NUM_ROWS_OR_COLUMNS: u8 = 3;
    // NOTE: be sure to also add more entries to COLUMN_HEADERS if increasing max num cols
    const MAX_NUM_ROWS_OR_COLUMNS: u8 = 8;
    const COLUMN_HEADERS: [char; 8] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];

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
            turn_number: 1,
            winner: None,
        })
    }
}

// add helper fn for getting current turn player (use turn number -1 to index into players vec,
//   protecting against overflow from zero to neg 1)
// use official turn-building logic even during init?
// add ability to render a game board, incl coords
// refactor out to multiple modules/files
// refactor to smaller functions, esp with more `new()` functions
// add test coverage
// remove many Debug derives
// is it possible to fully avoid use of unwrap?
