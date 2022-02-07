use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*, rngs::ThreadRng};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::{Duration, Instant};
use std::{borrow::Borrow, io};
use std::{
    collections::{HashMap, HashSet},
    fs,
};
// use std::{fmt::Result, sync::mpsc};
// use thiserror::Error;
// use tui::{
//     backend::CrosstermBackend,
//     layout::{Alignment, Constraint, Direction, Layout},
//     style::{Color, Modifier, Style},
//     text::{Span, Spans},
//     widgets::{
//         Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
//     },
//     Terminal,
// };

use std::os::unix::io::{IntoRawFd, RawFd};

use libc::size_t;

use std::process::Command;

use anyhow::{anyhow, Context, Result};

use walkdir::{DirEntry, WalkDir};

const DB_PATH: &str = "./data/db.json";
#[derive(Debug)]
pub struct FileDesc {
    fd: RawFd,
    close_on_drop: bool,
}

// #[derive(Error, Debug)]
// pub enum Error {
//     #[error("error reading the DB file: {0}")]
//     ReadDBError(#[from] io::Error),
//     #[error("error parsing the DB file: {0}")]
//     ParseDBError(#[from] serde_json::Error),
// }

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Serialize, Deserialize, Clone)]
struct Pet {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Pets,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Pets => 1,
        }
    }
}

fn is_not_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() == 0 || !s.starts_with("."))
        .unwrap_or(false)
}

impl FileDesc {
    /// Constructs a new `FileDesc` with the given `RawFd`.
    ///
    /// # Arguments
    ///
    /// * `fd` - raw file descriptor
    /// * `close_on_drop` - specify if the raw file descriptor should be closed once the `FileDesc` is dropped
    pub fn new(fd: RawFd, close_on_drop: bool) -> FileDesc {
        FileDesc { fd, close_on_drop }
    }

    pub fn read(&self, buffer: &mut [u8], size: usize) -> Result<usize> {
        let result = unsafe {
            libc::read(
                self.fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                size as size_t,
            ) as isize
        };

        if result < 0 {
            Err(anyhow!("bla"))
        } else {
            Ok(result as usize)
        }
    }

    /// Returns the underlying file descriptor.
    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }
}

pub fn tty_fd() -> Result<FileDesc> {
    let (fd, close_on_drop) = if unsafe { libc::isatty(libc::STDIN_FILENO) == 1 } {
        println!("STDIN===");
        (libc::STDIN_FILENO, false)
    } else {
        println!("TTY===");
        (
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/tty")?
                .into_raw_fd(),
            true,
        )
    };

    Ok(FileDesc::new(fd, close_on_drop))
}

#[derive(Debug)]
struct KernelCmdline {
    params: HashMap<String, Option<String>>,
}

impl KernelCmdline {
    fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }
    fn parse(mut self) -> Result<Self> {
        let raw = fs::read_to_string("/proc/cmdline").context("cannot open /proc/cmdline")?;
        let split: Vec<&str> = raw.trim().split(' ').collect();
        split.iter().for_each(|e| {
            if let Some((key, value)) = e.split_once('=') {
                self.params.insert(key.to_string(), Some(value.to_string()));
            } else {
                self.params.insert(e.to_string(), None);
            }
        });
        Ok(self)
    }
    fn get_bool(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    fn get_str(&self, key: &str) -> Option<String> {
        self.params.get(key).and_then(|e| e.to_owned())
    }
}

#[derive(Debug, Default)]
struct InstallerConfig {
    eve_nuke_disks: Option<Vec<String>>,
    eve_nuke_all_disks: bool,
    eve_install_disk: Option<String>,
    eve_persist_disk: Option<Vec<String>>,
    eve_install_server: Option<String>,
    eve_install_skip_rootfs: bool,
    eve_install_skip_config: bool,
    eve_install_skip_persist: bool,
    eve_pause_before_install: bool,
    eve_pause_after_install: bool,
    eve_blackbox: bool,
    eve_soft_serial: Option<String>,
    eve_reboot_after_install: bool,
    // helper fields
    persist_fs_zfs: bool,
}

