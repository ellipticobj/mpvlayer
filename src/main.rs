// main.rs
// entry point

mod backend;
mod frontend;
mod models;

use std::io::{self, Write};

use anyhow::Result;
use backend::Backend;
use frontend::runfrontend;

const DEBUG: bool = false;

fn main() -> Result<()> {
    if DEBUG {
        println!("--- debug CLI ---");
        println!("available commands: play, pause, next, prev, state, add <url>, queue, exit, help");
    
        // initialize the backend
        let mut backend = Backend::new();

        // dummy track
        let testtrack = models::Track { title: "losing interest".to_string(), artist: "adore".to_string(), url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string() };
        let testtrack1 = models::Track { title: "losing interest but again".to_string(), artist: "adore".to_string(), url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string() };
        backend::set::addplaylist(&mut backend, models::Playlist { name: "sigma 1".to_string(), tracks: vec![testtrack.clone()] });
        backend::set::addplaylist(&mut backend, models::Playlist { name: "sigma 2".to_string(), tracks: vec![testtrack1.clone()] });
        backend::set::addtoqueue(&mut backend, testtrack);
        backend::set::addtoqueue(&mut backend, testtrack1);
    
        let mut inputbuffer = String::new();
    
        // main command loop
        loop {
            print!("> "); // prompt
            io::stdout().flush()?;
    
            // clear the buffer and read the next line
            inputbuffer.clear();
            if io::stdin().read_line(&mut inputbuffer)? == 0 {
                // handle eof
                println!("exiting...");
                break;
            }
    
            // parse the input
            let parts: Vec<&str> = inputbuffer.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
    
            let command = parts[0];
            let args = &parts[1..];
    
            // execute commands
            match command {
                "play" => {
                    if let Err(e) = backend::set::playpause(&mut backend, ) {
                        println!("error playing: {}", e);
                    } else {
                        println!("play command sent.");
                    }
                    println!("current playing state: {}", backend::get::playingstate(&backend));
                }
                "pause" => {
                    if let Err(e) = backend::set::playpause(&mut backend, ) {
                        println!("error pausing: {}", e);
                    } else {
                        println!("pause command sent.");
                    }
                    println!("current playing state: {}", backend::get::playingstate(&backend));
                }
                "next" => {
                    if let Err(e) = backend::set::next(&mut backend, ) {
                        println!("error going to next track: {}", e);
                    } else {
                        println!("next command sent.");
                    }
                    println!("current track: {:?}", backend::get::currentsong(&backend));
                }
                "prev" => {
                    if let Err(e) = backend::set::prev(&mut backend, ) {
                        println!("error going to previous track: {}", e);
                    } else {
                        println!("prev command sent.");
                    }
                    println!("current track: {:?}", backend::get::currentsong(&backend));
                }
                "state" => {
                    // print the current backend state
                    println!("{:#?}", backend::get::state(&backend));
                }
                "add" => {
                    if args.len() == 1 {
                        let url = args[0].to_string();
                        // create a dummy track for testing
                        let track = models::Track {
                            title: format!("track at {}", url),
                            artist: "unknown".to_string(),
                            url,
                        };
                        backend::set::addtoqueue(&mut backend, track);
                        println!("added track to queue.");
                    } else {
                        println!("usage: add <url>");
                    }
                    println!("current queue:");
                    for (i, track) in backend::get::state(&backend).player.queuestate.queue.iter().enumerate() {
                        println!("  {}: {} - {}", i, track.artist, track.title);
                    }
                }
                "queue" => {
                    println!("current queue:");
                    for (i, track) in backend::get::state(&backend).player.queuestate.queue.iter().enumerate() {
                        println!("  {}: {} - {}", i, track.artist, track.title);
                    }
                }
                "exit" | "quit" | "q" => {
                    println!("exiting...");
                    break;
                }
                "help" => {
                    println!("available commands: play, pause, next, prev, state, add <url>, queue, exit, help");
                }
                _ => {
                    println!("unknown command: {}", command);
                }
            }
        }
    } else {
        // initialize backend
        let mut backend = Backend::new();

        // dummy track
        let testtrack = models::Track { title: "losing interest".to_string(), artist: "adore".to_string(), url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string() };
        let testtrack1 = models::Track { title: "losing interest but again".to_string(), artist: "adore".to_string(), url: "https://www.youtube.com/watch?v=HtR4PkPJiBk".to_string() };
        backend::set::addplaylist(&mut backend, models::Playlist { name: "sigma 1".to_string(), tracks: vec![testtrack.clone()] });
        backend::set::addplaylist(&mut backend, models::Playlist { name: "sigma 2".to_string(), tracks: vec![testtrack1.clone()] });
        backend::set::addtoqueue(&mut backend, testtrack);
        backend::set::addtoqueue(&mut backend, testtrack1);

        // start frontend, passing the backend for interaction
        runfrontend(backend)?;
    } 

    Ok(())
}
