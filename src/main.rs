extern crate ncurses;

use std::str;
use std::panic;
use std::process::{Command, Stdio};
use std::io::Write;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use dirs::home_dir;
use std::time::SystemTime;
use chrono;
use ncurses::{initscr, endwin, noecho, echo, stdscr, refresh, mvaddstr, getch, cbreak, nocbreak, keypad, curs_set, CURSOR_VISIBILITY};
use serde::{Serialize, Deserialize};
use serde_json;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn get_choices() -> Result<Vec<String>> {
    let output = Command::new("apg")
        .args(["-a", "1", "-m", "12"])
        .output()?;
    let ret = str::from_utf8(&output.stdout)?
        .split("\n")
        .map(|x| x.to_string())
        .filter(|s| s.len() > 0)
        .collect();
    Ok(ret)
}

fn draw_cursor(pos: i32, nchoices: i32) {
    for i in 0..nchoices {
        if i == pos {
            mvaddstr(i, 0, "==>");
        } else {
            mvaddstr(i, 0, "   ");
        }
    }
}

fn curse() {
    initscr();
    noecho();
    cbreak();
    keypad(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
}

fn uncurse() {
    // echo();
    // nocbreak();
    // keypad(stdscr(), false);
    // curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    endwin();
}

#[derive(Serialize, Deserialize, Debug)]
struct Pw {
    pw: String,
    created_at: chrono::DateTime<chrono::Local>,
}

fn copy_to_clipboard(pw: &str) -> Result<()> {
    // We need to use xclip
    // because there is no library that will copy to the clipboard
    // and have it survive the process.
    // For cli_clipboard, see https://github.com/ActuallyAllie/cli-clipboard/issues/7
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn().expect("xclip");
    let child_stdin = child.stdin.as_mut().expect("xclip stdin");
    child_stdin.write_all(pw.as_bytes())?;
    Ok(())
}

fn write_to_history_file(pw: &str) -> Result<()> {
    // Make ~/.pw if needed:
    let mut pw_dir = home_dir().expect("no home");
    pw_dir.push(".pw");
    fs::create_dir_all(pw_dir.clone())?;
    let mut perms = fs::metadata(pw_dir.clone())?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(pw_dir.clone(), perms)?;

    // Append to ~/.pw/history:
    let mut hist_file = pw_dir.clone();
    hist_file.push("history");
    let mut f = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(hist_file)?;
    let ent = Pw { pw: pw.to_string(), created_at: chrono::offset::Local::now() };
    write!(f, "{}\n", serde_json::to_string(&ent)?);
    Ok(())
}

fn choose_pw(pw: &str) -> Result<()> {
    copy_to_clipboard(pw)?;
    write_to_history_file(pw)?;
    Ok(())
}

fn main() -> Result<()> {
    let choices = get_choices()?;
    let nchoices: i32 = choices.len().try_into()?;
    // println!("{:?}", choices);

    curse();
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        uncurse();
        prev_hook(info);
    }));

    // Draw the choices:
    for (i, ch) in choices.iter().enumerate() {
        mvaddstr(i.try_into()?, 4, ch);
    }
    let mut cur_pos: i32 = 0;
    draw_cursor(cur_pos, nchoices);
    refresh();
    loop {
        match getch() {
            113 => {    // q
                break;
            },
            259 | 107 => { // up | k
                cur_pos = (cur_pos - 1).rem_euclid(nchoices);
                draw_cursor(cur_pos, nchoices);
            },
            258 | 106 => { // down | j
                cur_pos = (cur_pos + 1) % nchoices;
                draw_cursor(cur_pos, nchoices);
            },
            343 | 10 | 13 => {    // enter | \n | \r respectively
                choose_pw(&choices[cur_pos as usize])?;
                break;
            },
            _ => {
                // do nothing
            },
        }
    }
    uncurse();
    Ok(())
}
