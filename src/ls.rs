use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;

use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Offset};

use colored::*;

/// Permissions trio struct
pub struct Permissions {read: bool, write: bool, execute: bool}

/// Render a file size
fn render_file_size(size: usize, flags: &Flags) -> Result<String, String>
{
    if flags.byte_sizes || !flags.binary_sizes
    {
        Ok(format!("{}  ", size))
    }
    else
    {
        // Units are K,M,G,T,P,E,Z,Y (powers of 1024) or KB,MB,... (powers of 1000).
        let result: String;
        let val: f64;
        let unit: &str;

        if size as u64 > 1024u64.pow(6)
        {
            val = size as f64 / 1024f64.powi(6);
            unit = "E";
        }
        else if size as u64 > 1024u64.pow(5)
        {
            val = size as f64 / 1024f64.powi(5);
            unit = "P";
        }
        else if size as u64 > 1024u64.pow(4)
        {
            val = size as f64 / 1024f64.powi(4);
            unit = "T";
        }
        else if size as u64 > 1024u64.pow(3)
        {
            val = size as f64 / 1024f64.powi(3);
            unit = "G";
        }
        else if size as u64 > 1024u64.pow(2)
        {
            val = size as f64 / 1024f64.powi(2);
            unit = "M";
        }
        else if size as u64 > 1024u64.pow(1)
        {
            val = size as f64 / 1024f64.powi(1);
            unit = "K";
        }
        else
        {
            val = size as f64;
            unit = "";
        }

        if unit == ""
        {
            result = format!("{}", val as usize);
        }
        else
        {
            result = format!("{}{}", ((val * 10.0) as usize) as f64 / 10.0, unit);
        }

        Ok(result)
    }
}

/// Render a datetime from utc
fn render_date(dt: i64) -> Result<String, String>
{
    let offset = chrono::Local.timestamp(0, 0).offset().fix().local_minus_utc() as i64 + 3600;
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let current_utc = since_the_epoch.as_secs() as i64;

    let current_date_time_obj = chrono::DateTime::<chrono::Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(current_utc + offset, 0), chrono::Utc).naive_local();
    let date_time_obj = chrono::DateTime::<chrono::Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(dt + offset, 0), chrono::Utc).naive_local();

    if format!("{}", current_date_time_obj.format("%Y")) != format!("{}", date_time_obj.format("%Y"))
    {
        Ok(format!("{}", date_time_obj.format("%e %b  %Y")))
    }
    else
    {
        Ok(format!("{}", date_time_obj.format("%e %b %R")))
    }
}