impl InstallerConfig {
    fn from_cmdline(cmdline: &KernelCmdline) -> Self {
        let mut config = Self::default();
        config.eve_nuke_all_disks = cmdline.get_bool("eve_nuke_all_disks");
        config.eve_blackbox = cmdline.get_bool("eve_blackbox");
        config.eve_install_skip_config = cmdline.get_bool("eve_install_skip_config");
        config.eve_install_skip_persist = cmdline.get_bool("eve_install_skip_persist");
        config.eve_install_skip_rootfs = cmdline.get_bool("eve_install_skip_rootfs");
        config.eve_pause_after_install = cmdline.get_bool("eve_pause_after_install");
        config.eve_pause_before_install = cmdline.get_bool("eve_pause_before_install");
        config.eve_soft_serial = cmdline.get_str("eve_soft_serial");

        config.eve_install_server = cmdline.get_str("eve_install_server");
        config.eve_install_disk = cmdline.get_str("eve_install_disk");

        let eve_persist_disk = cmdline.get_str("eve_persist_disk");

        config.eve_persist_disk = eve_persist_disk
            .as_ref()
            .map(|e| e.trim().split(",").map(|e| e.to_string()).collect());

        config.persist_fs_zfs = eve_persist_disk.map_or(false, |e| e.trim().ends_with(","));

        config.eve_blackbox = cmdline.get_bool("eve_blackbox");
        config
    }
}

fn nuke_partition(disk: &str) -> Result<()> {
    Ok(())
}

fn run_os_command(cmdline: &str) -> Result<()> {
    let output = if let Some((cmd, params)) = cmdline.trim().split_once(' ') {
        Command::new(cmd).args(params.split(" ")).output()?
    } else {
        Command::new(cmdline.trim()).output()?
    };

    if output.status.success() {
        String::from_utf8(output.stdout)?
            .lines()
            .for_each(|line| println!("{}", line));
    } else {
        String::from_utf8(output.stderr)?
            .lines()
            .for_each(|line| println!("{}", line));
    }

    Ok(())
}

