use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::stdout;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::abort;
use std::sync::atomic::Ordering;

use super::input_handler;
use super::widgets::input_field;
use super::widgets::input_field::InputField;

use crate::debug;
use crate::log;
use crate::search::engines::SearchEngine;
use crate::tui::widgets;
use crate::utils::read_lines;
use crate::utils::Dimensions;

use cb::channel::unbounded;
use debug::*;
use phf::set::Iter;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read};
use std::sync::{atomic, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam as cb;

use crossterm as ct;
use crossterm::cursor;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::{self as cte, Event};
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::ExecutableCommand;

use anyhow as ah;
use anyhow::Context;
use widgets::previewer::ViewPort;



#[derive(Debug, Clone)]
pub enum SearchThreadMessage {
    Query(String),
    Results(Vec<(usize, usize)>),
    Kill,
}

#[derive(Debug)]
pub struct TuiState {
    /// Kills the event loop when set to true. This will also set st_kill, and
    /// wait for the search thread to join before properly exiting.
    pub el_kill: bool,

    pub input_field: InputField,

    pub search_query: String,
    pub search_results: Vec<(usize, usize)>, // Row, Col
    pub search_result_index: usize,
    pub search_buffer: String,
    pub search_cursor_index: usize,

    pub st_handle: Option<JoinHandle<()>>,
    pub st_kill: Arc<atomic::AtomicBool>,

    pub search_buffer_history: Vec<String>,

    pub doc_lines: Vec<(usize, String)>,

    pub preview_jump: bool,
    pub preview_viewport: ViewPort,

    /// How many digits are required to rerpesent the maximum line count.
    pub linum_digits: usize,

    pub preview_offset: isize,
    pub preview_context: usize,
    pub terminal_size: (u16, u16),
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            input_field: InputField::default(),

            el_kill: false,
            st_kill: Arc::new(atomic::AtomicBool::new(false)),
            preview_context: 3,
            search_query: String::new(),
            search_results: Vec::new(),
            search_result_index: 0,
            search_buffer: String::new(),
            search_cursor_index: 0,
            st_handle: None,
            search_buffer_history: Vec::new(),
            doc_lines: Vec::new(),
            linum_digits: 0,
            preview_offset: 0,
            terminal_size: (13u16, 80u16),
            preview_viewport: ViewPort {
                start_line: 0,
                context: 7,
            },
            preview_jump: false,
        }
    }
}

