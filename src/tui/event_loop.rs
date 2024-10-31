use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::stdout;
use std::path::PathBuf;
use std::process::abort;
use std::sync::atomic::Ordering;

use super::input_handler;
use super::render;
use super::render::input_field;

use crate::search::engines::SearchEngine;
use crate::utils::Dimensions;

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

#[derive(Debug, Clone)]
pub enum ViMode {
    Normal,
    Insert,
}

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

    pub search_query: String,
    pub search_results: Vec<(usize, usize)>, // Row, Col
    pub search_buffer: String,
    pub search_cursor_index: usize,

    pub st_handle: Option<JoinHandle<()>>,
    pub st_kill: Arc<atomic::AtomicBool>,

    pub search_buffer_history: Vec<String>,

    pub vi_mode: ViMode,
    pub vi_chord: Vec<char>,

    pub preview_lines: Vec<String>,
    pub preview_context: usize,
    pub preview_dimensions: Dimensions,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            el_kill: false,
            vi_chord: Vec::<char>::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_buffer_history: Vec::new(),
            search_cursor_index: 0,
            search_buffer: String::new(),
            preview_lines: Vec::new(),
            preview_context: 0,
            vi_mode: ViMode::Normal,
            preview_dimensions: Dimensions::default(),
            st_handle: None,
            st_kill: Arc::new(atomic::AtomicBool::new(false)),
        }
    }
}

pub fn event_loop<SE: SearchEngine>(subject: PathBuf) -> ah::Result<()> {
    let mut state = TuiState::default();

    let file = OpenOptions::new()
        .read(true)
        .open(&subject)
        .context("Failed to open file")?;

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    // We'll send queries to the thread using st_query_send, that one's
    // for us. The search thread will receive queries from st_query_recv,
    // that one's for the thread. We'll move st_query_recv into the thread.
    let (st_query_send, st_query_recv) = cb::channel::unbounded::<String>();

    // The thread will send results to us using st_result_send, that one's
    // for the thread. We'll receive them using st_result_recv, that one's
    // for us. We'll move st_result_send into the thread.
    let (st_result_send, st_result_recv) = cb::channel::unbounded::<Vec<(usize, usize)>>();

    // We'll also make a copy of the atomic kill switch for the thread to
    // know when to stop looping and die.
    let st_kill = state.st_kill.clone();

    state.st_handle = Some(thread::spawn(move || {
        let st_query_recv = st_query_recv;
        let st_result_send = st_result_send;
        let st_kill = st_kill;

        // The search thread needs a search engine to use.
        let mut st_search_engine = SE::default();

        while !st_kill.load(Ordering::SeqCst) {
            match st_query_recv.try_recv() {
                Ok(query) => {
                    let results = st_search_engine.search(&lines, &query);

                    if let Err(e) = st_result_send.send(results) {
                        // We can't send results back to the main thread.
                        // We should kill the thread and abort the program.
                        // Without the ability to send results back, then
                        // the program is in an unrecoverable state.

                        eprintln!("Failed to send results back to main thread: {:?}", e);
                        abort();
                    }
                }

                Err(cb::channel::TryRecvError::Empty) => {
                    // We don't want to overwhelm the CPU, so we'll sleep.
                    thread::sleep(Duration::from_millis(5));
                }

                Err(cb::channel::TryRecvError::Disconnected) => {
                    // If the main thread receiver has disconnected, then
                    // we should definitely kill the thread.
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

    let anchor: (usize, usize) = (anchor.0 as usize, anchor.1 as usize);

    // Also enable raw mode for the terminal.
    enable_raw_mode()?;
    stdout().execute(Hide)?;

    let mut hasher = DefaultHasher::new();
    state.search_buffer.hash(&mut hasher);
    let mut prev_hash = hasher.finish();

    while !state.el_kill {
        input_handler::handle_inputs(&mut state)?;
        // --- Receive Search Results ----------------------------------------- 
        // It makes more sense to do this first, as we avoid sleeping through
        // the time taken to handle input events, and then render the TUI. It
        // gives the search thread a little more time to do its work.

        if let Ok(results) = st_result_recv.try_recv() {
            state.search_results = results;
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
        render::components(&mut state, &anchor)?;
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
    render::cleanup()?;

    disable_raw_mode()?;
    stdout().execute(Show)?;

    Ok(())
}
