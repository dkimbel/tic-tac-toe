use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::io::{self, Write};
use std::str::FromStr;

use anyhow::{anyhow, Context, Error, Result};
use colored::*;
use regex::Regex;

fn main() -> Result<()> {
    let mut game = Game::new(3)?;
    while game.outcome == GameOutcome::InProgress {
        try_execute_turn(&mut game)?;
    }
    // render game board one last time to display final result
    try_execute_turn(&mut game)?;
    Ok(())
}

fn try_execute_turn(game: &mut Game) -> Result<()> {
    clearscreen::clear()?;
    let current_player = game.get_current_turn_player();
    println!(); // newline to ensure a command-line prompt doesn't skew first line of game board
    println!("{}", game.render_board());
    // print notification, if any
    if let Some(notification) = &game.notification {
        println!("{}", notification);
        println!();
    }
    // having printed prior notification, clear it
    game.notification = None;
    if game.outcome == GameOutcome::InProgress {
        print!(
            "{}, enter coordinates to place your {}: ",
            current_player.to_string().bold(),
            current_player.mark
        );
        io::stdout().flush()?;
        let mut unparsed_coords = String::new();
        io::stdin().read_line(&mut unparsed_coords)?;
        let coords_result = Coordinates::from_user_input(&unparsed_coords);
        if let Err(error) = coords_result {
            handle_error(game, error, None);
            return try_execute_turn(game);
        }
        let coords = coords_result.unwrap(); // safe thanks to line above
        if let Err(error) = game.update_board(coords, current_player) {
            handle_error(game, error, Some(coords));
            return try_execute_turn(game);
        }
        game.update_outcome();
    }
    Ok(())
}

fn handle_error(game: &mut Game, error: Error, maybe_coords: Option<Coordinates>) -> () {
    game.notification = Some(Notification {
        message: error.to_string(),
        notification_type: NotificationType::Error,
    });
    if let Some(coords) = maybe_coords {
        if let Some(mut error_tile) = game.board.tiles.get_mut(&coords) {
            error_tile.display_state = TileDisplayState::Error;
        }
    }
}

// zero-indexed!
struct Indices {
    column: usize,
    row: usize,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Coordinates {
    column: char,
    row: usize, // one-indexed!
}

impl Coordinates {
    const COLUMN_LETTERS: [char; Game::MAX_NUM_ROWS_OR_COLUMNS] =
        ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];

    fn from_indices(indices: &Indices) -> Result<Coordinates> {
        let row = indices.row + 1;
        let &column = Self::COLUMN_LETTERS.get(indices.column).context(format!(
            "No letter found to match zero-indexed column number {}; there are {} letter(s) total.",
            indices.column,
            Self::COLUMN_LETTERS.len()
        ))?;
        Ok(Coordinates { row, column })
    }

    fn from_user_input(input: &str) -> Result<Coordinates> {
        // Match a single alphabetical character followed by a number with one or more digits;
        // whitespace and arbitrary punctuation are allowed at the beginning, end, and in between
        // the character and digits (just not within the digits).
        let re =
            Regex::new(r"^[\s|[[:punct:]]]*([[:alpha:]])[\s|[[:punct:]]]*(\d+)[\s|[[:punct:]]]*$")?;
        let cap = re.captures(input).context(format!(
            "Could not parse '{}' as coordinates. Valid example: A1",
            input.trim()
        ))?;
        let column = char::from_str(&cap[1])?.to_ascii_uppercase();
        let row = usize::from_str(&cap[2])?;
        Ok(Self { column, row })
    }
}

impl fmt::Display for Coordinates {
    // print coords as e.g. "A1"
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.column, self.row)
    }
}

struct Tile {
    occupation_state: TileOccupationState,
    display_state: TileDisplayState,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use TileDisplayState::*;
        use TileOccupationState::*;
        write!(
            f,
            "{}",
            match self.occupation_state {
                Empty => " ".normal(),
                Occupied(player) => {
                    let mark = String::from(player.mark);
                    match self.display_state {
                        NewlyCreated => mark.bold(),
                        Victory => mark.green().bold(),
                        Error => mark.red().bold(),
                        Normal => mark.normal(),
                    }
                }
            }
        )
    }
}