pub fn event_loop<SE: SearchEngine>(subject: PathBuf) -> ah::Result<()> {
    let mut state = TuiState::default();

    {
        let lines = read_lines(&subject)?;
        let longest = lines.iter().map(String::len).max().unwrap_or(0);
        state.linum_digits = (longest as f64).log10().ceil() as usize + 1;
        state.doc_lines = lines.into_iter().enumerate().collect();
    }

    let (st_query_send, st_query_recv) = unbounded::<String>();
    let (st_result_send, st_result_recv) = unbounded::<Vec<(usize, usize)>>();

    // Make copies for thread to own.
    let st_kill = state.st_kill.clone();
    let doc_lines = state.doc_lines.clone();

    state.st_handle = Some(thread::spawn(move || {
        // Transfer ownership of variables to the thread.
        let st_query_recv = st_query_recv;
        let st_result_send = st_result_send;
        let st_kill = st_kill;
        let doc_lines = doc_lines;

        // Instantiate the search engine.
        let mut st_search_engine = SE::default();

        while !st_kill.load(Ordering::SeqCst) {
            match st_query_recv.try_recv() {
                Ok(query) => {
                    let results = st_search_engine.search(&doc_lines, &query);
                    state.preview_offset = 0;

                    if let Err(e) = st_result_send.send(results) {
                        eprintln!("Failed to send results back to main thread: {:?}", e);
                        abort();
                    }
                }

                Err(cb::channel::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(5));
                }

                Err(cb::channel::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
    }));

    // Before any rendering is done, we need to determine the "anchor"
    // row and column, which is where all rendering will be done relative
    // to, rather than rendering being done with absolute coordinates.

    let Ok(anchor) = cursor::position() else {
        ah::bail!("Failed to get cursor position");
    };

    let mut anchor: (usize, usize) = (anchor.0 as usize, anchor.1 as usize);

    // Also enable raw mode for the terminalterm.
    enable_raw_mode()?;
    stdout().execute(Hide)?;

    let mut hasher = DefaultHasher::new();
    state.search_buffer.hash(&mut hasher);
    let mut prev_hash = hasher.finish();

    let (mut avail_cols, mut avail_rows) =
        term_size::dimensions().context("Attempt to initially retrieve terminal size.")?;

    let (ccol, crow) =
        ct::cursor::position().context("Getting cursor location in render loop initially")?;
    log!("IC {}R {}C", crow, ccol);
    log!("ID {}R {}C", avail_rows, avail_cols);

    while !state.el_kill {
        let (cols, rows) =
            term_size::dimensions().context("Attempt to retrieve terminal size for comparison.")?;

        let (nccol, ncrow) =
            ct::cursor::position().context("Getting cursor location in render.")?;

        log!("SC {}R {}C", ncrow, nccol);

        if rows != avail_rows || cols != avail_cols {
            log!("{}R -> {}R, {}C -> C{}", avail_rows, rows, avail_cols, cols);

            avail_rows = rows;
            avail_cols = cols;
            // if cols != avail_cols {
            //     let shrank: bool = cols < avail_cols;
            //     let delta = cols.max(avail_cols) - cols.min(avail_cols);
            //
            //     // Terminal size decreased, our reference point, while valid, is
            //     // no longer going to redraw the rows
            //
        }

        input_handler::handle_inputs(&mut state)?;
        // --- Receive Search Results -----------------------------------------
        // It makes more sense to do this first, as we avoid sleeping through
        // the time taken to handle input events, and then render the TUI. It
        // gives the search thread a little more time to do its work.

        if let Ok(results) = st_result_recv.try_recv() {
            state.search_results = results;

            if state.search_results.is_empty() {
                state.search_result_index = 0
            } else if state.search_result_index >= state.search_results.len() {
                state.search_result_index = state.search_results.len() - 1;
            }
        }

        // --- Send Search Query ----------------------------------------------
        //  TODO: Implement a debounce mechanism to not spam the sarch thread.
        //  In other words, only pass the search buffer to the search thread
        //  after an amount of milliseconds has passed since the last change.
        state.search_buffer.hash(&mut hasher);

        let new_hash = hasher.finish();

        if new_hash != prev_hash {
            st_query_send.send(state.search_buffer.clone()).unwrap();
            prev_hash = new_hash;
        }

        // Render the TUI based on the state of the event loop, i.e. state.
        // The rendering logic should be apart from the event loop logic.
        // Its sole responsibility is to render state.
        //
        log!("Prior Anchor {:?}", anchor);
        let (ncr, ncc) = ct::cursor::position().context("Getting cursor location in render.")?;
        log!("Post CPos {}R, {}C", ncr, ncc);
        widgets::components(&mut state, &anchor)?;
        let (ncr, ncc) = ct::cursor::position().context("Getting cursor location in render.")?;
        log!("Post Set CPos {}R, {}C", ncr, ncc);
        log!("Post Anchor {:?}", anchor);

        let Ok(na) = cursor::position() else {
            ah::bail!("Failed to get cursor position");
        };

        anchor = (na.0 as usize, na.1 as usize);
        log!("Post Set Anchor {:?}", anchor);
    }

    // If we're here, then the event loop has been killed.
    // We should also kill the search thread.

    state.st_kill.store(true, Ordering::SeqCst);

    state
        .st_handle
        .take()
        .context("Atempted to take the st_handle")?
        .join()
        .ok()
        .context("Failed to join search thread")?;

    // We should also make sure to clean up the TUI before exiting.
    widgets::cleanup()?;

    disable_raw_mode()?;
    stdout().execute(Show)?;

    Ok(())
}
