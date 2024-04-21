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
A fast file system search tool

Usage: where-finder [OPTIONS] --expression <EXPRESSION>

Options:
  -t, --type <TYPE>
          Type of search to perform. One of "dir", "file", "content"
          
          [default: dir]

  -e, --expression <EXPRESSION>
          Regular expression to search for

  -m, --max-depth <MAX_DEPTH>
          Max depth to search
          
          [default: 10]

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
```

### Performance advice

File and dir search works great, but file content search might be really slow, there are a lot of variables from file lenght to the complexity of the regex expression. Again this is a personal project, if you would like to contribute feel free to open a PR. 

# Gotchas
- try to implement LRU cache to save results instead of vec.
- try to implement rayon into_par_iter for better performance at least reading files.
- format and fix spagetti code