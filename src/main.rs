// main.rs
// entry point

mod backend;
mod frontend;
mod models;

use std::io::{self, Write};

use anyhow::Result;
use backend::Backend;
use frontend::runfrontend;

const DEBUG: bool = true;

fn main() -> Result<()> {
    if !DEBUG {
        // initialize backend
        let mut backend = Backend::new();

        // start frontend, passing the backend for interaction
        runfrontend(&mut backend)?;

        // cleanup (handled by frontend for now, or call backend.shutdown() if needed)
        // backend cleanup
        backend.shutdown()?;

    } else {
        println!("--- debug CLI ---");
        println!("available commands: play, pause, next, prev, state, add <url>, queue, exit, help");
    
        // initialize the backend
        let mut backend = Backend::new();
        // TODO: Maybe load some initial test data into the backend if needed
        backend.addplaylist(models::Playlist { name: "Test Playlist".to_string(), tracks: Vec::new() });
    
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
                    if let Err(e) = backend.play() {
                        println!("error playing: {}", e);
                    } else {
                        println!("play command sent.");
                    }
                    println!("current playing state: {}", backend.getplayingstate());
                }
                "pause" => {
                    if let Err(e) = backend.pause() {
                        println!("error pausing: {}", e);
                    } else {
                        println!("pause command sent.");
                    }
                    println!("current playing state: {}", backend.getplayingstate());
                }
                "next" => {
                    if let Err(e) = backend.nexttrack() {
                        println!("error going to next track: {}", e);
                    } else {
                        println!("next command sent.");
                    }
                    println!("current track: {:?}", backend.getcurrentsong());
                }
                "prev" => {
                    if let Err(e) = backend.prevtrack() {
                        println!("error going to previous track: {}", e);
                    } else {
                        println!("prev command sent.");
                    }
                }
                "state" => {
                    // print the current backend state
                    println!("{:#?}", backend.getstate());
                }
                "add" => {
                    if args.len() == 1 {
                        let url = args[0].to_string();
                        // create a dummy track for testing
                        let track = models::Track {
                            title: format!("track at {}", url),
                            artist: "unknown".to_string(),
                            duration: 0,
                            url,
                        };
                        backend.addtoqueue(track);
                        println!("added track to queue.");
                    } else {
                        println!("usage: add <url>");
                    }
                    println!("current queue:");
                    for (i, track) in backend.getstate().player.queue.iter().enumerate() {
                        println!("  {}: {} - {}", i, track.artist, track.title);
                    }
                }
                "queue" => {
                    println!("current queue:");
                    for (i, track) in backend.getstate().player.queue.iter().enumerate() {
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
    }

    Ok(())
}
