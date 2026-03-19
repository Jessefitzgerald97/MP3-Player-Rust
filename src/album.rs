pub struct Song {
    pub song_name: String,
    pub song_length: String,
    pub song_file_path: String,
    pub cover_img_path: String,
}

impl Song {
    pub fn new(name: &str, length: &str, file: &str, img: &str) -> Song {
        Song {
            song_name: name.to_string(),
            song_length: length.to_string(),
            song_file_path: file.to_string(),
            cover_img_path: img.to_string(),
        }
    }
}

pub struct Playlist {
    pub name: String,
    songs: Vec<Song>,
    current_index: Option<usize>, // Track which song is playing
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            songs: Vec::new(),
            current_index: None,
        }
    }

    pub fn get_songs(&self) -> &Vec<Song> {
        &self.songs
    }

    pub fn get_current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn play_song_at_index(&mut self, index: usize) {
        if index < self.songs.len() {
            self.current_index = Some(index);
        }
    }

    pub fn add_song(&mut self, song: Song) {
        self.songs.push(song);
    }

    pub fn get_current_song(&self) -> Option<&Song> {
        self.current_index.map(|index| &self.songs[index])
    }

    pub fn next_song(&mut self) -> Option<&Song> {
        if self.songs.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                // Start playing from beginning
                self.current_index = Some(0);
            }
            Some(index) => {
                // Move to next song if available
                if index + 1 < self.songs.len() {
                    self.current_index = Some(index + 1);
                }
            }
        }

        self.get_current_song()
    }

    pub fn previous_song(&mut self) -> Option<&Song> {
        if let Some(index) = self.current_index {
            if index > 0 {
                self.current_index = Some(index - 1);
            }
        }
        self.get_current_song()
    }
}

//fn main() {
//    let all_songs: Vec<Song> = Playlist::new();
//    let all_playlists: Vec<Playlist> = Vec::new();
//}
//
//fn add_new_song(name: &str, album: &str, length: &str) {
//    let song = Song::new(name, album, length);
//    all_songs.push(song);
//}
//
//fn create_playlist() {
//    let playlist = Playlist::new();
//    all_playlists.push(playlist);
//}
//
//fn add_to_playlist(song: Song, playlist: Playlist) {
//    playlist.push(song);
//}
