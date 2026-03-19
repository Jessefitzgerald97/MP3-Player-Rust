use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::path::Path;
use std::fs;
use crate::album::{Song, Playlist};

pub struct AudioManager {
    _stream: OutputStream,        // Kept to maintain the audio connection
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,          // Optional since we might not always have a sink
    pub current_song: Option<String>, // Track what's currently playing
    is_playing: bool,
    pub current_playlist: Option<Playlist>,
}

impl AudioManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;

        Ok(Self {
            _stream,
            stream_handle,
            sink: None,
            current_song: None,
            is_playing: false,
            current_playlist: None,
        })
    }

    pub fn play_song_at_index(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        if let Some(playlist) = &mut self.current_playlist {
            playlist.play_song_at_index(index);
            self.play_current_song()?;
        }
        Ok(())
    }

    pub fn play(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            self.is_playing = true;
        }
    }

    pub fn play_current_song(&mut self) -> Result<(), Box<dyn Error>> {
        // First get the file path we need
        let file_path = if let Some(playlist) = &self.current_playlist {
            if let Some(song) = playlist.get_current_song() {
                song.song_file_path.clone()
            } else {
                return Ok(());  // No current song
            }
        } else {
            return Ok(());  // No playlist
        };

        self.load_and_play_file(&file_path)?;
        self.current_song = Some(file_path);
        self.is_playing = true;
        Ok(())
    }

    pub fn next_song(&mut self) -> Result<(), Box<dyn Error>> {
        // First get the next song's file path
        let next_file_path = if let Some(playlist) = &mut self.current_playlist {
            if let Some(next_song) = playlist.next_song() {
                next_song.song_file_path.clone()
            } else {
                return Ok(());  // No next song
            }
        } else {
            return Ok(());  // No playlist
        };

        // Now use the file path
        self.load_and_play_file(&next_file_path)?;
        self.current_song = Some(next_file_path);
        self.is_playing = true;
        Ok(())
    }

    pub fn previous_song(&mut self) -> Result<(), Box<dyn Error>> {
        // First get the previous song's file path
        let prev_file_path = if let Some(playlist) = &mut self.current_playlist {
            if let Some(prev_song) = playlist.previous_song() {
                prev_song.song_file_path.clone()
            } else {
                return Ok(());  // No previous song
            }
        } else {
            return Ok(());  // No playlist
        };

        // Now use the file path
        self.load_and_play_file(&prev_file_path)?;
        self.current_song = Some(prev_file_path);
        self.is_playing = true;
        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.is_playing = false;
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn load_and_play_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        // Open the file and create a buffer reader
        let file = BufReader::new(File::open(file_path)?);

        // Decode the audio file
        let source = Decoder::new(file)?;

        // Create a new sink for playback
        let sink = Sink::try_new(&self.stream_handle)?;

        // Load the source into the sink
        sink.append(source);

        // Store the sink and start playing
        self.sink = Some(sink);
        self.play();

        println!("now playing - {}", file_path);

        Ok(())
    }

    pub fn scan_directory(&mut self, directory: &str) -> Result<(), Box<dyn Error>> {
        // Create a new playlist for all songs
        let mut playlist = Playlist::new("All Songs".to_string());

        // Walk through the directory recursively
        self.scan_directory_recursive(Path::new(directory), &mut playlist)?;

        // Store the playlist
        self.current_playlist = Some(playlist);
        Ok(())
    }

    fn scan_directory_recursive(&self, path: &Path, playlist: &mut Playlist) -> Result<(), Box<dyn Error>> {
        // If the path is a directory, scan its contents
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                self.scan_directory_recursive(&entry.path(), playlist)?;
            }
        } else if let Some(extension) = path.extension() {
            // Check if the file is an audio file
            if self.is_audio_file(extension) {
                if let Some(song) = self.create_song_from_path(path)? {
                    playlist.add_song(song);
                }
            }
        }
        Ok(())
    }

    fn is_audio_file(&self, extension: &std::ffi::OsStr) -> bool {
        // Convert extension to lowercase string for comparison
        let ext = extension.to_string_lossy().to_lowercase();
        // List of supported audio formats
        matches!(ext.as_str(), "mp3" | "wav" | "flac" | "ogg")
    }

    fn create_song_from_path(&self, path: &Path) -> Result<Option<Song>, Box<dyn Error>> {
        // Get the song's directory - this is where we'll look for cover art
        let parent_dir = path.parent()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Parent directory not found"))?;

        // Get the song name (without extension)
        let song_name = path.file_stem()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        // Convert song path to string
        let song_file_path = path.to_string_lossy().into_owned();
        println!("found song! - {}", song_file_path);

        // Look for cover art in the directory
        let cover_img_path = Self::find_cover_art(parent_dir)?
            .unwrap_or_else(|| "../assets/default_cover.png".to_string());

        // I cant figure out how to get the artist so this will have to do for now
        Ok(Some(Song::new(
                    &song_name,
                    "0:00",  // Placeholder duration cus i dont know how to do this
                    &song_file_path,
                    &cover_img_path,
        )))
    }

    // Helper function to find cover art
    fn find_cover_art(directory: &Path) -> Result<Option<String>, Box<dyn Error>> {
        // Common names for cover art files
        let cover_filenames = [
            "cover", "album", "folder", "artwork", "front", "albumart",
            "Cover", "Album", "Folder", "Artwork", "Front", "AlbumArt"
        ];

        // Common image extensions
        let image_extensions = ["jpg", "jpeg", "png", "gif"];

        // First, look for exact matches of common cover art filenames
        for name in cover_filenames.iter() {
            for ext in image_extensions.iter() {
                let potential_path = directory.join(format!("{}.{}", name, ext));
                if potential_path.exists() {
                    return Ok(Some(potential_path.to_string_lossy().into_owned()));
                }
            }
        }

        // If no standard names found, look for any image file in the directory
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();

            // Check if this is a file (not a directory) and has an image extension
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if image_extensions.contains(&extension.to_string_lossy().to_lowercase().as_str()) {
                        return Ok(Some(path.to_string_lossy().into_owned()));
                    }
                }
            }
        }

        // If no cover art found, return None (will use default cover)
        Ok(None)
    }
}

