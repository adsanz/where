# Where 

A simple CLI tool to find files, folders and contents on files using regex and walkdir.

You can:
- Find that config file which you don't remember the path, by just using any of the contents as the search query
- Find all those target folders that are eating away your disk

You get the idea... Is a personal project for something that I find myself doing a lot, finding info on files, finding files or directories and recurse using regex as the expression. This can be done with find, but it's far easier with where. 

## Requirements

This tool requires rust installed. You can find guides here: https://www.rust-lang.org/tools/install

The tools is tested on linux machines (Debian) but it should work on other linux systems, mac and could work on windows but didn't test it.

## Building

Clone this repo, and proceed with these commands

```
cargo build --release
cd target/release
sudo cp where /usr/local/bin/where-finder # you can use "where" but may mess up with already present binaries
where-finder --help
A fast file system search tool. Remember to set WHERE_TO_FIND env variable to the directories you want to search in.

Usage: where [OPTIONS] --expression <EXPRESSION>

Options:
  -t, --type <TYPE>
          Type of search to perform. One of "dir", "file", "content"
          
          [default: dir]

  -e, --expression <EXPRESSION>
          Regular expression to search for

  -m, --max-depth <MAX_DEPTH>
          Max depth to search
          
          [default: 10]

  -v, --verbose
          Verbose mode - allows first line of content matched by regex to be displayed. May decrease performance Only works with content search

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## Usage

The juicy part about this tool is the regex usage. For files and directories is used on the file name level, for content is used by reading the file contents. Be aware, the tool will omit binary files, it looks at the first 1024 bytes and checks for null bytes. If you find a better way please feel free to open a Pull request!

Alright, so in order to decide which folders to use, the tool uses an environment variable, named "WHERE_TO_FIND" which is defined as a ":" separated string of paths (like PATH variable in linux) you can use absolute or relative paths, the tool takes care of converting it to absolute paths. *If the variable is not defined, the tool will use the current path of execution*

A full example would be something like: WHERE_TO_FIND="~/test" where-finder --type content -e '10.11.1.1'

This will iterate over all folders of "~/test" (by default 10 levels of depth) and inspect the contents of each file looking up for the ip '10.11.1.1'

### More examples

```bash
# find ip in files inside folder ~/test with default 10 levels of depth
WHERE_TO_FIND="~/test" where-finder --type content -e '10.11.1.1'
# find all "target" directories in home dir
WHERE_TO_FIND="~/" where-finder --type dir -e 'target'
# find all python files in test and test2 folder 
WHERE_TO_FIND="~/test:~/test2" where-finder --type file -e '.py'
# find in current path all files that contain IPs with depth 2
where-finder --type content -e '\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b' -m 2
# on content searches you can also use "-v" or verbose so it will give you back the first line matched. Be aware that might affecte performance.
time WHERE_TO_FIND="~/test" where-finder --type content -e 'super-secret-key' -v 
```

### Performance 

Rayon is used on file and content searches. Directories do not have any performance improvement. 