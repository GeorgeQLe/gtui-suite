//! Bundled cheat sheets.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CheatSheet {
    pub topic: String,
    pub category: String,
    pub source: Source,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum Source {
    Bundled,
    User { path: PathBuf },
}

impl Source {
    pub fn label(&self) -> &str {
        match self {
            Source::Bundled => "bundled",
            Source::User { .. } => "user",
        }
    }
}

pub fn bundled_cheatsheets() -> Vec<CheatSheet> {
    vec![
        CheatSheet {
            topic: "git".into(),
            category: "Version Control".into(),
            source: Source::Bundled,
            content: r#"# Git Cheat Sheet

## Configuration
```bash
git config --global user.name "Name"
git config --global user.email "email@example.com"
git config --list
```

## Basic Commands
```bash
git init                    # Initialize repository
git clone <url>             # Clone repository
git status                  # Check status
git add <file>              # Stage file
git add .                   # Stage all changes
git commit -m "message"     # Commit with message
git push                    # Push to remote
git pull                    # Pull from remote
```

## Branching
```bash
git branch                  # List branches
git branch <name>           # Create branch
git checkout <branch>       # Switch branch
git checkout -b <branch>    # Create and switch
git merge <branch>          # Merge branch
git branch -d <branch>      # Delete branch
```

## History
```bash
git log                     # View history
git log --oneline           # Compact history
git log --graph             # Graph view
git diff                    # Show changes
git show <commit>           # Show commit details
```

## Undo Changes
```bash
git checkout -- <file>      # Discard changes
git reset HEAD <file>       # Unstage file
git reset --soft HEAD~1     # Undo last commit (keep changes)
git reset --hard HEAD~1     # Undo last commit (discard)
git revert <commit>         # Revert commit
```

## Stashing
```bash
git stash                   # Stash changes
git stash list              # List stashes
git stash pop               # Apply and remove stash
git stash apply             # Apply stash
git stash drop              # Remove stash
```
"#.into(),
        },
        CheatSheet {
            topic: "docker".into(),
            category: "Containers".into(),
            source: Source::Bundled,
            content: r#"# Docker Cheat Sheet

## Images
```bash
docker images               # List images
docker pull <image>         # Pull image
docker build -t <name> .    # Build image
docker rmi <image>          # Remove image
docker image prune          # Remove unused images
```

## Containers
```bash
docker ps                   # List running containers
docker ps -a                # List all containers
docker run <image>          # Run container
docker run -d <image>       # Run detached
docker run -it <image> sh   # Interactive shell
docker stop <container>     # Stop container
docker start <container>    # Start container
docker rm <container>       # Remove container
docker logs <container>     # View logs
docker exec -it <c> sh      # Execute in container
```

## Docker Compose
```bash
docker compose up           # Start services
docker compose up -d        # Start detached
docker compose down         # Stop services
docker compose ps           # List services
docker compose logs         # View logs
docker compose build        # Build services
```

## Volumes
```bash
docker volume ls            # List volumes
docker volume create <name> # Create volume
docker volume rm <name>     # Remove volume
docker volume prune         # Remove unused
```

## Networks
```bash
docker network ls           # List networks
docker network create <n>   # Create network
docker network rm <name>    # Remove network
docker network inspect <n>  # Inspect network
```
"#.into(),
        },
        CheatSheet {
            topic: "vim".into(),
            category: "Editors".into(),
            source: Source::Bundled,
            content: r#"# Vim Cheat Sheet

## Modes
```
i       Insert mode (before cursor)
a       Insert mode (after cursor)
o       Insert line below
O       Insert line above
Esc     Normal mode
v       Visual mode
V       Visual line mode
Ctrl+v  Visual block mode
:       Command mode
```

## Navigation
```
h j k l     Left, Down, Up, Right
w / b       Next/previous word
e           End of word
0 / $       Start/end of line
gg / G      Start/end of file
Ctrl+d/u    Page down/up
```

## Editing
```
x           Delete character
dd          Delete line
yy          Yank (copy) line
p           Paste after
P           Paste before
u           Undo
Ctrl+r      Redo
.           Repeat last command
```

## Search & Replace
```
/pattern    Search forward
?pattern    Search backward
n / N       Next/previous match
:%s/old/new/g   Replace all
:s/old/new/g    Replace in line
```

## Files
```
:w          Save
:q          Quit
:wq         Save and quit
:q!         Quit without saving
:e file     Open file
:bn / :bp   Next/previous buffer
```

## Windows & Tabs
```
:split      Horizontal split
:vsplit     Vertical split
Ctrl+w h/j/k/l  Navigate windows
:tabnew     New tab
gt / gT     Next/previous tab
```
"#.into(),
        },
        CheatSheet {
            topic: "tmux".into(),
            category: "Multiplexers".into(),
            source: Source::Bundled,
            content: r#"# Tmux Cheat Sheet

## Sessions
```bash
tmux                        # Start new session
tmux new -s name            # Named session
tmux ls                     # List sessions
tmux attach -t name         # Attach to session
tmux kill-session -t name   # Kill session
```

## Prefix Key: Ctrl+b

## Windows (Ctrl+b, then:)
```
c           Create window
n / p       Next/previous window
0-9         Go to window N
,           Rename window
&           Kill window
w           List windows
```

## Panes (Ctrl+b, then:)
```
%           Split horizontal
"           Split vertical
o           Next pane
;           Last pane
x           Kill pane
z           Toggle zoom
{  }        Swap pane
Arrow       Navigate panes
Ctrl+Arrow  Resize pane
```

## Copy Mode (Ctrl+b, then:)
```
[           Enter copy mode
Space       Start selection
Enter       Copy selection
]           Paste
```

## Misc
```
d           Detach
t           Show clock
?           List keybindings
:           Command prompt
```
"#.into(),
        },
        CheatSheet {
            topic: "bash".into(),
            category: "Shell".into(),
            source: Source::Bundled,
            content: r#"# Bash Cheat Sheet

## Navigation
```bash
cd <dir>            # Change directory
cd ..               # Parent directory
cd ~                # Home directory
cd -                # Previous directory
pwd                 # Print working directory
ls                  # List files
ls -la              # List all with details
```

## File Operations
```bash
cp <src> <dst>      # Copy file
cp -r <src> <dst>   # Copy directory
mv <src> <dst>      # Move/rename
rm <file>           # Remove file
rm -rf <dir>        # Remove directory
mkdir <dir>         # Create directory
mkdir -p <path>     # Create nested dirs
touch <file>        # Create empty file
```

## Text Processing
```bash
cat <file>          # Display file
head -n 10 <file>   # First 10 lines
tail -n 10 <file>   # Last 10 lines
tail -f <file>      # Follow file
grep <pattern> <f>  # Search in file
wc -l <file>        # Count lines
sort <file>         # Sort lines
uniq                # Remove duplicates
```

## Redirection
```bash
cmd > file          # Redirect stdout
cmd >> file         # Append stdout
cmd 2> file         # Redirect stderr
cmd &> file         # Redirect all
cmd < file          # Input from file
cmd1 | cmd2         # Pipe
```

## Variables
```bash
VAR=value           # Set variable
echo $VAR           # Use variable
export VAR          # Export to env
$?                  # Last exit code
$$                  # Current PID
$0                  # Script name
$1, $2, ...         # Arguments
$@                  # All arguments
```

## Control Flow
```bash
if [ condition ]; then
    commands
elif [ condition ]; then
    commands
else
    commands
fi

for i in 1 2 3; do
    echo $i
done

while [ condition ]; do
    commands
done
```
"#.into(),
        },
        CheatSheet {
            topic: "rust".into(),
            category: "Languages".into(),
            source: Source::Bundled,
            content: r#"# Rust Cheat Sheet

## Cargo Commands
```bash
cargo new <name>        # New project
cargo build             # Build project
cargo run               # Build and run
cargo test              # Run tests
cargo check             # Check without building
cargo fmt               # Format code
cargo clippy            # Lint code
cargo doc --open        # Generate docs
cargo add <crate>       # Add dependency
```

## Variables
```rust
let x = 5;              // Immutable
let mut y = 5;          // Mutable
const MAX: u32 = 100;   // Constant
static NAME: &str = ""; // Static
```

## Types
```rust
i8, i16, i32, i64, i128, isize  // Signed
u8, u16, u32, u64, u128, usize  // Unsigned
f32, f64                         // Float
bool                             // Boolean
char                             // Character
&str, String                     // Strings
```

## Collections
```rust
let arr = [1, 2, 3];           // Array
let vec = vec![1, 2, 3];       // Vector
let map = HashMap::new();       // HashMap
let set = HashSet::new();       // HashSet
```

## Option & Result
```rust
Some(value) / None              // Option
Ok(value) / Err(error)          // Result

// Pattern matching
match option {
    Some(v) => println!("{}", v),
    None => println!("None"),
}

// If let
if let Some(v) = option {
    println!("{}", v);
}

// Unwrap (panics on None/Err)
option.unwrap()
result.unwrap()

// Safe alternatives
option.unwrap_or(default)
result.unwrap_or_else(|e| handle(e))
option?  // Propagate None
result?  // Propagate Err
```

## Structs & Enums
```rust
struct Point { x: i32, y: i32 }

enum Status {
    Active,
    Inactive(String),
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
```

## Traits
```rust
trait Speak {
    fn speak(&self) -> String;
}

impl Speak for Dog {
    fn speak(&self) -> String {
        "Woof!".to_string()
    }
}
```
"#.into(),
        },
        CheatSheet {
            topic: "curl".into(),
            category: "Tools".into(),
            source: Source::Bundled,
            content: r#"# curl Cheat Sheet

## Basic Requests
```bash
curl <url>                      # GET request
curl -o file <url>              # Save to file
curl -O <url>                   # Save with remote name
curl -L <url>                   # Follow redirects
curl -I <url>                   # Headers only
curl -v <url>                   # Verbose output
```

## HTTP Methods
```bash
curl -X GET <url>
curl -X POST <url>
curl -X PUT <url>
curl -X DELETE <url>
curl -X PATCH <url>
```

## Headers & Data
```bash
curl -H "Content-Type: application/json" <url>
curl -H "Authorization: Bearer <token>" <url>
curl -d "key=value" <url>               # Form data
curl -d '{"key":"value"}' <url>         # JSON data
curl --data-binary @file <url>          # File content
```

## Authentication
```bash
curl -u user:pass <url>                 # Basic auth
curl -H "Authorization: Bearer <t>" <url>
curl --negotiate -u : <url>             # Kerberos
```

## SSL/TLS
```bash
curl -k <url>                   # Insecure (skip verify)
curl --cacert ca.crt <url>      # Custom CA
curl --cert client.crt <url>    # Client cert
```

## Cookies
```bash
curl -c cookies.txt <url>       # Save cookies
curl -b cookies.txt <url>       # Send cookies
curl -b "name=value" <url>      # Send cookie
```

## Upload
```bash
curl -F "file=@path" <url>      # Upload file
curl -T file <url>              # PUT file
```

## Output
```bash
curl -w "%{http_code}" <url>    # Print status code
curl -s <url>                   # Silent mode
curl -S <url>                   # Show errors
```
"#.into(),
        },
        CheatSheet {
            topic: "kubectl".into(),
            category: "Containers".into(),
            source: Source::Bundled,
            content: r#"# kubectl Cheat Sheet

## Context & Config
```bash
kubectl config get-contexts
kubectl config use-context <ctx>
kubectl config current-context
kubectl cluster-info
```

## Get Resources
```bash
kubectl get pods
kubectl get pods -A                 # All namespaces
kubectl get pods -o wide            # More info
kubectl get pods -o yaml            # YAML output
kubectl get svc,deploy,pods
kubectl get all
```

## Describe & Logs
```bash
kubectl describe pod <name>
kubectl logs <pod>
kubectl logs -f <pod>               # Follow
kubectl logs <pod> -c <container>   # Specific container
```

## Create & Apply
```bash
kubectl apply -f <file.yaml>
kubectl create -f <file.yaml>
kubectl create deployment <n> --image=<img>
kubectl expose deploy <n> --port=80
```

## Edit & Delete
```bash
kubectl edit <resource> <name>
kubectl delete <resource> <name>
kubectl delete -f <file.yaml>
```

## Exec & Port Forward
```bash
kubectl exec -it <pod> -- sh
kubectl port-forward <pod> 8080:80
kubectl cp <pod>:/path ./local
```

## Scale & Rollout
```bash
kubectl scale deploy <n> --replicas=3
kubectl rollout status deploy <n>
kubectl rollout history deploy <n>
kubectl rollout undo deploy <n>
```

## Namespaces
```bash
kubectl get ns
kubectl create ns <name>
kubectl -n <ns> get pods
```
"#.into(),
        },
        CheatSheet {
            topic: "jq".into(),
            category: "Tools".into(),
            source: Source::Bundled,
            content: r#"# jq Cheat Sheet

## Basic Usage
```bash
cat file.json | jq '.'          # Pretty print
jq '.' file.json                # From file
echo '{"a":1}' | jq '.a'        # Get field
```

## Selectors
```bash
jq '.key'                       # Object key
jq '.key.nested'                # Nested key
jq '.[0]'                       # Array index
jq '.[]'                        # All elements
jq '.key[]'                     # Array in object
```

## Filters
```bash
jq 'select(.age > 30)'          # Filter objects
jq 'map(.name)'                 # Transform array
jq 'map(select(.active))'       # Filter and map
jq '.[] | select(.x == "y")'    # Pipe and filter
```

## Output
```bash
jq -r '.name'                   # Raw output (no quotes)
jq -c '.'                       # Compact output
jq -s '.'                       # Slurp (array of inputs)
jq --tab '.'                    # Tab indentation
```

## Constructing
```bash
jq '{name: .title}'             # New object
jq '[.items[].id]'              # New array
jq '{a, b}'                     # Shorthand
jq '. + {new: "field"}'         # Add field
jq 'del(.unwanted)'             # Remove field
```

## Functions
```bash
jq 'length'                     # Length
jq 'keys'                       # Object keys
jq 'values'                     # Object values
jq 'type'                       # Value type
jq 'sort'                       # Sort array
jq 'unique'                     # Unique values
jq 'reverse'                    # Reverse array
jq 'flatten'                    # Flatten nested
jq 'group_by(.key)'             # Group by field
```

## Conditionals
```bash
jq 'if .x then .y else .z end'
jq '.x // "default"'            # Default value
jq '.x? // "default"'           # Suppress errors
```
"#.into(),
        },
        CheatSheet {
            topic: "find".into(),
            category: "Tools".into(),
            source: Source::Bundled,
            content: r#"# find Cheat Sheet

## Basic Usage
```bash
find .                          # All files in current dir
find /path                      # All files in path
find . -name "*.txt"            # By name pattern
find . -iname "*.txt"           # Case insensitive
```

## By Type
```bash
find . -type f                  # Files only
find . -type d                  # Directories only
find . -type l                  # Symlinks only
```

## By Size
```bash
find . -size +10M               # Larger than 10MB
find . -size -1k                # Smaller than 1KB
find . -empty                   # Empty files/dirs
```

## By Time
```bash
find . -mtime -7                # Modified in last 7 days
find . -mtime +30               # Modified over 30 days ago
find . -mmin -60                # Modified in last hour
find . -newer file              # Newer than file
```

## By Permissions
```bash
find . -perm 644                # Exact permissions
find . -perm -644               # At least these perms
find . -user username           # By owner
find . -group groupname         # By group
```

## Actions
```bash
find . -name "*.log" -delete    # Delete matches
find . -exec cmd {} \;          # Run cmd for each
find . -exec cmd {} +           # Run cmd with all
find . -print0 | xargs -0 cmd   # Handle spaces
```

## Combining
```bash
find . -name "*.js" -o -name "*.ts"   # OR
find . -name "*.log" ! -name "*.gz"   # NOT
find . -name "*.txt" -a -size +1M     # AND
find . \( -name "a" -o -name "b" \)   # Grouping
```

## Depth Control
```bash
find . -maxdepth 2              # Max 2 levels deep
find . -mindepth 1              # Skip current dir
```

## Examples
```bash
# Find and delete old logs
find /var/log -name "*.log" -mtime +30 -delete

# Find large files
find / -type f -size +100M 2>/dev/null

# Find and replace in files
find . -name "*.txt" -exec sed -i 's/old/new/g' {} +
```
"#.into(),
        },
    ]
}

pub fn load_user_cheatsheets(path: &std::path::Path) -> Vec<CheatSheet> {
    let mut sheets = Vec::new();

    if !path.exists() {
        return sheets;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.extension().map_or(false, |e| e == "md") {
                if let Ok(content) = std::fs::read_to_string(&file_path) {
                    let topic = file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    sheets.push(CheatSheet {
                        topic,
                        category: "User".into(),
                        source: Source::User { path: file_path },
                        content,
                    });
                }
            }
        }
    }

    sheets
}