#[derive(Copy, Clone)]
enum TileOccupationState {
    Empty,
    Occupied(Player),
}

// lets us render the winning line of tiles in green
#[derive(Clone, Copy)]
enum TileDisplayState {
    Error,
    NewlyCreated,
    Normal,
    Victory,
}

struct Board {
    tiles: HashMap<Coordinates, Tile>,
}

#[derive(PartialEq, Clone, Copy)]
struct Player {
    number: u8,
    mark: char,
}

impl fmt::Display for Player {
    // print as e.g. "Player 1"
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Player {}", self.number)
    }
}

enum NotificationType {
    Success,
    Info,
    Error,
}

struct Notification {
    message: String,
    notification_type: NotificationType,
}

impl fmt::Display for Notification {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use NotificationType::*;
        let message = &self.message;
        write!(
            f,
            "{}",
            match self.notification_type {
                Success => message.green().bold(),
                Info => message.normal(),
                Error => format!("{} {}", "Error!".red().bold(), message).normal(),
            }
        )
    }
}

#[derive(PartialEq)]
enum GameOutcome {
    InProgress,
    Draw,
    Victory(Player),
}

struct Game {
    pub players: Vec<Player>,
    board: Board,
    notification: Option<Notification>,
    grid_dimensions: usize,
    turn_number: usize,
    outcome: GameOutcome,
}

impl Game {
    const MIN_NUM_ROWS_OR_COLUMNS: usize = 1;
    const MAX_NUM_ROWS_OR_COLUMNS: usize = 8;

    fn update_board(
        &mut self,
        coords_for_move: Coordinates,
        player_for_move: Player,
    ) -> Result<()> {
        // validate against both the "coordinates don't even exist on game board"
        // and the "coordinates refer to an already-occupied tile" edge cases
        let tile_for_move: &Tile = self.board.tiles.get(&coords_for_move).context(format!(
            "Could not find coordinates {} on game board.",
            coords_for_move
        ))?;
        if let TileOccupationState::Occupied(player) = tile_for_move.occupation_state {
            return Err(anyhow!(
                "Tile {} is already occupied by {}.",
                coords_for_move,
                player
            ));
        }

        // assemble new game board, clearing any previous display states from tiles
        let mut new_tiles = HashMap::new();
        for (coords, old_tile) in self.board.tiles.iter() {
            let new_tile = if coords == &coords_for_move {
                Tile {
                    occupation_state: TileOccupationState::Occupied(player_for_move),
                    display_state: TileDisplayState::NewlyCreated,
                }
            } else {
                Tile {
                    occupation_state: old_tile.occupation_state,
                    display_state: TileDisplayState::Normal,
                }
            };
            new_tiles.insert(coords.clone(), new_tile);
        }
        self.board = Board { tiles: new_tiles };
        Ok(())
    }

