use std::convert::identity;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::stdout;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::abort;
use std::rc::Rc;
use std::sync::atomic::Ordering;

use cb::channel::unbounded;
use crossterm::cursor;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::poll;
use crossterm::event::{self as cte, Event};
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::ExecutableCommand;
use debug::*;
use phf::set::Iter;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read};
use std::sync::{atomic, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam as cb;
use crossterm as ct;

use anyhow as ah;
use anyhow::Context;

use super::components::input_field::*;
use super::input_handler;
use crate::debug;
use crate::log;
use crate::search_engines::SearchEngine;
use crate::tui::component::Component;
use crate::tui::point::Point;
use crate::utils::{percentage_of_columns, read_lines};

#[derive(Debug)]
pub enum SearchThreadMessage {
    Query(String),
    Results(Vec<(usize, usize)>),
    Kill,
}

#[derive(Debug, Default)]
pub struct Tui {
    pub opts: TuiOptions,
    pub terminal_size: Point,

    pub thread_kill_tui: Arc<atomic::AtomicBool>,
    pub thread_kill_search: Arc<atomic::AtomicBool>,

    thread_jh_search: Option<JoinHandle<()>>,

    // ---- Components ------------------------
    pub input_field: InputField,
}

#[derive(Debug)]
pub struct TuiOptions {
    /// How large, horizontally and veritcally, is the TUI allowed to before
    /// the UI components cut off.
    max_dimensions: Point,
    search_engine: SearchEngine,
}

impl Default for TuiOptions {
    fn default() -> Self {
        Self {
            max_dimensions: Point {
                row: 15,
                col: percentage_of_columns(37.0).unwrap_or(35) as u16,
            },
            search_engine: SearchEngine::Fuzzy,
        }
    }
}

impl Tui {
    fn new(opts: TuiOptions) -> ah::Result<Self> {
        let mut tui = Tui::default();
        tui.terminal_size = Point::from(ct::terminal::size()?);
        tui.opts = opts;
        Ok(tui)
    }

    pub fn start() -> ah::Result<()> {
        Ok(())
    }
}

// pub fn event_loop<SE: SearchEngine>(subject: PathBuf) -> ah::Result<()> {
//     let mut tui = TuiState::default();
//
//     let subject_lines = read_lines(&subject)?;
//
//     {
//         // tui.linum_digits = (longest as f64).log10().ceil() as usize + 1;
//         // tui.markdown_lines = lines.into_iter().enumerate().collect();
//     }
//
//     let (st_query_send, st_query_recv) = unbounded::<String>();
//     let (st_result_send, st_result_recv) = unbounded::<Vec<(usize, usize)>>();
//
//     // Make copies for thread to own.
//     // let doc_lines = tui.markdown_lines.clone();
//
//     tui.searcher_join_handle = Some(thread::spawn(move || {
//         // Transfer ownership of variables to the thread.
//         let st_query_recv = st_query_recv;
//         let st_result_send = st_result_send;
//
//         let kill = tui.kill_se_thread.clone();
//         let lines = subject_lines.clone();
//
//         let st_search_engine = SE::default();
//
//         while !kill.load(Ordering::SeqCst) {
//             match st_query_recv.try_recv() {
//                 Ok(query) => {
//                     let results = st_search_engine.search(&lines, &query);
//
//                     if let Err(e) = st_result_send.send(results) {
//                         eprintln!("Failed to send results back to main thread: {:?}", e);
//                         abort();
//                     }
//                 }
//
//                 Err(cb::channel::TryRecvError::Empty) => {
//                     thread::sleep(Duration::from_millis(5));
//                 }
//
//                 Err(cb::channel::TryRecvError::Disconnected) => {
//                     break;
//                 }
//             }
//         }
//     }));
//
//     // Before any rendering is done, we need to determine the "anchor"
//     // row and column, which is where all rendering will be done relative
//     // to, rather than rendering being done with absolute coordinates.
//
//     let Ok(anchor) = cursor::position().map(Point::from) else {
//         ah::bail!("Cannot get cursor position");
//     };
//
//     // Also enable raw mode for the terminalterm.
//     enable_raw_mode()?;
//     stdout().execute(Hide)?;
//
//     let (mut avail_cols, mut avail_rows) =
//         term_size::dimensions().context("Attempt to initially retrieve terminal size.")?;
//
//     let (ccol, crow) =
//         ct::cursor::position().context("Getting cursor location in render loop initially")?;
//
//     #[rustfmt::skip]
//     let mut widget_layout: Vec<Box<dyn Component>> = vec![
//         Box::new(InputField::default()),
//     ];
//
//     while !tui.kill_event_loop {
//         let event: Option<Event> = poll(Duration::from_millis(10))
//             .is_ok_and(identity)
//             .then(|| ct::event::read().ok())
//             .flatten();
//
//         // Rationale: 6d12599a
//         let (tcols, trows) = term_size::dimensions()
//             .context("Attempt to determine terminal size during event loop.")?;
//
//         // cursor::position().map_err(|e| cursor::MoveTo(0,0)).map_err(op):
//
//         tui.terminal_size = Point::from((tcols as u16, trows as u16));
//
//         // log!("SC {}R {}C", ncrow, nccol);
//
//         // if trows != avail_rows || tcols != avail_cols {
//         //     log!(
//         //         "{}R -> {}R, {}C -> C{}",
//         //         avail_rows,
//         //         trows,
//         //         avail_cols,
//         //         tcols
//         //     );
//         //
//         //     avail_rows = trows;
//         //     avail_cols = tcols;
//         // }
//         //
//         // input_handler::handle_inputs(&mut tui)?;
//         // --- Receive Search Results -----------------------------------------
//         // It makes more sense to do this first, as we avoid sleeping through
//         // the time taken to handle input events, and then render the TUI. It
//         // gives the search thread a little more time to do its work.
//
//         // if let Ok(results) = st_result_recv.try_recv() {
//         //     tui.search_results = results;
//         //
//         //     if tui.search_results.is_empty() {
//         //         tui.search_result_index = 0
//         //     } else if tui.search_result_index >= tui.search_results.len() {
//         //         tui.search_result_index = tui.search_results.len() - 1;
//         //     }
//         // }
//
//         // Render the TUI based on the state of the event loop, i.e. state.
//         // The rendering logic should be apart from the event loop logic.
//         let Ok(na) = cursor::position() else {
//             ah::bail!("Failed to get cursor position");
//         };
//
//         log!("Post Set Anchor {:?}", anchor);
//     }
//
//     // If we're here, then the event loop has been killed.
//     // We should also kill the search thread.
//
//     tui.kill_se_thread.store(true, Ordering::SeqCst);
//
//     tui.searcher_join_handle
//         .take()
//         .context("Atempted to take the st_handle")?
//         .join()
//         .ok()
//         .context("Failed to join search thread")?;
//
//     // We should also make sure to clean up the TUI before exiting.
//     ::cleanup()?;
//
//     disable_raw_mode()?;
//     stdout().execute(Show)?;
//
//     Ok(())
// }