/// Render the extra information for a file or directory displayed by using the
/// '-l' or long argument
fn render_long(path: String, flags: &Flags) -> Result<StringData, String>
{
    let metadata = match std::fs::metadata(path.clone())
    {
        Ok(meta) => meta,
        Err(e) => {return Err(format!("{:?}", e));}
    };

    let file_size = metadata.len();
    let is_directory = metadata.is_dir();
    let perm_mode = (metadata.permissions().mode() & 0o777) as u16;
    let user_perms = Permissions{read: (perm_mode & 0b100000000) != 0, 
                                                write: (perm_mode & 0b010000000) != 0,
                                                execute: (perm_mode & 0b001000000) != 0};
    let group_perms = Permissions{read: (perm_mode & 0b000100000) != 0, 
                                                write: (perm_mode & 0b000010000) != 0,
                                                execute: (perm_mode & 0b000001000) != 0};
    let other_perms = Permissions{read: (perm_mode & 0b000000100) != 0, 
                                                write: (perm_mode & 0b000000010) != 0,
                                                execute: (perm_mode & 0b000000001) != 0};


    let mut string_data = StringData
    {
        colored_string: String::from(""),
        raw_string: String::from(""),
        length: 0
    };

    // Inode Section
    if flags.inode
    {
        // Add Spacing
        string_data.colored_string += &format!("{:9}", format!("{}", metadata.ino()).bright_purple());
        string_data.raw_string += "  ";
        string_data.length += 9;

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    // Permissions section
    if !flags.no_perms
    {
        // Directory bit of the permissions
        if is_directory
        {
            string_data.colored_string += &format!("{}", "d".bright_blue());
            string_data.raw_string += "d";
            string_data.length += 1;
        }
        else
        {
            string_data.colored_string += ".";
            string_data.raw_string += ".";
            string_data.length += 1;
        }

        // Loop over all of the permissions, and add those
        for permission in &vec![user_perms, group_perms, other_perms]
        {
            // Read Permissions
            if permission.read
            {
                string_data.colored_string += &format!("{}", "r".bright_yellow());
                string_data.raw_string += "r";
                string_data.length += 1;
            }
            else
            {
                string_data.colored_string += "-";
                string_data.raw_string += "-";
                string_data.length += 1;
            }

            // Write Permissions
            if permission.write
            {
                string_data.colored_string += &format!("{}", "w".bright_red());
                string_data.raw_string += "w";
                string_data.length += 1;
            }
            else
            {
                string_data.colored_string += "-";
                string_data.raw_string += "-";
                string_data.length += 1;
            }

            // Execute Permissions
            if permission.execute
            {
                string_data.colored_string += &format!("{}", "x".bright_green());
                string_data.raw_string += "x";
                string_data.length += 1;
            }
            else
            {
                string_data.colored_string += "-";
                string_data.raw_string += "-";
                string_data.length += 1;
            }
        }

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    // Octal Permissions
    if flags.octal_perms
    {
        let colored_octal = format!("[{}]", format!("{:03o}", perm_mode).bright_blue());
        // Add Octal Permissions
        string_data.colored_string += &colored_octal;
        string_data.raw_string += &format!("[{:03o}]", perm_mode);
        string_data.length += 5;

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    // Link Section
    if flags.show_links
    {
        // Links
        string_data.colored_string += &format!("{:4}", format!("{}", metadata.nlink()).bright_red());
        string_data.raw_string += "    ";
        string_data.length += 4;

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    // File size
    if !flags.no_size
    {
        let mut size_str = format!("{:6}", render_file_size(file_size as usize, flags)?);

        let colored = if std::path::Path::new(&(path.clone())).is_file()
        {
            format!("{}", size_str.bright_green())
        }
        else
        {
            size_str = String::from("-     ");
            size_str.clone()
        };

        // Add Size
        string_data.colored_string += &colored;
        string_data.raw_string += &size_str.clone();
        string_data.length += size_str.len();

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    // Block Section
    if flags.blocks
    {
        if metadata.is_file()
        {
            // Blocks
            string_data.colored_string += &format!("{:6}", format!("{}", metadata.blocks()).bright_blue());
            string_data.raw_string += "  ";
            string_data.length += 6;
        }
        else
        {
            // Blocks
            string_data.colored_string += "-     ";
            string_data.raw_string += "-     ";
            string_data.length += 6;
        }

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    // User Section
    if !flags.no_user
    {
        // User
        let user = metadata.uid();

        let user_str = match users::get_user_by_uid(user)
        {
            Some(user) => format!("{:8}", user.name().to_str().unwrap()),
            None => String::from("unknown ")
        };

        string_data.colored_string += &format!("{}", user_str.bright_yellow());
        string_data.raw_string += &user_str.clone();
        string_data.length += 8;

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    // Timestamp Section
    if !flags.no_time
    {
        let modified_time: i64 = metadata.mtime();
        let date_str = render_date(modified_time)?;

        // Timestamp
        string_data.colored_string += &format!("{:13}", format!("{}", date_str).bright_blue());
        string_data.raw_string += "  ";
        string_data.length += 13;

        // Add Spacing
        string_data.colored_string += "  ";
        string_data.raw_string += "  ";
        string_data.length += 2;
    }

    Ok(string_data)
}

/// Colored and non colored strings
#[derive(Debug)]
pub struct StringData
{
    /// Colored string data
    colored_string: String,
    /// Raw string data
    raw_string: String,
    /// String length
    length: usize
}

/// Stores the flags and options set by command line arguments to custom_ls
#[derive(Debug, Clone)]
pub struct Flags
{
    /// Vector of files passed to custom_ls
    files: Vec<String>,
    /// Go through subdirectories recursively
    recursive: bool,
    /// Display files and directories beginning with '.'
    all: bool,
    /// Display all information about a given file
    long: bool,
    /// Display one file per line
    one_per_line: bool,
    /// Show only directories
    only_dirs: bool,
    /// Binary file sizes
    binary_sizes: bool,
    /// Byte file sizes
    byte_sizes: bool,
    /// Shows headers to columns in long display
    headers: bool,
    /// Display number of links
    show_links: bool,
    /// Show inode
    inode: bool,
    /// Show number of blocks
    blocks: bool,
    /// Don't show the permissions
    no_perms: bool,
    /// Don't show the filesize
    no_size: bool,
    /// Don't show the user
    no_user: bool,
    /// Don't show the time stamp
    no_time: bool,
    /// Show octal permission data
    octal_perms: bool
}

/// The mode to run custom_ls in
#[derive(Debug, Clone, Copy)]
pub enum Mode
{
    /// Default mode, lists the files in the specified directory
    List,
    /// Displays the help for custom_ls
    Help,
    /// Displays the version for custom_ls
    Version
}


/// Information to be stored about each file
#[derive(Debug)]
pub struct File
{
    /// File Name
    name: String,
    /// File Path as String
    path_str: String
}

impl File
{
    /// Renderes file into
    pub fn render(&self, flags: &Flags) -> Result<StringData, String>
    {   
        let mut string_data = StringData
        {
            colored_string: self.name.clone(),
            raw_string: self.name.clone(),
            length: self.name.len()
        };

        if self.name.ends_with(".md") || self.name.ends_with(".toml") || self.name == ".gitignore" || self.name == "makefile"
            || self.name == "Makefile"
            {
                string_data.colored_string = format!("{}", string_data.colored_string.bright_yellow().underline());
            }

            if self.name.ends_with(".png") || self.name.ends_with(".bmp") || self.name.ends_with(".jpg")
                || self.name.ends_with(".jpeg") || self.name.ends_with(".svg")
            {
                string_data.colored_string = format!("{}", string_data.colored_string.bright_purple());
            }

            if std::fs::metadata(&self.path_str).unwrap().permissions().mode() & 0o111 > 0
            {
                string_data.colored_string = format!("{}", string_data.colored_string.bright_green());
                string_data.colored_string += "*";
                string_data.raw_string += "*";
                string_data.length += 1;
            }

        if flags.long
        {
            let long_data = render_long(self.path_str.clone(), flags)?;

            string_data.colored_string = long_data.colored_string + &string_data.colored_string;
            string_data.raw_string = long_data.raw_string + &string_data.raw_string;
            string_data.length = long_data.length + string_data.length;
        }

        Ok(string_data)
    }
}

/// Information to be stored about each directory
#[derive(Debug)]
pub struct Directory
{
    /// Directory Name
    name: String,
    /// Directory Path as String
    path_str: String
}

impl Directory
{
    /// Renderes directory into
    pub fn render(&self, flags: &Flags) -> Result<StringData, String>
    {
        let mut colored_string = format!("{}", self.name.bright_blue().bold());
        let mut raw_string = self.name.clone();

        colored_string += "/";
        raw_string += "/";

        let mut string_data = StringData
        {
            colored_string: colored_string,
            raw_string: raw_string.clone(),
            length: raw_string.len()
        };

        if flags.long
        {
            let long_data = render_long(self.path_str.clone(), flags)?;

            string_data.colored_string = long_data.colored_string + &string_data.colored_string;
            string_data.raw_string = long_data.raw_string + &string_data.raw_string;
            string_data.length = long_data.length + string_data.length;
        }

        Ok(string_data)
    }
}

/// Stores the discovered files and folders to be displayed
#[derive(Debug)]
pub struct Display
{
    /// List of files to display
    files: Vec<File>,
    /// List of directories to display
    directories: Vec<Directory>
}

impl Display
{
    /// Display all file into
    pub fn display(&self, flags: &Flags) -> Result<(), String>
    {   
        let max_line_length = 80usize;

        let mut longest_file_name = 0usize;
        let mut rendered_names: Vec<StringData> = vec![];

        for file in &self.files
        {
            if file.name.starts_with(".") && !flags.all
            {
                continue;
            }

            let rendered = file.render(flags)?;

            if rendered.length > longest_file_name
            {
                longest_file_name = rendered.length;
            }

            rendered_names.push(rendered);
        }

        for directory in &self.directories
        {
            if directory.name.starts_with(".") && !flags.all
            {
                continue;
            }

            let rendered = directory.render(flags)?;

            if rendered.length > longest_file_name
            {
                longest_file_name = rendered.length;
            }

            rendered_names.push(rendered);
        }

        // Header
        if flags.long && flags.headers
        {
            let mut header = String::from("");

            if flags.inode
            {
                header += &format!("{}      ", String::from("inode").white().underline());
            }

            if !flags.no_perms
            {
                header += &format!("{} ", String::from("Permissions").white().underline());
            }

            if flags.octal_perms
            {
                header += &format!("{}  ", String::from("Octal").white().underline());
            }

            if flags.show_links
            {
                header += &format!("{}  ", String::from("Link").white().underline());
            }

            if !flags.no_size
            {
                header += &format!("{}    ", String::from("Size").white().underline());
            }

            if flags.blocks
            {
                header += &format!("{}  ", String::from("Blocks").white().underline());
            }

            if !flags.no_user
            {
                header += &format!("{}      ", String::from("User").white().underline());
            }

            if !flags.no_time
            {
                header += &format!("{}       ", String::from("Modified").white().underline());
            }

            header += &format!("{}", String::from("Name").white().underline());
            
            println!("{}", header);
        }

        let num_per_line = max_line_length / longest_file_name;
        let mut current_line = 0usize;

        for rendered in &rendered_names
        {
            let mut padding = String::from("");

            for _ in 0..(longest_file_name - rendered.length + 2)
            {
                padding += " ";
            }

            print!("{}{}", rendered.colored_string, padding);

            if flags.one_per_line
            {
                println!("");
            }
            else
            {
                current_line += 1;

                if current_line >= num_per_line
                {
                    println!("");
                    current_line = 0;
                }
            } 
        }

        if !flags.one_per_line
        {
            println!("");
        }
        
        Ok(())
    }
}

/// Stores the information about running custom_ls, such as the command line
/// arguments, along with the various functions which will be called
#[derive(Debug)]
pub struct Utility
{
    /// Raw Arguments
    raw_args: Vec<String>,
    /// Flags generated from the command line arguments
    flags: Flags,
    /// Mode for custom_ls to be run in
    mode: Mode,
    /// Display object
    display: Display
}

impl Utility
{
    /// Generates a new Utility object from the command line arguments
    pub fn new(arguments: Vec<String>) -> Utility
    {
        let mut new_args: Vec<String> = vec![];

        for arg in &arguments
        {
            if !arg.starts_with("--") && arg.starts_with("-") && arg.len() > 2
            {
                for c in arg.chars()
                {
                    if c != '-'
                    {
                        let mut s = String::from("-");
                        s += &c.to_string();
                        new_args.push(s);
                    }
                }
            }
            else
            {
                new_args.push((&arg).to_string());
            }
        }

        let mut flags = Flags
        {
            files: vec![],
            recursive: new_args.contains(&String::from("-R")) || new_args.contains(&String::from("--recursive")),
            all: new_args.contains(&String::from("-a")) || new_args.contains(&String::from("--all")),
            long: new_args.contains(&String::from("-l")) || new_args.contains(&String::from("--long")),
            one_per_line: new_args.contains(&String::from("-l")) || new_args.contains(&String::from("-1")),
            only_dirs: new_args.contains(&String::from("-D")) || new_args.contains(&String::from("--only-dirs")),
            binary_sizes: new_args.contains(&String::from("-b")) || new_args.contains(&String::from("--binary")),
            byte_sizes: new_args.contains(&String::from("-B")) || new_args.contains(&String::from("--bytes")),
            headers: new_args.contains(&String::from("-h")) || new_args.contains(&String::from("--header")),
            show_links: new_args.contains(&String::from("-H")) || new_args.contains(&String::from("--links")),
            inode: new_args.contains(&String::from("-i")) || new_args.contains(&String::from("--inode")),
            blocks: new_args.contains(&String::from("-S")) || new_args.contains(&String::from("--blocks")),
            no_perms: new_args.contains(&String::from("--no-permissions")),
            no_size: new_args.contains(&String::from("--no-filesize")),
            no_user: new_args.contains(&String::from("--no-user")),
            no_time: new_args.contains(&String::from("--no-time")),
            octal_perms: new_args.contains(&String::from("-O")) || new_args.contains(&String::from("--octal"))
        };

        let mut in_files = false;
        for arg in &new_args[1..]
        {
            if !arg.starts_with("-")
            {
                in_files = true;
            }
            
            if in_files
            {
                flags.files.push(arg.to_string());
            }
        }

        if flags.files.len() == 0
        {
            flags.files = vec![String::from("./")];
        }

        let mode: Mode =
            if new_args.contains(&String::from("--help"))
            {
                Mode::Help
            }
            else if new_args.contains(&String::from("--version"))
            {
                Mode::Version
            }
            else
            {
                Mode::List
            };

        Utility
        {
            raw_args: new_args,
            flags,
            mode,
            display: Display
            {
                files: vec![],
                directories: vec![]
            }
        }
    }

    /// Execute the utility
    pub fn execute(&mut self) -> Result<(), String>
    {
        match self.mode
        {
            Mode::Help => 
            {
                self._help()
            },
            Mode::List =>
            {
                self._list()
            },
            Mode::Version =>
            {
                self._version()
            }
        }
    }

    /// Lists the files specified
    pub fn _list(&mut self) -> Result<(), String>
    {
        let mut files_to_handle: Vec<String> = vec![];
        let mut dirs_to_handle: Vec<String> = vec![];

        for file_path in &self.flags.files
        {
            let path = std::path::Path::new(&file_path);

            if !path.exists()
            {
                return Err(format!("Path '{}' does not exist", file_path));
            }

            if path.is_dir()
            {
                dirs_to_handle.push(file_path.clone());
            }
            else if path.is_file()
            {
                files_to_handle.push(file_path.clone());
            }
            else
            {
                return Err(String::from("The file path given is not a directory and not a file, what the hell?!"));
            }
        }

        for file in files_to_handle
        {
            match self._handle_file(&std::path::Path::new(&file))
                {
                    Ok(_) => {},
                    Err(e) => {return Err(e);}
                }
        }

        for dir in dirs_to_handle
        {
            match self._handle_dir(&std::path::Path::new(&dir))
            {
                Ok(_) => {},
                Err(e) => {return Err(e);}
            }
        }

        self.display.display(&self.flags)?;

        Ok(())
    }

    /// Handle Directory
    pub fn _handle_dir(&mut self, path: &std::path::Path) -> Result<(), String>
    {
        self._display_dir(&path)?;
        let items = std::fs::read_dir(path).unwrap();

        for path_entity in items
        {
            let path = path_entity.unwrap().path();
            if path.is_dir()
            {
                if self.flags.recursive
                {
                    self._handle_dir(&path)?;
                }
                else
                {
                    self._display_dir(&path)?;
                }
            }
            else if path.is_file()
            {
                self._handle_file(&path)?;
            }
            else
            {
                return Err(String::from("The file path given is not a directory and not a file, what the hell?!"));
            }
        }

        Ok(())
    }

    // Handle File
    pub fn _handle_file(&mut self, path: &std::path::Path) -> Result<(), String>
    {
        if !self.flags.only_dirs
        {
            self.display.files.push(File 
                {
                    name: String::from(path.file_name().unwrap().to_str().unwrap()),
                    path_str: String::from(path.as_os_str().to_str().unwrap())
                });
        }
        Ok(())
    }

    /// Display Directory dat
    fn _display_dir(&mut self, path: &std::path::Path) -> Result<(), String>
    {
        let mut path_str = String::from(path.to_str().unwrap());

        if path_str.ends_with("/")
        {
		    path_str.pop();            
        }

        let mut final_str = path_str.clone();

        for val in path_str.split("/")
        {
            final_str = String::from(val);
        }

        self.display.directories.push(
            Directory
            {
                name: final_str,
                path_str: String::from(path.as_os_str().to_str().unwrap())
            });

        Ok(())
    }

    /// Displays the help for custom_ls
    fn _help(&self) -> Result<(), String>
    {
        println!("Usage: custom_ls [OPTION]... [FILE]...");
        println!("Displays information about the FILEs (Will default to the current directory).");
        println!("");
        println!("  {:4}{:27}{}", "-a,", "--all", "Includes files and directories starting with '.'");
        println!("  {:4}{:27}{}", "-b,", "--binary", "Show file sizes with binary prefixes");
        println!("  {:4}{:27}{}", "-B,", "--bytes", "Show files sizes always in bytes");
        println!("  {:4}{:27}{}", "-D,", "--only-dirs", "List only directories");
        println!("  {:4}{:27}{}", "-h,", "--headers", "Displays headers on long view");
        println!("  {:4}{:27}{}", "", "--help", "Displays the help page");
        println!("  {:4}{:27}{}", "-H,", "--links", "Display number of hard links");
        println!("  {:4}{:27}{}", "-i,", "--inode", "Display inode");
        println!("  {:4}{:27}{}", "-l,", "--long", "Displays more information about the files");
        println!("  {:4}{:27}{}", "", "--no-filesize", "Don't show filesize");
        println!("  {:4}{:27}{}", "", "--no-permissions", "Don't show permissions");
        println!("  {:4}{:27}{}", "", "--no-time", "Don't show timestamp");
        println!("  {:4}{:27}{}", "", "--no-user", "Don't show user");
        println!("  {:4}{:27}{}", "-O,", "--octal", "Display octal permissions");
        println!("  {:4}{:27}{}", "-R,", "--recursive", "Go through subdirectories recursively");
        println!("  {:4}{:27}{}", "-S,", "--blocks", "Show number of blocks");
        println!("  {:4}{:27}{}", "", "--version", "Displays the version page");
        println!("  {:4}{:27}{}", "-1", "", "Display one file per line");

        Ok(())
    }

    /// Displays the version and program info
    fn _version(&self) -> Result<(), String>
    {
        println!("Custom ls v.{}", env!("CARGO_PKG_VERSION"));
        Ok(())
    }
}