    // checks for all possible victory states:
    //   - either player occupies every tile in a single row
    //   - either player occupies every tile in a single column
    //   - either player occupies every tile in a full-length diagonal
    // also checks for a draw (all tiles are occupied, but there is no victor)
    fn update_outcome(&mut self) {
        let mut possible_winning_indices_sets: Vec<Vec<Indices>> = Vec::new();

        // build a list of indices for each row of game board
        for row_index in 0..self.grid_dimensions {
            let mut row_indices: Vec<Indices> = Vec::new();
            for column_index in 0..self.grid_dimensions {
                row_indices.push(Indices {
                    row: row_index,
                    column: column_index,
                });
            }
            possible_winning_indices_sets.push(row_indices);
        }

        // build a list of indices for each column of game board
        for column_index in 0..self.grid_dimensions {
            let mut column_indices: Vec<Indices> = Vec::new();
            for row_index in 0..self.grid_dimensions {
                column_indices.push(Indices {
                    row: row_index,
                    column: column_index,
                });
            }
            possible_winning_indices_sets.push(column_indices);
        }

        // build lists of indices for the upper-left-to-lower-right and lower-left-to-
        // upper-right diagonals; both rely on the game board having equal length and width
        let mut upper_left_diagonal_indices: Vec<Indices> = Vec::new();
        let mut lower_left_diagonal_indices: Vec<Indices> = Vec::new();
        let max_index = self.grid_dimensions - 1;
        for index in 0..self.grid_dimensions {
            upper_left_diagonal_indices.push(Indices {
                row: index,
                column: index,
            });
            lower_left_diagonal_indices.push(Indices {
                row: max_index - index,
                column: index,
            });
        }
        possible_winning_indices_sets.push(upper_left_diagonal_indices);
        possible_winning_indices_sets.push(lower_left_diagonal_indices);

        for indices_set in possible_winning_indices_sets {
            let maybe_winner = self.single_player_occupying_indices(&indices_set);
            if let Some(player) = maybe_winner {
                self.outcome = GameOutcome::Victory(player);
                self.notification = Some(Notification {
                    message: format!("{} wins!", player),
                    notification_type: NotificationType::Success,
                });
                // update the tiles from the winning row/column/diagonal to render as winners
                let coordinates_set = indices_set
                    .iter()
                    .map(|indices| Coordinates::from_indices(indices).unwrap())
                    .collect::<Vec<Coordinates>>();
                for coordinates in coordinates_set {
                    let mut tile = self.board.tiles.get_mut(&coordinates).unwrap();
                    tile.display_state = TileDisplayState::Victory;
                }
                return;
            }
        }

        // we will only reach this point if no one has won yet; if every tile is in fact
        // occupied, the game must be a draw
        let all_tiles_occupied =
            self.board
                .tiles
                .iter()
                .all(|(_, tile)| match tile.occupation_state {
                    TileOccupationState::Occupied(_) => true,
                    _ => false,
                });
        if all_tiles_occupied {
            self.outcome = GameOutcome::Draw;
            self.notification = Some(Notification {
                message: format!("The game ends in a draw!"),
                notification_type: NotificationType::Info,
            });
            return;
        }

        // we will only reach this point if the game is still InProgress
        self.advance_turn();
    }

    // If every tile for the given indices is occupied AND is occupied by the
    // same player, return that player. Else return None.
    fn single_player_occupying_indices(&self, indices: &[Indices]) -> Option<Player> {
        let mut maybe_running_occupier: Option<Player> = None;
        let tiles = indices
            .iter()
            .map(|indices| Coordinates::from_indices(indices).unwrap())
            .map(|coords| self.board.tiles.get(&coords).unwrap())
            .collect::<Vec<&Tile>>();
        for tile in tiles {
            use TileOccupationState::*;
            match (maybe_running_occupier, tile.occupation_state) {
                (_, Empty) => return None,
                (None, Occupied(tile_occupier)) => maybe_running_occupier = Some(tile_occupier),
                (Some(running_occupier), Occupied(tile_occupier)) => {
                    if running_occupier != tile_occupier {
                        return None;
                    } else {
                        continue;
                    }
                }
            }
        }
        maybe_running_occupier
    }

    fn advance_turn(&mut self) {
        self.turn_number += 1;
    }

    fn get_current_turn_player(&self) -> Player {
        let turn_index = (self.turn_number - 1) % 2;
        self.players[turn_index]
    }

    fn render_board(&self) -> String {
        let mut rendered_grid = String::new();
        let column_headers = Coordinates::COLUMN_LETTERS
            .iter()
            .take(self.grid_dimensions)
            .map(|char| format!(" {} ", char))
            .collect::<Vec<_>>()
            .join(" ");
        let column_header_row = format!("   {}\n", column_headers.dimmed());
        rendered_grid.push_str(&column_header_row);
        rendered_grid.push_str("\n");
        // NOTE: we operate on the assumption that the board is a square -- its
        // number of rows and columns are equal, and every one of them contains
        // the same number of items
        for row_index in 0..self.grid_dimensions {
            let mut cells: Vec<String> = Vec::new();
            for column_index in 0..self.grid_dimensions {
                let coords = Coordinates::from_indices(&Indices {
                    row: row_index,
                    column: column_index,
                })
                .unwrap();
                let tile = self.board.tiles.get(&coords).unwrap();
                cells.push(format!(" {} ", tile));
            }
            let row_number = (row_index + 1).to_string();
            let tiles = cells.join("|");
            let tiles_row = format!("{}  {}\n", row_number.dimmed(), tiles);
            rendered_grid.push_str(&tiles_row);
            // If the row we just added wasn't the last one...
            if row_index + 1 < self.grid_dimensions {
                // ... then build and add a divider row.
                let divider = vec!["---"; self.grid_dimensions].join("+");
                let divider_row = format!("   {}\n", divider);
                rendered_grid.push_str(&divider_row);
            }
        }
        rendered_grid
    }

