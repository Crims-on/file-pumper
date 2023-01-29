use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.contains(&"-h".to_owned()) || args.contains(&"--help".to_owned()) {
        println!("Usage: file-pumper.exe <add|remove> <file path> [desired size]");
        return Ok(());
}

    let mode = &args[1];
    let path = Path::new(&args[2]);
    let desired_size = if args.len() == 4 {
        Some(args[3].parse::<u64>().unwrap())
    } else {
        None
    };

    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut current_size = file.metadata()?.len();
    const BUFFER_SIZE:u64 = 8192;
    let buffer = vec![0u8; BUFFER_SIZE as usize];
    let padding = vec![0u8; (desired_size.unwrap_or(current_size) % BUFFER_SIZE) as usize];

    if mode == "add" {
        file.seek(std::io::SeekFrom::End(0))?;

        let desired_size = desired_size.unwrap_or(current_size);
        while current_size < desired_size {
            let write_size = std::cmp::min(buffer.len() as u64, desired_size - current_size);
            file.write_all(&buffer[..write_size as usize])?;
            current_size = file.metadata()?.len();
        }

        if padding.len() > 0 {
            file.write_all(&padding)?;
        }
    } else if mode == "remove" {
        let mut end_pos = file.seek(std::io::SeekFrom::End(0))?;
        let mut current_pos = 0;
        let mut buffer = [0u8; BUFFER_SIZE as usize];

        while current_pos < end_pos {
            let read_size = std::cmp::min(BUFFER_SIZE, end_pos - current_pos);
            file.seek(std::io::SeekFrom::Start(current_pos))?;
            file.read_exact(&mut buffer[..read_size as usize])?;

            let mut all_zero = true;
            for i in 0..read_size as usize {
                if buffer[i] != 0 {
                    all_zero = false;
                    break;
                }
            }

            if all_zero {
                file.seek(std::io::SeekFrom::End(- (read_size as i64)))?;
                file.set_len(end_pos - read_size)?;
                end_pos -= read_size;
            } else {
                current_pos += read_size;
            }
        }
    } else {
        println!("Invalid mode. Mode must be either 'add' or 'remove'");
        return Ok(());
    }

    Ok(())
}