fn main() -> Result<()> {
    let cmd_line = KernelCmdline::new().parse()?;
    let config = InstallerConfig::from_cmdline(&cmd_line);
    println!("{:#?}", cmd_line);
    println!("{:#?}", config);

    run_os_command("mount")?;
    run_os_command("lsblk -anlb -o TYPE,NAME,SIZE,TRAN")?;
    let fd = tty_fd()?;
    println!("Raw desc: {}", fd.raw_fd());
    Ok(())
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     WalkDir::new("/dev")
//         .follow_links(true)
//         .into_iter()
//         .filter_entry(|e| is_not_hidden(e))
//         .filter_map(|v| v.ok())
//         .for_each(|x| println!("{}", x.path().display()));

//     let fd = tty_fd()?;
//     println!("Raw desc: {}", fd.raw_fd());

//     enable_raw_mode().context(format!("cannot switch to RAW mode"))?;

//     let (tx, rx) = mpsc::channel();
//     let tick_rate = Duration::from_millis(200);
//     thread::spawn(move || {
//         let mut last_tick = Instant::now();
//         loop {
//             let timeout = tick_rate
//                 .checked_sub(last_tick.elapsed())
//                 .unwrap_or_else(|| Duration::from_secs(0));

//             if event::poll(timeout).expect("poll works") {
//                 if let CEvent::Key(key) = event::read().expect("can read events") {
//                     tx.send(Event::Input(key)).expect("can send events");
//                 }
//             }

//             if last_tick.elapsed() >= tick_rate {
//                 if let Ok(_) = tx.send(Event::Tick) {
//                     last_tick = Instant::now();
//                 }
//             }
//         }
//     });

//     let stdout = io::stdout();
//     let backend = CrosstermBackend::new(stdout);
//     let mut terminal = Terminal::new(backend)?;
//     terminal.clear()?;

//     let menu_titles = vec!["Home", "Pets", "Add", "Delete", "Quit"];
//     let mut active_menu_item = MenuItem::Home;
//     let mut pet_list_state = ListState::default();
//     pet_list_state.select(Some(0));

//     let msize = terminal.size()?;

//     terminal
//         .current_buffer_mut()
//         .set_style(msize, Style::default().bg(Color::Black));

//     loop {
//         terminal.draw(|rect| {
//             let size = rect.size();

//             let chunks = Layout::default()
//                 .direction(Direction::Vertical)
//                 .margin(2)
//                 .constraints(
//                     [
//                         Constraint::Length(3),
//                         Constraint::Min(2),
//                         Constraint::Length(3),
//                     ]
//                     .as_ref(),
//                 )
//                 .split(size);

//             let copyright = Paragraph::new("pet-CLI 2022 - all rights reserved")
//                 .style(Style::default().fg(Color::LightCyan).bg(Color::Black))
//                 .alignment(Alignment::Center)
//                 .block(
//                     Block::default()
//                         .borders(Borders::ALL)
//                         .style(Style::default().fg(Color::White).bg(Color::Black))
//                         .title("Copyright")
//                         .border_type(BorderType::Plain),
//                 );

//             let menu = menu_titles
//                 .iter()
//                 .map(|t| {
//                     let (first, rest) = t.split_at(1);
//                     Spans::from(vec![
//                         Span::styled(
//                             first,
//                             Style::default()
//                                 .fg(Color::Yellow)
//                                 .bg(Color::Black)
//                                 .add_modifier(Modifier::UNDERLINED),
//                         ),
//                         Span::styled(rest, Style::default().fg(Color::White)),
//                     ])
//                 })
//                 .collect();

//             let tabs = Tabs::new(menu)
//                 .select(active_menu_item.into())
//                 .block(Block::default().title("Menu").borders(Borders::ALL))
//                 .style(Style::default().fg(Color::White).bg(Color::Black))
//                 .highlight_style(Style::default().fg(Color::Yellow))
//                 .divider(Span::raw("|"));

//             rect.render_widget(tabs, chunks[0]);
//             match active_menu_item {
//                 MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
//                 MenuItem::Pets => {
//                     let pets_chunks = Layout::default()
//                         .direction(Direction::Horizontal)
//                         .constraints(
//                             [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
//                         )
//                         .split(chunks[1]);
//                     let (left, right) = render_pets(&pet_list_state);
//                     rect.render_stateful_widget(left, pets_chunks[0], &mut pet_list_state);
//                     rect.render_widget(right, pets_chunks[1]);
//                 }
//             }
//             rect.render_widget(copyright, chunks[2]);
//         })?;

//         match rx.recv()? {
//             Event::Input(event) => match event.code {
//                 KeyCode::Char('q') => {
//                     disable_raw_mode()?;
//                     terminal.show_cursor()?;
//                     break;
//                 }
//                 KeyCode::Char('h') => active_menu_item = MenuItem::Home,
//                 KeyCode::Char('p') => active_menu_item = MenuItem::Pets,
//                 KeyCode::Char('a') => {
//                     add_random_pet_to_db().expect("can add new random pet");
//                 }
//                 KeyCode::Char('d') => {
//                     remove_pet_at_index(&mut pet_list_state).expect("can remove pet");
//                 }
//                 KeyCode::Down => {
//                     if let Some(selected) = pet_list_state.selected() {
//                         let amount_pets = read_db().expect("can fetch pet list").len();
//                         if selected >= amount_pets - 1 {
//                             pet_list_state.select(Some(0));
//                         } else {
//                             pet_list_state.select(Some(selected + 1));
//                         }
//                     }
//                 }
//                 KeyCode::Up => {
//                     if let Some(selected) = pet_list_state.selected() {
//                         let amount_pets = read_db().expect("can fetch pet list").len();
//                         if selected > 0 {
//                             pet_list_state.select(Some(selected - 1));
//                         } else {
//                             pet_list_state.select(Some(amount_pets - 1));
//                         }
//                     }
//                 }
//                 _ => {}
//             },
//             Event::Tick => {}
//         }
//     }

//     Ok(())
// }

// fn render_home<'a>() -> Paragraph<'a> {
//     let home = Paragraph::new(vec![
//         Spans::from(vec![Span::raw("")]),
//         Spans::from(vec![Span::raw("Welcome")]),
//         Spans::from(vec![Span::raw("")]),
//         Spans::from(vec![Span::raw("to")]),
//         Spans::from(vec![Span::raw("")]),
//         Spans::from(vec![Span::styled(
//             "pet-CLI",
//             Style::default().fg(Color::LightBlue),
//         )]),
//         Spans::from(vec![Span::raw("")]),
//         Spans::from(vec![Span::raw("Press 'p' to access pets, 'a' to add random new pets and 'd' to delete the currently selected pet.")]),
//     ])
//     .alignment(Alignment::Center)
//     .block(
//         Block::default()
//             .borders(Borders::ALL)
//             .style(Style::default().fg(Color::White).bg(Color::Black))
//             .title("Home")
//             .border_type(BorderType::Plain),
//     );
//     home
// }

// fn render_pets<'a>(pet_list_state: &ListState) -> (List<'a>, Table<'a>) {
//     let pets = Block::default()
//         .borders(Borders::ALL)
//         .style(Style::default().fg(Color::White).bg(Color::Black))
//         .title("Pets")
//         .border_type(BorderType::Plain);

//     let pet_list = read_db().expect("can fetch pet list");
//     let items: Vec<_> = pet_list
//         .iter()
//         .map(|pet| {
//             ListItem::new(Spans::from(vec![Span::styled(
//                 pet.name.clone(),
//                 Style::default().bg(Color::Black),
//             )]))
//         })
//         .collect();

//     let selected_pet = pet_list
//         .get(
//             pet_list_state
//                 .selected()
//                 .expect("there is always a selected pet"),
//         )
//         .expect("exists")
//         .clone();

//     let list = List::new(items).block(pets).highlight_style(
//         Style::default()
//             .bg(Color::Yellow)
//             .fg(Color::Black)
//             .add_modifier(Modifier::BOLD),
//     );

//     let pet_detail = Table::new(vec![Row::new(vec![
//         Cell::from(Span::raw(selected_pet.id.to_string())),
//         Cell::from(Span::raw(selected_pet.name)),
//         Cell::from(Span::raw(selected_pet.category)),
//         Cell::from(Span::raw(selected_pet.age.to_string())),
//         Cell::from(Span::raw(selected_pet.created_at.to_string())),
//     ])])
//     .header(Row::new(vec![
//         Cell::from(Span::styled(
//             "ID",
//             Style::default()
//                 .add_modifier(Modifier::BOLD)
//                 .bg(Color::Black),
//         )),
//         Cell::from(Span::styled(
//             "Name",
//             Style::default()
//                 .add_modifier(Modifier::BOLD)
//                 .bg(Color::Black),
//         )),
//         Cell::from(Span::styled(
//             "Category",
//             Style::default()
//                 .add_modifier(Modifier::BOLD)
//                 .bg(Color::Black),
//         )),
//         Cell::from(Span::styled(
//             "Age",
//             Style::default()
//                 .add_modifier(Modifier::BOLD)
//                 .bg(Color::Black),
//         )),
//         Cell::from(Span::styled(
//             "Created At",
//             Style::default()
//                 .add_modifier(Modifier::BOLD)
//                 .bg(Color::Black),
//         )),
//     ]))
//     .block(
//         Block::default()
//             .borders(Borders::ALL)
//             .style(Style::default().fg(Color::White).bg(Color::Black))
//             .title("Detail")
//             .border_type(BorderType::Plain),
//     )
//     .widths(&[
//         Constraint::Percentage(5),
//         Constraint::Percentage(20),
//         Constraint::Percentage(20),
//         Constraint::Percentage(5),
//         Constraint::Percentage(20),
//     ]);

//     (list, pet_detail)
// }

// fn read_db() -> Result<Vec<Pet>, Error> {
//     let db_content = fs::read_to_string(DB_PATH)?;
//     let parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
//     Ok(parsed)
// }

// fn add_random_pet_to_db() -> Result<Vec<Pet>, Error> {
//     let mut rng = rand::thread_rng();
//     let db_content = fs::read_to_string(DB_PATH)?;
//     let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
//     let catsdogs = match rng.gen_range(0..1) {
//         0 => "cats",
//         _ => "dogs",
//     };

//     let random_pet = Pet {
//         id: rng.gen_range(0..9999999),
//         name: "hello".to_owned(), //rng.sample_iter(Alphanumeric).take(10).collect(),
//         category: catsdogs.to_owned(),
//         age: rng.gen_range(1..15),
//         created_at: Utc::now(),
//     };

//     parsed.push(random_pet);
//     fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
//     Ok(parsed)
// }

// fn remove_pet_at_index(pet_list_state: &mut ListState) -> Result<(), Error> {
//     if let Some(selected) = pet_list_state.selected() {
//         let db_content = fs::read_to_string(DB_PATH)?;
//         let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
//         parsed.remove(selected);
//         fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
//         pet_list_state.select(Some(selected - 1));
//     }
//     Ok(())
// }