    fn new(num_rows_or_columns: usize) -> Result<Self> {
        if num_rows_or_columns < Self::MIN_NUM_ROWS_OR_COLUMNS
            || num_rows_or_columns > Self::MAX_NUM_ROWS_OR_COLUMNS
        {
            return Err(anyhow!(
                "Number of rows/columns on game board must be between {} and {}.",
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
                    Coordinates::from_indices(&Indices {
                        row: row_index,
                        column: column_index,
                    })
                    .unwrap(),
                    Tile {
                        occupation_state: TileOccupationState::Empty,
                        display_state: TileDisplayState::Normal,
                    },
                );
            }
        }
        Ok(Self {
            players,
            board: Board { tiles },
            notification: None,
            grid_dimensions: num_rows_or_columns,
            turn_number: 1,
            outcome: GameOutcome::InProgress,
        })
    }
}

// flesh out README
// try embedding an mp4 in README
// can I clean up 'cell' logic using map + join, so I join with '|' char?
// clear tile states at start of turn, so e.g. an earlier error's red tile won't show up for a
//   later coords-parsing error?
// refactor try_execute_turn error-handling logic to not repeat handle_error call, and to not
//   need `unwrap`; one tricky point is conditionally having coords to turn red, and having
//   different success types from our two results (Coordinates versus ()).
// refactor column headers out to coordinates, renamed to something else?
// refactor away `row_index + 1` in favor of something leveraging Coordinates
// refactor render_board into a new Board.render fn, or even better, impl Display for board
// refactor out opening three spaces from every row into something shared
// find a way to mutate tiles, not rebuild whole board? only if I can prevent inconsistent state
//   from partially-completed update, though. beware current way I set victory tiles one by one.
// can I nuke Indices struct? if not, add comments to it and Coordinates about which is user-facing
// refactor get_tile_from_indices to use Indices struct?
// refactor to not call try_execute_turn from itself? rather only call into a smaller helper fun?
// refactor so you'd only have a Coordinates object if it fit within game's board dimensions?
// only build vec of vecs for rows/cols/diagonals that can result in victory one time, on game init
//   and try to do 1x pass to build col + row vecs, not 2x passes
// refactor update_outcome into several fns? it does a lot. also figure out how best to advnce turn
// refactor try_execute_turn to be a method on game? only question is whether `main` should be the
//   one dealing with user inputs or whether that really does belong in Game instead
// any way to combine victory checks with draw check, to iterate through all tiles just once?
// try to have current-tile lookup and tile-replacement logic share a single get call if possible
// don't recompile regex on every turn
// sort out naming of 'indices' versus 'indices set'; latter isn't actually a set (but could be!)
// use official turn-building logic even during init?
// try to insert more context into my various error messages -- like, include any relevant coordinates
// stash max_index somewhere so it's only defined once? and/or have a fn to calculate it?
// do I need both a get_tile_from_indices and a get_mut_tile_from_indices? how to obviate need for both?
// refactor out to multiple modules/files
// refactor win-detection logic to something more elegant, that at least doesn't repeat self.winner assignment
// fix up cross-talk between Coordinates and Game (Coordinates refers to an attr of Game)
// share any coordinates-validating logic between initial game setup and within-turn user-input validation
// refactor to smaller functions, esp with more `new()` functions
// add test coverage, esp for coordinates parsing and full-game execution
// maybe clean up logic for how we look up tiles -- by indices always? by 'coords' always?
// maybe clean up how we build coords, so it's more foolproof about adding 1 to convert 0-indexed to 1
// is it possible to fully avoid use of unwrap?
// somehow make get_tile_from_indices less dangerous, by letting it have named parmams via some kind of struct?
//   and/or make it return a Result?
// choose carefully between iter, into_iter
// am I handling every possible error? see context, anyhow!, unwrap, `?`
